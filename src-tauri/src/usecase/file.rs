use std::io::prelude::*;
use std::{collections::HashMap, sync::Arc, time::Instant};

use derive_new::new;
use tauri::AppHandle;

use crate::domain::all_game_cache::{AllGameCache, AllGameCacheOne};
use crate::domain::file::{
    get_file_created_at_sync, get_file_name_without_extension, get_game_candidates_by_exe_path,
    PlayHistory,
};
use crate::domain::pubsub::{ProgressLivePayload, ProgressPayload, PubSubService};
use crate::{
    domain::{
        collection::{CollectionElement, NewCollectionElement},
        distance::get_comparable_distance,
        explorer::file::FileExplorer,
        file::{
            get_file_paths_by_exts, get_lnk_metadatas, get_most_probable_game_candidate,
            get_play_history_path, normalize, save_icon_to_png, start_process,
        },
        Id,
    },
    infrastructure::explorerimpl::explorer::ExplorersExt,
};

#[derive(new)]
pub struct FileUseCase<R: ExplorersExt> {
    explorers: Arc<R>,
}

type FilePathString = String;
type ErogamescapeID = i32;

const IGNORE_WORD_WHEN_CONFLICT: [&str; 29] = [
    "設定",
    "チェック",
    "インスト",
    "削除",
    "ファイル",
    "ください",
    "下さい",
    "マニュアル",
    "アップデート",
    "システム",
    "check",
    "setting",
    "config",
    "update",
    "inst",
    "tool",
    "support",
    "setup",
    "unins",
    "define",
    "bhvc",
    "bootstrap",
    "file",
    "exhibit",
    "ihs",
    "launcher",
    "syscfg",
    "updchk",
    "acmp",
];

const SHOULD_UPDATE_WORD_WHEN_CONFLICT: [&str; 6] = ["adv", "64", "cmvs", "bgi", "実行", "起動"];

fn emit_progress_with_time<P: PubSubService>(
    pubsub: &P,
    start: Instant,
    base_announce: &str,
) -> anyhow::Result<()> {
    let end = start.elapsed();
    pubsub.notify("progress", ProgressPayload::new(format!(
        "{}累計{}.{:03}秒経過しました.",
        base_announce,
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    )))
}

