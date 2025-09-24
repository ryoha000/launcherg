use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn get_game_candidates(
    modules: State<'_, Arc<Modules>>,
    filepath: String,
) -> anyhow::Result<Vec<(i32, String)>, CommandError> {
    let info = domain::game_matcher::extract_file_info(&filepath)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let mut queries: Vec<String> = Vec::new();
    if !info.skip_filename {
        queries.push(info.filename);
    }
    queries.push(info.parent_dir);
    let result = modules
        .game_matcher()
        .find_candidates(&queries)
        .into_iter()
        .map(|(c, _d)| (c.id, c.gamename))
        .collect();
    Ok(result)
}

#[tauri::command]
pub async fn get_game_candidates_by_name(
    modules: State<'_, Arc<Modules>>,
    game_name: String,
) -> anyhow::Result<Vec<(i32, String)>, CommandError> {
    let normalized_name = domain::game_matcher::normalize(&game_name);
    let result = modules
        .game_matcher()
        .find_candidates(&[normalized_name])
        .into_iter()
        .take(20)
        .map(|(c, _d)| (c.id, c.gamename))
        .collect();
    Ok(result)
}


