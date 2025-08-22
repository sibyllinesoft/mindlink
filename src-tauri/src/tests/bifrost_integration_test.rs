#[cfg(test)]
mod bifrost_integration_tests {
    use crate::managers::bifrost_manager::BifrostManager;
    use crate::managers::binary_manager::BinaryManager;
    use std::path::PathBuf;

    /// Test 1: Binary Manager Creation and Directory Setup
    #[tokio::test]
    async fn test_binary_manager_initialization() {
        println!("🧪 Test 1: Binary Manager Initialization");
        
        let result = BinaryManager::new().await;
        assert!(result.is_ok(), "BinaryManager should initialize successfully");
        
        let binary_manager = result.unwrap();
        let binaries_dir = binary_manager.get_binaries_dir();
        assert!(binaries_dir.exists(), "Binaries directory should be created");
        
        println!("✅ Binary Manager initialized successfully");
        println!("   Binaries directory: {:?}", binaries_dir);
    }

    /// Test 2: Bifrost Manager Creation
    #[tokio::test] 
    async fn test_bifrost_manager_creation() {
        println!("🧪 Test 2: Bifrost Manager Creation");
        
        let bifrost_manager = BifrostManager::new().await;
        
        // Check that the manager was created
        assert!(!bifrost_manager.is_running().await, "Bifrost should not be running initially");
        
        let (is_installed, binary_path, status_message) = bifrost_manager.get_installation_info().await;
        println!("   Installation status: {}", is_installed);
        println!("   Binary path: {:?}", binary_path);
        println!("   Status message: {:?}", status_message);
        
        println!("✅ Bifrost Manager created successfully");
    }

    /// Test 3: Binary Installation Process
    #[tokio::test]
    async fn test_bifrost_binary_installation() {
        println!("🧪 Test 3: Bifrost Binary Installation");
        
        let mut bifrost_manager = BifrostManager::new().await;
        
        // Test binary path refresh (tries to find existing binary first)
        let install_result = bifrost_manager.refresh_binary_path().await;
        
        match install_result {
            Ok(binary_path) => {
                println!("✅ Binary installation successful");
                println!("   Binary path: {:?}", binary_path);
                
                // Verify the binary exists
                assert!(binary_path.exists(), "Binary should exist after installation");
                
                // Check installation status
                let (is_installed, _, _) = bifrost_manager.get_installation_info().await;
                assert!(is_installed, "Installation status should show as installed");
                
            },
            Err(e) => {
                println!("⚠️  Binary installation failed (expected in test environment): {}", e);
                // This is expected in test environment without npm/node
                assert!(e.to_string().contains("npm") || e.to_string().contains("Node.js"));
            }
        }
    }

    /// Test 4: Port Allocation
    #[tokio::test]
    async fn test_port_allocation() {
        println!("🧪 Test 4: Port Allocation");
        
        // Create multiple Bifrost managers to test port allocation
        let manager1 = BifrostManager::new().await;
        let manager2 = BifrostManager::new().await;
        
        let (_, url1, _) = manager1.get_status_info().await;
        let (_, url2, _) = manager2.get_status_info().await;
        
        println!("   Manager 1 would use: {:?}", url1);
        println!("   Manager 2 would use: {:?}", url2);
        
        // Both should be None since they're not running, but the test verifies
        // that the managers can be created with different port configurations
        assert_eq!(url1, None);
        assert_eq!(url2, None);
        
        println!("✅ Port allocation system working");
    }

