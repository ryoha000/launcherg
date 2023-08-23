use std::io::prelude::*;
use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc, time::Instant};

use derive_new::new;

use crate::domain::all_game_cache::AllGameCache;
use crate::domain::file::{get_file_created_at_sync, get_game_candidates_by_exe_path, PlayHistory};
use crate::{
    domain::{
        collection::{CollectionElement, NewCollectionElement},
        distance::get_comparable_distance,
        explorer::file::FileExplorer,
        file::{
            filter_game_path, get_file_paths_by_exts, get_lnk_metadatas, get_play_history_path,
            normalize, save_icon_to_png, start_process,
        },
        network::ErogamescapeIDNamePair,
        Id,
    },
    infrastructure::explorerimpl::explorer::ExplorersExt,
};

use super::error::UseCaseError;

#[derive(new)]
pub struct FileUseCase<R: ExplorersExt> {
    explorers: Arc<R>,
}

type FilePathString = String;
type ErogamescapeID = i32;

const IGNORE_WORD_WHEN_CONFLICT: [&str; 7] = [
    "設定",
    "チェック",
    "アンインスト",
    "削除",
    "ファイル",
    "ください",
    "マニュアル",
];

fn emit_progress_with_time(
    f: Arc<impl Fn(String) -> anyhow::Result<()>>,
    start: Instant,
    base_announce: &str,
) -> anyhow::Result<()> {
    let end = start.elapsed();
    f(format!(
        "{}累計{}.{:03}秒経過しました.",
        base_announce,
        end.as_secs(),
        end.subsec_nanos() / 1_000_000
    ))
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
    pub async fn concurency_get_path_game_map<F: Fn() -> anyhow::Result<()> + Send + 'static>(
        &self,
        normalized_all_games: Arc<Vec<ErogamescapeIDNamePair>>,
        files: Vec<String>,
        callback: Arc<Mutex<F>>,
    ) -> anyhow::Result<HashMap<ErogamescapeID, FilePathString>> {
        let get_game_id_tasks = files.into_iter().map(|path| {
            let all = normalized_all_games.clone();
            let mutex_cb = Arc::clone(&callback);
            tauri::async_runtime::spawn(async move {
                let res = filter_game_path(&all, path)?;
                match mutex_cb.lock() {
                    Ok(cb) => {
                        if let Err(e) = cb() {
                            return Err(e);
                        };
                        Ok(res)
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!(e.to_string()));
                    }
                }
            })
        });

        let id_path_pairs: Vec<(ErogamescapeIDNamePair, FilePathString)> =
            futures::future::try_join_all(get_game_id_tasks)
                .await?
                .into_iter()
                .filter_map(|v| v.ok().and_then(|res| res))
                .collect();

        let mut id_path_map: HashMap<ErogamescapeID, FilePathString> = HashMap::new();
        for pair in id_path_pairs.into_iter() {
            let before = id_path_map.get(&pair.0.id);
            match before {
                Some(before) => {
                    let mut must_update = false;
                    let mut not_must_update = false;
                    for ignore_word in IGNORE_WORD_WHEN_CONFLICT {
                        if before.contains(ignore_word) {
                            must_update = true;
                            break;
                        }
                        if pair.1.contains(ignore_word) {
                            not_must_update = true;
                            break;
                        }
                    }
                    if must_update {
                        id_path_map.insert(pair.0.id, pair.1);
                    } else if !not_must_update {
                        let before_distance = get_comparable_distance(&before, &pair.0.gamename);
                        let after_distance = get_comparable_distance(&pair.1, &pair.0.gamename);
                        println!("conflicted. gamename: {}, before: {}, before_distance: {}, after: {}, after_distance: {}", pair.0.gamename, before, before_distance, pair.1, after_distance);
                        if before_distance < after_distance {
                            id_path_map.insert(pair.0.id, pair.1);
                        }
                    }
                }
                None => {
                    id_path_map.insert(pair.0.id, pair.1);
                }
            }
        }
        Ok(id_path_map)
    }
    pub async fn concurency_get_game_candidates_map(
        &self,
        normalized_all_games: Arc<Vec<ErogamescapeIDNamePair>>,
        files: Vec<String>,
    ) -> anyhow::Result<HashMap<String, Vec<ErogamescapeIDNamePair>>> {
        let get_game_id_tasks = files.into_iter().map(|path| {
            let all = normalized_all_games.clone();
            tauri::async_runtime::spawn(async move {
                match get_game_candidates_by_exe_path(&all, &path, 0.5, 5) {
                    Ok(pairs) => {
                        return Ok((path, pairs));
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!(e.to_string()));
                    }
                }
            })
        });

        Ok(futures::future::try_join_all(get_game_id_tasks)
            .await?
            .into_iter()
            .filter_map(|v| v.ok())
            .collect())
    }
    pub async fn filter_files_to_collection_elements<
        F: Fn() -> anyhow::Result<()> + Send + 'static,
    >(
        &self,
        files: Vec<String>,
        all_game_cache: AllGameCache,
        emit_progress: Arc<impl Fn(String) -> anyhow::Result<()>>,
        process_each_game_file_callback: Arc<Mutex<F>>,
    ) -> anyhow::Result<Vec<NewCollectionElement>> {
        let start = Instant::now();

        let normalized_all_games = Arc::new(
            all_game_cache
                .iter()
                .map(|pair| ErogamescapeIDNamePair {
                    id: pair.id,
                    gamename: normalize(&pair.gamename),
                })
                .collect::<Vec<ErogamescapeIDNamePair>>(),
        );
        let all_erogamescape_game_map: HashMap<i32, String> = all_game_cache
            .into_iter()
            .map(|v| (v.id, v.gamename))
            .collect();

        let (lnk_id_path_vec, exe_id_path_vec): (Vec<(i32, String)>, Vec<(i32, String)>) = self
            .concurency_get_path_game_map(
                normalized_all_games,
                files,
                process_each_game_file_callback,
            )
            .await?
            .into_iter()
            .partition(|(_id, path)| path.to_lowercase().ends_with("lnk"));

        emit_progress_with_time(
            emit_progress.clone(),
            start,
            ".lnk, .exe ファイルのゲームとの紐づけが完了しました。",
        )?;

        let lnk_path_vec: Vec<&str> = lnk_id_path_vec
            .iter()
            .map(|(_, lnk_path)| lnk_path.as_str())
            .collect();
        let lnk_metadatas = get_lnk_metadatas(lnk_path_vec)?;
        if lnk_id_path_vec.len() != lnk_metadatas.len() {
            emit_progress(format!(
                "lnk ファイルの数と lnk のターゲットファイルの数が一致しません。リンクファイル数: {}, ターゲットファイル数: {}", lnk_id_path_vec.len(), lnk_metadatas.len()
            ))?;
        }

        let mut save_icon_tasks = vec![];
        for (id, icon_src_path) in exe_id_path_vec.iter() {
            let task = save_icon_to_png(&icon_src_path, &Id::new(*id))?;
            save_icon_tasks.push(task);
        }
        for (id, lnk_path) in lnk_id_path_vec.iter() {
            if let Some(metadata) = lnk_metadatas.get(lnk_path.as_str()) {
                let task = save_icon_to_png(&metadata.icon, &Id::new(*id))?;
                save_icon_tasks.push(task);
            }
        }
        futures::future::try_join_all(save_icon_tasks)
            .await?
            .into_iter()
            .collect::<anyhow::Result<()>>()?;

        emit_progress_with_time(emit_progress.clone(), start, "icon の保存が完了しました。")?;

        let collection_elements: Vec<NewCollectionElement> = lnk_id_path_vec
            .iter()
            .filter_map(|(id, lnk_path)| {
                if let Some(metadata) = lnk_metadatas.get(lnk_path.as_str()) {
                    return Some((*id, metadata.path.clone()));
                }
                None
            })
            .chain(exe_id_path_vec.into_iter().map(|(id, path)| (id, path)))
            .filter_map(|(id, path)| {
                if let Some(gamename) = all_erogamescape_game_map.get(&id) {
                    let install_at = get_file_created_at_sync(&path);
                    return Some(NewCollectionElement {
                        id: Id::new(id),
                        gamename: gamename.clone(),
                        path,
                        install_at,
                    });
                }
                None
            })
            .collect();

        Ok(collection_elements)
    }
    pub async fn upload_image(&self, id: i32, base64_image: String) -> anyhow::Result<String> {
        let path = self.explorers.file_explorer().get_save_image_path(id)?;
        self.explorers
            .file_explorer()
            .save_base64_image(&path, base64_image)?;
        Ok(path)
    }
    pub fn start_game(
        &self,
        collection_element: CollectionElement,
        is_run_as_admin: bool,
    ) -> anyhow::Result<()> {
        let exist = std::path::Path::new(&collection_element.path).exists();
        if !exist {
            return Err(UseCaseError::IsNotValidPath(collection_element.path).into());
        }
        let play_history_path = get_play_history_path(&collection_element.id);
        Ok(start_process(
            is_run_as_admin,
            &collection_element.path,
            &play_history_path,
        )?)
    }
    pub fn get_play_time_minutes(
        &self,
        collection_element_id: &Id<CollectionElement>,
    ) -> anyhow::Result<f32> {
        let path = get_play_history_path(collection_element_id);

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
