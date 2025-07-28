use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcTailError {
    #[error("Failed to connect to ProcTail service: {0}")]
    ConnectionFailed(String),
    #[error("ProcTail service error: {0}")]
    ServiceError(String),
    #[error("Invalid response from ProcTail service: {0}")]
    InvalidResponse(String),
    #[error("Process not found: {0}")]
    ProcessNotFound(u32),
    #[error("Tag not found: {0}")]
    TagNotFound(String),
    #[error("Tag already exists: {0}")]
    TagAlreadyExists(String),
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("ProcTail service is not running")]
    ServiceNotRunning,
    #[error("Timeout while communicating with ProcTail service")]
    Timeout,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Base event data types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "$type")]
pub enum ProcTailEvent {
    #[serde(rename = "FileEvent")]
    File(FileEventData),
    #[serde(rename = "ProcessStart")]
    ProcessStart(ProcessStartEventData),
    #[serde(rename = "ProcessEnd")]
    ProcessEnd(ProcessEndEventData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileEventData {
    pub timestamp: String,
    pub process_id: u32,
    pub event_type: String,
    pub file_path: String,
    pub operation: String,
    pub file_size: i64,
    #[serde(default)]
    pub file_attributes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProcessStartEventData {
    pub timestamp: String,
    pub process_id: u32,
    pub event_type: String,
    pub process_name: String,
    pub parent_process_id: u32,
    pub command_line: String,
    #[serde(default)]
    pub executable_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProcessEndEventData {
    pub timestamp: String,
    pub process_id: u32,
    pub event_type: String,
    pub process_name: String,
    pub exit_code: i32,
    pub execution_time: String, // Duration as string
}

// Watch target information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WatchTarget {
    pub tag: String,
    pub process_id: u32,
    pub process_name: String,
    pub start_time: String,
    pub is_running: bool,
}

// Service status structures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceStatus {
    pub service: ServiceInfo,
    pub monitoring: MonitoringInfo,
    pub resources: ResourceInfo,
    pub ipc: IpcInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ServiceInfo {
    pub status: String,
    pub version: String,
    pub start_time: String,
    pub uptime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MonitoringInfo {
    pub etw_session_active: bool,
    pub active_tags: u32,
    pub active_processes: u32,
    pub total_events: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceInfo {
    #[serde(rename = "MemoryUsageMB")]
    pub memory_usage_mb: f64,
    #[serde(rename = "CpuUsagePercent")]
    pub cpu_usage_percent: f64,
    pub estimated_memory_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IpcInfo {
    pub named_pipe_active: bool,
    pub connected_clients: u32,
    pub total_requests: u64,
}

// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HealthCheckResult {
    pub status: String,
    pub check_time: String,
    pub details: HashMap<String, String>,
}

// Main ProcTail trait
#[cfg_attr(test, mockall::automock)]
pub trait ProcTail: Send + Sync {
    /// Add a process to monitoring targets
    async fn add_watch_target(
        &self,
        process_id: u32,
        tag: &str,
    ) -> Result<WatchTarget, ProcTailError>;

    /// Remove a watch target by tag
    async fn remove_watch_target(&self, tag: &str) -> Result<u32, ProcTailError>;

    /// Get all current watch targets
    async fn get_watch_targets(&self) -> Result<Vec<WatchTarget>, ProcTailError>;

    /// Get recorded events for a specific tag
    async fn get_recorded_events<'a>(
        &self,
        tag: &str,
        count: Option<u32>,
        event_type: Option<&'a str>,
    ) -> Result<Vec<ProcTailEvent>, ProcTailError>;

    /// Clear events for a specific tag
    async fn clear_events(&self, tag: &str) -> Result<u32, ProcTailError>;

    /// Get service status
    async fn get_status(&self) -> Result<ServiceStatus, ProcTailError>;

    /// Perform health check
    async fn health_check(&self) -> Result<HealthCheckResult, ProcTailError>;

    /// Check if ProcTail service is available
    async fn is_service_available(&self) -> bool;
}
