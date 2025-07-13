use std::sync::Arc;

use anyhow::Result;
use derive_new::new;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::domain::pubsub::PubSubService;

#[derive(new, Clone)]
pub struct PubSub {
    handle: Arc<AppHandle>,
}

pub trait PubSubExt {
    type PubSubService: PubSubService;
    fn pubsub(&self) -> &Self::PubSubService;
}

impl PubSubExt for PubSub {
    type PubSubService = PubSub;
    fn pubsub(&self) -> &Self::PubSubService {
        self
    }
}

impl PubSubService for PubSub {
    fn notify<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<()> {
        self.handle
            .emit(event, payload)
            .map_err(|e| anyhow::anyhow!("Failed to emit event {}: {}", event, e.to_string()))?;
        Ok(())
    }
}