use crate::domain::works::WorkDetails;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkDetailsVm {
    pub id: i32,
    pub title: String,
    pub dmm: Option<DmmSideVm>,
    pub dlsite: Option<DlsiteSideVm>,
    pub collection_element_id: Option<i32>,
    pub is_dmm_omitted: bool,
    pub is_dlsite_omitted: bool,
    pub is_dmm_pack: bool,
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

impl From<WorkDetails> for WorkDetailsVm {
    fn from(w: WorkDetails) -> Self {
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
            is_dmm_omitted: w.is_dmm_omitted,
            is_dlsite_omitted: w.is_dlsite_omitted,
            is_dmm_pack: w.is_dmm_pack,
        }
    }
}


