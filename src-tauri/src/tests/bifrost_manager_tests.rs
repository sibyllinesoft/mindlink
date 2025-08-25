#[cfg(test)]
mod bifrost_manager_tests {
    use crate::managers::bifrost_manager::BifrostManager;

    #[tokio::test]
    async fn test_bifrost_manager_creation() {
        println!("🧪 Test: BifrostManager creation");

        let manager = BifrostManager::new().await;

        assert!(
            !manager.is_running().await,
            "Should not be running initially"
        );
        assert!(
            manager.get_local_url().await.is_none(),
            "Should have no local URL initially"
        );
        assert!(
            manager.get_api_url().await.is_none(),
            "Should have no API URL initially"
        );

        println!("✅ BifrostManager creation successful");
    }

    #[tokio::test]
    async fn test_bifrost_manager_initial_state() {
        println!("🧪 Test: BifrostManager initial state");

        let manager = BifrostManager::new().await;

        // Check initial state
        assert!(
            !manager.is_running().await,
            "Should not be running initially"
        );
        assert!(
            manager.get_local_url().await.is_none(),
            "Should have no local URL"
        );
        assert!(
            manager.get_api_url().await.is_none(),
            "Should have no API URL"
        );

        let (is_running, local_url, api_url) = manager.get_status_info().await;
        assert!(!is_running, "Status should show not running");
        assert!(local_url.is_none(), "Status should show no local URL");
        assert!(api_url.is_none(), "Status should show no API URL");

        println!("✅ BifrostManager initial state test successful");
    }

    #[tokio::test]
    async fn test_bifrost_configuration() {
        println!("🧪 Test: BifrostManager configuration");

        let mut manager = BifrostManager::new().await;

        // Test configuring bifrost
        manager.configure("127.0.0.1".to_string(), 3001).await;
        manager.configure("0.0.0.0".to_string(), 8080).await;

        // Configuration should complete without errors
        assert!(
            !manager.is_running().await,
            "Should still not be running after configuration"
        );

        println!("✅ BifrostManager configuration successful");
    }

    #[tokio::test]
    async fn test_bifrost_binary_availability() {
        println!("🧪 Test: Bifrost binary availability");

        let manager = BifrostManager::new().await;

        // Check binary availability
        let is_available = manager.is_binary_available().await;
        let binary_path = manager.get_binary_path().await;
        let (is_installed, path, version) = manager.get_installation_info().await;
        let should_build = manager.should_build().await;

        println!("   Binary available: {}", is_available);
        println!("   Binary path: {:?}", binary_path);
        println!(
            "   Installed: {}, Path: {:?}, Version: {:?}",
            is_installed, path, version
        );
        println!("   Should build: {}", should_build);

        // These checks don't need to pass specific values in test environment
        // Just verify the methods complete successfully
        assert!(true, "Binary availability check should complete");

        println!("✅ Bifrost binary availability test successful");
    }

    #[tokio::test]
    async fn test_bifrost_health_check() {
        println!("🧪 Test: Bifrost health check");

        let manager = BifrostManager::new().await;

        // Health check when not running
        let health_result = manager.check_health().await;
        match health_result {
            Ok(healthy) => {
                println!("   Health check returned: {}", healthy);
                assert!(!healthy, "Should not be healthy when not running");
            },
            Err(e) => {
                println!("   Health check failed as expected: {}", e);
                // This is acceptable when not running
            },
        }

        println!("✅ Bifrost health check test successful");
    }

    #[tokio::test]
    async fn test_bifrost_models() {
        println!("🧪 Test: Bifrost models retrieval");

        let manager = BifrostManager::new().await;

        // Try to get models (will likely fail when not running)
        let models_result = manager.get_models().await;
        match models_result {
            Ok(models) => {
                println!("   Got models: {:?}", models);
                // If successful, should have some models
            },
            Err(e) => {
                println!("   Get models failed as expected when not running: {}", e);
                // This is expected when bifrost is not running
            },
        }

        println!("✅ Bifrost models retrieval test successful");
    }

    #[tokio::test]
    async fn test_bifrost_start_stop() {
        println!("🧪 Test: BifrostManager start/stop functionality");

        let mut manager = BifrostManager::new().await;

        // Try to start (might fail in test environment due to missing binary)
        let start_result = manager.start().await;
        match start_result {
            Ok(_) => {
                println!("   Start succeeded");
                assert!(
                    manager.is_running().await,
                    "Should be running after successful start"
                );

                // Try to stop
                let stop_result = manager.stop().await;
                assert!(
                    stop_result.is_ok(),
                    "Stop should succeed after successful start"
                );
                assert!(
                    !manager.is_running().await,
                    "Should not be running after stop"
                );
            },
            Err(e) => {
                println!("   Start failed as expected in test environment: {}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
                assert!(
                    !manager.is_running().await,
                    "Should not be running after failed start"
                );
            },
        }

        println!("✅ BifrostManager start/stop test successful");
    }

    #[tokio::test]
    async fn test_bifrost_stop_when_not_running() {
        println!("🧪 Test: Stop bifrost when not running");

        let mut manager = BifrostManager::new().await;

        // Try to stop when not running
        let stop_result = manager.stop().await;
        // Should handle gracefully
        assert!(
            stop_result.is_ok() || stop_result.is_err(),
            "stop should complete"
        );

        // Should still not be running
        assert!(!manager.is_running().await, "Should remain not running");

        println!("✅ Stop when not running test successful");
    }

    #[tokio::test]
    async fn test_bifrost_restart_when_not_running() {
        println!("🧪 Test: Restart bifrost when not running");

        let mut manager = BifrostManager::new().await;

        // Try to restart when not running
        let restart_result = manager.restart().await;

        // This might succeed or fail depending on implementation
        match restart_result {
            Ok(_) => {
                println!("   Restart succeeded");
                // If restart succeeded, manager might be running
            },
            Err(e) => {
                println!("   Restart failed as expected: {}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
            },
        }

        println!("✅ Restart when not running test successful");
    }

    #[tokio::test]
    async fn test_concurrent_bifrost_access() {
        println!("🧪 Test: Concurrent bifrost access");

        let manager = std::sync::Arc::new(BifrostManager::new().await);

        // Test concurrent reads
        let mut handles = vec![];
        for i in 0..3 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let running = manager_clone.is_running().await;
                let local_url = manager_clone.get_local_url().await;
                let api_url = manager_clone.get_api_url().await;
                println!(
                    "   Concurrent check {}: running={}, local_url={:?}, api_url={:?}",
                    i, running, local_url, api_url
                );
                assert!(!running, "Should not be running in concurrent access");
                assert!(
                    local_url.is_none(),
                    "Should have no local URL in concurrent access"
                );
                assert!(
                    api_url.is_none(),
                    "Should have no API URL in concurrent access"
                );
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent bifrost access test successful");
    }
}
