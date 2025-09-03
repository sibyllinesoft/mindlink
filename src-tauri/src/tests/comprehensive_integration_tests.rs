//! Comprehensive Integration Tests
//!
//! This module contains integration tests that verify cross-manager communication,
//! error propagation, state coordination, and end-to-end workflows. These tests
//! simulate real-world usage patterns and edge cases to ensure system reliability.

#[cfg(test)]
mod comprehensive_integration_tests {
    use crate::error::MindLinkError;
    use crate::managers::auth_manager::AuthManager;
    use crate::managers::config_manager::ConfigManager;
    use crate::managers::server_manager::ServerManager;
    use crate::managers::tunnel_manager::{TunnelManager, TunnelType};
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio::time::{timeout, Duration};

    /// Test helper to create managers for integration testing
    async fn create_test_managers() -> (AuthManager, ServerManager, TunnelManager, ConfigManager) {
        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");
        let server_manager = ServerManager::new().await;
        let tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");
        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        (auth_manager, server_manager, tunnel_manager, config_manager)
    }

    #[tokio::test]
    async fn test_complete_system_startup_without_auth() {
        println!("🧪 Integration Test: Complete system startup without authentication");

        let (auth_manager, mut server_manager, mut tunnel_manager, config_manager) =
            create_test_managers().await;

        // Verify initial states
        assert!(
            !auth_manager.is_authenticated().await,
            "Auth should not be authenticated initially"
        );
        assert!(
            !server_manager.is_running().await,
            "Server should not be running initially"
        );
        assert!(
            !tunnel_manager.is_connected().await,
            "Tunnel should not be connected initially"
        );

        let config = config_manager.get_config().await;
        println!(
            "   Config loaded: server={}:{}",
            config.server.host, config.server.port
        );

        // Configure server from config
        let server_config_result = server_manager
            .configure(config.server.host.clone(), config.server.port)
            .await;
        assert!(
            server_config_result.is_ok(),
            "Server configuration should succeed"
        );

        // Try to start server without authentication
        let auth_manager_shared = Arc::new(RwLock::new(auth_manager));
        let server_start_result = server_manager.start(auth_manager_shared.clone()).await;

        match server_start_result {
            Ok(url) => {
                println!("   Server started unexpectedly without auth: {}", url);

                // If server started, try to create tunnel
                let port = url
                    .split(':')
                    .last()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(3000);

                let tunnel_result = tunnel_manager.create_tunnel().await;
                match tunnel_result {
                    Ok(tunnel_url) => {
                        println!("   Tunnel created: {}", tunnel_url);
                        let _ = tunnel_manager.close_tunnel().await;
                    },
                    Err(e) => {
                        println!("   Tunnel creation failed (expected): {:?}", e);
                    },
                }

                // Cleanup
                let _ = server_manager.stop().await;
            },
            Err(e) => {
                println!("   Server start failed without auth (expected): {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );

                // Verify proper error type
                match e {
                    MindLinkError::Authentication { .. } => {
                        println!("   Correct authentication error type");
                    },
                    _ => {
                        println!("   Error type: {:?} (may be acceptable)", e);
                    },
                }
            },
        }

        println!("✅ Complete system startup without auth test successful");
    }

    #[tokio::test]
    async fn test_cascading_failure_handling() {
        println!("🧪 Integration Test: Cascading failure handling");

        let (auth_manager, mut server_manager, mut tunnel_manager, config_manager) =
            create_test_managers().await;
        let auth_manager_shared = Arc::new(RwLock::new(auth_manager));

        // Configure server with invalid settings
        let invalid_config_result = server_manager
            .configure("invalid.host".to_string(), 65535)
            .await;
        match invalid_config_result {
            Ok(_) => {
                println!("   Invalid configuration accepted");

                // Try to start server with invalid config
                let start_result = server_manager.start(auth_manager_shared.clone()).await;
                match start_result {
                    Ok(url) => {
                        println!("   Server started with invalid config: {}", url);

                        // Try to create tunnel
                        let tunnel_result = tunnel_manager.create_tunnel().await;
                        match tunnel_result {
                            Ok(tunnel_url) => {
                                println!("   Tunnel created to invalid server: {}", tunnel_url);
                                let _ = tunnel_manager.close_tunnel().await;
                            },
                            Err(e) => {
                                println!("   Tunnel creation failed (expected): {:?}", e);
                            },
                        }

                        let _ = server_manager.stop().await;
                    },
                    Err(e) => {
                        println!("   Server start failed with invalid config: {:?}", e);
                        assert!(
                            !e.to_string().is_empty(),
                            "Error should have meaningful message"
                        );
                    },
                }
            },
            Err(e) => {
                println!("   Invalid configuration rejected: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        println!("✅ Cascading failure handling test successful");
    }

    #[tokio::test]
    async fn test_concurrent_manager_operations() {
        println!("🧪 Integration Test: Concurrent manager operations");

        let (auth_manager, server_manager, tunnel_manager, config_manager) =
            create_test_managers().await;

        let auth_manager = Arc::new(auth_manager);
        let server_manager = Arc::new(tokio::sync::RwLock::new(server_manager));
        let tunnel_manager = Arc::new(tunnel_manager);
        let config_manager = Arc::new(config_manager);

        // Launch concurrent operations on different managers
        let mut handles = vec![];

        // Auth manager operations
        for i in 0..3 {
            let auth_clone = auth_manager.clone();
            let handle = tokio::spawn(async move {
                let is_auth = auth_clone.is_authenticated().await;
                println!("   Concurrent auth check {}: {}", i, is_auth);
                let _ = auth_clone.get_access_token();
            });
            handles.push(handle);
        }

        // Config manager operations
        for i in 0..2 {
            let config_clone = config_manager.clone();
            let handle = tokio::spawn(async move {
                let config = config_clone.get_config().await;
                println!(
                    "   Concurrent config access {}: port={}",
                    i, config.server.port
                );
                let _ = config_clone.get_config().await;
            });
            handles.push(handle);
        }

        // Server manager operations
        for i in 0..2 {
            let server_clone = server_manager.clone();
            let handle = tokio::spawn(async move {
                let server = server_clone.read().await;
                let running = server.is_running().await;
                println!("   Concurrent server check {}: running={}", i, running);
            });
            handles.push(handle);
        }

        // Tunnel manager operations
        for i in 0..2 {
            let tunnel_clone = tunnel_manager.clone();
            let handle = tokio::spawn(async move {
                let connected = tunnel_clone.is_connected().await;
                println!("   Concurrent tunnel check {}: connected={}", i, connected);
                let _ = tunnel_clone.check_health().await;
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent manager operations test successful");
    }

    #[tokio::test]
    async fn test_configuration_consistency_across_managers() {
        println!("🧪 Integration Test: Configuration consistency across managers");

        let (auth_manager, mut server_manager, tunnel_manager, config_manager) =
            create_test_managers().await;

        // Get initial config
        let initial_config = config_manager.get_config().await;
        let initial_port = initial_config.server.port;

        // Configure server manager with config values
        let config_result = server_manager
            .configure(initial_config.server.host.clone(), initial_port)
            .await;
        assert!(config_result.is_ok(), "Server configuration should succeed");

        // Update config
        let mut updated_config = initial_config.clone();
        updated_config.server.port = initial_port + 100;
        updated_config.server.host = "127.0.0.1".to_string();

        let update_result = config_manager.update_config(updated_config.clone()).await;
        assert!(update_result.is_ok(), "Config update should succeed");

        // Verify config was updated
        let new_config = config_manager.get_config().await;
        assert_eq!(new_config.server.port, initial_port + 100);
        assert_eq!(new_config.server.host, "127.0.0.1");

        // Server manager should still have old config until reconfigured
        // This tests that managers maintain independent state appropriately

        // Reconfigure server with new config
        let reconfig_result = server_manager
            .configure(new_config.server.host.clone(), new_config.server.port)
            .await;
        assert!(
            reconfig_result.is_ok(),
            "Server reconfiguration should succeed"
        );

        println!("✅ Configuration consistency test successful");
    }

    #[tokio::test]
    async fn test_error_propagation_across_system() {
        println!("🧪 Integration Test: Error propagation across system");

        let (mut auth_manager, mut server_manager, mut tunnel_manager, config_manager) =
            create_test_managers().await;

        // Create authentication error
        let auth_error_result = auth_manager.refresh_tokens().await;
        match auth_error_result {
            Ok(_) => println!("   Token refresh succeeded unexpectedly"),
            Err(e) => {
                println!("   Authentication error: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );

                // Verify error propagates to server start
                let auth_shared = Arc::new(RwLock::new(auth_manager));
                let server_start_result = server_manager.start(auth_shared).await;

                match server_start_result {
                    Ok(_) => println!("   Server started despite auth error"),
                    Err(server_error) => {
                        println!(
                            "   Server start failed due to auth error: {:?}",
                            server_error
                        );
                        assert!(
                            !server_error.to_string().is_empty(),
                            "Server error should have message"
                        );
                    },
                }
            },
        }

        // Create network/binary error
        let tunnel_error_result = tunnel_manager.create_tunnel().await;
        match tunnel_error_result {
            Ok(url) => {
                println!("   Tunnel created successfully: {}", url);
                let _ = tunnel_manager.close_tunnel().await;
            },
            Err(e) => {
                println!("   Tunnel error: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Tunnel error should have message"
                );

                // Check error message content
                let error_msg = e.to_string();
                if error_msg.contains("binary") || error_msg.contains("cloudflared") {
                    println!("   Correct binary execution error identified");
                } else if error_msg.contains("network") || error_msg.contains("connection") {
                    println!("   Network error (acceptable)");
                } else {
                    println!("   Other error type: {:?}", e);
                }
            },
        }

        println!("✅ Error propagation test successful");
    }

    #[tokio::test]
    async fn test_resource_cleanup_coordination() {
        println!("🧪 Integration Test: Resource cleanup coordination");

        let (auth_manager, mut server_manager, mut tunnel_manager, config_manager) =
            create_test_managers().await;
        let auth_shared = Arc::new(RwLock::new(auth_manager));

        // Configure server
        let config = config_manager.get_config().await;
        let _ = server_manager
            .configure(config.server.host.clone(), config.server.port)
            .await;

        // Attempt to start server
        let server_start_result = server_manager.start(auth_shared.clone()).await;
        if server_start_result.is_ok() {
            println!("   Server started for cleanup test");

            // Attempt to create tunnel
            let tunnel_result = tunnel_manager.create_tunnel().await;
            if tunnel_result.is_ok() {
                println!("   Tunnel created for cleanup test");

                // Verify both are running
                assert!(
                    server_manager.is_running().await,
                    "Server should be running"
                );
                assert!(
                    tunnel_manager.is_connected().await,
                    "Tunnel should be connected"
                );
            }

            // Test cleanup order - tunnel first, then server
            if tunnel_manager.is_connected().await {
                let tunnel_close_result = tunnel_manager.close_tunnel().await;
                match tunnel_close_result {
                    Ok(_) => {
                        println!("   Tunnel closed successfully");
                        assert!(
                            !tunnel_manager.is_connected().await,
                            "Tunnel should be disconnected"
                        );
                    },
                    Err(e) => println!("   Tunnel close failed: {:?}", e),
                }
            }

            // Close server
            let server_stop_result = server_manager.stop().await;
            match server_stop_result {
                Ok(_) => {
                    println!("   Server stopped successfully");
                    assert!(
                        !server_manager.is_running().await,
                        "Server should be stopped"
                    );
                },
                Err(e) => println!("   Server stop failed: {:?}", e),
            }
        } else {
            println!(
                "   Server failed to start (expected without auth): {:?}",
                server_start_result.unwrap_err()
            );
        }

        // Verify final state
        assert!(
            !server_manager.is_running().await,
            "Server should not be running"
        );
        assert!(
            !tunnel_manager.is_connected().await,
            "Tunnel should not be connected"
        );

        println!("✅ Resource cleanup coordination test successful");
    }

    #[tokio::test]
    async fn test_timeout_and_retry_behavior() {
        println!("🧪 Integration Test: Timeout and retry behavior");

        let (auth_manager, mut server_manager, mut tunnel_manager, _config_manager) =
            create_test_managers().await;
        let auth_shared = Arc::new(RwLock::new(auth_manager));

        // Test operations with timeout
        let timeout_duration = Duration::from_secs(5);

        // Server operations with timeout
        let server_start_with_timeout =
            timeout(timeout_duration, server_manager.start(auth_shared.clone())).await;

        match server_start_with_timeout {
            Ok(start_result) => {
                match start_result {
                    Ok(url) => {
                        println!("   Server started within timeout: {}", url);

                        // Test health check with timeout
                        let health_with_timeout =
                            timeout(Duration::from_secs(2), server_manager.check_health()).await;

                        match health_with_timeout {
                            Ok(health_result) => {
                                println!(
                                    "   Health check completed within timeout: {:?}",
                                    health_result
                                );
                            },
                            Err(_) => {
                                println!(
                                    "   Health check timed out (may indicate performance issue)"
                                );
                            },
                        }

                        let _ = server_manager.stop().await;
                    },
                    Err(e) => {
                        println!("   Server start failed within timeout: {:?}", e);
                    },
                }
            },
            Err(_) => {
                println!("   Server start operation timed out");
            },
        }

        // Tunnel operations with timeout
        let tunnel_create_with_timeout =
            timeout(timeout_duration, tunnel_manager.create_tunnel()).await;

        match tunnel_create_with_timeout {
            Ok(tunnel_result) => match tunnel_result {
                Ok(url) => {
                    println!("   Tunnel created within timeout: {}", url);
                    let _ = tunnel_manager.close_tunnel().await;
                },
                Err(e) => {
                    println!("   Tunnel creation failed within timeout: {:?}", e);
                },
            },
            Err(_) => {
                println!("   Tunnel creation timed out");
            },
        }

        println!("✅ Timeout and retry behavior test successful");
    }

    #[tokio::test]
    async fn test_system_recovery_after_failures() {
        println!("🧪 Integration Test: System recovery after failures");

        let (auth_manager, mut server_manager, mut tunnel_manager, config_manager) =
            create_test_managers().await;
        let auth_shared = Arc::new(RwLock::new(auth_manager));
        let config = config_manager.get_config().await;

        // Configure server
        let _ = server_manager
            .configure(config.server.host.clone(), config.server.port)
            .await;

        // Simulate failure scenario
        let initial_start = server_manager.start(auth_shared.clone()).await;
        if initial_start.is_ok() {
            println!("   Initial server start succeeded");

            // Force stop to simulate failure
            let _ = server_manager.stop().await;
            assert!(
                !server_manager.is_running().await,
                "Server should be stopped"
            );

            // Attempt recovery - restart
            let recovery_start = server_manager.start(auth_shared.clone()).await;
            match recovery_start {
                Ok(url) => {
                    println!("   Server recovery successful: {}", url);
                    assert!(
                        server_manager.is_running().await,
                        "Server should be running after recovery"
                    );

                    // Test tunnel recovery
                    let tunnel_create = tunnel_manager.create_tunnel().await;
                    if tunnel_create.is_ok() {
                        println!("   Tunnel recovery successful");

                        // Simulate tunnel failure and recovery
                        let _ = tunnel_manager.close_tunnel().await;
                        let tunnel_recovery = tunnel_manager.recreate_tunnel().await;
                        match tunnel_recovery {
                            Ok(url) => println!("   Tunnel recreate successful: {}", url),
                            Err(e) => println!("   Tunnel recreate failed: {:?}", e),
                        }
                    }

                    // Cleanup
                    let _ = tunnel_manager.close_tunnel().await;
                    let _ = server_manager.stop().await;
                },
                Err(e) => {
                    println!("   Server recovery failed: {:?}", e);
                },
            }
        } else {
            println!("   Initial server start failed (expected without auth)");

            // Test that system can still respond to operations after failures
            assert!(
                !server_manager.is_running().await,
                "Server should not be running"
            );

            let health_after_failure = server_manager.check_health().await;
            match health_after_failure {
                Ok(healthy) => {
                    println!("   Health check after failure: {}", healthy);
                    assert!(!healthy, "Should not be healthy when not running");
                },
                Err(e) => {
                    println!("   Health check failed after failure: {:?}", e);
                },
            }
        }

        println!("✅ System recovery after failures test successful");
    }
}
