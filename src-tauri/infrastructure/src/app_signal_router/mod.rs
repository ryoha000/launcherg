pub mod endpoint;
pub mod interprocess;

/// フロントエンドへブロードキャストするイベント名。
pub const APP_SIGNAL_EVENT: &str = "appSignal";
pub const APP_SIGNAL_SHOW_MESSAGE_EVENT: &str = "appSignal:showMessage";
pub const APP_SIGNAL_SHOW_ERROR_MESSAGE_EVENT: &str = "appSignal:showErrorMessage";
pub const APP_SIGNAL_REFETCH_WORK_EVENT: &str = "appSignal:refetchWork";
pub const APP_SIGNAL_REFETCH_WORKS_EVENT: &str = "appSignal:refetchWorks";

#[cfg(test)]
pub(crate) mod test_support;
