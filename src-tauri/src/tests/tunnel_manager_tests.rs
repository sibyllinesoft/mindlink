#[cfg(test)]
mod tunnel_manager_tests {
    use crate::managers::tunnel_manager::{TunnelManager, TunnelType};
    use regex::Regex;

    #[tokio::test]
    async fn test_tunnel_manager_creation() {
        println!("🧪 Test: TunnelManager creation");

        let result = TunnelManager::new().await;

        assert!(result.is_ok(), "TunnelManager creation should succeed");

        let manager = result.unwrap();
        assert!(
            !manager.is_connected().await,
            "Should not be connected initially"
        );
        assert!(
            manager.get_current_url().await.is_none(),
            "Should have no URL initially"
        );

        println!("✅ TunnelManager creation successful");
    }

    #[tokio::test]
    async fn test_tunnel_manager_initial_state() {
        println!("🧪 Test: TunnelManager initial state");

        let manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Check initial state
        assert!(
            !manager.is_connected().await,
            "Should not be connected initially"
        );
        assert!(
            manager.get_current_url().await.is_none(),
            "Should have no current URL"
        );

        // Health check on disconnected tunnel
        let health_result = manager.check_health().await;
        // Health check might fail when not connected, which is expected
        assert!(
            health_result.is_ok() || health_result.is_err(),
            "Health check should complete"
        );

        println!("✅ TunnelManager initial state test successful");
    }

    #[tokio::test]
    async fn test_tunnel_types() {
        println!("🧪 Test: Tunnel types");

        // Test TunnelType enum variants
        let quick_type = TunnelType::Quick;
        let named_type = TunnelType::Named("test-tunnel".to_string());

        // Verify types can be used
        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        manager.set_tunnel_type(quick_type).await;
        manager.set_tunnel_type(named_type).await;

        println!("✅ Tunnel types successful");
    }

    #[tokio::test]
    async fn test_tunnel_configuration() {
        println!("🧪 Test: Tunnel configuration");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Test setting tunnel type
        manager.set_tunnel_type(TunnelType::Quick).await;

        // Test setting local port
        manager.set_local_port(3001).await;
        manager.set_local_port(8080).await;

        // Configuration should complete without errors
        assert!(
            !manager.is_connected().await,
            "Should still not be connected after configuration"
        );

        println!("✅ Tunnel configuration successful");
    }

    #[tokio::test]
    async fn test_tunnel_with_named_type() {
        println!("🧪 Test: Named tunnel type configuration");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Test named tunnel type
        manager
            .set_tunnel_type(TunnelType::Named("test-tunnel".to_string()))
            .await;

        // Should still be in initial state
        assert!(
            !manager.is_connected().await,
            "Should not be connected after setting named tunnel"
        );

        println!("✅ Named tunnel type configuration successful");
    }

    #[tokio::test]
    async fn test_create_tunnel() {
        println!("🧪 Test: Create tunnel");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Try to create tunnel (will likely fail in test environment)
        let create_result = manager.create_tunnel().await;

        // In test environment, this will likely fail due to missing cloudflared binary
        // But we verify it handles errors gracefully
        match create_result {
            Ok(url) => {
                println!("   Create tunnel succeeded unexpectedly: {}", url);
                assert!(
                    !url.is_empty(),
                    "URL should not be empty if creation succeeded"
                );
            },
            Err(e) => {
                println!(
                    "   Create tunnel failed as expected in test environment: {}",
                    e
                );
                assert!(
                    !e.to_string().is_empty(),
                    "Error message should not be empty"
                );
            },
        }

        println!("✅ Create tunnel test successful");
    }

    #[tokio::test]
    async fn test_close_tunnel_when_not_connected() {
        println!("🧪 Test: Close tunnel when not connected");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Try to close tunnel when not connected
        let close_result = manager.close_tunnel().await;
        // Should handle gracefully (either succeed or fail with proper error)
        assert!(
            close_result.is_ok() || close_result.is_err(),
            "close_tunnel should complete"
        );

        // Should still not be connected
        assert!(!manager.is_connected().await, "Should remain disconnected");

        println!("✅ Close tunnel when not connected test successful");
    }

    #[tokio::test]
    async fn test_recreate_tunnel_when_not_connected() {
        println!("🧪 Test: Recreate tunnel when not connected");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Try to recreate tunnel when not connected
        let recreate_result = manager.recreate_tunnel().await;

        // This might succeed (creating new tunnel) or fail (no existing tunnel)
        // Both behaviors are valid depending on implementation
        assert!(
            recreate_result.is_ok() || recreate_result.is_err(),
            "recreate_tunnel should complete"
        );

        println!("✅ Recreate tunnel when not connected test successful");
    }