impl<R: ExplorersExt> FileUseCase<R> {
    pub async fn concurency_get_file_paths(
        &self,
        explore_dir_paths: Vec<String>,
    ) -> anyhow::Result<Vec<String>> {
        let files_tasks = explore_dir_paths.into_iter().map(|dir_path| {
            tauri::async_runtime::spawn(async move {
                get_file_paths_by_exts(dir_path, vec!["lnk".to_string(), "exe".to_string()])
            })
        });
        Ok(futures::future::try_join_all(files_tasks)
            .await?
            .into_iter()
            .filter_map(|v| v.ok())
            .flatten()
            .collect())
    }
    // 各ゲームとして一番の候補の配列から重複したものを踏まえて各ゲームに対して最高1つのファイルパスにする
    pub fn get_map_of_one_filepath_per_game(
        &self,
        cache_path_pairs: Vec<(AllGameCacheOne, String)>,
    ) -> HashMap<ErogamescapeID, FilePathString> {
        let mut res: HashMap<ErogamescapeID, FilePathString> = HashMap::new();
        for (cache, filepath_unnormalized) in cache_path_pairs {
            match res.get(&cache.id) {
                Some(current_filepath) => {
                    let current_filepath =
                        get_file_name_without_extension(&normalize(&current_filepath))
                            .unwrap_or_default();
                    let filepath =
                        get_file_name_without_extension(&normalize(&filepath_unnormalized))
                            .unwrap_or_default();

                    let mut must_update = false;
                    let mut not_must_update = false;
                    // 競合時に無視する単語をチェックする
                    for ignore_word in IGNORE_WORD_WHEN_CONFLICT {
                        if current_filepath.contains(ignore_word) {
                            must_update = true;
                            break;
                        }
                        if filepath.contains(ignore_word) {
                            not_must_update = true;
                            break;
                        }
                    }
                    // 競合時に更新する単語をチェックする
                    for update_word in SHOULD_UPDATE_WORD_WHEN_CONFLICT {
                        if current_filepath.contains(update_word) {
                            not_must_update = true;
                            break;
                        }
                        if filepath.contains(update_word) {
                            must_update = true;
                            break;
                        }
                    }
                    if must_update && !not_must_update {
                        res.insert(cache.id, filepath_unnormalized);
                    } else if !not_must_update {
                        let gamename = &cache.gamename;
                        let current_distance = get_comparable_distance(&current_filepath, gamename);
                        let distance = get_comparable_distance(&filepath, gamename);
                        if current_distance < distance {
                            res.insert(cache.id, filepath_unnormalized);
                        }
                    }
                }
                None => {
                    res.insert(cache.id, filepath_unnormalized);
                }
            }
        }
        res
    }
    pub async fn concurency_get_path_game_map<P: PubSubService + 'static>(
        &self,
        normalized_all_games: Arc<AllGameCache>,
        files: Vec<String>,
        pubsub: Arc<P>,
    ) -> anyhow::Result<HashMap<ErogamescapeID, FilePathString>> {
        let get_game_id_tasks = files.into_iter().map(|path| {
            let all = normalized_all_games.clone();
            let pubsub_clone = Arc::clone(&pubsub);
            tauri::async_runtime::spawn(async move {
                let res = get_most_probable_game_candidate(&all, path)?;
                if let Err(e) = pubsub_clone.notify("progresslive", ProgressLivePayload::new(None)) {
                    return Err(e);
                }
                Ok(res)
            })
        });

        let most_probable_game_filepath_pairs: Vec<(AllGameCacheOne, FilePathString)> =
            futures::future::try_join_all(get_game_id_tasks)
                .await?
                .into_iter()
                .filter_map(|v| v.ok().and_then(|res| res))
                .collect();

        Ok(self.get_map_of_one_filepath_per_game(most_probable_game_filepath_pairs))
    }
    pub async fn get_game_candidates(
        &self,
        all_game_cache: AllGameCache,
        file: String,
    ) -> anyhow::Result<AllGameCache> {
        let normalized_all_games = all_game_cache
            .iter()
            .map(|pair| AllGameCacheOne {
                id: pair.id,
                gamename: normalize(&pair.gamename),
            })
            .collect::<AllGameCache>();
        get_game_candidates_by_exe_path(&normalized_all_games, &file, 0.2, 5)
    }
    pub async fn filter_files_to_collection_elements<P: PubSubService + 'static>(
        &self,
        handle: &Arc<AppHandle>,
        files: Vec<String>,
        all_game_cache: AllGameCache,
        pubsub: Arc<P>,
    ) -> anyhow::Result<Vec<NewCollectionElement>> {
        let start = Instant::now();

        let normalized_all_games = Arc::new(
            all_game_cache
                .iter()
                .map(|pair| AllGameCacheOne {
                    id: pair.id,
                    gamename: normalize(&pair.gamename),
                })
                .collect::<AllGameCache>(),
        );
        let all_erogamescape_game_map: HashMap<i32, String> = all_game_cache
            .into_iter()
            .map(|v| (v.id, v.gamename))
            .collect();

        let (exe_id_path_vec, lnk_id_path_vec): (Vec<(i32, String)>, Vec<(i32, String)>) = self
            .concurency_get_path_game_map(
                normalized_all_games,
                files,
                pubsub.clone(),
            )
            .await?
            .into_iter()
            .partition(|(_id, path)| path.to_lowercase().ends_with("exe"));

        emit_progress_with_time(
            pubsub.as_ref(),
            start,
            ".lnk, .exe ファイルのゲームとの紐づけが完了しました。",
        )?;

        let lnk_path_vec: Vec<&str> = lnk_id_path_vec
            .iter()
            .map(|(_, lnk_path)| lnk_path.as_str())
            .collect();
        let lnk_metadatas = get_lnk_metadatas(lnk_path_vec)?;
        if lnk_id_path_vec.len() != lnk_metadatas.len() {
            pubsub.notify("progress", ProgressPayload::new(format!(
                "lnk ファイルの数と lnk のターゲットファイルの数が一致しません。リンクファイル数: {}, ターゲットファイル数: {}", lnk_id_path_vec.len(), lnk_metadatas.len()
            )))?;
        }

        let mut collection_elements = vec![];
        let mut save_icon_tasks = vec![];
        for (id, exe_path) in exe_id_path_vec.into_iter() {
            // icon
            let task = save_icon_to_png(handle, &exe_path, &Id::new(id))?;
            save_icon_tasks.push(task);

            // new collection element
            if let Some(gamename) = all_erogamescape_game_map.get(&id) {
                let install_at = get_file_created_at_sync(&exe_path);
                collection_elements.push(NewCollectionElement::new(
                    Id::new(id),
                    gamename.clone(),
                    Some(exe_path),
                    None,
                    install_at,
                ));
            }
        }
        for (id, lnk_path) in lnk_id_path_vec.iter() {
            let id = Id::new(*id);
            let install_at;
            // icon
            if let Some(metadata) = lnk_metadatas.get(lnk_path.as_str()) {
                let task;
                if metadata.icon.to_lowercase().ends_with("ico") {
                    task = save_icon_to_png(handle, &metadata.icon, &id)?;
                } else {
                    task = save_icon_to_png(handle, &metadata.path, &id)?;
                }
                save_icon_tasks.push(task);

                install_at = get_file_created_at_sync(&metadata.path);
            } else {
                install_at = None;
            }

            if let Some(gamename) = all_erogamescape_game_map.get(&id.value) {
                collection_elements.push(NewCollectionElement::new(
                    id,
                    gamename.clone(),
                    None,
                    Some(lnk_path.clone()),
                    install_at,
                ));
            }
        }
        futures::future::try_join_all(save_icon_tasks)
            .await?
            .into_iter()
            .collect::<anyhow::Result<()>>()?;

        emit_progress_with_time(pubsub.as_ref(), start, "icon の保存が完了しました。")?;

        Ok(collection_elements)
    }
    pub fn get_new_upload_image_path(
        &self,
        handle: &Arc<AppHandle>,
        id: i32,
    ) -> anyhow::Result<String> {
        self.explorers
            .file_explorer()
            .get_save_image_path(handle, id)
    }
    pub async fn upload_image(
        &self,
        handle: &Arc<AppHandle>,
        id: i32,
        base64_image: String,
    ) -> anyhow::Result<String> {
        let path = self
            .explorers
            .file_explorer()
            .get_save_image_path(handle, id)?;
        self.explorers
            .file_explorer()
            .save_base64_image(&path, base64_image)?;
        Ok(path)
    }
    pub fn start_game(
        &self,
        collection_element: CollectionElement,
        is_run_as_admin: bool,
    ) -> anyhow::Result<Option<u32>> {
        start_process(
            is_run_as_admin,
            collection_element.exe_path,
            collection_element.lnk_path,
        )
    }
    pub fn get_play_time_minutes(
        &self,
        handle: &Arc<AppHandle>,
        collection_element_id: &Id<CollectionElement>,
    ) -> anyhow::Result<f32> {
        let path = get_play_history_path(handle, collection_element_id);

        let exist = std::path::Path::new(&path).exists();
        if !exist {
            return Ok(0.0);
        }

        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);

        let mut histories = vec![];
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(history) = serde_json::from_str::<PlayHistory>(&line) {
                    histories.push(history)
                }
            }
        }

        Ok(histories.into_iter().map(|v| v.minutes).sum())
    }
}
