use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub allowed_extension_ids: Vec<String>,
    pub rate_limit_per_minute: u32,
    pub max_message_size: usize,
    pub require_signature: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_extension_ids: vec![
                // Chrome拡張機能のIDを設定（実際の拡張機能開発時に更新）
                "your-extension-id-here".to_string(),
            ],
            rate_limit_per_minute: 100,
            max_message_size: 1024 * 1024, // 1MB
            require_signature: false, // 開発時はfalse、本番ではtrueに
        }
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    limit_per_minute: u32,
}

impl RateLimiter {
    pub fn new(limit_per_minute: u32) -> Self {
        Self {
            requests: HashMap::new(),
            limit_per_minute,
        }
    }

    pub fn check_rate_limit(&mut self, client_id: &str) -> bool {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);
        
        let requests = self.requests.entry(client_id.to_string()).or_insert_with(Vec::new);
        
        // 1分以内のリクエストのみを保持
        requests.retain(|&timestamp| timestamp > minute_ago);
        
        if requests.len() >= self.limit_per_minute as usize {
            false
        } else {
            requests.push(now);
            true
        }
    }
}

#[derive(Debug)]
pub struct SecurityValidator {
    config: SecurityConfig,
    rate_limiter: RateLimiter,
}

impl SecurityValidator {
    pub fn new(config: SecurityConfig) -> Self {
        let rate_limiter = RateLimiter::new(config.rate_limit_per_minute);
        Self {
            config,
            rate_limiter,
        }
    }

    pub fn validate_extension_id(&self, extension_id: &str) -> Result<()> {
        if self.config.allowed_extension_ids.contains(&extension_id.to_string()) {
            Ok(())
        } else {
            Err(anyhow!("Unauthorized extension ID: {}", extension_id))
        }
    }

    pub fn validate_message_size(&self, message_size: usize) -> Result<()> {
        if message_size <= self.config.max_message_size {
            Ok(())
        } else {
            Err(anyhow!(
                "Message size {} exceeds limit {}",
                message_size,
                self.config.max_message_size
            ))
        }
    }

    pub fn check_rate_limit(&mut self, client_id: &str) -> Result<()> {
        if self.rate_limiter.check_rate_limit(client_id) {
            Ok(())
        } else {
            Err(anyhow!("Rate limit exceeded for client: {}", client_id))
        }
    }

    pub fn validate_request(&mut self, extension_id: &str, message_size: usize) -> Result<()> {
        self.validate_extension_id(extension_id)?;
        self.validate_message_size(message_size)?;
        self.check_rate_limit(extension_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);
        
        assert!(limiter.check_rate_limit("client1"));
        assert!(limiter.check_rate_limit("client1"));
        assert!(!limiter.check_rate_limit("client1")); // 制限に達する
        
        // 異なるクライアントは独立
        assert!(limiter.check_rate_limit("client2"));
    }

    #[test]
    fn test_security_validator() {
        let config = SecurityConfig {
            allowed_extension_ids: vec!["test-extension".to_string()],
            rate_limit_per_minute: 1,
            max_message_size: 100,
            require_signature: false,
        };
        
        let mut validator = SecurityValidator::new(config);
        
        // 正常なケース
        assert!(validator.validate_request("test-extension", 50).is_ok());
        
        // レート制限
        assert!(validator.validate_request("test-extension", 50).is_err());
        
        // 不正な拡張機能ID
        assert!(validator.validate_request("invalid-extension", 50).is_err());
        
        // メッセージサイズ超過
        let mut validator2 = SecurityValidator::new(SecurityConfig::default());
        assert!(validator2.validate_request("your-extension-id-here", 2000000).is_err());
    }
}