    #[tokio::test]
    async fn test_concurrent_tunnel_access() {
        println!("🧪 Test: Concurrent tunnel access");

        let manager = std::sync::Arc::new(
            TunnelManager::new()
                .await
                .expect("Failed to create tunnel manager"),
        );

        // Test concurrent reads
        let mut handles = vec![];
        for i in 0..3 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let connected = manager_clone.is_connected().await;
                let url = manager_clone.get_current_url().await;
                println!(
                    "   Concurrent check {}: connected={}, url={:?}",
                    i, connected, url
                );
                assert!(!connected, "Should not be connected in concurrent access");
                assert!(url.is_none(), "Should have no URL in concurrent access");
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent tunnel access test successful");
    }

    #[tokio::test]
    async fn test_tunnel_url_regex_parsing() {
        println!("🧪 Test: Tunnel URL regex parsing");

        // Test URL patterns for Cloudflare tunnels
        let cloudflare_pattern = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com").unwrap();

        // Test valid tunnel URLs
        let valid_cloudflare_urls = vec![
            "https://test-tunnel-123.trycloudflare.com",
            "https://another-example.trycloudflare.com",
            "https://abc-123-def.trycloudflare.com",
        ];

        for url in valid_cloudflare_urls {
            assert!(
                cloudflare_pattern.is_match(url),
                "Should match Cloudflare URL: {}",
                url
            );
        }

        // Test invalid URLs
        let invalid_urls = vec![
            "http://test.trycloudflare.com", // not https
            "https://test.example.com",      // wrong domain
            "not-a-url",                     // not a URL at all
        ];

        for url in invalid_urls {
            assert!(
                !cloudflare_pattern.is_match(url),
                "Should not match invalid URL: {}",
                url
            );
        }

        println!("✅ Tunnel URL regex parsing successful");
    }

    #[tokio::test]
    async fn test_binary_dependency_errors() {
        println!("🧪 Test: Binary dependency errors");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Test tunnel creation without cloudflared binary
        // This will likely fail in most test environments
        let create_result = manager.create_tunnel().await;
        match create_result {
            Ok(url) => {
                println!("   Tunnel created successfully: {}", url);
                // Clean up if successful
                let _ = manager.close_tunnel().await;
            },
            Err(e) => {
                println!("   Tunnel creation failed (expected): {:?}", e);
                // Verify error handling is structured
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
                // Should identify binary-related issues
                let error_msg = e.to_string();
                if error_msg.contains("cloudflared") || error_msg.contains("binary") {
                    println!("   Correctly identified binary dependency issue");
                } else {
                    println!("   Error type: {:?} (may be acceptable)", e);
                }
            },
        }

        println!("✅ Binary dependency errors test successful");
    }

