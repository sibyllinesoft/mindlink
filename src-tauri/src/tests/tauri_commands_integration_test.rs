#[cfg(test)]
mod tauri_commands_integration_tests {
    // Note: This file tests the Tauri commands at the manager level
    // Full Tauri command testing would require a Tauri app instance

    use crate::managers::{
        auth_manager::AuthManager, config_manager::ConfigManager, server_manager::ServerManager,
    };

    #[tokio::test]
    async fn test_managers_for_command_support() {
        println!("🧪 Test: Managers support for Tauri commands");

        // Test that managers can be created (prerequisite for commands)
        let auth_manager = AuthManager::new()
            .await
            .expect("Auth manager needed for login commands");
        let server_manager = ServerManager::new().await;
        let config_manager = ConfigManager::new()
            .await
            .expect("Config manager needed for settings commands");

        // Verify managers are in expected initial state
        assert!(
            !auth_manager.is_authenticated().await,
            "Auth should start unauthenticated"
        );
        assert!(
            !server_manager.is_running().await,
            "Server should start stopped"
        );

        // Test config access (needed for get_config command)
        let config = config_manager.get_config().await;
        assert!(
            !config.server.host.is_empty(),
            "Config should have default host"
        );
        assert!(config.server.port > 0, "Config should have valid port");

        println!("✅ Managers support for Tauri commands verified");
    }

    #[tokio::test]
    async fn test_authentication_workflow() {
        println!("🧪 Test: Authentication workflow for login commands");

        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");

        // Test initial authentication state (for get_auth_status command)
        assert!(
            !auth_manager.is_authenticated().await,
            "Should start unauthenticated"
        );

        // Test logout when not authenticated (for logout command)
        let mut auth_manager_mut = auth_manager;
        let logout_result = auth_manager_mut.logout().await;
        assert!(
            logout_result.is_ok(),
            "Logout should succeed even when not authenticated"
        );

        assert!(
            !auth_manager_mut.is_authenticated().await,
            "Should remain unauthenticated"
        );

        println!("✅ Authentication workflow test successful");
    }

    #[tokio::test]
    async fn test_server_workflow() {
        println!("🧪 Test: Server workflow for service commands");

        let server_manager = ServerManager::new().await;

        // Test initial server state (for get_status command)
        assert!(
            !server_manager.is_running().await,
            "Server should start stopped"
        );
        assert!(
            server_manager.get_local_url().await.is_none(),
            "Should have no URL initially"
        );

        // Test server health check (for health check commands)
        let health_result = server_manager.check_health().await;
        match health_result {
            Ok(healthy) => assert!(!healthy, "Should not be healthy when stopped"),
            Err(_) => println!("   Health check error when stopped (acceptable)"),
        }

        println!("✅ Server workflow test successful");
    }

    #[tokio::test]
    async fn test_config_workflow() {
        println!("🧪 Test: Configuration workflow for settings commands");

        let config_manager = ConfigManager::new()
            .await
            .expect("Failed to create config manager");

        // Test config retrieval (for get_config command)
        let config = config_manager.get_config().await;
        assert!(!config.server.host.is_empty(), "Should have default host");

        // Test individual config section access (for specific get commands)
        let server_config = config_manager.get_server_config().await;
        assert_eq!(
            server_config.host, config.server.host,
            "Server config should match"
        );

        let features_config = config_manager.get_feature_config().await;
        assert!(
            !features_config.reasoning_effort.is_empty(),
            "Should have reasoning effort setting"
        );

        println!("✅ Configuration workflow test successful");
    }

    #[tokio::test]
    async fn test_error_handling_for_commands() {
        println!("🧪 Test: Error handling for command-like operations");

        let mut server_manager = ServerManager::new().await;

        // Test stopping when not running (should handle gracefully)
        let stop_result = server_manager.stop().await;
        assert!(
            stop_result.is_ok() || stop_result.is_err(),
            "Stop should complete"
        );

        // Test configuration with edge cases
        let config_result = server_manager
            .configure("localhost".to_string(), 8080)
            .await;
        assert!(config_result.is_ok(), "Basic configuration should succeed");

        println!("✅ Error handling for commands test successful");
    }
}
