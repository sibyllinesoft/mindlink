// Tauri commands - replaces Electron IPC handlers
use crate::AppState;
use crate::error::MindLinkError;
use crate::logging::{get_logger, LogEntry, LogLevel, LogCategory};
use crate::managers::config_manager::ConfigSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub is_serving: bool,
    pub is_authenticated: bool,
    pub tunnel_url: Option<String>,
    pub server_url: Option<String>,
    pub bifrost_url: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub success: bool,
    pub message: Option<String>,
    pub server_url: Option<String>,
    pub tunnel_url: Option<String>,
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    let is_serving = *state.is_serving.read().await;
    let last_error = state.last_error.read().await.clone();
    
    let is_authenticated = {
        let auth_manager = state.auth_manager.read().await;
        auth_manager.is_authenticated().await
    };
    
    let tunnel_url = {
        let tunnel_manager = state.tunnel_manager.read().await;
        tunnel_manager.get_current_url().await
    };
    
    let server_url = {
        let server_manager = state.server_manager.read().await;
        server_manager.get_local_url().await
    };
    
    let bifrost_url = {
        let bifrost_manager = state.bifrost_manager.read().await;
        bifrost_manager.get_local_url().await
    };
    
    Ok(StatusResponse {
        is_serving,
        is_authenticated,
        tunnel_url,
        server_url,
        bifrost_url,
        last_error,
    })
}

#[tauri::command]
pub async fn login_and_serve(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    // Log user action
    if let Some(logger) = get_logger() {
        logger.log_user_action("login_and_serve", None);
    }
    
    // Check authentication first
    let is_authenticated = {
        let mut auth_manager = state.auth_manager.write().await;
        if !auth_manager.is_authenticated().await {
            match auth_manager.login().await {
                Ok(_) => true,
                Err(e) => {
                    let auth_error = MindLinkError::Authentication {
                        message: "Login failed".to_string(),
                        source: Some(e),
                    };
                    
                    if let Some(logger) = get_logger() {
                        logger.log_error("Auth", &auth_error, None);
                    }
                    
                    return Ok(ServiceResponse {
                        success: false,
                        message: Some(auth_error.user_message()),
                        server_url: None,
                        tunnel_url: None,
                    });
                }
            }
        } else {
            true
        }
    };
    
    if !is_authenticated {
        let auth_error = MindLinkError::Authentication {
            message: "Authentication required".to_string(),
            source: None,
        };
        
        return Ok(ServiceResponse {
            success: false,
            message: Some(auth_error.user_message()),
            server_url: None,
            tunnel_url: None,
        });
    }
    
    // Start server
    let server_url = {
        let mut server_manager = state.server_manager.write().await;
        match server_manager.start(state.auth_manager.clone()).await {
            Ok(url) => {
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Info,
                        LogCategory::System,
                        format!("Server started successfully at {}", url),
                    ).with_component("Server");
                    logger.log(entry);
                }
                Some(url)
            },
            Err(e) => {
                let server_error = MindLinkError::Internal {
                    message: "Server failed to start".to_string(),
                    component: Some("ServerManager".to_string()),
                    source: Some(e.into()),
                };
                
                if let Some(logger) = get_logger() {
                    logger.log_error("Server", &server_error, None);
                }
                
                return Ok(ServiceResponse {
                    success: false,
                    message: Some(server_error.user_message()),
                    server_url: None,
                    tunnel_url: None,
                });
            }
        }
    };
    
    // Create tunnel (non-fatal if it fails)
    let tunnel_url = {
        let mut tunnel_manager = state.tunnel_manager.write().await;
        match tunnel_manager.create_tunnel().await {
            Ok(url) => {
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Info,
                        LogCategory::Network,
                        format!("Tunnel created successfully: {}", url),
                    ).with_component("Tunnel");
                    logger.log(entry);
                }
                Some(url)
            },
            Err(e) => {
                let tunnel_error = MindLinkError::Tunnel {
                    message: "Tunnel creation failed".to_string(),
                    tunnel_type: Some("quick".to_string()),
                    local_port: Some(3001),
                    source: Some(e),
                };
                
                if let Some(logger) = get_logger() {
                    logger.log_error("Tunnel", &tunnel_error, None);
                }
                
                // Non-fatal, continue without tunnel
                None
            }
        }
    };
    
    // Update serving state
    *state.is_serving.write().await = true;
    
    if let Some(logger) = get_logger() {
        let entry = LogEntry::new(
            LogLevel::Info,
            LogCategory::System,
            "Services started successfully".to_string(),
        ).with_component("Main");
        logger.log(entry);
    }
    
    Ok(ServiceResponse {
        success: true,
        message: Some("Services started successfully".to_string()),
        server_url,
        tunnel_url,
    })
}

