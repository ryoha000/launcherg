use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct NativeMessageTs {
    pub request_id: String,
    pub message: NativeMessageCase,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "case", content = "value")] // Buf JSON と互換なタグ付け
pub enum NativeMessageCase {
    SyncDmmGames(super::sync::DmmSyncGamesRequestTs),
    SyncDlsiteGames(super::sync::DlsiteSyncGamesRequestTs),
    GetStatus(GetStatusRequestTs),
    SetConfig(super::status::ExtensionConfigTs),
    HealthCheck(HealthCheckRequestTs),
    GetDmmPackIds(super::packs::GetDmmPackIdsRequestTs),
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct GetStatusRequestTs {}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct HealthCheckRequestTs {}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct NativeResponseTs {
    pub success: bool,
    pub error: String,
    pub request_id: String,
    pub response: Option<NativeResponseCase>,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "case", content = "value")] // Buf JSON と互換
pub enum NativeResponseCase {
    SyncGamesResult(super::sync::SyncBatchResultTs),
    StatusResult(super::status::SyncStatusTs),
    ConfigResult(super::status::ConfigUpdateResultTs),
    HealthCheckResult(HealthCheckResultTs),
    DmmPackIds(super::packs::DmmPackIdsResponseTs),
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct HealthCheckResultTs {
    pub message: String,
    pub version: String,
}

