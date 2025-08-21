use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DenyListItemVm {
    pub id: i32,
    pub store_type: i32,
    pub store_id: String,
}

impl From<domain::deny_list::DenyListEntry> for DenyListItemVm {
    fn from(value: domain::deny_list::DenyListEntry) -> Self {
        Self { id: value.id.value, store_type: value.store_type as i32, store_id: value.store_id }
    }
}


