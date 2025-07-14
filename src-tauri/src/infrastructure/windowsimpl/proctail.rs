use crate::domain::windows::proctail::{
    HealthCheckResult, IpcInfo, MonitoringInfo, ProcTail, ProcTailError, ProcTailEvent,
    ResourceInfo, ServiceInfo, ServiceStatus, WatchTarget,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\ProcTail";
const DEFAULT_TIMEOUT_MS: u64 = 5000;

// Generic Request structure
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ProcTailRequest<T> {
    request_type: String,
    #[serde(flatten)]
    payload: T,
}

// Request payload types
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct AddWatchTargetPayload {
    process_id: u32,
    tag_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct RemoveWatchTargetPayload {
    tag_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetRecordedEventsPayload {
    tag_name: String,
    max_count: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ClearEventsPayload {
    tag_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct EmptyPayload {}

// Type aliases for specific requests
type AddWatchTargetRequest = ProcTailRequest<AddWatchTargetPayload>;
type RemoveWatchTargetRequest = ProcTailRequest<RemoveWatchTargetPayload>;
type GetWatchTargetsRequest = ProcTailRequest<EmptyPayload>;
type GetRecordedEventsRequest = ProcTailRequest<GetRecordedEventsPayload>;
type ClearEventsRequest = ProcTailRequest<ClearEventsPayload>;
type GetStatusRequest = ProcTailRequest<EmptyPayload>;
type HealthCheckRequest = ProcTailRequest<EmptyPayload>;

// Generic Response structure
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ProcTailResponse<T> {
    success: bool,
    #[serde(default)]
    error_message: Option<String>,
    #[serde(flatten)]
    data: T,
}

// Response data types
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ServiceStatusData {
    is_running: bool,
    is_etw_monitoring: bool,
    is_pipe_server_running: bool,
    active_watch_targets: i32,
    total_tags: i32,
    total_events: i32,
    #[serde(rename = "EstimatedMemoryUsageMB")]
    estimated_memory_usage_mb: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetWatchTargetsData {
    watch_targets: Vec<WatchTargetData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetRecordedEventsData {
    events: Vec<ProcTailEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EmptyData {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WatchTargetData {
    process_id: u32,
    process_name: String,
    executable_path: String,
    start_time: String,
    tag_name: String,
}

// Type aliases for specific responses
type ServiceStatusResponse = ProcTailResponse<ServiceStatusData>;
type GetWatchTargetsResponse = ProcTailResponse<GetWatchTargetsData>;
type GetRecordedEventsResponse = ProcTailResponse<GetRecordedEventsData>;
type BasicResponse = ProcTailResponse<EmptyData>;

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
        let pipe_name_for_error = pipe_name.clone();
        
        let result = timeout(
            Duration::from_millis(self.timeout_ms),
            async {
                ClientOptions::new().open(&pipe_name)
            },
        )
        .await;
        
        match result {
            Ok(Ok(client)) => Ok(client),
            Ok(Err(e)) => {
                Err(ProcTailError::ConnectionFailed(format!(
                    "Pipe {} connection failed. Please ensure ProcTail service is fully initialized. Error: {}", 
                    pipe_name_for_error, e
                )))
            }
            Err(_) => {
                Err(ProcTailError::Timeout)
            }
        }
    }

    async fn send_on_pipe(&self, client: &mut NamedPipeClient, request: &str) -> Result<String, ProcTailError> {
        // Send message length first (as in C# implementation)
        let request_bytes = request.as_bytes();
        let message_length = request_bytes.len() as i32;
        let length_bytes = message_length.to_le_bytes();
        
        // Send message length
        timeout(
            Duration::from_millis(self.timeout_ms),
            client.write_all(&length_bytes),
        )
        .await
        .map_err(|_| ProcTailError::Timeout)?
        .map_err(|e| ProcTailError::IoError(e))?;
        
        // Send message body
        timeout(
            Duration::from_millis(self.timeout_ms),
            client.write_all(request_bytes),
        )
        .await
        .map_err(|_| ProcTailError::Timeout)?
        .map_err(|e| ProcTailError::IoError(e))?;
        
        // Flush the stream
        timeout(
            Duration::from_millis(self.timeout_ms),
            client.flush(),
        )
        .await
        .map_err(|_| ProcTailError::Timeout)?
        .map_err(|e| ProcTailError::IoError(e))?;

        // Read message length (4 bytes)
        let mut length_buffer = [0u8; 4];
        let mut bytes_read = 0;
        
        while bytes_read < 4 {
            let read = timeout(
                Duration::from_millis(self.timeout_ms),
                client.read(&mut length_buffer[bytes_read..]),
            )
            .await
            .map_err(|_| ProcTailError::Timeout)?
            .map_err(|e| ProcTailError::IoError(e))?;
            
            if read == 0 {
                return Err(ProcTailError::ServiceError("メッセージ長の受信が予期せず終了しました".to_string()));
            }
            bytes_read += read;
        }

        let message_length = i32::from_le_bytes(length_buffer);
        
        if message_length <= 0 || message_length > 10 * 1024 * 1024 {
            return Err(ProcTailError::ServiceError(format!("無効なメッセージ長: {}", message_length)));
        }

        // Read message body
        let mut message_buffer = vec![0u8; message_length as usize];
        let mut bytes_read = 0;
        
        while bytes_read < message_length as usize {
            let read = timeout(
                Duration::from_millis(self.timeout_ms),
                client.read(&mut message_buffer[bytes_read..]),
            )
            .await
            .map_err(|_| ProcTailError::Timeout)?
            .map_err(|e| ProcTailError::IoError(e))?;
            
            if read == 0 {
                return Err(ProcTailError::ServiceError("メッセージ受信が予期せず終了しました".to_string()));
            }
            bytes_read += read;
        }

        String::from_utf8(message_buffer)
            .map_err(|e| ProcTailError::ServiceError(format!("Invalid UTF-8 in response: {}", e)))
    }
}

#[async_trait]
impl ProcTail for ProcTailImpl {
    async fn add_watch_target(&self, process_id: u32, tag: &str) -> Result<WatchTarget, ProcTailError> {
        let request = AddWatchTargetRequest {
            request_type: "AddWatchTarget".to_string(),
            payload: AddWatchTargetPayload {
                process_id,
                tag_name: tag.to_string(),
            },
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let basic_response: BasicResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !basic_response.success {
            return Err(ProcTailError::ServiceError(basic_response.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }
        
        // 成功時は現在のプロセス情報から推測してWatchTargetを作成
        let process_name = format!("process_{}", process_id);
        
        Ok(WatchTarget {
            tag: tag.to_string(),
            process_id,
            process_name,
            start_time: chrono::Utc::now().to_rfc3339(),
            is_running: true,
        })
    }

    async fn remove_watch_target(&self, tag: &str) -> Result<u32, ProcTailError> {
        let request = RemoveWatchTargetRequest {
            request_type: "RemoveWatchTarget".to_string(),
            payload: RemoveWatchTargetPayload {
                tag_name: tag.to_string(),
            },
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let basic_response: BasicResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !basic_response.success {
            return Err(ProcTailError::ServiceError(basic_response.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }
        
        Ok(1) // 成功時は1を返す
    }

    async fn get_watch_targets(&self) -> Result<Vec<WatchTarget>, ProcTailError> {
        let request = GetWatchTargetsRequest {
            request_type: "GetWatchTargets".to_string(),
            payload: EmptyPayload {},
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let response: GetWatchTargetsResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !response.success {
            return Err(ProcTailError::ServiceError(response.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }
        
        // WatchTargetDataをWatchTargetに変換
        let targets = response.data.watch_targets.into_iter().map(|data| WatchTarget {
            tag: data.tag_name,
            process_id: data.process_id,
            process_name: data.process_name,
            start_time: data.start_time,
            is_running: true, // 監視中なのでtrueと仮定
        }).collect();
        
        Ok(targets)
    }

    async fn get_recorded_events(
        &self,
        tag: &str,
        count: Option<u32>,
        _event_type: Option<&str>,
    ) -> Result<Vec<ProcTailEvent>, ProcTailError> {
        let request = GetRecordedEventsRequest {
            request_type: "GetRecordedEvents".to_string(),
            payload: GetRecordedEventsPayload {
                tag_name: tag.to_string(),
                max_count: count.unwrap_or(100) as i32,
            },
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let response: GetRecordedEventsResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !response.success {
            return Err(ProcTailError::ServiceError(response.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }
        
        Ok(response.data.events)
    }

    async fn clear_events(&self, tag: &str) -> Result<u32, ProcTailError> {
        let request = ClearEventsRequest {
            request_type: "ClearEvents".to_string(),
            payload: ClearEventsPayload {
                tag_name: tag.to_string(),
            },
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let basic_response: BasicResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !basic_response.success {
            return Err(ProcTailError::ServiceError(basic_response.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }
        
        Ok(0) // 成功時は0を返す（クリアされた数は不明）
    }

    async fn get_status(&self) -> Result<ServiceStatus, ProcTailError> {
        let request = GetStatusRequest {
            request_type: "GetStatus".to_string(),
            payload: EmptyPayload {},
        };

        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let status_response: ServiceStatusResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !status_response.success {
            return Err(ProcTailError::ServiceError(status_response.error_message.unwrap_or_else(|| "Unknown error".to_string())));
        }
        
        // ServiceStatusResponseから必要なフィールドを抽出してServiceStatusに変換
        Ok(ServiceStatus {
            service: ServiceInfo {
                status: if status_response.data.is_running { "Running".to_string() } else { "Stopped".to_string() },
                version: "Unknown".to_string(),
                start_time: "Unknown".to_string(),
                uptime: "Unknown".to_string(),
            },
            monitoring: MonitoringInfo {
                etw_session_active: status_response.data.is_etw_monitoring,
                active_tags: status_response.data.active_watch_targets as u32,
                active_processes: status_response.data.active_watch_targets as u32,
                total_events: status_response.data.total_events as u64,
            },
            resources: ResourceInfo {
                memory_usage_mb: status_response.data.estimated_memory_usage_mb as f64,
                cpu_usage_percent: 0.0,
                estimated_memory_usage: status_response.data.estimated_memory_usage_mb as u64,
            },
            ipc: IpcInfo {
                named_pipe_active: status_response.data.is_pipe_server_running,
                connected_clients: 0,
                total_requests: 0,
            },
        })
    }

    async fn health_check(&self) -> Result<HealthCheckResult, ProcTailError> {
        let request = HealthCheckRequest {
            request_type: "GetStatus".to_string(), // Use GetStatus instead of HealthCheck
            payload: EmptyPayload {},
        };
        
        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await;
        
        let response_json = response_json?;
        let status_response: ServiceStatusResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        Ok(HealthCheckResult {
            status: if status_response.data.is_running { "Healthy".to_string() } else { "Unhealthy".to_string() },
            check_time: chrono::Utc::now().to_rfc3339(),
            details: std::collections::HashMap::new(),
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
}