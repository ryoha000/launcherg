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

const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\ProcTail";
const ALTERNATIVE_PIPE_NAME: &str = r"\\.\pipe\proctail";
const DEFAULT_TIMEOUT_MS: u64 = 5000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct AddWatchTargetRequest {
    request_type: String,
    process_id: u32,
    tag_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct RemoveWatchTargetRequest {
    request_type: String,
    tag_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetWatchTargetsRequest {
    request_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetRecordedEventsRequest {
    request_type: String,
    tag_name: String,
    max_count: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ClearEventsRequest {
    request_type: String,
    tag_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetStatusRequest {
    request_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct HealthCheckRequest {
    request_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ShutdownRequest {
    request_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ServiceStatusResponse {
    success: bool,
    is_running: bool,
    is_etw_monitoring: bool,
    is_pipe_server_running: bool,
    active_watch_targets: i32,
    total_tags: i32,
    total_events: i32,
    #[serde(rename = "EstimatedMemoryUsageMB")]
    estimated_memory_usage_mb: i64,
    message: String,
    #[serde(default)]
    error_message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetWatchTargetsResponse {
    watch_targets: Vec<WatchTargetData>,
    success: bool,
    error_message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct WatchTargetData {
    process_id: u32,
    process_name: String,
    executable_path: String,
    start_time: String,
    tag_name: String,
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

    pub async fn detect_pipe_name(&self) -> Option<String> {
        let candidate_pipes = vec![
            DEFAULT_PIPE_NAME.to_string(),
            ALTERNATIVE_PIPE_NAME.to_string(),
        ];
        
        for pipe_name in candidate_pipes {
            if self.test_pipe_exists(&pipe_name).await {
                return Some(pipe_name);
            }
        }
        
        None
    }

    async fn test_pipe_exists(&self, pipe_name: &str) -> bool {
        match timeout(
            Duration::from_millis(1000),
            async {
                ClientOptions::new().open(pipe_name)
            },
        )
        .await
        {
            Ok(Ok(_)) => true,
            Ok(Err(_)) => false,
            Err(_) => false,
        }
    }

    pub fn with_config(pipe_name: String, timeout_ms: u64) -> Self {
        Self {
            pipe_name,
            timeout_ms,
            connection_cache: Arc::new(Mutex::new(None)),
        }
    }

    async fn send_request<T, R>(&self, request: R) -> Result<T, ProcTailError>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize,
    {
        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;

        let response: ProcTailResponse<T> = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;

        if !response.success {
            return Err(self.map_error_code(response.error_code.as_deref(), &response.message));
        }

        response.data.ok_or_else(|| ProcTailError::InvalidResponse(format!("Response missing data field. Full response: {}", response_json)))
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
            println!("Named Pipe Info (existing client): {:?}", client.info());
            match self.send_on_pipe(&mut client, request).await {
                Ok(response) => {
                    // Put the connection back in cache
                    *cache = Some(client);
                    return Ok(response);
                }
                Err(err) => {
                    println!("failed to send: {:?}", err);
                    // Connection failed, will create new one
                }
            }
        }

        // Create new connection
        let mut client = self.connect_to_pipe().await?;
        println!("Named Pipe Info (new connection): {:?}", client.info());
        let response = self.send_on_pipe(&mut client, request).await?;
        
        // Cache the connection for future use
        *cache = Some(client);
        
        Ok(response)
    }

    async fn connect_to_pipe(&self) -> Result<NamedPipeClient, ProcTailError> {
        let pipe_name = self.pipe_name.clone();
        let pipe_name_for_error = pipe_name.clone();
        
        // Retry connection with exponential backoff for OS error 233
        let max_retries = 3;
        let mut retry_count = 0;
        
        while retry_count < max_retries {
            let result = timeout(
                Duration::from_millis(self.timeout_ms),
                async {
                    ClientOptions::new().open(&pipe_name)
                },
            )
            .await;
            
            match result {
                Ok(Ok(client)) => return Ok(client),
                Ok(Err(e)) => {
                    // Check for OS error 233 (ERROR_PIPE_NOT_CONNECTED)
                    if e.raw_os_error() == Some(233) {
                        retry_count += 1;
                        if retry_count < max_retries {
                            // Wait before retry with exponential backoff
                            let delay = Duration::from_millis(100 * (1 << retry_count));
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    
                    return Err(ProcTailError::ConnectionFailed(format!(
                        "Pipe {} connection failed. Please ensure ProcTail service is fully initialized. Error: {}", 
                        pipe_name_for_error, e
                    )));
                }
                Err(_) => {
                    return Err(ProcTailError::Timeout);
                }
            }
        }
        
        Err(ProcTailError::ConnectionFailed(format!(
            "Failed to connect to pipe {} after {} retries. ProcTail service may not be responding.", 
            pipe_name_for_error, max_retries
        )))
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
        
        println!("Sent request: {} bytes", request_bytes.len());

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
            println!("Read {} bytes for message length", read);
            
            if read == 0 {
                return Err(ProcTailError::ServiceError("メッセージ長の受信が予期せず終了しました".to_string()));
            }
            bytes_read += read;
        }
        println!("Received message length: {:?}", length_buffer);

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

        println!("Received message of {} bytes", message_length);
        String::from_utf8(message_buffer)
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
        let request = AddWatchTargetRequest {
            request_type: "AddWatchTarget".to_string(),
            process_id,
            tag_name: tag.to_string(),
        };

        let data: AddWatchTargetData = self.send_request(request).await?;
        
        Ok(WatchTarget {
            tag: data.tag,
            process_id: data.process_id,
            process_name: data.process_name,
            start_time: data.start_time,
            is_running: true,
        })
    }

    async fn remove_watch_target(&self, tag: &str) -> Result<u32, ProcTailError> {
        let request = RemoveWatchTargetRequest {
            request_type: "RemoveWatchTarget".to_string(),
            tag_name: tag.to_string(),
        };

        let data: RemoveWatchTargetData = self.send_request(request).await?;
        Ok(data.removed_process_count)
    }

    async fn get_watch_targets(&self) -> Result<Vec<WatchTarget>, ProcTailError> {
        let request = GetWatchTargetsRequest {
            request_type: "GetWatchTargets".to_string(),
        };

        // GetWatchTargetsは直接レスポンスを返す
        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let response: GetWatchTargetsResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !response.success {
            return Err(ProcTailError::ServiceError(response.error_message));
        }
        
        // WatchTargetDataをWatchTargetに変換
        let targets = response.watch_targets.into_iter().map(|data| WatchTarget {
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
            tag_name: tag.to_string(),
            max_count: count.unwrap_or(100) as i32,
        };

        let data: GetRecordedEventsData = self.send_request(request).await?;
        Ok(data.events)
    }

    async fn clear_events(&self, tag: &str) -> Result<u32, ProcTailError> {
        let request = ClearEventsRequest {
            request_type: "ClearEvents".to_string(),
            tag_name: tag.to_string(),
        };

        let data: ClearEventsData = self.send_request(request).await?;
        Ok(data.cleared_event_count)
    }

    async fn get_status(&self) -> Result<ServiceStatus, ProcTailError> {
        let request = GetStatusRequest {
            request_type: "GetStatus".to_string(),
        };

        // GetStatusはServiceStatusResponseを直接返す
        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await?;
        
        let status_response: ServiceStatusResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        if !status_response.success {
            return Err(ProcTailError::ServiceError(status_response.error_message));
        }
        
        // ServiceStatusResponseから必要なフィールドを抽出してServiceStatusに変換
        Ok(ServiceStatus {
            service: ServiceInfo {
                status: if status_response.is_running { "Running".to_string() } else { "Stopped".to_string() },
                version: "Unknown".to_string(),
                start_time: "Unknown".to_string(),
                uptime: "Unknown".to_string(),
            },
            monitoring: MonitoringInfo {
                etw_session_active: status_response.is_etw_monitoring,
                active_tags: status_response.active_watch_targets as u32,
                active_processes: status_response.active_watch_targets as u32,
                total_events: status_response.total_events as u64,
            },
            resources: ResourceInfo {
                memory_usage_mb: status_response.estimated_memory_usage_mb as f64,
                cpu_usage_percent: 0.0,
                estimated_memory_usage: status_response.estimated_memory_usage_mb as u64,
            },
            ipc: IpcInfo {
                named_pipe_active: status_response.is_pipe_server_running,
                connected_clients: 0,
                total_requests: 0,
            },
        })
    }

    async fn health_check(&self) -> Result<HealthCheckResult, ProcTailError> {
        // Try with current pipe name first
        let request = HealthCheckRequest {
            request_type: "GetStatus".to_string(), // Use GetStatus instead of HealthCheck
        };
        
        let request_json = serde_json::to_string(&request)
            .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;

        let response_json = self.send_raw_request(&request_json).await;
        
        // If failed, try to detect alternative pipe name
        if let Err(ref err) = response_json {
            // Check if it's OS error 233 - service may be starting up
            if let ProcTailError::IoError(io_err) = err {
                if io_err.raw_os_error() == Some(233) {
                    // Wait a bit and retry once for service initialization
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    let retry_request = HealthCheckRequest {
                        request_type: "GetStatus".to_string(),
                    };
                    let retry_request_json = serde_json::to_string(&retry_request)
                        .map_err(|e| ProcTailError::ServiceError(format!("Failed to serialize request: {}", e)))?;
                    let retry_response_json = self.send_raw_request(&retry_request_json).await?;
                    
                    let status_response: ServiceStatusResponse = serde_json::from_str(&retry_response_json)
                        .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, retry_response_json)))?;
                    
                    return Ok(HealthCheckResult {
                        status: if status_response.is_running { "Healthy".to_string() } else { "Unhealthy".to_string() },
                        check_time: chrono::Utc::now().to_rfc3339(),
                        details: std::collections::HashMap::new(),
                    });
                }
            }
        }
        
        let response_json = response_json?;
        let status_response: ServiceStatusResponse = serde_json::from_str(&response_json)
            .map_err(|e| ProcTailError::InvalidResponse(format!("Failed to parse JSON response: {}. Original response: {}", e, response_json)))?;
        
        Ok(HealthCheckResult {
            status: if status_response.is_running { "Healthy".to_string() } else { "Unhealthy".to_string() },
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

    #[tokio::test]
    async fn test_proctail_with_config() {
        let custom_pipe = r"\\.\pipe\CustomProcTail";
        let custom_timeout = 10000;
        let proctail = ProcTailImpl::with_config(custom_pipe.to_string(), custom_timeout);
        assert_eq!(proctail.pipe_name, custom_pipe);
        assert_eq!(proctail.timeout_ms, custom_timeout);
    }
}