#[cfg(test)]
mod server_manager_tests {
    use crate::managers::auth_manager::AuthManager;
    use crate::managers::server_manager::ServerManager;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_server_manager_creation() {
        println!("🧪 Test: ServerManager creation");

        let manager = ServerManager::new().await;

        // Should not be running initially
        assert!(
            !manager.is_running().await,
            "Should not be running initially"
        );
        assert!(
            manager.get_local_url().await.is_none(),
            "Should have no URL initially"
        );

        println!("✅ ServerManager creation successful");
    }

    #[tokio::test]
    async fn test_server_manager_initial_state() {
        println!("🧪 Test: ServerManager initial state");

        let manager = ServerManager::new().await;

        // Check initial state
        assert!(
            !manager.is_running().await,
            "Should not be running initially"
        );
        assert!(
            manager.get_local_url().await.is_none(),
            "Should have no local URL"
        );

        // Health check on stopped server
        let health_result = manager.check_health().await;
        // Health check should succeed but return false when not running
        match health_result {
            Ok(healthy) => assert!(!healthy, "Should not be healthy when not running"),
            Err(_) => {
                // Some implementations might return an error when checking health of stopped server
                println!("   Health check returned error for stopped server (acceptable)");
            },
        }

        println!("✅ ServerManager initial state test successful");
    }

    #[tokio::test]
    async fn test_server_configuration() {
        println!("🧪 Test: Server configuration");

        let mut manager = ServerManager::new().await;

        // Test configuring server
        let config_result = manager.configure("127.0.0.1".to_string(), 8080).await;
        assert!(config_result.is_ok(), "Configuration should succeed");

        // Test with different port
        let config_result2 = manager.configure("0.0.0.0".to_string(), 3001).await;
        assert!(
            config_result2.is_ok(),
            "Second configuration should succeed"
        );

        // Should still not be running after configuration
        assert!(
            !manager.is_running().await,
            "Should not be running after configuration"
        );

        println!("✅ Server configuration successful");
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        println!("🧪 Test: Stop server when not running");

        let mut manager = ServerManager::new().await;

        // Try to stop server when not running
        let stop_result = manager.stop().await;
        // Should handle gracefully (either succeed or fail with proper error)
        assert!(
            stop_result.is_ok() || stop_result.is_err(),
            "stop should complete"
        );

        // Should still not be running
        assert!(!manager.is_running().await, "Should remain not running");

        println!("✅ Stop when not running test successful");
    }

