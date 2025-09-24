use std::sync::Arc;
use tauri::State;

use crate::domain::Id;
use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn get_erogamescape_id_by_collection_id(
    modules: State<'_, Arc<Modules>>,
    collection_element_id: i32,
) -> anyhow::Result<Option<i32>, CommandError> {
    Ok(modules
        .collection_use_case()
        .get_erogamescape_id_by_collection_id(&Id::new(collection_element_id))
        .await?)
}
