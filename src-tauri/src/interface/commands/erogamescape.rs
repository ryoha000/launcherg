use std::sync::Arc;

use serde::Deserialize;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertErogamescapeInformationInput {
    pub erogamescape_id: i32,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

#[tauri::command]
pub async fn upsert_erogamescape_information(
    modules: State<'_, Arc<Modules>>,
    details: Vec<UpsertErogamescapeInformationInput>,
) -> anyhow::Result<(), CommandError> {
    use crate::domain::erogamescape::NewErogamescapeInformation;
    let inputs: Vec<NewErogamescapeInformation> = details
        .into_iter()
        .map(|d| {
            NewErogamescapeInformation::new(
                d.erogamescape_id,
                d.gamename_ruby,
                d.brandname,
                d.brandname_ruby,
                d.sellday,
                d.is_nukige,
            )
        })
        .collect();

    modules
        .erogamescape_use_case()
        .upsert_information_batch(&inputs)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn get_not_registered_erogamescape_information_ids(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<i32>, CommandError> {
    modules
        .erogamescape_use_case()
        .find_missing_information_ids()
        .await
        .map_err(Into::into)
}
