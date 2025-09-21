use std::sync::Arc;

use anyhow::Result;
use derive_new::new;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use domain::pubsub::{PubSubEvent, PubSubService};

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

impl PubSub {
    fn emit<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<()> {
        self.handle
            .emit(event, payload)
            .map_err(|e| anyhow::anyhow!("Failed to emit event {}: {}", event, e.to_string()))?;
        Ok(())
    }
}

impl PubSubService for PubSub {
    fn notify(&self, event: PubSubEvent) -> Result<()> {
        match event {
            PubSubEvent::Progress(payload) => self.emit("progress", payload),
            PubSubEvent::ProgressLive(payload) => self.emit("progresslive", payload),
            PubSubEvent::ScanProgress(payload) => self.emit("scanProgress", payload),
            PubSubEvent::ScanLog(payload) => self.emit("scanLog", payload),
            PubSubEvent::ScanSummary(payload) => self.emit("scanSummary", payload),
            PubSubEvent::ScanPhaseTiming(payload) => self.emit("scanPhaseTiming", payload),
            PubSubEvent::ScanEnrichResult(payload) => self.emit("scanEnrichResult", payload),
            PubSubEvent::ScanDedup(payload) => self.emit("scanDedup", payload),
            PubSubEvent::ExtensionConnectionStatus(payload) => {
                self.emit("extension-connection-status", payload)
            }
            PubSubEvent::ImageQueueWorkerStarted(payload) => {
                self.emit("imageQueueWorkerStarted", payload)
            }
            PubSubEvent::ImageQueueWorkerFinished(payload) => {
                self.emit("imageQueueWorkerFinished", payload)
            }
            PubSubEvent::ImageQueueItemStarted(payload) => {
                self.emit("imageQueueItemStarted", payload)
            }
            PubSubEvent::ImageQueueItemSucceeded(payload) => {
                self.emit("imageQueueItemSucceeded", payload)
            }
            PubSubEvent::ImageQueueItemFailed(payload) => {
                self.emit("imageQueueItemFailed", payload)
            }
            PubSubEvent::AppSignal(payload) => self.emit("appSignal", payload),
            PubSubEvent::AppSignalShowMessage(payload) => {
                self.emit("appSignal:showMessage", payload)
            }
            PubSubEvent::AppSignalShowErrorMessage(payload) => {
                self.emit("appSignal:showErrorMessage", payload)
            }
        }
    }
}
