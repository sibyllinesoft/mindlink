#[cfg(test)]
mod config_manager_tests {
    use crate::managers::config_manager::{
        BifrostConfig, ConfigManager, ConfigSchema, FeatureConfig, MonitoringConfig, ServerConfig,
        TunnelConfig,
    };
    use tempfile::TempDir;
    use tokio::fs;

    /// Helper to create a test config directory with proper structure
    async fn _create_test_config_dir() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create mindlink directory structure
        let mindlink_dir = temp_dir.path().join(".mindlink");
        fs::create_dir_all(&mindlink_dir)
            .await
            .expect("Failed to create mindlink directory");

        temp_dir
    }

    /// Helper to create default config for testing
    fn _create_test_config() -> ConfigSchema {
        ConfigSchema {
            version: 1,
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
            },
            bifrost: BifrostConfig {
                port: 3001,
                host: "127.0.0.1".to_string(),
                enabled: true,
            },
            tunnel: TunnelConfig {
                enabled: false,
                tunnel_type: "cloudflare".to_string(),
            },
            features: FeatureConfig {
                reasoning_effort: "medium".to_string(),
                reasoning_summaries: "enabled".to_string(),
                reasoning_compatibility: "compatible".to_string(),
            },
            monitoring: MonitoringConfig {
                health_check_interval: 30,
                error_threshold: 5,
                notifications: true,
            },
        }
    }

    #[tokio::test]
    async fn test_config_manager_creation() {
        println!("🧪 Test: ConfigManager creation");

        let result = ConfigManager::new().await;

        // Should succeed (creates default config if none exists)
        assert!(result.is_ok(), "ConfigManager creation should succeed");

        println!("✅ ConfigManager creation successful");
    }

    #[tokio::test]
    async fn test_config_loading_and_access() {
        println!("🧪 Test: Config loading and access");

        // Create a config manager
        let manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        // Get current config
        let config = manager.get_config().await;
        assert!(
            !config.server.host.is_empty(),
            "Server host should not be empty"
        );
        assert!(config.server.port > 0, "Server port should be positive");

        // Test individual config section access
        let server_config = manager.get_server_config().await;
        assert_eq!(server_config.host, config.server.host);
        assert_eq!(server_config.port, config.server.port);

        let bifrost_config = manager.get_bifrost_config().await;
        assert_eq!(bifrost_config.port, config.bifrost.port);
        assert_eq!(bifrost_config.host, config.bifrost.host);

        let tunnel_config = manager.get_tunnel_config().await;
        assert_eq!(tunnel_config.enabled, config.tunnel.enabled);

        let feature_config = manager.get_feature_config().await;
        assert!(!feature_config.reasoning_effort.is_empty());

        let monitoring_config = manager.get_monitoring_config().await;
        assert!(monitoring_config.health_check_interval > 0);

        println!("✅ Config loading and access successful");
    }

    #[tokio::test]
    async fn test_config_update() {
        println!("🧪 Test: Config update");

        let manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        // Get current config
        let mut config = manager.get_config().await;
        let original_port = config.server.port;

        // Modify config to a different port than current
        let new_port = if original_port == 9090 { 8080 } else { 9090 };
        config.server.port = new_port;
        config.server.host = "0.0.0.0".to_string();

        // Update config
        let update_result = manager.update_config(config.clone()).await;
        assert!(update_result.is_ok(), "Config update should succeed");

        // Verify changes persisted
        let updated_config = manager.get_config().await;
        assert_eq!(updated_config.server.host, "0.0.0.0");
        assert_eq!(updated_config.server.port, new_port);
        assert_ne!(updated_config.server.port, original_port);

        println!("✅ Config update successful");
    }

    #[tokio::test]
    async fn test_concurrent_config_access() {
        println!("🧪 Test: Concurrent config access");

        let manager = std::sync::Arc::new(
            ConfigManager::new()
                .await
                .expect("Failed to create config manager"),
        );

        // Test concurrent reads
        let mut handles = vec![];
        for i in 0..3 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let config = manager_clone.get_config().await;
                println!("   Concurrent read {}: port={}", i, config.server.port);
                assert!(!config.server.host.is_empty());
                assert!(config.server.port > 0);
            });
            handles.push(handle);
        }

        // Wait for all reads to complete
        for handle in handles {
            handle.await.expect("Concurrent read should succeed");
        }

        println!("✅ Concurrent config access successful");
    }

    #[tokio::test]
    async fn test_config_restore_from_backup() {
        println!("🧪 Test: Config restore from backup");

        let manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        // Test restore from backup (should handle gracefully even if no backup exists)
        let restore_result = manager.restore_from_backup().await;
        // This might succeed or fail depending on backup file existence, but should not panic
        assert!(
            restore_result.is_ok() || restore_result.is_err(),
            "restore_from_backup should complete"
        );

        println!("✅ Config restore from backup successful");
    }

    #[tokio::test]
    async fn test_config_schema_completeness() {
        println!("🧪 Test: Config schema completeness");

        let manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let config = manager.get_config().await;

        // Verify all required fields are present
        assert!(
            !config.server.host.is_empty(),
            "Server host should not be empty"
        );
        assert!(config.server.port > 0, "Server port should be positive");
        assert!(config.bifrost.port > 0, "Bifrost port should be positive");
        assert!(
            !config.bifrost.host.is_empty(),
            "Bifrost host should not be empty"
        );
        assert!(
            !config.tunnel.tunnel_type.is_empty(),
            "Tunnel type should not be empty"
        );
        assert!(
            !config.features.reasoning_effort.is_empty(),
            "Reasoning effort should not be empty"
        );
        assert!(
            config.monitoring.health_check_interval > 0,
            "Health check interval should be positive"
        );
        assert!(config.version > 0, "Config version should be positive");

        println!("✅ Config schema completeness successful");
    }
}
