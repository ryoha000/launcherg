use std::io::{self, BufReader, Write, Read};
use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Result};
use serde_json;
use chrono::Utc;

use crate::domain::{collection::DLStoreType, Id};
// use crate::usecase::collection::CollectionUseCase;
// TODO: 実際の実装時に有効化
// use crate::infrastructure::repositoryimpl::{
//     repository::RepositoryImpl, 
//     collection::CollectionRepository
// };

use super::protocol::{
    NativeMessage, NativeResponse, MessageType, SyncGamesRequest, 
    ExtractedGameData, SyncBatchResult, SyncStatus, ExtensionConfig
};
use super::security::{SecurityValidator, SecurityConfig};

// 暫定的な実装 - 後で完全実装する予定
#[derive(Debug)]
pub struct NativeMessagingHost {
    security_validator: Arc<Mutex<SecurityValidator>>,
    // TODO: 実際のCollectionUseCaseを使用
    // collection_usecase: CollectionUseCase<RepositoryImpl<crate::domain::collection::Collection>>,
    config: ExtensionConfig,
}

impl NativeMessagingHost {
    pub async fn new() -> Result<Self> {
        let security_config = SecurityConfig::default();
        let _security_validator = Arc::new(Mutex::new(SecurityValidator::new(security_config)));
        
        // 簡略化されたセットアップ（実際のデータベース接続は後で実装）
        // TODO: 実際のデータベース接続を実装
        return Err(anyhow!("Native Messaging Host is not fully implemented yet"));
    }

    pub async fn run(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdin_reader = BufReader::new(stdin.lock());
        
        loop {
            // Native Messaging Protocolでは、最初の4バイトがメッセージ長
            let mut length_bytes = [0u8; 4];
            if stdin_reader.read_exact(&mut length_bytes).is_err() {
                break; // stdin closed
            }
            
            let length = u32::from_le_bytes(length_bytes) as usize;
            
            // セキュリティ: メッセージサイズを検証
            if let Err(e) = self.security_validator.lock().unwrap().validate_message_size(length) {
                self.send_error_response("", &format!("Message size validation failed: {}", e))?;
                continue;
            }
            
            // メッセージ本体を読み取り
            let mut message_bytes = vec![0u8; length];
            if stdin_reader.read_exact(&mut message_bytes).is_err() {
                break; // stdin closed or read error
            }
            
            let message_str = String::from_utf8(message_bytes)
                .map_err(|e| anyhow!("Invalid UTF-8: {}", e))?;
            
            // メッセージを処理
            match self.handle_message(&message_str).await {
                Ok(response) => {
                    if let Err(e) = self.send_response(&response) {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to handle message: {}", e);
                    if let Err(send_err) = self.send_error_response("", &e.to_string()) {
                        eprintln!("Failed to send error response: {}", send_err);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn handle_message(&mut self, message_str: &str) -> Result<NativeResponse> {
        let message: NativeMessage = serde_json::from_str(message_str)
            .map_err(|e| anyhow!("Failed to parse message: {}", e))?;

        match message.type_ {
            MessageType::SyncGames => self.handle_sync_games(&message).await,
            MessageType::GetStatus => self.handle_get_status(&message).await,
            MessageType::SetConfig => self.handle_set_config(&message).await,
            MessageType::HealthCheck => self.handle_health_check(&message).await,
        }
    }

    async fn handle_sync_games(&mut self, message: &NativeMessage) -> Result<NativeResponse> {
        let request: SyncGamesRequest = serde_json::from_value(message.payload.clone())
            .map_err(|e| anyhow!("Failed to parse sync games request: {}", e))?;

        // セキュリティ検証
        self.security_validator
            .lock()
            .unwrap()
            .validate_request(&request.extension_id, message.payload.to_string().len())?;

        let store_type = match request.store.as_str() {
            "DMM" => DLStoreType::DMM,
            "DLSite" => DLStoreType::DLSite,
            _ => return Err(anyhow!("Invalid store type: {}", request.store)),
        };

        let mut result = SyncBatchResult {
            success_count: 0,
            error_count: 0,
            errors: Vec::new(),
            synced_games: Vec::new(),
        };

        for game_data in request.games {
            match self.sync_single_game(&game_data, &store_type).await {
                Ok(_) => {
                    result.success_count += 1;
                    result.synced_games.push(game_data.title.clone());
                }
                Err(e) => {
                    result.error_count += 1;
                    result.errors.push(format!("Failed to sync {}: {}", game_data.title, e));
                }
            }
        }

        let response_data = serde_json::to_value(result)?;
        Ok(NativeResponse::success(response_data, message.request_id.clone()))
    }

    async fn sync_single_game(&self, game_data: &ExtractedGameData, _store_type: &DLStoreType) -> Result<Id<crate::domain::collection::CollectionElement>> {
        // TODO: 実際の同期処理を実装
        Err(anyhow!("Sync not yet implemented for game: {}", game_data.title))
    }

    async fn handle_get_status(&self, message: &NativeMessage) -> Result<NativeResponse> {
        let status = SyncStatus {
            last_sync: Some(Utc::now()), // 実際には最後の同期時刻を記録
            total_synced: 0, // 実際には同期したゲーム数を記録
            connected_extensions: vec![],
            is_running: true,
            connection_status: crate::native_messaging::protocol::ExtensionConnectionStatus::Connected,
            error_message: None,
        };

        let response_data = serde_json::to_value(status)?;
        Ok(NativeResponse::success(response_data, message.request_id.clone()))
    }

    async fn handle_set_config(&mut self, message: &NativeMessage) -> Result<NativeResponse> {
        let new_config: ExtensionConfig = serde_json::from_value(message.payload.clone())
            .map_err(|e| anyhow!("Failed to parse config: {}", e))?;

        self.config = new_config;
        
        let response_data = serde_json::to_value("Config updated successfully")?;
        Ok(NativeResponse::success(response_data, message.request_id.clone()))
    }

    async fn handle_health_check(&self, message: &NativeMessage) -> Result<NativeResponse> {
        let response_data = serde_json::to_value("OK")?;
        Ok(NativeResponse::success(response_data, message.request_id.clone()))
    }

    fn send_response(&self, response: &NativeResponse) -> Result<()> {
        let response_str = serde_json::to_string(response)?;
        let response_bytes = response_str.as_bytes();
        let length = response_bytes.len() as u32;
        
        let mut stdout = io::stdout();
        stdout.write_all(&length.to_le_bytes())?;
        stdout.write_all(response_bytes)?;
        stdout.flush()?;
        
        Ok(())
    }

    fn send_error_response(&self, request_id: &str, error: &str) -> Result<()> {
        let response = NativeResponse::error(error.to_string(), request_id.to_string());
        self.send_response(&response)
    }
}