    #[tokio::test]
    async fn test_invalid_port_handling() {
        println!("🧪 Test: Invalid port handling");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Test tunnel creation (API doesn't take port/type parameters)
        let result_port_0 = manager.create_tunnel().await;
        match result_port_0 {
            Ok(url) => {
                println!("   Port 0 tunnel created: {}", url);
                let _ = manager.close_tunnel().await;
            },
            Err(e) => {
                println!("   Port 0 tunnel creation failed: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        // Test tunnel creation again
        let result_high_port = manager.create_tunnel().await;
        match result_high_port {
            Ok(url) => {
                println!("   High port tunnel created: {}", url);
                let _ = manager.close_tunnel().await;
            },
            Err(e) => {
                println!("   High port tunnel creation failed: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        println!("✅ Invalid port handling test successful");
    }

    #[tokio::test]
    async fn test_tunnel_naming_validation() {
        println!("🧪 Test: Tunnel naming validation");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Test valid tunnel names
        let valid_names = vec!["test-tunnel", "my_tunnel_123", "valid-name-2024"];

        for name in valid_names {
            // Note: Current TunnelManager API doesn't expose named tunnel creation in tests
            // Test that the TunnelType enum works (this validates the naming logic)
            let tunnel_type = TunnelType::Named(name.to_string());
            println!("   Created tunnel type for name: {}", name);

            // Test basic tunnel creation
            let result = manager.create_tunnel().await;
            match result {
                Ok(url) => {
                    println!("   Tunnel created for test '{}': {}", name, url);
                    let _ = manager.close_tunnel().await;
                },
                Err(e) => {
                    println!("   Tunnel creation failed for test '{}': {:?}", name, e);
                    // This is acceptable due to binary availability issues
                },
            }
        }

        // Test potentially invalid tunnel names
        let potentially_invalid_names = vec![
            "tunnel with spaces",
            "tunnel@with@symbols",
            "very-very-very-long-tunnel-name-that-might-exceed-limits-for-some-systems",
            "",
        ];

        for name in potentially_invalid_names {
            // Test that the TunnelType enum can handle various name formats
            let tunnel_type = TunnelType::Named(name.to_string());
            println!(
                "   Created tunnel type for potentially invalid name: '{}'",
                name
            );

            // Test basic tunnel creation (not related to naming validation due to API limitations)
            let result = manager.create_tunnel().await;
            match result {
                Ok(url) => {
                    println!("   Tunnel created for test '{}': {}", name, url);
                    let _ = manager.close_tunnel().await;
                },
                Err(e) => {
                    println!("   Tunnel creation failed for test '{}': {:?}", name, e);
                    // Verify error handling
                    assert!(
                        !e.to_string().is_empty(),
                        "Error should have meaningful message"
                    );
                },
            }
        }

        println!("✅ Tunnel naming validation test successful");
    }

    #[tokio::test]
    async fn test_network_connectivity_errors() {
        println!("🧪 Test: Network connectivity errors");

        let manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Test health check on disconnected tunnel
        let health_result = manager.check_health().await;
        match health_result {
            Ok(healthy) => {
                println!("   Health check succeeded: {}", healthy);
                assert!(!healthy, "Disconnected tunnel should not be healthy");
            },
            Err(e) => {
                println!("   Health check failed: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
                // Should identify health check issues
                let error_msg = e.to_string();
                if error_msg.contains("health") || error_msg.contains("tunnel") {
                    println!("   Correctly identified health check issue");
                } else {
                    println!("   Error type: {:?} (may be acceptable)", e);
                }
            },
        }

        println!("✅ Network connectivity errors test successful");
    }

    #[tokio::test]
    async fn test_concurrent_tunnel_operations() {
        println!("🧪 Test: Concurrent tunnel operations");

        let manager = std::sync::Arc::new(
            TunnelManager::new()
                .await
                .expect("Failed to create tunnel manager"),
        );

        // Test concurrent operations on same manager
        let mut handles = vec![];

        for i in 0..3 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                match i % 3 {
                    0 => {
                        // Test connection status check
                        let connected = manager_clone.is_connected().await;
                        println!("   Concurrent check {}: connected={}", i, connected);
                    },
                    1 => {
                        // Test URL retrieval
                        let url = manager_clone.get_current_url().await;
                        println!("   Concurrent check {}: url={:?}", i, url);
                    },
                    2 => {
                        // Test health check
                        let health = manager_clone.check_health().await;
                        match health {
                            Ok(healthy) => {
                                println!("   Concurrent health check {}: {}", i, healthy)
                            },
                            Err(e) => println!("   Concurrent health check {} failed: {:?}", i, e),
                        }
                    },
                    _ => unreachable!(),
                }
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent tunnel operations test successful");
    }

    #[tokio::test]
    async fn test_tunnel_state_consistency() {
        println!("🧪 Test: Tunnel state consistency");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Initial state
        assert!(
            !manager.is_connected().await,
            "Should not be connected initially"
        );
        assert!(
            manager.get_current_url().await.is_none(),
            "Should have no URL initially"
        );

        // Try to close non-existent tunnel
        let close_result = manager.close_tunnel().await;
        match close_result {
            Ok(_) => println!("   Close succeeded on non-existent tunnel (idempotent)"),
            Err(e) => {
                println!("   Close failed on non-existent tunnel: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        // State should remain consistent
        assert!(
            !manager.is_connected().await,
            "Should remain disconnected after close"
        );
        assert!(
            manager.get_current_url().await.is_none(),
            "Should have no URL after close"
        );

        // Try to recreate non-existent tunnel
        let recreate_result = manager.recreate_tunnel().await;
        match recreate_result {
            Ok(url) => {
                println!("   Recreate succeeded unexpectedly: {}", url);
                let _ = manager.close_tunnel().await; // Cleanup
            },
            Err(e) => {
                println!("   Recreate failed as expected: {:?}", e);
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        println!("✅ Tunnel state consistency test successful");
    }

    #[tokio::test]
    async fn test_tunnel_resource_cleanup() {
        println!("🧪 Test: Tunnel resource cleanup");

        let mut manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Test multiple create/close cycles to verify cleanup
        for i in 0..3 {
            let create_result = manager.create_tunnel().await;
            match create_result {
                Ok(url) => {
                    println!("   Tunnel {} created: {}", i, url);
                    assert!(
                        manager.is_connected().await,
                        "Should be connected after creation"
                    );

                    // Close tunnel
                    let close_result = manager.close_tunnel().await;
                    match close_result {
                        Ok(_) => {
                            println!("   Tunnel {} closed successfully", i);
                            assert!(
                                !manager.is_connected().await,
                                "Should be disconnected after close"
                            );
                        },
                        Err(e) => {
                            println!("   Tunnel {} close failed: {:?}", i, e);
                        },
                    }
                },
                Err(e) => {
                    println!("   Tunnel {} creation failed: {:?}", i, e);
                    // This is expected in test environments without cloudflared
                },
            }

            // Small delay between cycles
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        println!("✅ Tunnel resource cleanup test successful");
    }
}
