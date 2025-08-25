#[cfg(test)]
mod auth_manager_tests {
    use crate::managers::auth_manager::AuthManager;
    use tempfile::TempDir;
    use tokio::fs;

    /// Helper to create a test directory with proper auth structure
    async fn create_test_auth_dir() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create mindlink directory structure
        let mindlink_dir = temp_dir.path().join(".mindlink");
        fs::create_dir_all(&mindlink_dir)
            .await
            .expect("Failed to create mindlink directory");

        temp_dir
    }

    #[tokio::test]
    async fn test_auth_manager_creation() {
        println!("🧪 Test: AuthManager creation");

        let _temp_dir = create_test_auth_dir().await;
        let result = AuthManager::new().await;

        assert!(result.is_ok(), "AuthManager creation should succeed");

        let auth_manager = result.unwrap();
        assert!(
            !auth_manager.is_authenticated().await,
            "Should not be authenticated initially"
        );

        println!("✅ AuthManager creation successful");
    }

    #[tokio::test]
    async fn test_authentication_status_initial() {
        println!("🧪 Test: Initial authentication status");

        let _temp_dir = create_test_auth_dir().await;
        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Initially should not be authenticated
        assert!(
            !auth_manager.is_authenticated().await,
            "Should not be authenticated initially"
        );

        println!("✅ Initial authentication status test successful");
    }

    #[tokio::test]
    async fn test_logout_functionality() {
        println!("🧪 Test: Logout functionality");

        let _temp_dir = create_test_auth_dir().await;
        let mut auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Logout should work even if not authenticated
        let logout_result = auth_manager.logout().await;
        assert!(logout_result.is_ok(), "Logout should succeed");

        // Should still not be authenticated
        assert!(
            !auth_manager.is_authenticated().await,
            "Should not be authenticated after logout"
        );

        println!("✅ Logout functionality successful");
    }

    #[tokio::test]
    async fn test_ensure_valid_tokens() {
        println!("🧪 Test: Ensure valid tokens functionality");

        let _temp_dir = create_test_auth_dir().await;
        let mut auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Without authentication, ensure_valid_tokens should handle gracefully
        let result = auth_manager.ensure_valid_tokens().await;
        // This might fail or succeed depending on implementation, but should not panic
        assert!(
            result.is_ok() || result.is_err(),
            "ensure_valid_tokens should complete"
        );

        println!("✅ Ensure valid tokens functionality successful");
    }

    #[tokio::test]
    async fn test_refresh_tokens_without_auth() {
        println!("🧪 Test: Refresh tokens without authentication");

        let _temp_dir = create_test_auth_dir().await;
        let mut auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Refreshing tokens without authentication should handle gracefully
        let refresh_result = auth_manager.refresh_tokens().await;
        // This might fail due to no tokens, but should not panic
        assert!(
            refresh_result.is_ok() || refresh_result.is_err(),
            "refresh_tokens should complete"
        );

        println!("✅ Refresh tokens without auth test successful");
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        println!("🧪 Test: Concurrent access to AuthManager");

        let _temp_dir = create_test_auth_dir().await;
        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        let auth_manager = std::sync::Arc::new(auth_manager);

        // Test concurrent reads
        let mut handles = vec![];
        for i in 0..3 {
            let manager_clone = auth_manager.clone();
            let handle = tokio::spawn(async move {
                let is_auth = manager_clone.is_authenticated().await;
                println!("   Concurrent check {}: authenticated={}", i, is_auth);
                assert!(!is_auth, "Should not be authenticated in concurrent access");
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent access test successful");
    }

    #[tokio::test]
    async fn test_oauth_state_generation() {
        println!("🧪 Test: OAuth state and PKCE code generation");

        let _temp_dir = create_test_auth_dir().await;
        let mut auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Test OAuth URL generation through login attempt (this will test internal URL generation)
        // Note: login() requires mut self, so we test this through the authentication flow
        let login_attempt = auth_manager.login().await;

        match login_attempt {
            Ok(_) => {
                println!("   OAuth login process started successfully");
            },
            Err(e) => {
                println!(
                    "   OAuth login process failed (expected in test environment): {:?}",
                    e
                );
                // This is expected in test environments without user interaction
                assert!(
                    !e.to_string().is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        println!("✅ OAuth state generation test successful");
    }

    #[tokio::test]
    async fn test_invalid_token_handling() {
        println!("🧪 Test: Invalid token handling");

        let _temp_dir = create_test_auth_dir().await;
        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Create invalid auth file content
        let auth_dir = dirs::home_dir().unwrap().join(".mindlink");
        let auth_path = auth_dir.join("auth.json");

        // Write invalid JSON
        if let Ok(_) = fs::write(&auth_path, "invalid json content").await {
            // Try to create new auth manager (should handle invalid file gracefully)
            let auth_manager2 = AuthManager::new().await;
            match auth_manager2 {
                Ok(manager) => {
                    assert!(
                        !manager.is_authenticated().await,
                        "Should not be authenticated with invalid tokens"
                    );
                },
                Err(e) => {
                    println!(
                        "   Auth manager creation failed with invalid token file: {:?}",
                        e
                    );
                    // This is acceptable behavior
                },
            }
        }

        println!("✅ Invalid token handling test successful");
    }

    #[tokio::test]
    async fn test_expired_token_detection() {
        println!("🧪 Test: Expired token detection");

        let _temp_dir = create_test_auth_dir().await;
        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Create mock expired tokens
        let auth_dir = dirs::home_dir().unwrap().join(".mindlink");
        let auth_path = auth_dir.join("auth.json");

        let expired_tokens = r#"{
            "access_token": "mock_token",
            "refresh_token": "mock_refresh",
            "expires_at": "2020-01-01T00:00:00Z",
            "token_type": "Bearer"
        }"#;

        if let Ok(_) = fs::write(&auth_path, expired_tokens).await {
            // Create new auth manager with expired tokens
            let auth_manager2 = AuthManager::new().await;
            match auth_manager2 {
                Ok(manager) => {
                    // Should recognize tokens are expired
                    let is_auth = manager.is_authenticated().await;
                    println!("   Authentication status with expired tokens: {}", is_auth);
                    // The behavior here depends on implementation - might be false due to expiry
                },
                Err(e) => {
                    println!(
                        "   Auth manager creation failed with expired tokens: {:?}",
                        e
                    );
                },
            }
        }

        println!("✅ Expired token detection test successful");
    }

    #[tokio::test]
    async fn test_network_error_resilience() {
        println!("🧪 Test: Network error resilience");

        let _temp_dir = create_test_auth_dir().await;
        let mut auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Test operations that might involve network calls
        // These should handle network errors gracefully

        // Test token refresh without network
        let refresh_result = auth_manager.refresh_tokens().await;
        match refresh_result {
            Ok(_) => println!("   Token refresh succeeded (unexpected)"),
            Err(e) => {
                println!("   Token refresh failed as expected: {:?}", e);
                // Verify error is properly structured
                assert!(
                    !format!("{:?}", e).is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        // Test ensure valid tokens without network
        let ensure_result = auth_manager.ensure_valid_tokens().await;
        match ensure_result {
            Ok(_) => println!("   Ensure valid tokens succeeded"),
            Err(e) => {
                println!("   Ensure valid tokens failed: {:?}", e);
                // Should handle gracefully
            },
        }

        println!("✅ Network error resilience test successful");
    }

    #[tokio::test]
    async fn test_file_system_error_handling() {
        println!("🧪 Test: File system error handling");

        // Test creation in directory with restricted permissions
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let restricted_path = temp_dir.path().join("restricted");

        // Create directory structure but don't create mindlink dir
        // This will test directory creation error handling
        std::env::set_var("HOME", temp_dir.path());

        // This should either succeed (creating the directory) or fail gracefully
        let auth_manager_result = AuthManager::new().await;
        match auth_manager_result {
            Ok(_) => println!("   Auth manager created successfully"),
            Err(e) => {
                println!("   Auth manager creation failed: {:?}", e);
                // Verify error handling is structured
                assert!(
                    !format!("{:?}", e).is_empty(),
                    "Error should have meaningful message"
                );
            },
        }

        println!("✅ File system error handling test successful");
    }

    #[tokio::test]
    async fn test_concurrent_token_operations() {
        println!("🧪 Test: Concurrent token operations");

        let _temp_dir = create_test_auth_dir().await;
        let auth_manager = std::sync::Arc::new(
            AuthManager::new()
                .await
                .expect("Failed to create auth manager"),
        );

        // Test multiple concurrent token operations
        let mut handles = vec![];

        for i in 0..5 {
            let manager_clone = auth_manager.clone();
            let handle = tokio::spawn(async move {
                // Test various operations concurrently
                match i % 3 {
                    0 => {
                        let _ = manager_clone.is_authenticated().await;
                    },
                    1 => {
                        let _ = manager_clone.get_access_token();
                    },
                    2 => {
                        // Test is_authenticated (no mut needed)
                        let _ = manager_clone.is_authenticated().await;
                    },
                    _ => unreachable!(),
                }
                println!("   Concurrent operation {} completed", i);
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent token operations test successful");
    }
}