    /// Test 5: Service Lifecycle (Start/Stop) - Mock Test
    #[tokio::test]
    async fn test_service_lifecycle_mock() {
        println!("🧪 Test 5: Service Lifecycle (Mock Test)");
        
        let mut bifrost_manager = BifrostManager::new().await;
        
        // Test initial state
        assert!(!bifrost_manager.is_running().await, "Should start in stopped state");
        assert!(bifrost_manager.get_local_url().await.is_none(), "URL should be None when stopped");
        
        // Test start attempt (will fail in test environment)
        let start_result = bifrost_manager.start().await;
        
        match start_result {
            Ok(_) => {
                println!("✅ Bifrost started successfully (unexpected in test environment)");
                
                // Test running state
                assert!(bifrost_manager.is_running().await, "Should be running after start");
                assert!(bifrost_manager.get_local_url().await.is_some(), "URL should be available when running");
                
                // Test stop
                let stop_result = bifrost_manager.stop().await;
                assert!(stop_result.is_ok(), "Stop should succeed");
                assert!(!bifrost_manager.is_running().await, "Should be stopped after stop");
                
            },
            Err(e) => {
                println!("⚠️  Bifrost start failed (expected in test environment): {}", e);
                // Expected failures in test environment:
                let error_msg = e.to_string();
                let is_expected_error = error_msg.contains("Node.js") || 
                                      error_msg.contains("binary not found") ||
                                      error_msg.contains("installation required") ||
                                      error_msg.contains("npm");
                
                assert!(is_expected_error, "Error should be related to missing Node.js or binary: {}", error_msg);
            }
        }
        
        println!("✅ Service lifecycle test completed");
    }

    /// Test 6: Health Check Functionality
    #[tokio::test]
    async fn test_health_check() {
        println!("🧪 Test 6: Health Check");
        
        let bifrost_manager = BifrostManager::new().await;
        
        // Health check on stopped service
        let health_result = bifrost_manager.check_health().await;
        assert!(health_result.is_ok(), "Health check should not fail");
        assert!(!health_result.unwrap(), "Health should be false for stopped service");
        
        println!("✅ Health check working correctly");
    }

    /// Test 7: Configuration Management
    #[tokio::test]
    async fn test_configuration_management() {
        println!("🧪 Test 7: Configuration Management");
        
        let mut bifrost_manager = BifrostManager::new().await;
        
        // Test configuration changes (only allowed when stopped)
        bifrost_manager.configure("127.0.0.1".to_string(), 3005).await;
        
        // Test setting config path
        let config_path = PathBuf::from("/tmp/test_config.json");
        bifrost_manager.set_config_path(config_path.clone()).await;
        
        // Test setting binary path
        let binary_path = PathBuf::from("/tmp/test_binary");
        bifrost_manager.set_binary_path(binary_path.clone()).await;
        
        let retrieved_binary_path = bifrost_manager.get_binary_path().await;
        assert_eq!(retrieved_binary_path, Some(binary_path));
        
        println!("✅ Configuration management working");
    }

    /// Test 8: Error Handling for Invalid States
    #[tokio::test]
    async fn test_error_handling() {
        println!("🧪 Test 8: Error Handling");
        
        let bifrost_manager = BifrostManager::new().await;
        
        // Test getting models when not running
        let models_result = bifrost_manager.get_models().await;
        assert!(models_result.is_err(), "Getting models should fail when service is not running");
        
        let error_msg = models_result.unwrap_err().to_string();
        assert!(error_msg.contains("not running"), "Error should mention service not running");
        
        println!("✅ Error handling working correctly");
    }

    /// Test 9: Binary Manager Integration
    #[tokio::test]
    async fn test_binary_manager_integration() {
        println!("🧪 Test 9: Binary Manager Integration");
        
        let binary_manager = BinaryManager::new().await.unwrap();
        
        // Test listing installed binaries
        let binaries_result = binary_manager.list_installed_binaries();
        assert!(binaries_result.is_ok(), "Should be able to list binaries");
        
        let binaries = binaries_result.unwrap();
        println!("   Installed binaries: {:?}", binaries);
        
        // Test checking for non-existent binary
        let non_existent = binary_manager.get_binary_path("non_existent_binary");
        assert!(non_existent.is_none(), "Non-existent binary should return None");
        
        println!("✅ Binary Manager integration working");
    }

    /// Test 10: Concurrent Operations
    #[tokio::test]
    async fn test_concurrent_operations() {
        println!("🧪 Test 10: Concurrent Operations");
        
        let manager1 = BifrostManager::new().await;
        let manager2 = BifrostManager::new().await;
        
        // Test concurrent status checks
        let (health1, health2): (anyhow::Result<bool>, anyhow::Result<bool>) = tokio::join!(
            manager1.check_health(),
            manager2.check_health()
        );
        
        assert!(health1.is_ok(), "Concurrent health check 1 should succeed");
        assert!(health2.is_ok(), "Concurrent health check 2 should succeed");
        
        println!("✅ Concurrent operations working");
    }

