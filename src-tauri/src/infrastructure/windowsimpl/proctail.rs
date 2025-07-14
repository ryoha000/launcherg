use crate::domain::windows::proctail::{
    HealthCheckResult, IpcInfo, MonitoringInfo, ProcTail, ProcTailError, ProcTailEvent,
    ResourceInfo, ServiceInfo, ServiceStatus, WatchTarget,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\ProcTailIPC";
const DEFAULT_TIMEOUT_MS: u64 = 5000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ProcTailRequest {
    request_type: String,
    parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ProcTailResponse<T> {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    timestamp: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AddWatchTargetData {
    process_id: u32,
    tag: String,
    process_name: String,
    start_time: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RemoveWatchTargetData {
    tag: String,
    removed_process_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetWatchTargetsData {
    targets: Vec<WatchTarget>,
    total_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetRecordedEventsData {
    tag: String,
    events: Vec<ProcTailEvent>,
    total_count: u32,
    has_more: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ClearEventsData {
    tag: String,
    cleared_event_count: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetStatusData {
    service: ServiceInfo,
    monitoring: MonitoringInfo,
    resources: ResourceInfo,
    ipc: IpcInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct HealthCheckData {
    status: String,
    check_time: String,
    details: HashMap<String, String>,
}

pub struct ProcTailImpl {
    pipe_name: String,
    timeout_ms: u64,
    connection_cache: Arc<Mutex<Option<NamedPipeClient>>>,
}

impl ProcTailImpl {
    pub fn new() -> Self {
        Self {
            pipe_name: DEFAULT_PIPE_NAME.to_string(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
            connection_cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_config(pipe_name: String, timeout_ms: u64) -> Self {
        Self {
            pipe_name,
            timeout_ms,
            connection_cache: Arc::new(Mutex::new(None)),
        }
    }

    async fn send_request<T>(&self, request_type: &str, parameters: serde_json::Value) -> Result<T, ProcTailError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let request = ProcTailRequest {
            request_type: request_type.to_string(),
            parameters,
            request_id: Some(uuid::Uuid::new_v4().to_string()),
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;

        let response: ProcTailResponse<T> = serde_json::from_str(&response_json)
            .map_err(|_| ProcTailError::InvalidResponse)?;

        if !response.success {
            return Err(self.map_error_code(response.error_code.as_deref(), &response.message));
        }

        response.data.ok_or(ProcTailError::InvalidResponse)
    }

    async fn send_raw_request(&self, request: &str) -> Result<String, ProcTailError> {
        let mut retries = 2;
        
        while retries > 0 {
            match self.try_send_request(request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if retries == 1 {
                        return Err(e);
                    }
                    // Clear cached connection on error
                    let mut cache = self.connection_cache.lock().await;
                    *cache = None;
                    retries -= 1;
                }
            }
        }

        Err(ProcTailError::ServiceError("Failed after retries".to_string()))
    }

    async fn try_send_request(&self, request: &str) -> Result<String, ProcTailError> {
        let mut cache = self.connection_cache.lock().await;
        
        // Try to use cached connection first
        if let Some(mut client) = cache.take() {
            match self.send_on_pipe(&mut client, request).await {
                Ok(response) => {
                    // Put the connection back in cache
                    *cache = Some(client);
                    return Ok(response);
                }
                Err(_) => {
                    // Connection failed, will create new one
                }
            }
        }

        // Create new connection
        let mut client = self.connect_to_pipe().await?;
        let response = self.send_on_pipe(&mut client, request).await?;
        
        // Cache the connection for future use
        *cache = Some(client);
        
        Ok(response)
    }

    async fn connect_to_pipe(&self) -> Result<NamedPipeClient, ProcTailError> {
        let pipe_name = self.pipe_name.clone();
        let client = timeout(
            Duration::from_millis(self.timeout_ms),
            async move {
                ClientOptions::new().open(&pipe_name)
            },
        )
        .await
        .map_err(|_| ProcTailError::Timeout)?
        .map_err(|e| ProcTailError::ConnectionFailed(format!("Failed to connect to pipe: {}", e)))?;

        Ok(client)
    }

    async fn send_on_pipe(&self, client: &mut NamedPipeClient, request: &str) -> Result<String, ProcTailError> {
        // Send request
        let request_bytes = request.as_bytes();
        timeout(
            Duration::from_millis(self.timeout_ms),
            client.write_all(request_bytes),
        )
        .await
        .map_err(|_| ProcTailError::Timeout)?
        .map_err(|e| ProcTailError::IoError(e))?;

        // Read response
        let mut response_buffer = vec![0u8; 65536];
        let bytes_read = timeout(
            Duration::from_millis(self.timeout_ms),
            client.read(&mut response_buffer),
        )
        .await
        .map_err(|_| ProcTailError::Timeout)?
        .map_err(|e| ProcTailError::IoError(e))?;

        if bytes_read == 0 {
            return Err(ProcTailError::ServiceError("Empty response from service".to_string()));
        }

        response_buffer.truncate(bytes_read);
        String::from_utf8(response_buffer)
            .map_err(|e| ProcTailError::ServiceError(format!("Invalid UTF-8 in response: {}", e)))
    }

    fn map_error_code(&self, error_code: Option<&str>, message: &str) -> ProcTailError {
        match error_code {
            Some("PROCESS_NOT_FOUND") => {
                if let Some(pid) = message.split_whitespace().last().and_then(|s| s.parse::<u32>().ok()) {
                    ProcTailError::ProcessNotFound(pid)
                } else {
                    ProcTailError::ServiceError(message.to_string())
                }
            }
            Some("TAG_NOT_FOUND") => ProcTailError::TagNotFound(message.split('"').nth(1).unwrap_or("").to_string()),
            Some("TAG_ALREADY_EXISTS") => ProcTailError::TagAlreadyExists(message.split('"').nth(1).unwrap_or("").to_string()),
            Some("INSUFFICIENT_PERMISSIONS") => ProcTailError::InsufficientPermissions,
            Some("SERVICE_NOT_RUNNING") => ProcTailError::ServiceNotRunning,
            _ => ProcTailError::ServiceError(message.to_string()),
        }
    }
}

#[async_trait]
impl ProcTail for ProcTailImpl {
    async fn add_watch_target(&self, process_id: u32, tag: &str) -> Result<WatchTarget, ProcTailError> {
        let parameters = serde_json::json!({
            "ProcessId": process_id,
            "Tag": tag
        });

        let data: AddWatchTargetData = self.send_request("AddWatchTarget", parameters).await?;
        
        Ok(WatchTarget {
            tag: data.tag,
            process_id: data.process_id,
            process_name: data.process_name,
            start_time: data.start_time,
            is_running: true,
        })
    }

    async fn remove_watch_target(&self, tag: &str) -> Result<u32, ProcTailError> {
        let parameters = serde_json::json!({
            "Tag": tag
        });

        let data: RemoveWatchTargetData = self.send_request("RemoveWatchTarget", parameters).await?;
        Ok(data.removed_process_count)
    }

    async fn get_watch_targets(&self) -> Result<Vec<WatchTarget>, ProcTailError> {
        let parameters = serde_json::json!({});
        let data: GetWatchTargetsData = self.send_request("GetWatchTargets", parameters).await?;
        Ok(data.targets)
    }

    async fn get_recorded_events(
        &self,
        tag: &str,
        count: Option<u32>,
        event_type: Option<&str>,
    ) -> Result<Vec<ProcTailEvent>, ProcTailError> {
        let mut parameters = serde_json::json!({
            "Tag": tag
        });

        if let Some(count) = count {
            parameters["Count"] = serde_json::json!(count);
        }

        if let Some(event_type) = event_type {
            parameters["EventType"] = serde_json::json!(event_type);
        }

        let data: GetRecordedEventsData = self.send_request("GetRecordedEvents", parameters).await?;
        Ok(data.events)
    }

    async fn clear_events(&self, tag: &str) -> Result<u32, ProcTailError> {
        let parameters = serde_json::json!({
            "Tag": tag
        });

        let data: ClearEventsData = self.send_request("ClearEvents", parameters).await?;
        Ok(data.cleared_event_count)
    }

    async fn get_status(&self) -> Result<ServiceStatus, ProcTailError> {
        let parameters = serde_json::json!({});
        let data: GetStatusData = self.send_request("GetStatus", parameters).await?;
        
        Ok(ServiceStatus {
            service: data.service,
            monitoring: data.monitoring,
            resources: data.resources,
            ipc: data.ipc,
        })
    }

    async fn health_check(&self) -> Result<HealthCheckResult, ProcTailError> {
        let parameters = serde_json::json!({});
        let data: HealthCheckData = self.send_request("HealthCheck", parameters).await?;
        
        Ok(HealthCheckResult {
            status: data.status,
            check_time: data.check_time,
            details: data.details,
        })
    }

    async fn is_service_available(&self) -> bool {
        matches!(self.health_check().await, Ok(result) if result.status == "Healthy")
    }
}

impl Default for ProcTailImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proctail_creation() {
        let proctail = ProcTailImpl::new();
        assert_eq!(proctail.pipe_name, DEFAULT_PIPE_NAME);
        assert_eq!(proctail.timeout_ms, DEFAULT_TIMEOUT_MS);
    }

    #[tokio::test]
    async fn test_proctail_with_config() {
        let custom_pipe = r"\\.\pipe\CustomProcTail";
        let custom_timeout = 10000;
        let proctail = ProcTailImpl::with_config(custom_pipe.to_string(), custom_timeout);
        assert_eq!(proctail.pipe_name, custom_pipe);
        assert_eq!(proctail.timeout_ms, custom_timeout);
    }
}