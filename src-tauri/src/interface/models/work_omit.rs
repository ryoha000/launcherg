use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkOmitItemVm { pub id: i32, pub work_id: i32 }

impl From<domain::work_omit::WorkOmit> for WorkOmitItemVm {
    fn from(value: domain::work_omit::WorkOmit) -> Self { Self { id: value.id.value, work_id: value.work_id.value } }
}


