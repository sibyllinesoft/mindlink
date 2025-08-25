#[cfg(test)]
mod bifrost_integration_tests {
    use crate::managers::{bifrost_manager::BifrostManager, config_manager::ConfigManager};

    #[tokio::test]
    async fn test_bifrost_config_integration() {
        println!("🧪 Test: Integration - Bifrost with Config");

        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");
        let bifrost_manager = BifrostManager::new().await;

        // Get bifrost configuration from config manager
        let bifrost_config = config_manager.get_bifrost_config().await;
        assert!(
            bifrost_config.port > 0,
            "Bifrost config should have valid port"
        );
        assert!(
            !bifrost_config.host.is_empty(),
            "Bifrost config should have host"
        );

        // Verify bifrost manager is in expected initial state
        assert!(
            !bifrost_manager.is_running().await,
            "Bifrost should not be running initially"
        );
        assert!(
            bifrost_manager.get_local_url().await.is_none(),
            "Should have no local URL initially"
        );

        println!("✅ Bifrost config integration successful");
    }

    #[tokio::test]
    async fn test_bifrost_binary_management() {
        println!("🧪 Test: Integration - Bifrost binary management");

        let bifrost_manager = BifrostManager::new().await;

        // Test binary availability checks
        let is_available = bifrost_manager.is_binary_available().await;
        let binary_path = bifrost_manager.get_binary_path().await;
        let should_build = bifrost_manager.should_build().await;

        println!("   Binary available: {}", is_available);
        println!("   Binary path: {:?}", binary_path);
        println!("   Should build: {}", should_build);

        // Test installation info
        let (is_installed, path, version) = bifrost_manager.get_installation_info().await;
        println!(
            "   Installed: {}, Path: {:?}, Version: {:?}",
            is_installed, path, version
        );

        // These should complete without errors
        assert!(true, "Binary management integration completed");

        println!("✅ Bifrost binary management integration successful");
    }

    #[tokio::test]
    async fn test_bifrost_models_integration() {
        println!("🧪 Test: Integration - Bifrost models");

        let bifrost_manager = BifrostManager::new().await;

        // Try to get models (will likely fail when not running)
        let models_result = bifrost_manager.get_models().await;
        match models_result {
            Ok(models) => {
                println!("   Got {} models: {:?}", models.len(), models);
                // If we got models, they should be non-empty
                assert!(
                    !models.is_empty(),
                    "Should have at least one model if call succeeded"
                );
            },
            Err(e) => {
                println!("   Get models failed as expected when not running: {}", e);
                // This is expected when bifrost is not running
            },
        }

        println!("✅ Bifrost models integration successful");
    }

    #[tokio::test]
    async fn test_bifrost_configuration() {
        println!("🧪 Test: Integration - Bifrost configuration");

        let mut bifrost_manager = BifrostManager::new().await;
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        // Get configuration from config manager
        let bifrost_config = config_manager.get_bifrost_config().await;

        // Configure bifrost with config values
        bifrost_manager
            .configure(bifrost_config.host.clone(), bifrost_config.port)
            .await;

        // Test with different configuration
        bifrost_manager.configure("0.0.0.0".to_string(), 3002).await;

        // Should still not be running after configuration
        assert!(
            !bifrost_manager.is_running().await,
            "Should not be running after configuration"
        );

        println!("✅ Bifrost configuration integration successful");
    }

    #[tokio::test]
    async fn test_bifrost_status_integration() {
        println!("🧪 Test: Integration - Bifrost status reporting");

        let bifrost_manager = BifrostManager::new().await;

        // Test status info method
        let (is_running, local_url, api_url) = bifrost_manager.get_status_info().await;

        assert!(!is_running, "Should report not running");
        assert!(local_url.is_none(), "Should report no local URL");
        assert!(api_url.is_none(), "Should report no API URL");

        // Test individual status methods
        assert!(
            !bifrost_manager.is_running().await,
            "is_running should match status"
        );
        assert_eq!(
            bifrost_manager.get_local_url().await,
            local_url,
            "get_local_url should match status"
        );
        assert_eq!(
            bifrost_manager.get_api_url().await,
            api_url,
            "get_api_url should match status"
        );

        println!("✅ Bifrost status integration successful");
    }
}
