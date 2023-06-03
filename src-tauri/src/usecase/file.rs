use std::{collections::HashMap, sync::Arc, time::Instant};

use derive_new::new;

use crate::{
    domain::{
        collection::NewCollectionElement,
        distance::get_comparable_distance,
        explored_cache::ExploredCache,
        explorer::network::NetworkExplorer,
        file::{
            filter_game_path, get_file_paths_by_exts, get_lnk_metadatas, normalize,
            save_icon_to_png,
        },
        network::ErogamescapeIDNamePair,
        Id,
    },
    infrastructure::explorerimpl::explorer::ExplorersExt,
};

#[derive(new)]
pub struct FileUseCase<R: ExplorersExt> {
    explorers: Arc<R>,
}

type FilePathString = String;
type Gamename = String;
type ErogamescapeID = i32;

const IGNORE_WORD_WHEN_CONFLICT: [&str; 2] = ["設定", "チェック"];

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
    pub async fn concurency_get_path_game_map(
        &self,
        normalized_all_games: Arc<Vec<ErogamescapeIDNamePair>>,
        files: Vec<String>,
    ) -> anyhow::Result<HashMap<ErogamescapeID, FilePathString>> {
        let get_game_id_tasks = files.into_iter().map(|path| {
            let all = normalized_all_games.clone();
            tauri::async_runtime::spawn(async move { filter_game_path(&all, path) })
        });

        let id_path_pairs: Vec<(ErogamescapeIDNamePair, String)> =
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
                        let distance = get_comparable_distance(&before, &pair.0.gamename);
                        if before_distance < distance {
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
    pub async fn filter_files_to_collection_elements(
        &self,
        files: Vec<String>,
    ) -> anyhow::Result<Vec<NewCollectionElement>> {
        let start = Instant::now();

        let all_erogamescape_games_vec = self.explorers.network_explorer().get_all_games().await?;

        let normalized_all_games = Arc::new(
            all_erogamescape_games_vec
                .iter()
                .map(|pair| ErogamescapeIDNamePair {
                    id: pair.id,
                    gamename: normalize(&pair.gamename),
                })
                .collect::<Vec<ErogamescapeIDNamePair>>(),
        );
        let all_erogamescape_game_map: HashMap<i32, String> = all_erogamescape_games_vec
            .into_iter()
            .map(|v| (v.id, v.gamename))
            .collect();

        let end = start.elapsed();
        println!(
            "all_games の取得完了. {}.{:03}秒経過しました.",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );

        let (lnk_id_path_vec, exe_id_path_vec): (Vec<(i32, String)>, Vec<(i32, String)>) = self
            .concurency_get_path_game_map(normalized_all_games, files)
            .await?
            .into_iter()
            .partition(|(id, path)| path.to_lowercase().ends_with("lnk"));

        let end = start.elapsed();
        println!(
            "id との紐づけ 完了. {}.{:03}秒経過しました.",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );
        println!(
            "exe_id_path_vec.len(): {}, lnk_id_path_vec.len(): {}",
            exe_id_path_vec.len(),
            lnk_id_path_vec.len()
        );
        println!(
            "exe_id_path_vec: {:#?}, lnk_id_path_vec: {:#?}",
            exe_id_path_vec, lnk_id_path_vec
        );

        let (lnk_id_vec, lnk_path_vec): (Vec<ErogamescapeID>, Vec<String>) =
            lnk_id_path_vec.into_iter().unzip();

        let lnk_metadatas = get_lnk_metadatas(lnk_path_vec)?;
        if lnk_id_vec.len() != lnk_metadatas.len() {
            return Err(anyhow::anyhow!(
                "lnk ファイルの数と lnk のターゲットファイルの数が一致しません",
            ));
        }

        let mut save_icon_tasks = vec![];
        for icon_src_path in exe_id_path_vec.iter() {
            let task = save_icon_to_png(&icon_src_path.1, &Id::new(icon_src_path.0))?;
            save_icon_tasks.push(task);
        }
        for icon_src_path in lnk_id_vec.iter().zip(lnk_metadatas.iter().map(|v| &v.path)) {
            let task = save_icon_to_png(icon_src_path.1, &Id::new(*icon_src_path.0))?;
            save_icon_tasks.push(task);
        }
        futures::future::try_join_all(save_icon_tasks)
            .await?
            .into_iter()
            .collect::<anyhow::Result<()>>()?;

        let end = start.elapsed();
        println!(
            "icon の保存完了. {}.{:03}秒経過しました.",
            end.as_secs(),
            end.subsec_nanos() / 1_000_000
        );

        let collection_elements: Vec<NewCollectionElement> = lnk_id_vec
            .into_iter()
            .zip(lnk_metadatas.into_iter().map(|v| v.path))
            .chain(exe_id_path_vec)
            .filter_map(|(id, path)| {
                if let Some(gamename) = all_erogamescape_game_map.get(&id) {
                    return Some(NewCollectionElement {
                        id: Id::new(id),
                        gamename: gamename.clone(),
                        path,
                    });
                }
                None
            })
            .collect();

        Ok(collection_elements)
    }
}