#[tauri::command]
pub async fn stop_serving(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    // Stop tunnel
    {
        let mut tunnel_manager = state.tunnel_manager.write().await;
        if let Err(e) = tunnel_manager.close_tunnel().await {
            eprintln!("Failed to close tunnel: {}", e);
        }
    }
    
    // Stop server
    {
        let mut server_manager = state.server_manager.write().await;
        if let Err(e) = server_manager.stop().await {
            eprintln!("Failed to stop server: {}", e);
        }
    }
    
    // Update serving state
    *state.is_serving.write().await = false;
    
    Ok(ServiceResponse {
        success: true,
        message: Some("Services stopped successfully".to_string()),
        server_url: None,
        tunnel_url: None,
    })
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<HashMap<String, serde_json::Value>, String> {
    let config_manager = state.config_manager.read().await;
    let config = config_manager.get_config().await;
    // Convert ConfigSchema to HashMap
    let json_value = serde_json::to_value(&config).map_err(|e| format!("Serialization failed: {}", e))?;
    let map = json_value.as_object().unwrap().clone().into_iter()
        .map(|(k, v)| (k, v)).collect();
    Ok(map)
}

#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>, 
    config: HashMap<String, serde_json::Value>
) -> Result<(), String> {
    let config_manager = state.config_manager.write().await;
    // Convert HashMap to ConfigSchema first
    let config_json = serde_json::Value::Object(config.into_iter().collect());
    let config_schema: ConfigSchema = serde_json::from_value(config_json)
        .map_err(|e| format!("Invalid config format: {}", e))?;
    
    config_manager.update_config(config_schema).await
        .map_err(|e| format!("Failed to save config: {}", e))
}

#[tauri::command]
pub async fn show_notification(message: String) -> Result<(), String> {
    #[allow(unused_imports)]
    use tauri_plugin_notification::NotificationExt;
    // This will be called from the frontend to show notifications
    println!("Notification: {}", message);
    Ok(())
}

#[tauri::command]
pub async fn open_bifrost_dashboard(state: State<'_, AppState>) -> Result<(), String> {
    let bifrost_manager = state.bifrost_manager.read().await;
    if let Some(url) = bifrost_manager.get_local_url().await {
        if bifrost_manager.is_running().await {
            println!("Opening Bifrost dashboard: {}", url);
            // This command doesn't have access to shell directly, return URL for caller to open
            Ok(())
        } else {
            Err("Bifrost dashboard is not running".to_string())
        }
    } else {
        Err("Bifrost dashboard URL not available".to_string())
    }
}

#[tauri::command]
pub async fn copy_api_url(state: State<'_, AppState>) -> Result<String, String> {
    let tunnel_url = {
        let tunnel_manager = state.tunnel_manager.read().await;
        tunnel_manager.get_current_url().await
    };
    
    let server_url = {
        let server_manager = state.server_manager.read().await;
        server_manager.get_local_url().await
    };
    
    let api_url = tunnel_url.or(server_url)
        .map(|url| format!("{}/v1", url))
        .ok_or("No API URL available")?;
    
    // Copy to clipboard would be handled by frontend
    Ok(api_url)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCompletionRequest {
    pub message: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCompletionResponse {
    pub success: bool,
    pub response: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn test_completion(
    state: State<'_, AppState>,
    request: TestCompletionRequest,
) -> Result<TestCompletionResponse, String> {
    let is_serving = *state.is_serving.read().await;
    
    if !is_serving {
        return Ok(TestCompletionResponse {
            success: false,
            response: None,
            error: Some("API server not running".to_string()),
        });
    }
    
    let server_url = {
        let server_manager = state.server_manager.read().await;
        server_manager.get_local_url().await
    };
    
    let Some(base_url) = server_url else {
        return Ok(TestCompletionResponse {
            success: false,
            response: None,
            error: Some("Server URL not available".to_string()),
        });
    };
    
    // Make test request to API
    let client = reqwest::Client::new();
    let test_request = serde_json::json!({
        "model": request.model.unwrap_or_else(|| "gpt-5".to_string()),
        "messages": [{"role": "user", "content": request.message}],
        "stream": false
    });
    
    match client
        .post(&format!("{}/v1/chat/completions", base_url))
        .json(&test_request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let content = json
                            .pointer("/choices/0/message/content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("No response content");
                        
                        Ok(TestCompletionResponse {
                            success: true,
                            response: Some(content.to_string()),
                            error: None,
                        })
                    }
                    Err(e) => Ok(TestCompletionResponse {
                        success: false,
                        response: None,
                        error: Some(format!("Failed to parse response: {}", e)),
                    }),
                }
            } else {
                Ok(TestCompletionResponse {
                    success: false,
                    response: None,
                    error: Some(format!("API returned status: {}", response.status())),
                })
            }
        }
        Err(e) => Ok(TestCompletionResponse {
            success: false,
            response: None,
            error: Some(format!("Request failed: {}", e)),
        }),
    }
}