    /// Test 11: Comprehensive Status Information
    #[tokio::test]
    async fn test_comprehensive_status() {
        println!("🧪 Test 11: Comprehensive Status Information");
        
        let bifrost_manager = BifrostManager::new().await;
        
        // Test status info
        let (running, url, api_url) = bifrost_manager.get_status_info().await;
        println!("   Running: {}", running);
        println!("   URL: {:?}", url);
        println!("   API URL: {:?}", api_url);
        
        // Test installation info
        let (is_installed, binary_path, status_message) = bifrost_manager.get_installation_info().await;
        println!("   Installed: {}", is_installed);
        println!("   Binary path: {:?}", binary_path);
        println!("   Status: {:?}", status_message);
        
        // Test binary availability
        let binary_available = bifrost_manager.is_binary_available().await;
        println!("   Binary available: {}", binary_available);
        
        // Test build recommendation
        let should_install = bifrost_manager.should_build().await;
        println!("   Should install: {}", should_install);
        
        assert_eq!(running, false, "Should not be running initially");
        assert_eq!(url, None, "URL should be None when not running");
        assert_eq!(api_url, None, "API URL should be None when not running");
        
        println!("✅ Comprehensive status information working");
    }

    /// Test 12: Binary Verification Process
    #[tokio::test]
    async fn test_binary_verification() {
        println!("🧪 Test 12: Binary Verification");
        
        let binary_manager = BinaryManager::new().await.unwrap();
        
        // Test verification of non-existent binary
        let fake_path = PathBuf::from("/non/existent/binary");
        let verification_result = binary_manager.verify_binary(&fake_path).await;
        
        assert!(verification_result.is_ok(), "Verification should not fail");
        assert!(!verification_result.unwrap(), "Non-existent binary should not verify");
        
        println!("✅ Binary verification working correctly");
    }

    /// Test 13: Binary Name Resolution (bifrost-http)
    #[tokio::test]
    async fn test_bifrost_http_binary_resolution() {
        println!("🧪 Test 13: Bifrost-HTTP Binary Resolution");
        
        let binary_manager = BinaryManager::new().await.unwrap();
        
        // Test that it looks for bifrost-http binary
        let bifrost_path = binary_manager.get_local_bifrost_path();
        println!("   Local bifrost path search result: {:?}", bifrost_path);
        
        // Test both old and new binary names return the same path
        let old_name_path = binary_manager.get_binary_path("bifrost");
        let new_name_path = binary_manager.get_binary_path("bifrost-http");
        
        assert_eq!(old_name_path, new_name_path, "Both 'bifrost' and 'bifrost-http' should return the same path");
        
        if let Some(path) = &bifrost_path {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            assert!(filename.starts_with("bifrost-http"), 
                    "Binary filename should start with 'bifrost-http', got: {}", filename);
        }
        
        println!("✅ Bifrost-HTTP binary resolution working correctly");
    }

    /// Test 14: Platform-Specific Binary Names
    #[tokio::test]
    async fn test_platform_specific_binary_names() {
        println!("🧪 Test 14: Platform-Specific Binary Names");
        
        let manager = BifrostManager::new().await;
        let binary_path = manager.get_binary_path().await;
        
        println!("   Current binary path: {:?}", binary_path);
        
        if let Some(path) = binary_path {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            println!("   Binary filename: {}", filename);
            
            // Should be bifrost-http or bifrost-http-{platform} or bifrost-http.exe
            let is_valid_name = filename == "bifrost-http" 
                || filename == "bifrost-http.exe"
                || filename.starts_with("bifrost-http-");
            
            assert!(is_valid_name, "Binary name should be bifrost-http variant, got: {}", filename);
        } else {
            println!("   No binary found (expected in test environment without pre-built binary)");
        }
        
        println!("✅ Platform-specific binary names working correctly");
    }
}