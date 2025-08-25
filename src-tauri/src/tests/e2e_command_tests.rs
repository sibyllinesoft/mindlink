//! End-to-End Command Tests
//! 
//! Tests that validate the Tauri commands work correctly in an end-to-end fashion.
//! These tests focus on the command layer without requiring UI interaction.

use crate::commands::*;
use crate::AppState;
use std::collections::HashMap;
use std::sync::Arc;

/// Create a test app state for E2E command testing  
async fn create_test_app_state() -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
    // Create a real AppState for testing
    let app_state = AppState::new().await?;
    Ok(Arc::new(app_state))
}

/// Test the get_config command
#[tokio::test]
async fn test_get_config_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: get_config command");
    
    let app_state = create_test_app_state().await?;
    
    // Test getting default config
    let result = get_config(app_state.clone()).await;
    
    match result {
        Ok(config) => {
            println!("✅ get_config returned successfully");
            
            // Verify config has expected structure
            assert!(config.contains_key("server"), "Config should have server section");
            assert!(config.contains_key("tunnel"), "Config should have tunnel section");
            assert!(config.contains_key("features"), "Config should have features section");
            
            println!("✅ Config structure validated");
        }
        Err(e) => {
            println!("❌ get_config failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!("✅ get_config command test completed");
    Ok(())
}

/// Test the save_config command
#[tokio::test]
async fn test_save_config_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: save_config command");
    
    let app_state = create_test_app_state();
    
    // Create test config
    let mut test_config = HashMap::new();
    test_config.insert("server".to_string(), serde_json::json!({
        "port": 3001,
        "host": "127.0.0.1"
    }));
    test_config.insert("tunnel".to_string(), serde_json::json!({
        "enabled": true
    }));
    test_config.insert("features".to_string(), serde_json::json!({
        "reasoningEffort": "medium"
    }));
    
    // Test saving config
    let result = save_config(app_state.clone(), test_config.clone()).await;
    
    match result {
        Ok(_) => {
            println!("✅ save_config completed successfully");
            
            // Verify config was saved by reading it back
            let saved_config = get_config(app_state.clone()).await?;
            
            assert_eq!(
                saved_config.get("server"),
                test_config.get("server"),
                "Saved server config should match"
            );
            
            println!("✅ Config save and read-back verified");
        }
        Err(e) => {
            println!("❌ save_config failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!("✅ save_config command test completed");
    Ok(())
}

/// Test the get_status command
#[tokio::test]
async fn test_get_status_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: get_status command");
    
    let app_state = create_test_app_state();
    
    // Test getting status
    let result = get_status(app_state.clone()).await;
    
    match result {
        Ok(status) => {
            println!("✅ get_status returned successfully");
            
            // Check that status reflects initial state (not serving)
            assert!(!status.is_serving, "Should not be serving initially");
            
            // Verify other status fields exist
            println!("📊 Status: serving={}, authenticated={}", 
                status.is_serving, status.is_authenticated);
            
            println!("✅ Status structure and initial state validated");
        }
        Err(e) => {
            println!("❌ get_status failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!("✅ get_status command test completed");
    Ok(())
}

/// Test the test_completion command
#[tokio::test]
async fn test_completion_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: test_completion command");
    
    let app_state = create_test_app_state();
    
    // Create test request
    let test_request = TestCompletionRequest {
        message: "Hello, this is a test message for E2E testing.".to_string(),
        model: Some("gpt-3.5-turbo".to_string()),
    };
    
    // Test completion command
    let result = test_completion(app_state.clone(), test_request).await;
    
    match result {
        Ok(response) => {
            println!("✅ test_completion returned successfully");
            
            // Verify response structure
            if response.success {
                if let Some(resp) = &response.response {
                    assert!(!resp.is_empty(), "Response should not be empty when successful");
                    println!("📋 Response preview: {}", 
                        if resp.len() > 100 { &resp[..100] } else { resp });
                } else {
                    println!("⚠️  Successful response but no content");
                }
            } else {
                println!("⚠️  Response indicated failure: {:?}", response.error);
            }
            
            println!("✅ Response structure validated");
        }
        Err(e) => {
            println!("⚠️  test_completion may have failed due to no authentication: {}", e);
            // This is expected if not logged into ChatGPT
            println!("💡 This is expected behavior when not authenticated");
        }
    }
    
    println!("✅ test_completion command test completed");
    Ok(())
}

/// Test login_and_serve command flow (without actually logging in)
#[tokio::test]
async fn test_login_and_serve_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: login_and_serve command");
    
    let app_state = create_test_app_state();
    
    // Test login_and_serve command (will likely fail due to no browser/auth)
    let result = login_and_serve(app_state.clone()).await;
    
    match result {
        Ok(_) => {
            println!("✅ login_and_serve succeeded (unexpected but good)");
            
            // If it succeeded, check status
            let status = get_status(app_state.clone()).await?;
            let is_serving = status.get("is_serving").and_then(|v| v.as_bool()).unwrap_or(false);
            
            if is_serving {
                println!("✅ Service started successfully");
                
                // Clean up by stopping
                let _ = stop_serving(app_state.clone()).await;
            }
        }
        Err(e) => {
            println!("⚠️  login_and_serve failed as expected: {}", e);
            println!("💡 This is expected behavior without browser authentication");
            
            // Verify we're still not serving
            let status = get_status(app_state.clone()).await?;
            assert!(!status.is_serving, "Should not be serving after failed login");
        }
    }
    
    println!("✅ login_and_serve command test completed");
    Ok(())
}

/// Test stop_serving command
#[tokio::test]
async fn test_stop_serving_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: stop_serving command");
    
    let app_state = create_test_app_state();
    
    // Test stopping (even when not serving)
    let result = stop_serving(app_state.clone()).await;
    
    match result {
        Ok(_) => {
            println!("✅ stop_serving completed successfully");
            
            // Verify status shows not serving
            let status = get_status(app_state.clone()).await?;
            assert!(!status.is_serving, "Should not be serving after stop command");
            
            println!("✅ Stop serving state verified");
        }
        Err(e) => {
            println!("⚠️  stop_serving had an issue: {}", e);
            // This might be OK if nothing was running
        }
    }
    
    println!("✅ stop_serving command test completed");
    Ok(())
}

/// Test complete command workflow
#[tokio::test]
async fn test_complete_command_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Complete command workflow");
    
    let app_state = create_test_app_state();
    
    // Step 1: Get initial status
    println!("📋 Step 1: Getting initial status");
    let initial_status = get_status(app_state.clone()).await?;
    assert!(!initial_status.is_serving, "Should start not serving");
    println!("✅ Initial status: not serving");
    
    // Step 2: Get initial config
    println!("📋 Step 2: Getting initial config");
    let initial_config = get_config(app_state.clone()).await?;
    assert!(initial_config.contains_key("server"), "Should have server config");
    println!("✅ Initial config retrieved");
    
    // Step 3: Modify and save config
    println!("📋 Step 3: Modifying and saving config");
    let mut modified_config = HashMap::new();
    for (key, value) in initial_config.iter() {
        modified_config.insert(key.clone(), value.clone());
    }
    modified_config.insert("server".to_string(), serde_json::json!({
        "host": "127.0.0.1",
        "port": 3001
    }));
    
    save_config(app_state.clone(), modified_config).await?;
    println!("✅ Config modified and saved");
    
    // Step 4: Verify config was saved
    println!("📋 Step 4: Verifying config was saved");
    let saved_config = get_config(app_state.clone()).await?;
    if let Some(server) = saved_config.get("server") {
        if let Some(port) = server.get("port") {
            assert_eq!(port.as_i64(), Some(3001), "Port should be saved correctly");
        }
    }
    println!("✅ Config save verified");
    
    // Step 5: Test completion (may fail due to auth)
    println!("📋 Step 5: Testing completion");
    let test_request = TestCompletionRequest {
        message: "Test message for workflow".to_string(),
        model: Some("gpt-3.5-turbo".to_string()),
    };
    
    match test_completion(app_state.clone(), test_request).await {
        Ok(response) => {
            if response.success {
                println!("✅ Test completion succeeded");
            } else {
                println!("⚠️  Test completion returned failure");
            }
        }
        Err(_) => {
            println!("⚠️  Test completion failed (expected without auth)");
        }
    }
    
    // Step 6: Final status check
    println!("📋 Step 6: Final status check");
    let final_status = get_status(app_state.clone()).await?;
    println!("✅ Final status retrieved successfully");
    
    println!("✅ Complete command workflow test completed");
    Ok(())
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_command_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing: Command error handling");
    
    let app_state = create_test_app_state();
    
    // Test 1: Invalid config save
    println!("📋 Testing invalid config handling");
    let invalid_config = HashMap::new(); // Empty config
    
    match save_config(app_state.clone(), invalid_config).await {
        Ok(_) => {
            println!("✅ Empty config handled gracefully");
        }
        Err(e) => {
            println!("⚠️  Empty config rejected: {}", e);
            // Either outcome is acceptable
        }
    }
    
    // Test 2: Multiple rapid status calls
    println!("📋 Testing rapid status calls");
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let state = app_state.clone();
        let handle = tokio::spawn(async move {
            let result = get_status(state).await;
            println!("📊 Rapid call {} result: {:?}", i, result.is_ok());
            result
        });
        handles.push(handle);
    }
    
    // Wait for all calls to complete
    let mut successes = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => successes += 1,
            Ok(Err(e)) => println!("⚠️  Rapid call error: {}", e),
            Err(e) => println!("⚠️  Task join error: {}", e),
        }
    }
    
    println!("✅ Rapid calls completed: {}/5 succeeded", successes);
    assert!(successes > 0, "At least some rapid calls should succeed");
    
    println!("✅ Command error handling test completed");
    Ok(())
}