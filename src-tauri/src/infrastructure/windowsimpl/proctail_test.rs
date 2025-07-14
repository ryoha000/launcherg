#[cfg(test)]
mod tests {
    use super::super::proctail::ProcTailImpl;
    use crate::domain::windows::proctail::ProcTail;

    #[tokio::test]
    #[ignore]
    async fn test_health_check_with_default_pipe() {
        let proctail = ProcTailImpl::new();
        
        let result = proctail.health_check().await;
        
        assert!(result.is_ok(), "Health check failed: {:?}", result.err());
        let health_check_result = result.unwrap();
        assert_eq!(health_check_result.status, "Healthy");
        assert!(!health_check_result.check_time.is_empty());
    }
}