#[tauri::command]
pub async fn start_bifrost(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    println!("🚀 Starting Bifrost LLM Router...");
    let mut bifrost_manager = state.bifrost_manager.write().await;
    
    if bifrost_manager.is_running().await {
        println!("ℹ️ Bifrost is already running");
        return Ok(ServiceResponse {
            success: true,
            message: Some("Bifrost is already running".to_string()),
            server_url: bifrost_manager.get_local_url().await,
            tunnel_url: None,
        });
    }
    
    // Check if binary is available
    if !bifrost_manager.is_binary_available().await {
        println!("❌ Bifrost binary not available - installation required");
        return Ok(ServiceResponse {
            success: false,
            message: Some("Bifrost binary not installed. Please install the binary first.".to_string()),
            server_url: None,
            tunnel_url: None,
        });
    }
    
    match bifrost_manager.start().await {
        Ok(()) => {
            let url = bifrost_manager.get_local_url().await;
            println!("✅ Bifrost LLM Router started successfully: {:?}", url);
            Ok(ServiceResponse {
                success: true,
                message: Some("Bifrost LLM Router started successfully".to_string()),
                server_url: url,
                tunnel_url: None,
            })
        }
        Err(e) => {
            println!("❌ Failed to start Bifrost: {}", e);
            Ok(ServiceResponse {
                success: false,
                message: Some(format!("Failed to start Bifrost: {}", e)),
                server_url: None,
                tunnel_url: None,
            })
        },
    }
}

#[tauri::command]
pub async fn stop_bifrost(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    let mut bifrost_manager = state.bifrost_manager.write().await;
    
    if !bifrost_manager.is_running().await {
        return Ok(ServiceResponse {
            success: true,
            message: Some("Bifrost is already stopped".to_string()),
            server_url: None,
            tunnel_url: None,
        });
    }
    
    match bifrost_manager.stop().await {
        Ok(()) => Ok(ServiceResponse {
            success: true,
            message: Some("Bifrost LLM Router stopped successfully".to_string()),
            server_url: None,
            tunnel_url: None,
        }),
        Err(e) => Ok(ServiceResponse {
            success: false,
            message: Some(format!("Failed to stop Bifrost: {}", e)),
            server_url: None,
            tunnel_url: None,
        }),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryInstallationResponse {
    pub success: bool,
    pub message: Option<String>,
    pub binary_path: Option<String>,
    pub is_installed: bool,
}

#[tauri::command]
pub async fn install_bifrost_binary(state: State<'_, AppState>) -> Result<BinaryInstallationResponse, String> {
    println!("🔧 Starting Bifrost binary build...");
    let mut bifrost_manager = state.bifrost_manager.write().await;
    
    // First try to refresh the binary path (in case it was already built)
    match bifrost_manager.refresh_binary_path().await {
        Ok(path) => {
            println!("✅ Found existing Bifrost binary at: {:?}", path);
            return Ok(BinaryInstallationResponse {
                success: true,
                message: Some("Existing Bifrost binary found".to_string()),
                binary_path: Some(path.to_string_lossy().to_string()),
                is_installed: true,
            });
        },
        Err(_) => {
            // Binary not found, need to build it
            println!("📦 Building Bifrost binary from source...");
            match bifrost_manager.rebuild_bifrost().await {
                Ok(path) => {
                    println!("✅ Bifrost binary built successfully at: {:?}", path);
                    Ok(BinaryInstallationResponse {
                        success: true,
                        message: Some("Bifrost binary built successfully".to_string()),
                        binary_path: Some(path.to_string_lossy().to_string()),
                        is_installed: true,
                    })
                },
                Err(e) => {
                    println!("❌ Failed to build Bifrost binary: {}", e);
                    Ok(BinaryInstallationResponse {
                        success: false,
                        message: Some(format!("Failed to build Bifrost: {}", e)),
                        binary_path: None,
                        is_installed: false,
                    })
                },
            }
        }
    }
}

#[tauri::command]
pub async fn get_bifrost_installation_status(state: State<'_, AppState>) -> Result<BinaryInstallationResponse, String> {
    let bifrost_manager = state.bifrost_manager.read().await;
    let (is_installed, binary_path, status_message) = bifrost_manager.get_installation_info().await;
    
    Ok(BinaryInstallationResponse {
        success: true,
        message: status_message,
        binary_path: binary_path.map(|p| p.to_string_lossy().to_string()),
        is_installed,
    })
}

#[tauri::command]
pub async fn reinstall_bifrost_binary(state: State<'_, AppState>) -> Result<BinaryInstallationResponse, String> {
    let mut bifrost_manager = state.bifrost_manager.write().await;
    
    match bifrost_manager.rebuild_bifrost().await {
        Ok(path) => Ok(BinaryInstallationResponse {
            success: true,
            message: Some("Bifrost binary rebuilt successfully".to_string()),
            binary_path: Some(path.to_string_lossy().to_string()),
            is_installed: true,
        }),
        Err(e) => Ok(BinaryInstallationResponse {
            success: false,
            message: Some(format!("Failed to rebuild Bifrost: {}", e)),
            binary_path: None,
            is_installed: false,
        }),
    }
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    let mut auth_manager = state.auth_manager.write().await;
    
    match auth_manager.logout().await {
        Ok(()) => Ok(ServiceResponse {
            success: true,
            message: Some("Logged out successfully".to_string()),
            server_url: None,
            tunnel_url: None,
        }),
        Err(e) => Ok(ServiceResponse {
            success: false,
            message: Some(format!("Logout failed: {}", e)),
            server_url: None,
            tunnel_url: None,
        }),
    }
}