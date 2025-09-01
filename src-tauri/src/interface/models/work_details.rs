use domain::service::save_path_resolver::{DirsSavePathResolver, SavePathResolver};

use crate::domain::works::WorkDetails;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkDetailsVm {
    pub id: i32,
    pub title: String,
    pub dmm: Option<DmmSideVm>,
    pub dlsite: Option<DlsiteSideVm>,
    pub collection_element_id: Option<i32>,
    pub erogamescape_id: Option<i32>,
    pub is_omitted: bool,
    pub is_dmm_pack: bool,
    pub thumbnail: Option<String>,
    pub latest_download_path: Option<LatestWorkDownloadPathVm>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmmSideVm {
    pub id: i32,
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub is_omitted: bool,
    pub is_pack: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DlsiteSideVm {
    pub id: i32,
    pub store_id: String,
    pub category: String,
    pub is_omitted: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestWorkDownloadPathVm {
    pub id: i32,
    pub work_id: i32,
    pub download_path: String,
}

impl From<WorkDetails> for WorkDetailsVm {
    fn from(w: WorkDetails) -> Self {
        let resolver = DirsSavePathResolver::default();
        let thumbnail = if let Some(dmm) = w.dmm.as_ref() {
            Some(resolver.thumbnail_alias_dmm_png_path(&dmm.category, &dmm.subcategory, &dmm.store_id))
        } else if let Some(dlsite) = w.dlsite.as_ref() {
            Some(resolver.thumbnail_alias_dlsite_png_path(&dlsite.category, &dlsite.store_id))
        } else if let Some(collection_element_id) = w.collection_element_id.as_ref() {
            Some(resolver.thumbnail_png_path(collection_element_id.value))
        } else {
            None
        };
        WorkDetailsVm {
            id: w.work.id.value,
            title: w.work.title,
            dmm: w.dmm.map(|d| DmmSideVm {
                id: d.id.value,
                store_id: d.store_id,
                category: d.category,
                subcategory: d.subcategory,
                is_omitted: false,
                is_pack: false,
            }),
            dlsite: w.dlsite.map(|d| DlsiteSideVm {
                id: d.id.value,
                store_id: d.store_id,
                category: d.category,
                is_omitted: false,
            }),
            collection_element_id: w.collection_element_id.map(|v| v.value),
            erogamescape_id: w.erogamescape.as_ref().map(|e| e.erogamescape_id),
            is_omitted: w.is_omitted,
            is_dmm_pack: w.is_dmm_pack,
            thumbnail: thumbnail,
            latest_download_path: w.latest_download_path.map(|p| LatestWorkDownloadPathVm {
                id: p.id.value,
                work_id: p.work_id.value,
                download_path: p.download_path,
            }),
        }
    }
}


