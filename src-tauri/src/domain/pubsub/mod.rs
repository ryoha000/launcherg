use derive_new::new;
use serde::Serialize;

#[derive(new, Clone, Serialize)]
pub struct ProgressPayload {
    pub message: String,
}

#[derive(new, Clone, Serialize)]
pub struct ProgressLivePayload {
    pub max: Option<i32>,
}

pub trait PubSubService: Send + Sync {
    fn notify<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<(), anyhow::Error>;
}
