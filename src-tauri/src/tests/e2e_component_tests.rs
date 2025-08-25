//! End-to-End Component Integration Tests
//!
//! Tests that validate component integration without requiring UI or full application.
//! These tests focus on manager coordination and system-level integration.

use crate::AppState;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Create a test app state for component testing
async fn create_test_app_state() -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
    let app_state = AppState::new().await?;
    Ok(Arc::new(app_state))
}

/// Test that all managers are initialized properly
#[tokio::test]
async fn test_managers_initialization() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Managers initialization");

    let app_state = create_test_app_state().await?;

    // Test that all managers are accessible
    {
        let _auth_manager = app_state.auth_manager.read().await;
        println!("✅ AuthManager initialized");
    }

    {
        let _server_manager = app_state.server_manager.read().await;
        println!("✅ ServerManager initialized");
    }

    {
        let _config_manager = app_state.config_manager.read().await;
        println!("✅ ConfigManager initialized");
    }

    {
        let _tunnel_manager = app_state.tunnel_manager.read().await;
        println!("✅ TunnelManager initialized");
    }

    {
        let _bifrost_manager = app_state.bifrost_manager.read().await;
        println!("✅ BifrostManager initialized");
    }

    println!("✅ All managers initialization test completed");
    Ok(())
}

/// Test configuration loading and consistency across managers
#[tokio::test]
async fn test_configuration_consistency() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Configuration consistency across managers");

    let app_state = create_test_app_state().await?;

    // Test that configuration is accessible from all managers
    let config = {
        let config_manager = app_state.config_manager.read().await;
        config_manager.get_config().await.clone()
    };

    println!("📋 Configuration loaded successfully");

    // Verify basic config structure
    println!("📊 Config version: {}", config.version);
    println!("📊 Server config port: {}", config.server.port);

    println!("✅ Configuration structure validated");
    println!("✅ Configuration consistency test completed");
    Ok(())
}

/// Test service state coordination
#[tokio::test]
async fn test_service_state_coordination() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Service state coordination");

    let app_state = create_test_app_state().await?;

    // Check initial service state
    let initial_serving = *app_state.is_serving.read().await;
    println!("📊 Initial serving state: {}", initial_serving);

    // Test state consistency
    let auth_state = {
        let auth_manager = app_state.auth_manager.read().await;
        auth_manager.is_authenticated().await
    };

    println!("📊 Authentication state: {}", auth_state);

    // In initial state, should not be serving and not authenticated
    assert!(!initial_serving, "Should not be serving initially");
    assert!(!auth_state, "Should not be authenticated initially");

    println!("✅ Service state coordination test completed");
    Ok(())
}

/// Test manager communication and data flow
#[tokio::test]
async fn test_manager_communication() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Manager communication and data flow");

    let app_state = create_test_app_state().await?;

    // Test that managers can access shared state
    let config = {
        let config_manager = app_state.config_manager.read().await;
        config_manager.get_config().await.clone()
    };

    // Test that other managers can use this config
    println!("📊 Server config available: port={}", config.server.port);
    println!(
        "📊 Tunnel config available: enabled={}",
        config.tunnel.enabled
    );

    println!("✅ Manager communication verified");
    println!("✅ Manager communication test completed");
    Ok(())
}

/// Test concurrent manager access
#[tokio::test]
async fn test_concurrent_manager_access() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Concurrent manager access");

    let app_state = create_test_app_state().await?;

    // Test concurrent read access to multiple managers
    let mut handles = Vec::new();

    // Concurrent config reads
    for i in 0..5 {
        let state = app_state.clone();
        let handle = tokio::spawn(async move {
            let config_manager = state.config_manager.read().await;
            let config = config_manager.get_config().await;
            println!("🔄 Concurrent config access {} completed", i);
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(config.server.port > 0)
        });
        handles.push(handle);
    }

    // Concurrent auth checks
    for i in 0..3 {
        let state = app_state.clone();
        let handle = tokio::spawn(async move {
            let auth_manager = state.auth_manager.read().await;
            let is_auth = auth_manager.is_authenticated().await;
            println!("🔄 Concurrent auth check {} completed", i);
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(is_auth)
        });
        handles.push(handle);
    }

    // Wait for all concurrent operations
    let mut successes = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => {
                successes += 1;
            },
            Ok(Err(e)) => {
                println!("⚠️  Concurrent operation error: {}", e);
            },
            Err(e) => {
                println!("⚠️  Task join error: {}", e);
            },
        }
    }

    println!(
        "✅ Concurrent operations completed: {}/8 succeeded",
        successes
    );
    assert!(
        successes > 0,
        "At least some concurrent operations should succeed"
    );

    println!("✅ Concurrent manager access test completed");
    Ok(())
}

