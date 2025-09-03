//! Local LLM Commands Tests
//! 
//! Tests for the local LLM management commands (Ollama and Llama.cpp).
//! These tests verify the commands work correctly without requiring the actual services to be running.

use crate::commands::*;
use std::sync::Arc;

/// Test the check_ollama_status command when Ollama is not running
#[tokio::test]
async fn test_check_ollama_status_not_running() {
    println!("🧪 Testing: check_ollama_status when service is not running");
    
    // This test assumes Ollama is not running on port 11434
    let result = check_ollama_status().await;
    
    match result {
        Ok(status) => {
            println!("✅ check_ollama_status returned successfully");
            
            // Should indicate service is not running
            assert!(!status.running, "Status should indicate Ollama is not running");
            assert!(status.version.is_none(), "Version should be None when not running");
            assert!(status.models.is_empty(), "Models should be empty when not running");
            
            println!("✅ Status structure validated for non-running service");
        }
        Err(e) => {
            panic!("check_ollama_status should not return error, got: {}", e);
        }
    }
    
    println!("✅ check_ollama_status test completed");
}

/// Test the check_llamacpp_status command when Llama.cpp is not running
#[tokio::test]
async fn test_check_llamacpp_status_not_running() {
    println!("🧪 Testing: check_llamacpp_status when service is not running");
    
    // This test assumes Llama.cpp is not running on port 8080
    let result = check_llamacpp_status().await;
    
    match result {
        Ok(running) => {
            println!("✅ check_llamacpp_status returned successfully");
            
            // Should indicate service is not running
            assert!(!running, "Should indicate Llama.cpp is not running");
            
            println!("✅ Status validated for non-running service");
        }
        Err(e) => {
            panic!("check_llamacpp_status should not return error, got: {}", e);
        }
    }
    
    println!("✅ check_llamacpp_status test completed");
}

/// Test the get_ollama_models command when Ollama is not running
#[tokio::test]
async fn test_get_ollama_models_not_running() {
    println!("🧪 Testing: get_ollama_models when service is not running");
    
    // This test assumes Ollama is not running
    let result = get_ollama_models().await;
    
    match result {
        Ok(_) => {
            // Should not succeed if Ollama is not running
            panic!("get_ollama_models should fail if Ollama is not running");
        }
        Err(error) => {
            println!("✅ get_ollama_models properly failed with error: {}", error);
            
            // Should get a connection error
            assert!(
                error.contains("Failed to connect to Ollama") || 
                error.contains("Ollama API returned error"),
                "Error message should indicate connection failure, got: {}", 
                error
            );
            
            println!("✅ Error message validated");
        }
    }
    
    println!("✅ get_ollama_models test completed");
}

/// Test start_ollama_service command error handling
#[tokio::test]
async fn test_start_ollama_service_error_handling() {
    println!("🧪 Testing: start_ollama_service error handling");
    
    // This test assumes 'ollama' command is not in PATH on test system
    let result = start_ollama_service().await;
    
    match result {
        Ok(response) => {
            println!("✅ start_ollama_service returned ServiceResponse");
            
            // Should have success field
            if response.success {
                println!("✅ Service started successfully (ollama was available)");
                assert!(response.message.is_some(), "Should have a success message");
                assert_eq!(response.server_url, Some("http://localhost:11434".to_string()), 
                    "Should return correct server URL");
            } else {
                println!("✅ Service failed to start (expected if ollama not installed)");
                assert!(response.message.is_some(), "Should have an error message");
                assert!(response.message.unwrap().contains("Failed to start Ollama"), 
                    "Error message should mention Ollama failure");
            }
        }
        Err(e) => {
            panic!("start_ollama_service should return ServiceResponse, not error: {}", e);
        }
    }
    
    println!("✅ start_ollama_service test completed");
}