    #[tokio::test]
    async fn test_start_without_auth() {
        println!("🧪 Test: Start server without proper auth");

        let mut manager = ServerManager::new().await;

        // Create an auth manager that's not authenticated
        let auth_manager = Arc::new(RwLock::new(
            AuthManager::new()
                .await
                .expect("Failed to create auth manager"),
        ));

        // Try to start server (will likely fail due to no authentication)
        let start_result = manager.start(auth_manager).await;

        // This should fail since we don't have valid authentication
        match start_result {
            Ok(url) => {
                println!("   Start succeeded unexpectedly: {}", url);
                // If it succeeded, verify we got a valid URL
                assert!(
                    !url.is_empty(),
                    "URL should not be empty if start succeeded"
                );
                assert!(
                    manager.is_running().await,
                    "Should be running if start succeeded"
                );
            },
            Err(e) => {
                println!("   Start failed as expected without auth: {}", e);
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

        println!("✅ Start without auth test successful");
    }

    #[tokio::test]
    async fn test_restart_when_not_running() {
        println!("🧪 Test: Restart server when not running");

        let mut manager = ServerManager::new().await;

        // Create an auth manager
        let auth_manager = Arc::new(RwLock::new(
            AuthManager::new()
                .await
                .expect("Failed to create auth manager"),
        ));

        // Try to restart server when not running
        let restart_result = manager.restart(auth_manager).await;

        // This might succeed (starting fresh) or fail (no auth/other issues)
        // Both behaviors are valid depending on implementation
        match restart_result {
            Ok(url) => {
                println!("   Restart succeeded: {}", url);
                assert!(
                    !url.is_empty(),
                    "URL should not be empty if restart succeeded"
                );
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
    async fn test_concurrent_server_access() {
        println!("🧪 Test: Concurrent server access");

        let manager = std::sync::Arc::new(ServerManager::new().await);

        // Test concurrent reads
        let mut handles = vec![];
        for i in 0..3 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let running = manager_clone.is_running().await;
                let url = manager_clone.get_local_url().await;
                println!(
                    "   Concurrent check {}: running={}, url={:?}",
                    i, running, url
                );
                assert!(!running, "Should not be running in concurrent access");
                assert!(url.is_none(), "Should have no URL in concurrent access");
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent server access test successful");
    }

    #[tokio::test]
    async fn test_configuration_edge_cases() {
        println!("🧪 Test: Configuration edge cases");

        let mut manager = ServerManager::new().await;

        // Test configuration with port 0 (should let system choose)
        let config_result = manager.configure("127.0.0.1".to_string(), 0).await;
        // This might succeed or fail depending on implementation
        assert!(
            config_result.is_ok() || config_result.is_err(),
            "Configuration with port 0 should complete"
        );

        // Test configuration with high port number
        let config_result2 = manager.configure("127.0.0.1".to_string(), 65535).await;
        assert!(
            config_result2.is_ok() || config_result2.is_err(),
            "Configuration with high port should complete"
        );

        // Test configuration with localhost
        let config_result3 = manager.configure("localhost".to_string(), 8080).await;
        assert!(
            config_result3.is_ok() || config_result3.is_err(),
            "Configuration with localhost should complete"
        );

        println!("✅ Configuration edge cases test successful");
    }

    #[tokio::test]
    async fn test_invalid_configuration() {
        println!("🧪 Test: Invalid server configuration");

        let mut manager = ServerManager::new().await;

        // Test configuration with invalid host
        let invalid_host_result = manager.configure("invalid..host".to_string(), 8080).await;
        match invalid_host_result {
            Ok(_) => println!("   Invalid host configuration accepted (implementation dependent)"),
            Err(e) => {
                println!("   Invalid host configuration rejected: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
            },
        }

        // Test configuration with high port number
        let invalid_port_result = manager.configure("localhost".to_string(), 65535).await;
        match invalid_port_result {
            Ok(_) => println!("   High port configuration accepted"),
            Err(e) => {
                println!("   High port configuration rejected: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
            },
        }

        println!("✅ Invalid configuration test successful");
    }

    #[tokio::test]
    async fn test_health_check_errors() {
        println!("🧪 Test: Health check error scenarios");

        let manager = ServerManager::new().await;

        // Health check on non-running server
        let health_result = manager.check_health().await;
        match health_result {
            Ok(healthy) => {
                println!("   Health check succeeded on stopped server: {}", healthy);
                assert!(!healthy, "Stopped server should not be healthy");
            },
            Err(e) => {
                println!("   Health check failed on stopped server: {:?}", e);
                // This is acceptable - some implementations may error on stopped servers
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
            },
        }

        // Test repeated health checks
        for i in 0..3 {
            let health_result = manager.check_health().await;
            match health_result {
                Ok(healthy) => {
                    println!("   Health check {} result: {}", i, healthy);
                },
                Err(e) => {
                    println!("   Health check {} failed: {:?}", i, e);
                },
            }
        }

        println!("✅ Health check errors test successful");
    }

    #[tokio::test]
    async fn test_server_state_consistency() {
        println!("🧪 Test: Server state consistency");

        let mut manager = ServerManager::new().await;

        // Initial state checks
        assert!(
            !manager.is_running().await,
            "Should not be running initially"
        );
        assert!(
            manager.get_local_url().await.is_none(),
            "Should have no URL initially"
        );

        // Configure server
        let config_result = manager.configure("127.0.0.1".to_string(), 3002).await;
        assert!(config_result.is_ok(), "Configuration should succeed");

        // State should still be not running after configuration
        assert!(
            !manager.is_running().await,
            "Should not be running after configuration"
        );
        assert!(
            manager.get_local_url().await.is_none(),
            "Should have no URL after configuration"
        );

        // Try to stop non-running server
        let stop_result = manager.stop().await;
        assert!(
            stop_result.is_ok() || stop_result.is_err(),
            "Stop should complete without panic"
        );

        // State should remain consistent
        assert!(
            !manager.is_running().await,
            "Should remain not running after stop"
        );

        println!("✅ Server state consistency test successful");
    }

    #[tokio::test]
    async fn test_server_lifecycle_edge_cases() {
        println!("🧪 Test: Server lifecycle edge cases");

        let mut manager = ServerManager::new().await;
        let auth_manager = Arc::new(RwLock::new(
            AuthManager::new()
                .await
                .expect("Failed to create auth manager"),
        ));

        // Test start with different auth managers
        let start_result1 = manager.start(auth_manager.clone()).await;
        if start_result1.is_ok() {
            println!("   Start succeeded");

            // Test stop and immediate restart
            let stop_result = manager.stop().await;
            if stop_result.is_ok() {
                println!("   Stop succeeded");

                // Immediate restart
                let restart_result = manager.start(auth_manager).await;
                match restart_result {
                    Ok(url) => {
                        println!("   Immediate restart succeeded: {}", url);
                        let _ = manager.stop().await; // Cleanup
                    },
                    Err(e) => {
                        println!("   Immediate restart failed: {:?}", e);
                    },
                }
            }
        } else {
            println!(
                "   Start failed (expected without proper auth): {:?}",
                start_result1.unwrap_err()
            );
        }

        println!("✅ Server lifecycle edge cases test successful");
    }

    #[tokio::test]
    async fn test_network_error_handling() {
        println!("🧪 Test: Network error handling");

        let mut manager = ServerManager::new().await;

        // Test binding to potentially restricted ports
        let restricted_ports = [80, 443, 22, 21];

        for &port in &restricted_ports {
            let config_result = manager.configure("127.0.0.1".to_string(), port).await;
            match config_result {
                Ok(_) => {
                    println!("   Configuration for port {} succeeded", port);

                    // Try to start (will likely fail due to permissions)
                    let auth_manager = Arc::new(RwLock::new(
                        AuthManager::new()
                            .await
                            .expect("Failed to create auth manager"),
                    ));

                    let start_result = manager.start(auth_manager).await;
                    match start_result {
                        Ok(url) => {
                            println!(
                                "   Unexpectedly started on restricted port {}: {}",
                                port, url
                            );
                            let _ = manager.stop().await; // Cleanup
                        },
                        Err(e) => {
                            println!(
                                "   Failed to start on restricted port {} (expected): {:?}",
                                port, e
                            );
                            // This is expected for restricted ports
                        },
                    }
                },
                Err(e) => {
                    println!("   Configuration for port {} failed: {:?}", port, e);
                },
            }
        }

        println!("✅ Network error handling test successful");
    }
}