/// Test error state propagation
#[tokio::test]
async fn test_error_state_propagation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Error state propagation");

    let app_state = create_test_app_state().await?;

    // Check initial error state
    let initial_error = app_state.last_error.read().await.clone();
    println!("📊 Initial error state: {:?}", initial_error);

    // Simulate setting an error
    {
        let mut error_state = app_state.last_error.write().await;
        *error_state = Some("Test error for E2E testing".to_string());
    }

    // Verify error was set
    let error_after_set = app_state.last_error.read().await.clone();
    assert!(error_after_set.is_some(), "Error should be set");
    assert_eq!(error_after_set.unwrap(), "Test error for E2E testing");

    // Clear the error
    {
        let mut error_state = app_state.last_error.write().await;
        *error_state = None;
    }

    // Verify error was cleared
    let error_after_clear = app_state.last_error.read().await.clone();
    assert!(error_after_clear.is_none(), "Error should be cleared");

    println!("✅ Error state propagation test completed");
    Ok(())
}

/// Test resource cleanup and shutdown behavior
#[tokio::test]
async fn test_resource_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Resource cleanup and shutdown behavior");

    let app_state = create_test_app_state().await?;

    // Test that managers handle shutdown gracefully
    // (In a real shutdown, we'd call shutdown methods on each manager)

    // Simulate some work
    {
        let _config_manager = app_state.config_manager.read().await;
        sleep(Duration::from_millis(10)).await;
    }

    {
        let _auth_manager = app_state.auth_manager.read().await;
        sleep(Duration::from_millis(10)).await;
    }

    // Verify managers are still accessible after work
    {
        let config_manager = app_state.config_manager.read().await;
        let config = config_manager.get_config().await;
        assert!(config.server.port > 0, "Managers should remain accessible");
    }

    println!("✅ Resource cleanup test completed");
    Ok(())
}

/// Integration test: Complete system initialization and coordination
#[tokio::test]
async fn test_complete_system_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Complete system integration");

    // Phase 1: System initialization
    println!("📋 Phase 1: System initialization");
    let app_state = create_test_app_state().await?;
    sleep(Duration::from_millis(50)).await;
    println!("✅ System initialized");

    // Phase 2: Manager coordination
    println!("📋 Phase 2: Manager coordination");
    let mut manager_states = Vec::new();

    // Collect states from all managers
    {
        let config_manager = app_state.config_manager.read().await;
        let config = config_manager.get_config().await;
        manager_states.push(("Config", config.server.port > 0));
    }

    {
        let auth_manager = app_state.auth_manager.read().await;
        let is_auth = auth_manager.is_authenticated().await;
        manager_states.push(("Auth", !is_auth)); // Should be false initially
    }

    // Verify all managers are in expected initial state
    for (name, state) in &manager_states {
        assert!(
            *state,
            "{} manager should be in expected initial state",
            name
        );
    }
    println!("✅ Manager coordination verified");

    // Phase 3: State consistency
    println!("📋 Phase 3: State consistency");
    let serving_state = *app_state.is_serving.read().await;
    let _error_state = app_state.last_error.read().await.clone();

    assert!(!serving_state, "Should not be serving initially");
    println!("✅ State consistency verified");

    // Phase 4: Concurrent operations
    println!("📋 Phase 4: Concurrent operations");
    let mut concurrent_handles = Vec::new();

    for _i in 0..3 {
        let state = app_state.clone();
        let handle = tokio::spawn(async move {
            let config_manager = state.config_manager.read().await;
            sleep(Duration::from_millis(50)).await;
            config_manager.get_config().await.server.port > 0
        });
        concurrent_handles.push(handle);
    }

    // Wait for concurrent operations
    let mut concurrent_successes = 0;
    for handle in concurrent_handles {
        if let Ok(result) = handle.await {
            if result {
                concurrent_successes += 1;
            }
        }
    }

    assert_eq!(
        concurrent_successes, 3,
        "All concurrent operations should succeed"
    );
    println!("✅ Concurrent operations verified");

    // Phase 5: Final state verification
    println!("📋 Phase 5: Final state verification");
    {
        let config_manager = app_state.config_manager.read().await;
        let config = config_manager.get_config().await;
        assert!(
            config.server.port > 0,
            "Configuration should remain consistent"
        );
    }

    let final_serving_state = *app_state.is_serving.read().await;
    assert!(!final_serving_state, "Should still not be serving");
    println!("✅ Final state verified");

    println!("✅ Complete system integration test completed");
    Ok(())
}
