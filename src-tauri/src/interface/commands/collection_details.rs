use std::sync::Arc;
use tauri::State;

use crate::domain::Id;
use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use crate::usecase::models::collection::CreateCollectionElementDetail;

#[tauri::command]
pub async fn upsert_collection_element_details(
    modules: State<'_, Arc<Modules>>,
    details: Vec<CreateCollectionElementDetail>,
) -> anyhow::Result<(), CommandError> {
    for detail in details {
        let info = domain::collection::NewCollectionElementInfo::new(
            Id::new(detail.collection_element_id),
            detail.gamename_ruby,
            detail.brandname,
            detail.brandname_ruby,
            detail.sellday,
            detail.is_nukige,
        );
        modules
            .collection_use_case()
            .upsert_collection_element_info(&info)
            .await?;
    }
    Ok(())
}

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
