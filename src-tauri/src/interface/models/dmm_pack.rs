use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmmPackMarkVm {
    pub id: i32,
    pub store_id: String,
    pub name: String,
}

impl From<crate::domain::dmm_pack::DmmPackMark> for DmmPackMarkVm {
    fn from(value: crate::domain::dmm_pack::DmmPackMark) -> Self {
        Self { id: value.id.value, store_id: value.store_id, name: value.name }
    }
}