/// Test stop_ollama_service command
#[tokio::test]
async fn test_stop_ollama_service() {
    println!("🧪 Testing: stop_ollama_service");
    
    let result = stop_ollama_service().await;
    
    match result {
        Ok(response) => {
            println!("✅ stop_ollama_service returned ServiceResponse");
            
            // Should succeed even if nothing to stop
            assert!(response.success, "Should succeed even if service wasn't running");
            assert!(response.message.is_some(), "Should have a message");
            
            // Message should indicate either stopped or wasn't running
            let message = response.message.unwrap();
            assert!(
                message.contains("stopped") || message.contains("not running"),
                "Message should indicate stop status, got: {}",
                message
            );
            
            println!("✅ Response validated");
        }
        Err(e) => {
            panic!("stop_ollama_service should not return error: {}", e);
        }
    }
    
    println!("✅ stop_ollama_service test completed");
}

/// Test check_bifrost_llm_provider command when Bifrost is not running
#[tokio::test]
async fn test_check_bifrost_llm_provider_not_running() {
    println!("🧪 Testing: check_bifrost_llm_provider when Bifrost is not running");
    
    let result = check_bifrost_llm_provider(
        "ollama".to_string(),
        "http://localhost:11434".to_string(),
    ).await;
    
    match result {
        Ok(configured) => {
            println!("✅ check_bifrost_llm_provider returned successfully");
            
            // Should indicate provider is not configured (since Bifrost isn't running)
            assert!(!configured, "Should indicate provider is not configured when Bifrost is not accessible");
            
            println!("✅ Configuration status validated");
        }
        Err(e) => {
            panic!("check_bifrost_llm_provider should not return error: {}", e);
        }
    }
    
    println!("✅ check_bifrost_llm_provider test completed");
}

/// Test configure_bifrost_llm_provider command (placeholder implementation)
#[tokio::test]
async fn test_configure_bifrost_llm_provider() {
    println!("🧪 Testing: configure_bifrost_llm_provider");
    
    let result = configure_bifrost_llm_provider(
        "ollama".to_string(),
        "http://localhost:11434".to_string(),
        "Ollama".to_string(),
    ).await;
    
    match result {
        Ok(()) => {
            println!("✅ configure_bifrost_llm_provider succeeded (placeholder implementation)");
        }
        Err(e) => {
            panic!("configure_bifrost_llm_provider should not fail in placeholder implementation: {}", e);
        }
    }
    
    println!("✅ configure_bifrost_llm_provider test completed");
}

/// Test data structure serialization/deserialization
#[test]
fn test_ollama_status_response_serialization() {
    println!("🧪 Testing: OllamaStatusResponse serialization");
    
    let status = OllamaStatusResponse {
        running: true,
        version: Some("0.1.0".to_string()),
        models: vec!["llama2".to_string(), "codellama".to_string()],
    };
    
    // Test serialization
    let json = serde_json::to_string(&status).expect("Should serialize successfully");
    println!("✅ Serialized to JSON: {}", json);
    
    // Test deserialization
    let deserialized: OllamaStatusResponse = serde_json::from_str(&json)
        .expect("Should deserialize successfully");
    
    assert_eq!(deserialized.running, status.running);
    assert_eq!(deserialized.version, status.version);
    assert_eq!(deserialized.models, status.models);
    
    println!("✅ Serialization/deserialization validated");
}

/// Test OllamaModel structure
#[test]
fn test_ollama_model_serialization() {
    println!("🧪 Testing: OllamaModel serialization");
    
    let model = OllamaModel {
        name: "llama2".to_string(),
        size: 3825819519,
        modified: "2023-12-07T09:32:18.757212583Z".to_string(),
        digest: "sha256:78e26419b4469263f75331927a00a0284ef6544c1975b826b15abdaef17bb962".to_string(),
        details: Some(serde_json::json!({
            "family": "llama",
            "parameter_size": "7B",
            "quantization_level": "Q4_0"
        })),
    };
    
    // Test serialization
    let json = serde_json::to_string(&model).expect("Should serialize successfully");
    println!("✅ Serialized OllamaModel to JSON");
    
    // Test deserialization
    let deserialized: OllamaModel = serde_json::from_str(&json)
        .expect("Should deserialize successfully");
    
    assert_eq!(deserialized.name, model.name);
    assert_eq!(deserialized.size, model.size);
    assert_eq!(deserialized.modified, model.modified);
    assert_eq!(deserialized.digest, model.digest);
    assert_eq!(deserialized.details, model.details);
    
    println!("✅ OllamaModel serialization/deserialization validated");
}