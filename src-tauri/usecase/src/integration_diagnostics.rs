use anyhow::{ensure, Context};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const MAX_LOG_BYTES: u64 = 1024 * 1024;

/// Deliberately contains progress metadata only, never store responses or credentials.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrationDiagnosticEvent {
    pub timestamp: String,
    pub store: String,
    pub stage: String,
    pub trigger: String,
    pub page_kind: String,
    pub count: Option<u32>,
    pub detail: String,
}

pub fn record_diagnostics(
    root: &Path,
    events: &[IntegrationDiagnosticEvent],
) -> anyhow::Result<()> {
    ensure!(
        !events.is_empty() && events.len() <= 100,
        "Invalid diagnostic batch size"
    );
    let mut bytes = Vec::new();
    for event in events {
        ensure!(
            matches!(event.store.as_str(), "dlsite" | "dmm" | "extension"),
            "Invalid store"
        );
        ensure!(
            matches!(
                event.trigger.as_str(),
                "auto" | "manual" | "periodic" | "background"
            ),
            "Invalid trigger"
        );
        ensure!(
            matches!(
                event.page_kind.as_str(),
                "library" | "work" | "other" | "background"
            ),
            "Invalid page kind"
        );
        ensure!(
            event.stage.len() <= 64
                && !event.stage.is_empty()
                && event
                    .stage
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c == b'_'),
            "Invalid diagnostic stage"
        );
        ensure!(
            event.timestamp.len() <= 40
                && chrono::DateTime::parse_from_rfc3339(&event.timestamp).is_ok(),
            "Invalid timestamp"
        );
        ensure!(event.detail.len() <= 2048, "Diagnostic detail too long");
        serde_json::to_writer(&mut bytes, event)?;
        bytes.push(b'\n');
    }
    let dir = root.join("diagnostics");
    fs::create_dir_all(&dir)?;
    let lock = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(dir.join("browser-sync.lock"))?;
    lock.lock_exclusive().context("Lock browser diagnostics")?;
    let current = dir.join("browser-sync.jsonl");
    let previous = dir.join("browser-sync.previous.jsonl");
    if fs::metadata(&current).map(|m| m.len()).unwrap_or(0) + bytes.len() as u64 > MAX_LOG_BYTES {
        if previous.exists() {
            fs::remove_file(&previous)?;
        }
        if current.exists() {
            fs::rename(&current, &previous)?;
        }
    }
    let mut file = OpenOptions::new().create(true).append(true).open(current)?;
    file.write_all(&bytes)?;
    file.flush()?;
    Ok(())
}
