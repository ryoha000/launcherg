use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmmWorkPackVm {
    pub id: i32,
    pub work_id: i32,
}

impl From<crate::domain::dmm_work_pack::DmmWorkPack> for DmmWorkPackVm {
    fn from(value: crate::domain::dmm_work_pack::DmmWorkPack) -> Self {
        Self { id: value.id.value, work_id: value.work_id.value }
    }
}


