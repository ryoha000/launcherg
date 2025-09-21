pub mod endpoint;
pub mod interprocess;

/// フロントエンドへブロードキャストするイベント名。
pub const APP_SIGNAL_EVENT: &str = "appSignal";
pub const APP_SIGNAL_SYNC_REQUESTED_EVENT: &str = "appSignal:syncRequested";

#[cfg(test)]
pub(crate) mod test_support;
