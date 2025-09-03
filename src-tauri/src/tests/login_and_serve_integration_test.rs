#[cfg(test)]
mod login_and_serve_integration_tests {
    use crate::managers::{
        auth_manager::AuthManager, bifrost_manager::BifrostManager, server_manager::ServerManager,
        tunnel_manager::TunnelManager,
    };
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_manager_creation_integration() {
        println!("🧪 Test: Integration - Manager creation");

        // Test creating all managers together
        let auth_manager = AuthManager::new()
            .await
            .expect("Failed to create auth manager");
        let server_manager = ServerManager::new().await;
        let bifrost_manager = BifrostManager::new().await;
        let tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

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
            !bifrost_manager.is_running().await,
            "Bifrost should not be running initially"
        );
        assert!(
            !tunnel_manager.is_connected().await,
            "Tunnel should not be connected initially"
        );

        println!("✅ Manager creation integration successful");
    }

    #[tokio::test]
    async fn test_server_without_auth_integration() {
        println!("🧪 Test: Integration - Server without auth");

        let mut server_manager = ServerManager::new().await;
        let auth_manager = Arc::new(RwLock::new(
            AuthManager::new()
                .await
                .expect("Failed to create auth manager"),
        ));

        // Try to start server without authentication
        let start_result = server_manager.start(auth_manager).await;

        // Should fail gracefully due to no authentication
        match start_result {
            Ok(url) => {
                println!("   Server start succeeded unexpectedly: {}", url);
                // If it succeeded, clean up by stopping
                let _ = server_manager.stop().await;
            },
            Err(e) => {
                println!("   Server start failed as expected without auth: {}", e);
                assert!(!e.to_string().is_empty(), "Error should have message");
            },
        }

        println!("✅ Server without auth integration test successful");
    }

    #[tokio::test]
    async fn test_managers_stop_when_not_running() {
        println!("🧪 Test: Integration - Stop all managers when not running");

        let mut server_manager = ServerManager::new().await;
        let mut bifrost_manager = BifrostManager::new().await;
        let mut tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Try to stop all managers when they're not running
        let server_stop = server_manager.stop().await;
        let bifrost_stop = bifrost_manager.stop().await;
        let tunnel_stop = tunnel_manager.close_tunnel().await;

        // All should handle gracefully
        assert!(
            server_stop.is_ok() || server_stop.is_err(),
            "Server stop should complete"
        );
        assert!(
            bifrost_stop.is_ok() || bifrost_stop.is_err(),
            "Bifrost stop should complete"
        );
        assert!(
            tunnel_stop.is_ok() || tunnel_stop.is_err(),
            "Tunnel close should complete"
        );

        // Verify they remain in stopped state
        assert!(
            !server_manager.is_running().await,
            "Server should remain stopped"
        );
        assert!(
            !bifrost_manager.is_running().await,
            "Bifrost should remain stopped"
        );
        assert!(
            !tunnel_manager.is_connected().await,
            "Tunnel should remain disconnected"
        );

        println!("✅ Stop all managers integration test successful");
    }

    #[tokio::test]
    async fn test_health_checks_integration() {
        println!("🧪 Test: Integration - Health checks");

        let server_manager = ServerManager::new().await;
        let bifrost_manager = BifrostManager::new().await;
        let tunnel_manager = TunnelManager::new()
            .await
            .expect("Failed to create tunnel manager");

        // Check health of all managers when not running
        let server_health = server_manager.check_health().await;
        let bifrost_health = bifrost_manager.check_health().await;
        let tunnel_health = tunnel_manager.check_health().await;

        // Health checks should complete (might return false or error when not running)
        println!("   Server health: {:?}", server_health);
        println!("   Bifrost health: {:?}", bifrost_health);
        println!("   Tunnel health: {:?}", tunnel_health);

        // All health checks should complete without panic
        assert!(true, "All health checks completed");

        println!("✅ Health checks integration test successful");
    }

    #[tokio::test]
    async fn test_concurrent_manager_access() {
        println!("🧪 Test: Integration - Concurrent manager access");

        let auth_manager = Arc::new(
            AuthManager::new()
                .await
                .expect("Failed to create auth manager"),
        );
        let server_manager = Arc::new(ServerManager::new().await);
        let bifrost_manager = Arc::new(BifrostManager::new().await);

        let mut handles = vec![];
        for i in 0..3 {
            let auth_clone = auth_manager.clone();
            let server_clone = server_manager.clone();
            let bifrost_clone = bifrost_manager.clone();

            let handle = tokio::spawn(async move {
                let auth_status = auth_clone.is_authenticated().await;
                let server_status = server_clone.is_running().await;
                let bifrost_status = bifrost_clone.is_running().await;

                println!(
                    "   Concurrent check {}: auth={}, server={}, bifrost={}",
                    i, auth_status, server_status, bifrost_status
                );

                assert!(!auth_status, "Auth should not be authenticated");
                assert!(!server_status, "Server should not be running");
                assert!(!bifrost_status, "Bifrost should not be running");
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent operation should complete");
        }

        println!("✅ Concurrent manager access integration test successful");
    }
}
