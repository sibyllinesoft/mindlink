#[cfg(test)]
mod live_e2e_integration_tests {
    use crate::managers::{
        bifrost_manager::BifrostManager,
        config_manager::ConfigManager,
        server_manager::ServerManager,
    };
    use reqwest;
    use serde_json::{self, Value};
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_live_bifrost_discovery_and_configuration() {
        println!("🔍 E2E Test: Live Bifrost Discovery and Configuration");
        
        // Step 1: Discover running Bifrost services
        println!("📋 Step 1: Discovering running Bifrost services on ports 3004-3009...");
        
        let mut active_bifrost_services = Vec::new();
        let client = reqwest::Client::new();
        
        for port in 3004..=3009 {
            let url = format!("http://127.0.0.1:{}", port);
            match client.get(&url).timeout(Duration::from_secs(2)).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("✅ Found active Bifrost service on port {}", port);
                        active_bifrost_services.push(port);
                        
                        // Try to get service info
                        let health_url = format!("http://127.0.0.1:{}/health", port);
                        match client.get(&health_url).send().await {
                            Ok(health_response) => {
                                if health_response.status().is_success() {
                                    println!("   💚 Health check passed for port {}", port);
                                } else {
                                    println!("   ⚠️  Health check returned: {} for port {}", health_response.status(), port);
                                }
                            },
                            Err(_) => {
                                println!("   ℹ️  No health endpoint available on port {}", port);
                            }
                        }
                    }
                },
                Err(_) => {
                    // Port not active - this is fine
                }
            }
        }
        
        println!("📊 Found {} active Bifrost services: {:?}", active_bifrost_services.len(), active_bifrost_services);
        
        // Step 2: Configure Bifrost manager with discovered services
        println!("📋 Step 2: Configuring Bifrost manager...");
        
        let config_manager = ConfigManager::new().await.expect("Failed to create config manager");
        let bifrost_config = config_manager.get_bifrost_config().await;
        
        println!("🔧 Current Bifrost configuration:");
        println!("   Host: {}", bifrost_config.host);
        println!("   Port: {}", bifrost_config.port);
        
        // Step 3: Test Bifrost manager integration
        println!("📋 Step 3: Testing Bifrost manager integration...");
        
        let bifrost_manager = BifrostManager::new().await;
        
        println!("📊 Bifrost Manager Status:");
        println!("   Running: {}", bifrost_manager.is_running().await);
        println!("   Binary Available: {}", bifrost_manager.is_binary_available().await);
        println!("   Local URL: {:?}", bifrost_manager.get_local_url().await);
        println!("   Should Build: {}", bifrost_manager.should_build().await);
        
        // Step 4: Try to interact with discovered Bifrost services
        if !active_bifrost_services.is_empty() {
            println!("📋 Step 4: Attempting to interact with live Bifrost services...");
            
            for &port in &active_bifrost_services {
                println!("🔍 Testing Bifrost service on port {}:", port);
                
                // Try to get models from this Bifrost instance
                let models_url = format!("http://127.0.0.1:{}/v1/models", port);
                match client.get(&models_url).send().await {
                    Ok(response) => {
                        println!("   📊 Models endpoint status: {}", response.status());
                        
                        if response.status().is_success() {
                            match response.text().await {
                                Ok(body) => {
                                    if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                        println!("   📋 Models response: {}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Invalid JSON".to_string()));
                                    } else {
                                        println!("   📋 Raw response: {}", body);
                                    }
                                },
                                Err(e) => println!("   ❌ Failed to read response body: {}", e),
                            }
                        } else {
                            println!("   ⚠️  Models endpoint returned: {}", response.status());
                        }
                    },
                    Err(e) => {
                        println!("   ❌ Failed to connect to models endpoint: {}", e);
                    }
                }
                
                // Try to get config from this Bifrost instance
                let config_url = format!("http://127.0.0.1:{}/config", port);
                match client.get(&config_url).send().await {
                    Ok(response) => {
                        println!("   🔧 Config endpoint status: {}", response.status());
                    },
                    Err(_) => {
                        println!("   ℹ️  No config endpoint available");
                    }
                }
            }
        }
        
        println!("✅ Live Bifrost discovery and configuration test completed!");
        assert!(!active_bifrost_services.is_empty(), "Should have found at least one active Bifrost service");
    }

    #[tokio::test]
    async fn test_live_ollama_integration_and_models() {
        println!("🤖 E2E Test: Live Ollama Integration and Model Discovery");
        
        // Step 1: Check if Ollama is available
        println!("📋 Step 1: Checking Ollama availability...");
        
        let client = reqwest::Client::new();
        let ollama_base_urls = vec![
            "http://127.0.0.1:11434",  // Default Ollama port
            "http://127.0.0.1:3001",   // Potential proxy port
            "http://localhost:11434",  // Alternative localhost
        ];
        
        let mut active_ollama_url = None;
        
        for base_url in &ollama_base_urls {
            let health_url = format!("{}/api/tags", base_url);  // Ollama models endpoint
            match client.get(&health_url).timeout(Duration::from_secs(3)).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("✅ Found active Ollama service at: {}", base_url);
                        active_ollama_url = Some(base_url.to_string());
                        break;
                    } else {
                        println!("   ⚠️  Ollama at {} returned: {}", base_url, response.status());
                    }
                },
                Err(e) => {
                    println!("   ❌ No Ollama service found at {}: {}", base_url, e);
                }
            }
        }
        
        let ollama_url = active_ollama_url.expect("No active Ollama service found");
        
        // Step 2: Fetch and display actual models
        println!("📋 Step 2: Fetching your actual Ollama models...");
        
        let models_url = format!("{}/api/tags", &ollama_url);
        match client.get(&models_url).send().await {
            Ok(response) => {
                println!("📊 Ollama models endpoint status: {}", response.status());
                
                if response.status().is_success() {
                    match response.text().await {
                        Ok(body) => {
                            println!("📋 Raw Ollama response: {}", body);
                            
                            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                println!("🎯 OLLAMA MODELS DISCOVERED:");
                                println!("{}", "=".repeat(50));
                                
                                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                                    for (i, model) in models.iter().enumerate() {
                                        println!("📦 Model {}: ", i + 1);
                                        
                                        if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
                                            println!("   Name: {}", name);
                                        }
                                        
                                        if let Some(size) = model.get("size").and_then(|s| s.as_u64()) {
                                            println!("   Size: {} MB", size / (1024 * 1024));
                                        }
                                        
                                        if let Some(modified) = model.get("modified_at").and_then(|m| m.as_str()) {
                                            println!("   Modified: {}", modified);
                                        }
                                        
                                        if let Some(digest) = model.get("digest").and_then(|d| d.as_str()) {
                                            println!("   Digest: {}...", &digest[..std::cmp::min(16, digest.len())]);
                                        }
                                        
                                        if let Some(details) = model.get("details") {
                                            if let Some(family) = details.get("family").and_then(|f| f.as_str()) {
                                                println!("   Family: {}", family);
                                            }
                                            if let Some(format) = details.get("format").and_then(|f| f.as_str()) {
                                                println!("   Format: {}", format);
                                            }
                                            if let Some(params) = details.get("parameter_size").and_then(|p| p.as_str()) {
                                                println!("   Parameters: {}", params);
                                            }
                                        }
                                        
                                        println!();
                                    }
                                    
                                    println!("📊 SUMMARY: Found {} Ollama models", models.len());
                                } else {
                                    println!("⚠️  No models array found in response");
                                }
                            } else {
                                println!("❌ Failed to parse JSON response");
                            }
                        },
                        Err(e) => {
                            println!("❌ Failed to read Ollama response: {}", e);
                        }
                    }
                } else {
                    println!("❌ Ollama models request failed with status: {}", response.status());
                    if let Ok(error_body) = response.text().await {
                        println!("Error details: {}", error_body);
                    }
                }
            },
            Err(e) => {
                println!("❌ Failed to connect to Ollama: {}", e);
            }
        }
        
        // Step 3: Test integration with MindLink server manager
        println!("📋 Step 3: Testing MindLink server manager integration...");
        
        let config_manager = ConfigManager::new().await.expect("Failed to create config manager");
        let server_manager = ServerManager::new().await;
        
        println!("🔧 Server Manager Status:");
        println!("   Running: {}", server_manager.is_running().await);
        
        let server_config = config_manager.get_server_config().await;
        println!("📊 Server Configuration:");
        println!("   Host: {}", server_config.host);
        println!("   Port: {}", server_config.port);
        
        // Step 4: Test OpenAI-compatible endpoint integration
        println!("📋 Step 4: Testing OpenAI-compatible endpoint integration...");
        
        let openai_models_url = format!("{}/v1/models", &ollama_url);
        match client.get(&openai_models_url).send().await {
            Ok(response) => {
                println!("📊 OpenAI-compatible models endpoint status: {}", response.status());
                
                if response.status().is_success() {
                    match response.text().await {
                        Ok(body) => {
                            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                println!("🔗 OpenAI-compatible format:");
                                if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                                    for model in data {
                                        if let Some(id) = model.get("id").and_then(|i| i.as_str()) {
                                            println!("   - {}", id);
                                        }
                                    }
                                }
                            }
                        },
                        Err(e) => println!("❌ Failed to read OpenAI response: {}", e),
                    }
                }
            },
            Err(e) => {
                println!("ℹ️  OpenAI endpoint not available: {}", e);
            }
        }
        
        println!("✅ Live Ollama integration and model discovery test completed!");
    }

    #[tokio::test]
    async fn test_end_to_end_service_integration() {
        println!("🎯 E2E Test: Complete Service Integration");
        
        // Step 1: Initialize all managers
        println!("📋 Step 1: Initializing all service managers...");
        
        let config_manager = ConfigManager::new().await.expect("Failed to create config manager");
        let bifrost_manager = BifrostManager::new().await;
        let server_manager = ServerManager::new().await;
        
        // Step 2: Display comprehensive service status
        println!("📋 Step 2: Comprehensive service status check...");
        
        println!("🔧 === MINDLINK SERVICE STATUS ===");
        
        // Config status
        let bifrost_config = config_manager.get_bifrost_config().await;
        let server_config = config_manager.get_server_config().await;
        
        println!("📊 Configuration:");
        println!("   Bifrost: {}:{}", bifrost_config.host, bifrost_config.port);
        println!("   Server: {}:{}", server_config.host, server_config.port);
        
        // Manager status
        println!("📊 Manager Status:");
        println!("   Bifrost Manager Running: {}", bifrost_manager.is_running().await);
        println!("   Bifrost Binary Available: {}", bifrost_manager.is_binary_available().await);
        println!("   Server Manager Running: {}", server_manager.is_running().await);
        
        // Step 3: Test service discovery
        println!("📋 Step 3: Service discovery across all active ports...");
        
        let client = reqwest::Client::new();
        let test_ports = vec![1420, 3001, 3002, 3004, 3005, 3006, 3007, 3008, 3009, 11434];
        
        let mut active_services = HashMap::new();
        
        for port in test_ports {
            let url = format!("http://127.0.0.1:{}", port);
            match client.get(&url).timeout(Duration::from_secs(1)).send().await {
                Ok(response) => {
                    let service_type = match port {
                        1420 => "Vite Dev Server",
                        3001 => "MindLink API/Ollama Proxy",
                        3002 => "MindLink Dashboard", 
                        3004..=3009 => "Bifrost Instance",
                        11434 => "Ollama Server",
                        _ => "Unknown Service",
                    };
                    
                    active_services.insert(port, (service_type, response.status()));
                    println!("   ✅ Port {}: {} ({})", port, service_type, response.status());
                },
                Err(_) => {
                    // Service not running - this is fine
                }
            }
        }
        
        println!("📊 Active Services Summary: {} services running", active_services.len());
        
        // Step 4: Test cross-service communication
        println!("📋 Step 4: Testing cross-service communication...");
        
        // Test if we can communicate between services
        for (&port, (service_type, _)) in &active_services {
            match service_type {
                &"Bifrost Instance" => {
                    println!("🔍 Testing Bifrost instance on port {}:", port);
                    
                    // Test health endpoint
                    let health_url = format!("http://127.0.0.1:{}/health", port);
                    match client.get(&health_url).send().await {
                        Ok(response) => {
                            println!("   Health: {}", response.status());
                        },
                        Err(_) => {
                            println!("   Health: No endpoint");
                        }
                    }
                    
                    // Test models endpoint
                    let models_url = format!("http://127.0.0.1:{}/v1/models", port);
                    match client.get(&models_url).send().await {
                        Ok(response) => {
                            println!("   Models: {}", response.status());
                        },
                        Err(_) => {
                            println!("   Models: No endpoint");
                        }
                    }
                },
                &"MindLink API/Ollama Proxy" => {
                    println!("🔍 Testing MindLink API on port {}:", port);
                    
                    let health_url = format!("http://127.0.0.1:{}/health", port);
                    match client.get(&health_url).send().await {
                        Ok(response) => {
                            println!("   API Health: {}", response.status());
                        },
                        Err(_) => {
                            println!("   API Health: No endpoint");
                        }
                    }
                },
                &"Ollama Server" => {
                    println!("🔍 Testing Ollama server on port {}:", port);
                    
                    let tags_url = format!("http://127.0.0.1:{}/api/tags", port);
                    match client.get(&tags_url).send().await {
                        Ok(response) => {
                            println!("   Ollama Models: {}", response.status());
                            
                            if response.status().is_success() {
                                if let Ok(body) = response.text().await {
                                    if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                        if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                                            println!("   Available Models: {}", models.len());
                                        }
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            println!("   Ollama Models: Connection failed");
                        }
                    }
                },
                _ => {
                    // Other services - basic check only
                }
            }
        }
        
        println!("✅ End-to-end service integration test completed!");
        
        // Ensure we found at least some active services
        assert!(!active_services.is_empty(), "Should have found at least one active service");
    }
}