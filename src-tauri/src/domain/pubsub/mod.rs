use serde::Serialize;

pub trait PubSubService: Send + Sync {
    fn notify<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<(), anyhow::Error>;
}