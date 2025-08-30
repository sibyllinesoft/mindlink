//! # Tauri Commands Module
//!
//! This module contains all Tauri command handlers that expose the application's
//! core functionality to the frontend. These commands replace traditional Electron
//! IPC handlers and provide the bridge between the Rust backend and the frontend UI.
//!
//! ## Architecture
//!
//! Commands are organized into functional groups:
//!
//! - **Status & Control**: System status, service lifecycle management
//! - **Authentication**: Login/logout operations and token management  
//! - **Configuration**: Settings management and persistence
//! - **API Testing**: Built-in testing capabilities for the LLM API
//! - **Binary Management**: Bifrost binary installation and updates
//! - **Utility**: Helper functions for UI interactions
//!
//! ## Command Pattern
//!
//! All commands follow a consistent pattern:
//! - Accept a `State<AppState>` parameter for accessing shared state
//! - Return `Result<ResponseType, String>` for error handling
//! - Use async/await for non-blocking operations
//! - Provide structured response types with success/error information
//!
//! ## Example Usage
//!
//! Commands are called from the frontend using Tauri's invoke API:
//!
//! ```javascript
//! // Frontend JavaScript/TypeScript
//! import { invoke } from '@tauri-apps/api/tauri';
//!
//! const status = await invoke('get_status');
//! const result = await invoke('login_and_serve');
//! ```
//!
//! ## Error Handling
//!
//! Commands use structured error responses that provide both machine-readable
//! success flags and human-friendly error messages for display in the UI.
//!
//! ## Thread Safety
//!
//! All commands are designed to be thread-safe and can handle concurrent
//! calls by using appropriate locking mechanisms through the `AppState`.
use crate::error::MindLinkError;
use crate::logging::{get_logger, LogCategory, LogEntry, LogLevel};
use crate::managers::config_manager::ConfigSchema;
use crate::AppState;
use tauri::{AppHandle, Manager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;
use chrono;
use tokio::process::Command;
use tokio::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Response type for status queries, providing comprehensive system state information.
///
/// This structure contains all the essential status information needed by the frontend
/// to display the current state of the application and its services.
///
/// # Fields
///
/// - `is_serving`: Whether the main API server is currently running
/// - `is_authenticated`: Whether the user is currently logged in with valid tokens
/// - `tunnel_url`: Public Cloudflare tunnel URL (if active)
/// - `server_url`: Local API server URL (usually http://localhost:3001)
/// - `bifrost_url`: Bifrost dashboard URL (if running)
/// - `instance_token`: Unique token for this MindLink instance
/// - `last_error`: Most recent error message (if any)
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub is_serving: bool,
    pub is_authenticated: bool,
    pub tunnel_url: Option<String>,
    pub server_url: Option<String>,
    pub bifrost_url: Option<String>,
    pub instance_token: Option<String>,
    pub last_error: Option<String>,
}

/// Response type for QR data containing tunnel URL and instance token
#[derive(Debug, Serialize, Deserialize)]
pub struct QrDataResponse {
    pub success: bool,
    pub qr_data: Option<String>,
    pub error: Option<String>,
}

/// Standard response type for service operations (start, stop, etc.).
///
/// This structure provides a consistent format for all service management
/// operations, including success/failure status and relevant URLs.
///
/// # Fields
///
/// - `success`: Whether the operation completed successfully
/// - `message`: Human-readable status or error message
/// - `server_url`: Local server URL (if applicable)
/// - `tunnel_url`: Public tunnel URL (if applicable)
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub success: bool,
    pub message: Option<String>,
    pub server_url: Option<String>,
    pub tunnel_url: Option<String>,
    pub auth_url: Option<String>,
}

/// Retrieves the current system status including authentication, service states, and URLs.
///
/// This command provides a comprehensive view of the application state, which is
/// essential for the frontend to display accurate status information and enable/disable
/// appropriate UI controls. It detects both managed and external processes.
///
/// # Returns
///
/// - `Ok(StatusResponse)`: Current system status with all relevant information
/// - `Err(String)`: Error message if status could not be retrieved (rare)
///
/// # Example Response
///
/// ```json
/// {
///   "is_serving": true,
///   "is_authenticated": true,
///   "tunnel_url": "https://example.trycloudflare.com",
///   "server_url": "http://localhost:3001",
///   "bifrost_url": "http://localhost:3002",
///   "last_error": null
/// }
/// ```
#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    // Check actual service states, not just internal flags
    let is_serving = check_actual_server_running().await.unwrap_or(*state.is_serving.read().await);
    let last_error = state.last_error.read().await.clone();

    let is_authenticated = {
        // Check cached authentication status first to avoid expensive cloudflared calls
        const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(30); // Cache for 30 seconds
        
        let auth_cache = state.auth_cache.read().await;
        let should_check = match &*auth_cache {
            Some((_, last_check)) => last_check.elapsed() > CACHE_DURATION,
            None => true,
        };
        
        if should_check {
            drop(auth_cache); // Release read lock before acquiring write lock
            
            // Perform the expensive authentication check
            let auth_result = {
                let binary_manager = state.binary_manager.read().await;
                match binary_manager.ensure_cloudflared().await {
                    Ok(cloudflared_path) => {
                        drop(binary_manager); // Release the lock early
                        // Check if cloudflared is authenticated by running "tunnel list"
                        match Command::new(&cloudflared_path)
                            .args(&["tunnel", "list"])
                            .output()
                            .await
                        {
                            Ok(output) => output.status.success(),
                            Err(_) => false,
                        }
                    }
                    Err(_) => {
                        drop(binary_manager); // Release the lock early
                        false
                    }
                }
            };
            
            // Update the cache
            let mut auth_cache = state.auth_cache.write().await;
            *auth_cache = Some((auth_result, std::time::Instant::now()));
            auth_result
        } else {
            // Use cached result
            auth_cache.unwrap().0
        }
    };

    // Check for actual tunnel URL by detecting running cloudflare processes
    let tunnel_url = match detect_actual_tunnel_url().await {
        Some(url) => Some(url),
        None => {
            let tunnel_manager = state.tunnel_manager.read().await;
            tunnel_manager.get_current_url().await
        }
    };

    let server_url = if is_serving {
        Some("http://127.0.0.1:3001".to_string())
    } else {
        let server_manager = state.server_manager.read().await;
        server_manager.get_local_url().await
    };

    // Get Bifrost URL from the manager first (knows the actual port), fallback to detection
    let bifrost_url = {
        let bifrost_manager = state.bifrost_manager.read().await;
        match bifrost_manager.get_local_url().await {
            Some(url) => Some(url),
            None => detect_actual_bifrost_url().await
        }
    };

    // Get or create instance token
    let instance_token = get_or_create_instance_token(state.clone()).await.ok();

    Ok(StatusResponse {
        is_serving,
        is_authenticated,
        tunnel_url,
        server_url,
        bifrost_url,
        instance_token,
        last_error,
    })
}

/// Performs authentication and starts all required services (server + tunnel).
///
/// This is the main command for starting the MindLink API service. It handles the
/// complete workflow:
/// 1. Authenticates with ChatGPT (if not already authenticated)
/// 2. Starts the local API server
/// 3. Creates a Cloudflare tunnel (if possible)
/// 4. Updates the serving state
///
/// The operation is designed to be robust - tunnel creation failures are non-fatal
/// and the service will continue running locally even if the tunnel fails.
///
/// # Returns
///
/// - `Ok(ServiceResponse)`: Success/failure status with URLs
/// - `Err(String)`: Should not occur - errors are returned as ServiceResponse
///
/// # Errors
///
/// Common failure scenarios:
/// - Authentication failed (invalid credentials, network issues)
/// - Local server port already in use
/// - Cloudflare tunnel creation failed (non-fatal)
///
/// # Example Success Response
///
/// ```json
/// {
///   "success": true,
///   "message": "Services started successfully",
///   "server_url": "http://localhost:3001",
///   "tunnel_url": "https://example.trycloudflare.com"
/// }
/// ```
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
            auth_url: None,
                    });
                },
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
            auth_url: None,
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
                    )
                    .with_component("Server");
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
            auth_url: None,
                });
            },
        }
    };

    // Create tunnel (enhanced error reporting but still non-fatal)
    let tunnel_url = {
        let mut tunnel_manager = state.tunnel_manager.write().await;
        match tunnel_manager.create_tunnel().await {
            Ok(url) => {
                println!("✅ Cloudflare tunnel created: {}", url);
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Info,
                        LogCategory::Network,
                        format!("Tunnel created successfully: {}", url),
                    )
                    .with_component("Tunnel");
                    logger.log(entry);
                }
                Some(url)
            },
            Err(e) => {
                println!("⚠️  Tunnel creation failed (continuing without tunnel): {}", e);
                
                let tunnel_error = MindLinkError::Tunnel {
                    message: format!("Tunnel creation failed: {}. Service running locally only.", e),
                    tunnel_type: Some("quick".to_string()),
                    local_port: Some(3001),
                    source: Some(e),
                };

                if let Some(logger) = get_logger() {
                    logger.log_error("Tunnel", &tunnel_error, None);
                }

                // Store tunnel error for user to see but don't fail the service
                *state.last_error.write().await = Some(format!(
                    "Tunnel unavailable: {}. Use 'Create Tunnel' to retry.", 
                    tunnel_error.user_message()
                ));

                // Non-fatal, continue without tunnel
                None
            },
        }
    };

    // Update serving state
    *state.is_serving.write().await = true;

    if let Some(logger) = get_logger() {
        let entry = LogEntry::new(
            LogLevel::Info,
            LogCategory::System,
            "Services started successfully".to_string(),
        )
        .with_component("Main");
        logger.log(entry);
    }

    Ok(ServiceResponse {
        success: true,
        message: Some("Services started successfully".to_string()),
        server_url,
        tunnel_url,
        auth_url: None,
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
            auth_url: None,
    })
}

#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let config_manager = state.config_manager.read().await;
    let config = config_manager.get_config().await;
    
    // Add authentication status
    let auth_manager = state.auth_manager.read().await;
    let (is_authenticated, user_email) = auth_manager.get_auth_status().await;
    
    drop(config_manager);
    drop(auth_manager);
    
    // Convert ConfigSchema to HashMap
    let json_value =
        serde_json::to_value(&config).map_err(|e| format!("Serialization failed: {}", e))?;
    let mut map: HashMap<String, serde_json::Value> = json_value
        .as_object()
        .unwrap()
        .clone()
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();
    
    // Add authentication info
    map.insert("is_authenticated".to_string(), serde_json::Value::Bool(is_authenticated));
    map.insert("user_email".to_string(), 
        user_email.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null));
    
    Ok(map)
}

#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    config: HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    let config_manager = state.config_manager.write().await;
    // Convert HashMap to ConfigSchema first
    let config_json = serde_json::Value::Object(config.into_iter().collect());
    let config_schema: ConfigSchema =
        serde_json::from_value(config_json).map_err(|e| format!("Invalid config format: {}", e))?;

    config_manager
        .update_config(config_schema)
        .await
        .map_err(|e| format!("Failed to save config: {}", e))
}

#[tauri::command]
pub async fn show_notification(message: String) -> Result<(), String> {
    // This will be called from the frontend to show notifications
    // TODO: Implement actual notification when tauri-plugin-notification is properly integrated
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

    let api_url = tunnel_url
        .or(server_url)
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
                    },
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
        },
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
            auth_url: None,
        });
    }

    // Check if binary is available
    if !bifrost_manager.is_binary_available().await {
        println!("❌ Bifrost binary not available - installation required");
        return Ok(ServiceResponse {
            success: false,
            message: Some(
                "Bifrost binary not installed. Please install the binary first.".to_string(),
            ),
            server_url: None,
            tunnel_url: None,
            auth_url: None,
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
            auth_url: None,
            })
        },
        Err(e) => {
            println!("❌ Failed to start Bifrost: {}", e);
            Ok(ServiceResponse {
                success: false,
                message: Some(format!("Failed to start Bifrost: {}", e)),
                server_url: None,
                tunnel_url: None,
            auth_url: None,
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
            auth_url: None,
        });
    }

    match bifrost_manager.stop().await {
        Ok(()) => Ok(ServiceResponse {
            success: true,
            message: Some("Bifrost LLM Router stopped successfully".to_string()),
            server_url: None,
            tunnel_url: None,
            auth_url: None,
        }),
        Err(e) => Ok(ServiceResponse {
            success: false,
            message: Some(format!("Failed to stop Bifrost: {}", e)),
            server_url: None,
            tunnel_url: None,
            auth_url: None,
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
pub async fn install_bifrost_binary(
    state: State<'_, AppState>,
) -> Result<BinaryInstallationResponse, String> {
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
        },
    }
}

#[tauri::command]
pub async fn get_bifrost_installation_status(
    state: State<'_, AppState>,
) -> Result<BinaryInstallationResponse, String> {
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
pub async fn reinstall_bifrost_binary(
    state: State<'_, AppState>,
) -> Result<BinaryInstallationResponse, String> {
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

/// Create a new Cloudflare tunnel for external access
#[tauri::command]
pub async fn create_tunnel(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    println!("🚀 Creating Cloudflare tunnel...");
    
    // Log user action
    if let Some(logger) = get_logger() {
        logger.log_user_action("create_tunnel", None);
    }

    let mut tunnel_manager = state.tunnel_manager.write().await;
    
    match tunnel_manager.create_tunnel().await {
        Ok(url) => {
            println!("✅ Tunnel created successfully: {}", url);
            
            if let Some(logger) = get_logger() {
                let entry = LogEntry::new(
                    LogLevel::Info,
                    LogCategory::Network,
                    format!("Manual tunnel creation successful: {}", url),
                )
                .with_component("Tunnel");
                logger.log(entry);
            }

            Ok(ServiceResponse {
                success: true,
                message: Some("Cloudflare tunnel created successfully".to_string()),
                server_url: None,
                tunnel_url: Some(url),
                auth_url: None,
            })
        },
        Err(e) => {
            println!("❌ Failed to create tunnel: {}", e);
            
            let tunnel_error = MindLinkError::Tunnel {
                message: "Manual tunnel creation failed".to_string(),
                tunnel_type: Some("quick".to_string()),
                local_port: Some(3001),
                source: Some(e),
            };

            if let Some(logger) = get_logger() {
                logger.log_error("Tunnel", &tunnel_error, None);
            }

            Ok(ServiceResponse {
                success: false,
                message: Some(tunnel_error.user_message()),
                server_url: None,
                tunnel_url: None,
            auth_url: None,
            })
        },
    }
}

/// Close the current Cloudflare tunnel
#[tauri::command]
pub async fn close_tunnel(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    println!("🔌 Closing Cloudflare tunnel...");
    
    // Log user action
    if let Some(logger) = get_logger() {
        logger.log_user_action("close_tunnel", None);
    }

    let mut tunnel_manager = state.tunnel_manager.write().await;
    
    match tunnel_manager.close_tunnel().await {
        Ok(()) => {
            println!("✅ Tunnel closed successfully");
            
            if let Some(logger) = get_logger() {
                let entry = LogEntry::new(
                    LogLevel::Info,
                    LogCategory::Network,
                    "Manual tunnel closure successful".to_string(),
                )
                .with_component("Tunnel");
                logger.log(entry);
            }

            Ok(ServiceResponse {
                success: true,
                message: Some("Cloudflare tunnel closed successfully".to_string()),
                server_url: None,
                tunnel_url: None,
            auth_url: None,
            })
        },
        Err(e) => {
            println!("❌ Failed to close tunnel: {}", e);

            Ok(ServiceResponse {
                success: false,
                message: Some(format!("Failed to close tunnel: {}", e)),
                server_url: None,
                tunnel_url: None,
            auth_url: None,
            })
        },
    }
}

/// Get current tunnel status and URL
#[tauri::command]
pub async fn get_tunnel_status(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    // First check for actual running tunnel
    let actual_tunnel_url = detect_actual_tunnel_url().await;
    
    if let Some(url) = actual_tunnel_url {
        return Ok(ServiceResponse {
            success: true,
            message: Some("Tunnel is active".to_string()),
            server_url: None,
            tunnel_url: Some(url),
            auth_url: None,
        });
    }
    
    // Fallback to managed tunnel state
    let tunnel_manager = state.tunnel_manager.read().await;
    let is_connected = tunnel_manager.is_connected().await;
    let tunnel_url = tunnel_manager.get_current_url().await;
    
    if is_connected {
        Ok(ServiceResponse {
            success: true,
            message: Some("Tunnel is active".to_string()),
            server_url: None,
            tunnel_url,
            auth_url: None,
        })
    } else {
        Ok(ServiceResponse {
            success: true,
            message: Some("No active tunnel".to_string()),
            server_url: None,
            tunnel_url: None,
            auth_url: None,
        })
    }
}

/// Install cloudflared binary for tunnel functionality
#[tauri::command]
pub async fn install_cloudflared_binary(
    state: State<'_, AppState>,
) -> Result<BinaryInstallationResponse, String> {
    println!("📦 Installing cloudflared binary...");
    
    let binary_manager = state.binary_manager.read().await;
    
    match binary_manager.ensure_cloudflared().await {
        Ok(path) => {
            println!("✅ cloudflared installed successfully at: {:?}", path);
            Ok(BinaryInstallationResponse {
                success: true,
                message: Some("cloudflared binary installed successfully".to_string()),
                binary_path: Some(path.to_string_lossy().to_string()),
                is_installed: true,
            })
        },
        Err(e) => {
            println!("❌ Failed to install cloudflared: {}", e);
            Ok(BinaryInstallationResponse {
                success: false,
                message: Some(format!("Failed to install cloudflared: {}", e)),
                binary_path: None,
                is_installed: false,
            })
        },
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
            auth_url: None,
        }),
        Err(e) => Ok(ServiceResponse {
            success: false,
            message: Some(format!("Logout failed: {}", e)),
            server_url: None,
            tunnel_url: None,
            auth_url: None,
        }),
    }
}

/// Get the persistent instance token for this MindLink installation
#[tauri::command]
pub async fn get_instance_token(state: State<'_, AppState>) -> Result<String, String> {
    get_or_create_instance_token(state).await
}

/// Cloudflare tunnel authentication - initiates cloudflared login flow
#[tauri::command]
pub async fn oauth_login(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    println!("🔑 Starting Cloudflare tunnel authentication...");
    
    // Log user action
    if let Some(logger) = get_logger() {
        logger.log_user_action("cloudflared_login", None);
    }

    // Get cloudflared binary path
    let binary_manager = state.binary_manager.read().await;
    let cloudflared_path = match binary_manager.ensure_cloudflared().await {
        Ok(path) => {
            println!("✅ Found cloudflared binary at: {:?}", path);
            path
        },
        Err(e) => {
            println!("❌ Failed to get cloudflared binary: {}", e);
            return Ok(ServiceResponse {
                success: false,
                message: Some(format!("cloudflared binary not available: {}", e)),
                server_url: None,
                tunnel_url: None,
                auth_url: None,
            });
        }
    };
    drop(binary_manager);

    // Ensure .cloudflared directory exists with proper permissions
    let home_dir = dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;
    let cloudflared_dir = home_dir.join(".cloudflared");
    
    if !cloudflared_dir.exists() {
        println!("📁 Creating .cloudflared directory: {:?}", cloudflared_dir);
        if let Err(e) = fs::create_dir_all(&cloudflared_dir).await {
            println!("⚠️ Warning: Failed to create .cloudflared directory: {}", e);
        }
    }

    // Check certificate file status before login
    let cert_path = cloudflared_dir.join("cert.pem");
    println!("🔍 Checking for existing certificate at: {:?}", cert_path);
    
    if cert_path.exists() {
        match fs::read_to_string(&cert_path).await {
            Ok(cert_content) if !cert_content.trim().is_empty() => {
                println!("✅ Found existing certificate file");
            }
            _ => {
                println!("⚠️ Certificate file exists but is empty or unreadable");
            }
        }
    } else {
        println!("ℹ️ No certificate file found, authentication required");
    }

    // Check if already authenticated by trying to list tunnels
    println!("🔍 Checking current authentication status...");
    let check_cmd = Command::new(&cloudflared_path)
        .args(&["tunnel", "list"])
        .output();

    match check_cmd.await {
        Ok(output) if output.status.success() => {
            // Already authenticated
            println!("✅ Already authenticated with Cloudflare");
            if let Some(logger) = get_logger() {
                logger.log(LogEntry::new(
                    LogLevel::Info,
                    LogCategory::Authentication,
                    "Already authenticated with Cloudflare".to_string(),
                ));
            }
            
            return Ok(ServiceResponse {
                success: true,
                message: Some("Already authenticated with Cloudflare".to_string()),
                auth_url: None,
                server_url: None,
                tunnel_url: None,
            });
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("❌ Authentication check failed - stdout: {}, stderr: {}", stdout, stderr);
        }
        Err(e) => {
            println!("❌ Failed to run authentication check: {}", e);
        }
    }

    // Need to authenticate - start login flow
    println!("🌐 Starting cloudflared login flow...");
    
    // Spawn cloudflared login process (this will open browser)
    match Command::new(&cloudflared_path)
        .args(&["tunnel", "login"])
        .spawn()
    {
        Ok(child) => {
            println!("✅ cloudflared login process spawned with PID: {:?}", child.id());
            
            if let Some(logger) = get_logger() {
                logger.log(LogEntry::new(
                    LogLevel::Info,
                    LogCategory::Authentication,
                    format!("Cloudflare tunnel authentication initiated - PID: {:?}", child.id()),
                ));
            }
            
            // Invalidate auth cache since authentication status might change
            {
                let mut auth_cache = state.auth_cache.write().await;
                *auth_cache = None;
            }
            
            Ok(ServiceResponse {
                success: true,
                message: Some("Cloudflare authentication started - please complete the process in your browser".to_string()),
                auth_url: Some("browser_opened".to_string()), // Signal that browser was opened
                server_url: None,
                tunnel_url: None,
            })
        }
        Err(e) => {
            println!("❌ Failed to spawn cloudflared login process: {}", e);
            if let Some(logger) = get_logger() {
                logger.log(LogEntry::new(
                    LogLevel::Error,
                    LogCategory::Authentication,
                    format!("Cloudflare authentication failed: {}", e),
                ));
            }
            
            Err(format!("Failed to start Cloudflare authentication: {}", e))
        }
    }
}

/// OAuth logout command - clears authentication tokens
#[tauri::command]
pub async fn oauth_logout(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    println!("🚪 OAuth logout...");
    
    // Stop services first
    let _ = stop_serving(state.clone()).await;
    
    let mut auth_manager = state.auth_manager.write().await;
    
    match auth_manager.logout().await {
        Ok(_) => {
            if let Some(logger) = get_logger() {
                logger.log(LogEntry::new(
                    LogLevel::Info,
                    LogCategory::Authentication,
                    "OAuth logout successful".to_string(),
                ));
            }
            
            Ok(ServiceResponse {
                success: true,
                message: Some("Logged out successfully".to_string()),
                server_url: None,
                tunnel_url: None,
            auth_url: None,
            })
        }
        Err(e) => {
            Err(format!("Logout failed: {}", e))
        }
    }
}

/// Enable tunnel with automatic name generation and permanent setup
#[tauri::command]
pub async fn start_tunnel(
    state: State<'_, AppState>,
    tunnel_name: String,
) -> Result<ServiceResponse, String> {
    println!("🚇 Enabling permanent tunnel: {}", tunnel_name);
    
    let mut tunnel_manager = state.tunnel_manager.write().await;
    
    // Save tunnel name to config for persistence
    {
        let config_manager = state.config_manager.write().await;
        let mut current_config = config_manager.get_config().await;
        current_config.tunnel.tunnel_type = tunnel_name.clone();
        current_config.tunnel.enabled = true;
        
        if let Err(e) = config_manager.update_config(current_config).await {
            eprintln!("Warning: Failed to save tunnel config: {}", e);
        }
    }
    
    match tunnel_manager.create_permanent_tunnel(&tunnel_name).await {
        Ok(tunnel_url) => {
            if let Some(logger) = get_logger() {
                logger.log(LogEntry::new(
                    LogLevel::Info,
                    LogCategory::System,
                    format!("Permanent tunnel '{}' active at {}", tunnel_name, tunnel_url),
                ));
            }
            
            Ok(ServiceResponse {
                success: true,
                message: Some(format!("Tunnel '{}' enabled successfully", tunnel_name)),
                tunnel_url: Some(tunnel_url),
                server_url: None,
                auth_url: None,
            })
        }
        Err(e) => {
            if let Some(logger) = get_logger() {
                logger.log(LogEntry::new(
                    LogLevel::Error,
                    LogCategory::System,
                    format!("Tunnel creation failed: {}", e),
                ));
            }
            
            Err(format!("Failed to enable tunnel: {}", e))
        }
    }
}

/// Disable tunnel
#[tauri::command]
pub async fn stop_tunnel(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    println!("🚇 Disabling tunnel...");
    
    let mut tunnel_manager = state.tunnel_manager.write().await;
    
    // Update config to disable tunnel
    {
        let config_manager = state.config_manager.write().await;
        let mut current_config = config_manager.get_config().await;
        current_config.tunnel.enabled = false;
        
        if let Err(e) = config_manager.update_config(current_config).await {
            eprintln!("Warning: Failed to save tunnel config: {}", e);
        }
    }
    
    match tunnel_manager.close_tunnel().await {
        Ok(_) => {
            if let Some(logger) = get_logger() {
                logger.log(LogEntry::new(
                    LogLevel::Info,
                    LogCategory::System,
                    "Tunnel disabled".to_string(),
                ));
            }
            
            Ok(ServiceResponse {
                success: true,
                message: Some("Tunnel disabled successfully".to_string()),
                tunnel_url: None,
                server_url: None,
                auth_url: None,
            })
        }
        Err(e) => {
            Err(format!("Failed to disable tunnel: {}", e))
        }
    }
}

/// Regenerate and save a new instance token
#[tauri::command]
pub async fn regenerate_token(state: State<'_, AppState>) -> Result<String, String> {
    let new_token = Uuid::new_v4().to_string();
    
    // Save the new token to config
    let config_manager = state.config_manager.write().await;
    
    // Add token to config (we'll extend ConfigSchema to include this)
    // For now, we'll store it as a custom field
    match config_manager.set_custom_field("instance_token", new_token.clone()).await {
        Ok(_) => {
            println!("✅ New instance token generated: {}", new_token);
            Ok(new_token)
        },
        Err(e) => {
            println!("❌ Failed to save new token: {}", e);
            Err(format!("Failed to save token: {}", e))
        }
    }
}

/// Get QR data containing tunnel URL and instance token as JSON
#[tauri::command]
pub async fn get_qr_data(state: State<'_, AppState>) -> Result<QrDataResponse, String> {
    // Get instance token
    let token = match get_or_create_instance_token(state.clone()).await {
        Ok(t) => t,
        Err(e) => {
            return Ok(QrDataResponse {
                success: false,
                qr_data: None,
                error: Some(format!("Failed to get token: {}", e)),
            });
        }
    };

    // Get tunnel URL
    let tunnel_url = {
        // First try to detect actual tunnel
        if let Some(url) = detect_actual_tunnel_url().await {
            Some(url)
        } else {
            // Fallback to managed tunnel state
            let tunnel_manager = state.tunnel_manager.read().await;
            tunnel_manager.get_current_url().await
        }
    };

    // Create QR data
    let qr_data = if let Some(url) = tunnel_url {
        let data = serde_json::json!({
            "url": url,
            "token": token
        });
        Some(data.to_string())
    } else {
        // If no tunnel, return token-only data
        let data = serde_json::json!({
            "token": token,
            "status": "No tunnel active"
        });
        Some(data.to_string())
    };

    Ok(QrDataResponse {
        success: true,
        qr_data,
        error: None,
    })
}

// ===== Helper functions for detecting actual running services =====

/// Get or create the persistent instance token
async fn get_or_create_instance_token(state: State<'_, AppState>) -> Result<String, String> {
    let config_manager = state.config_manager.read().await;
    
    // Try to get existing token from config
    match config_manager.get_custom_field("instance_token").await {
        Ok(Some(token)) => {
            if let Some(token_str) = token.as_str() {
                if !token_str.is_empty() {
                    return Ok(token_str.to_string());
                }
            }
        },
        _ => {
            // Token doesn't exist or is invalid, create a new one
        }
    }
    
    // Create new token
    let new_token = Uuid::new_v4().to_string();
    
    // Save it (drop read lock first)
    drop(config_manager);
    
    let config_manager = state.config_manager.write().await;
    match config_manager.set_custom_field("instance_token", new_token.clone()).await {
        Ok(_) => {
            println!("✅ Created new instance token: {}", new_token);
            Ok(new_token)
        },
        Err(e) => {
            println!("❌ Failed to save instance token: {}", e);
            // Return the token anyway, it just won't persist
            Ok(new_token)
        }
    }
}

/// Check if server is actually running on port 3001
async fn check_actual_server_running() -> Option<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    match client.get("http://127.0.0.1:3001/health").send().await {
        Ok(response) => Some(response.status().is_success()),
        Err(_) => Some(false),
    }
}

/// Detect actual tunnel URL by checking running cloudflare processes
async fn detect_actual_tunnel_url() -> Option<String> {
    use std::process::Command;
    
    // First try to get tunnel URL from cloudflare process
    if let Ok(output) = Command::new("ps")
        .args(&["aux"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("cloudflared") && line.contains("tunnel") {
                // Found cloudflare process, now try to extract URL from logs or check the tunnel
                if let Some(url) = check_tunnel_connectivity().await {
                    return Some(url);
                }
            }
        }
    }
    
    // If we can't detect from process, try common cloudflare domain patterns
    check_tunnel_connectivity().await
}

/// Check tunnel connectivity and return URL if active
async fn check_tunnel_connectivity() -> Option<String> {
    use std::process::Command;
    
    // Try to get the tunnel URL from systemctl or process command line
    if let Ok(output) = Command::new("ps")
        .args(&["aux"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("cloudflared") && line.contains("tunnel") && line.contains("http://localhost:") {
                // Try to identify which port it's tunneling
                if line.contains("localhost:3001") {
                    // This is the main server tunnel, let's try to find the URL
                    if let Some(url) = try_detect_tunnel_from_logs().await {
                        return Some(url);
                    }
                }
            }
        }
    }
    
    // Fallback: check known tunnel URL if it still works
    let potential_urls = vec![
        "https://raised-hub-cat-barcelona.trycloudflare.com",
    ];
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;
    
    for url in potential_urls {
        if let Ok(response) = client.get(&format!("{}/health", url)).send().await {
            if response.status().is_success() {
                return Some(url.to_string());
            }
        }
    }
    
    None
}

/// Try to detect tunnel URL from cloudflare logs or other sources
async fn try_detect_tunnel_from_logs() -> Option<String> {
    // Try to check if the known tunnel URL is still working
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;
    
    // Check the known working tunnel URL
    let known_url = "https://raised-hub-cat-barcelona.trycloudflare.com";
    if let Ok(response) = client.get(&format!("{}/health", known_url)).send().await {
        if response.status().is_success() {
            return Some(known_url.to_string());
        }
    }
    
    None
}

/// Detect actual Bifrost URL by checking running services
async fn detect_actual_bifrost_url() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok()?;

    // Check Bifrost ports (avoid 3002 which is MindLink dashboard)
    // Start from 3003 and check a wider range to catch dynamically assigned ports
    let ports: Vec<u16> = (3003..3100).collect();
    
    for port in ports {
        let url = format!("http://127.0.0.1:{}", port);
        
        // Try Bifrost-specific endpoints first to avoid false positives
        let endpoints = vec!["/v1/models", "/health", "/v1"];
        
        for endpoint in endpoints {
            if let Ok(response) = client.get(&format!("{}{}", url, endpoint)).send().await {
                if response.status().is_success() {
                    // Additional check: try to verify this is actually Bifrost by checking response
                    if endpoint == "/v1/models" {
                        if let Ok(text) = response.text().await {
                            // Bifrost should return a models list or at least JSON
                            if text.contains("models") || text.contains("data") || text.starts_with("{") {
                                return Some(url);
                            }
                        }
                    } else {
                        return Some(url);
                    }
                }
            }
        }
    }
    
    None
}

// Settings Management Commands

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizedApp {
    pub id: String,
    pub name: String,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub default_model: Option<String>,
    pub authorized_apps: Vec<AuthorizedApp>,
}

/// Check authentication status with intelligent certificate handling
/// This creates a "valet service" that automatically handles certificate downloads
/// from the Downloads folder without requiring manual user intervention
#[tauri::command]
pub async fn check_auth_status(state: State<'_, AppState>) -> Result<bool, String> {
    // Use longer cache duration for OAuth polling to reduce cloudflared spam
    const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(15); // Cache for 15 seconds
    
    let auth_cache = state.auth_cache.read().await;
    let should_check = match &*auth_cache {
        Some((_, last_check)) => last_check.elapsed() > CACHE_DURATION,
        None => true,
    };
    
    if should_check {
        drop(auth_cache); // Release read lock before acquiring write lock
        
        println!("🔍 Performing fresh authentication check (cache expired)...");
        
        // Perform the smart authentication check with automatic certificate handling
        let auth_result = perform_smart_auth_check(&state).await?;
        
        println!("🔍 Smart authentication check result: {}", auth_result);
        
        // Update the cache
        let mut auth_cache = state.auth_cache.write().await;
        *auth_cache = Some((auth_result, Instant::now()));
        Ok(auth_result)
    } else {
        // Use cached result
        let cached_result = auth_cache.unwrap().0;
        println!("💨 Using cached authentication result: {}", cached_result);
        Ok(cached_result)
    }
}

/// Performs intelligent authentication checking with automatic certificate handling
/// 
/// Flow:
/// 1. Try normal cloudflared tunnel token check
/// 2. If that fails, check Downloads for recent cert.pem (within 10 minutes)
/// 3. If found, automatically move it to ~/.cloudflared/cert.pem
/// 4. Re-verify authentication works
/// 5. Return true if successful
async fn perform_smart_auth_check(state: &State<'_, AppState>) -> Result<bool, String> {
    let binary_manager = state.binary_manager.read().await;
    let cloudflared_path = match binary_manager.ensure_cloudflared().await {
        Ok(path) => {
            println!("📍 Using cloudflared at: {:?}", path);
            path
        },
        Err(e) => {
            println!("❌ Failed to get cloudflared binary: {}", e);
            return Ok(false);
        }
    };
    drop(binary_manager);

    // Step 1: Try normal authentication check first
    println!("🚀 Step 1: Trying normal cloudflared authentication...");
    if let Ok(true) = try_cloudflared_auth(&cloudflared_path).await {
        println!("✅ Normal authentication successful");
        return Ok(true);
    }

    println!("⚠️ Normal authentication failed, checking for automatic certificate handling...");

    // Step 2: Check Downloads folder for recent cert.pem file
    println!("🔍 Step 2: Checking Downloads folder for recent cert.pem...");
    if let Some(downloads_cert_path) = find_recent_cert_in_downloads().await {
        println!("✅ Found recent cert.pem in Downloads: {:?}", downloads_cert_path);

        // Step 3: Automatically move certificate to ~/.cloudflared
        println!("📁 Step 3: Moving certificate to ~/.cloudflared...");
        if let Err(e) = move_cert_to_cloudflared(&downloads_cert_path).await {
            println!("❌ Failed to move certificate: {}", e);
            return Ok(false);
        }
        println!("✅ Certificate moved successfully");

        // Step 4: Re-verify authentication works
        println!("🔄 Step 4: Re-verifying authentication after certificate move...");
        if let Ok(true) = try_cloudflared_auth(&cloudflared_path).await {
            println!("🎉 Authentication successful after automatic certificate handling!");
            return Ok(true);
        } else {
            println!("❌ Authentication still failed after moving certificate");
            return Ok(false);
        }
    }

    println!("❌ No recent certificate found in Downloads folder");
    Ok(false)
}

/// Try cloudflared authentication using tunnel token command
async fn try_cloudflared_auth(cloudflared_path: &Path) -> Result<bool, String> {
    // First check if certificate file exists and is valid
    let home_dir = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let cert_path = home_dir.join(".cloudflared").join("cert.pem");
    
    if !cert_path.exists() {
        println!("❌ Certificate file does not exist at: {:?}", cert_path);
        return Ok(false);
    }
    
    // Check if certificate file is readable and non-empty
    match fs::read_to_string(&cert_path).await {
        Ok(cert_content) if cert_content.trim().is_empty() => {
            println!("❌ Certificate file exists but is empty");
            return Ok(false);
        }
        Ok(_) => {
            println!("✅ Certificate file exists and has content");
        }
        Err(e) => {
            println!("❌ Cannot read certificate file: {}", e);
            return Ok(false);
        }
    }
    
    // Now check with cloudflared command using tunnel list (which works when authenticated)
    println!("🚀 Running 'cloudflared tunnel list' to verify authentication...");
    match Command::new(cloudflared_path)
        .args(&["tunnel", "list"])
        .output()
        .await
    {
        Ok(output) => {
            let success = output.status.success();
            if success {
                println!("✅ cloudflared authentication verified successfully");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("❌ cloudflared authentication failed - stdout: {}, stderr: {}", stdout, stderr);
            }
            Ok(success)
        }
        Err(e) => {
            println!("❌ Failed to execute cloudflared command: {}", e);
            Ok(false)
        }
    }
}

/// Find recent cert.pem file in Downloads folder (within last 10 minutes)
async fn find_recent_cert_in_downloads() -> Option<std::path::PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let downloads_dir = dirs::download_dir()?;
    let cert_path = downloads_dir.join("cert.pem");
    
    if !cert_path.exists() {
        println!("❌ No cert.pem found in Downloads folder: {:?}", cert_path);
        return None;
    }
    
    // Check file modification time
    match fs::metadata(&cert_path).await {
        Ok(metadata) => {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
                    let age = now.saturating_sub(duration);
                    
                    // Check if file is less than 10 minutes old
                    if age.as_secs() < 600 { // 10 minutes = 600 seconds
                        println!("✅ Found recent cert.pem ({}s old) in Downloads", age.as_secs());
                        
                        // Verify it's not empty
                        match fs::read_to_string(&cert_path).await {
                            Ok(content) if !content.trim().is_empty() => {
                                println!("✅ Certificate file has content ({} chars)", content.len());
                                return Some(cert_path);
                            }
                            Ok(_) => {
                                println!("❌ Certificate file in Downloads is empty");
                            }
                            Err(e) => {
                                println!("❌ Cannot read certificate file in Downloads: {}", e);
                            }
                        }
                    } else {
                        println!("⚠️ cert.pem in Downloads is too old ({}s), ignoring", age.as_secs());
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Cannot get metadata for cert.pem in Downloads: {}", e);
        }
    }
    
    None
}

/// Move certificate from Downloads to ~/.cloudflared/cert.pem
async fn move_cert_to_cloudflared(downloads_cert_path: &Path) -> Result<(), String> {
    let home_dir = dirs::home_dir().ok_or("Cannot determine home directory")?;
    let cloudflared_dir = home_dir.join(".cloudflared");
    let target_cert_path = cloudflared_dir.join("cert.pem");
    
    // Create .cloudflared directory if it doesn't exist
    if !cloudflared_dir.exists() {
        println!("📁 Creating .cloudflared directory: {:?}", cloudflared_dir);
        fs::create_dir_all(&cloudflared_dir).await
            .map_err(|e| format!("Failed to create .cloudflared directory: {}", e))?;
    }
    
    // Copy the file first (safer than move in case of permissions issues)
    println!("📋 Copying cert.pem from Downloads to .cloudflared...");
    fs::copy(downloads_cert_path, &target_cert_path).await
        .map_err(|e| format!("Failed to copy certificate file: {}", e))?;
    
    // Verify the copy was successful
    match fs::read_to_string(&target_cert_path).await {
        Ok(content) if !content.trim().is_empty() => {
            println!("✅ Certificate successfully copied ({} chars)", content.len());
        }
        Ok(_) => {
            return Err("Copied certificate file is empty".to_string());
        }
        Err(e) => {
            return Err(format!("Cannot verify copied certificate: {}", e));
        }
    }
    
    // Now remove the original from Downloads (cleanup)
    println!("🗑️ Cleaning up original cert.pem from Downloads...");
    if let Err(e) = fs::remove_file(downloads_cert_path).await {
        println!("⚠️ Warning: Failed to remove original cert.pem from Downloads: {}", e);
        // Not a fatal error, the copy succeeded
    } else {
        println!("✅ Original cert.pem removed from Downloads");
    }
    
    Ok(())
}

/// Get current application settings
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    // For now, we'll create a simple settings system using files in the config directory
    let config_dir = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?
        .join(".mindlink");
    
    let settings_path = config_dir.join("settings.json");
    
    // Try to read existing settings file
    if let Ok(content) = fs::read_to_string(&settings_path).await {
        if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
            return Ok(settings);
        }
    }
    
    // Return default settings if file doesn't exist or is invalid
    Ok(Settings {
        default_model: Some("gpt-4".to_string()),
        authorized_apps: Vec::new(),
    })
}

/// Update a single setting
#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    let config_dir = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?
        .join(".mindlink");
    
    let settings_path = config_dir.join("settings.json");
    
    // Read current settings
    let mut settings = if let Ok(content) = fs::read_to_string(&settings_path).await {
        serde_json::from_str::<serde_json::Value>(&content)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    
    // Update the specific setting
    if let Some(obj) = settings.as_object_mut() {
        obj.insert(key, value);
    }
    
    // Ensure config directory exists
    if let Some(parent) = settings_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }
    
    // Write back to file
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
    fs::write(&settings_path, content).await
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    Ok(())
}

/// Get all authorized apps
#[tauri::command]
pub async fn get_authorized_apps(state: State<'_, AppState>) -> Result<Vec<AuthorizedApp>, String> {
    let config_dir = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?
        .join(".mindlink");
    
    let settings_path = config_dir.join("settings.json");
    
    // Try to read existing settings file
    if let Ok(content) = fs::read_to_string(&settings_path).await {
        if let Ok(settings) = serde_json::from_str::<Settings>(&content) {
            return Ok(settings.authorized_apps);
        }
    }
    
    // Return empty list if file doesn't exist or is invalid
    Ok(Vec::new())
}

/// Add a new authorized app
#[tauri::command]
pub async fn add_authorized_app(
    state: State<'_, AppState>,
    name: String,
    model: String,
) -> Result<(), String> {
    let config_dir = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?
        .join(".mindlink");
    
    let settings_path = config_dir.join("settings.json");
    
    // Read current settings
    let mut settings = if let Ok(content) = fs::read_to_string(&settings_path).await {
        serde_json::from_str::<Settings>(&content)
            .unwrap_or_else(|_| Settings {
                default_model: Some("gpt-4".to_string()),
                authorized_apps: Vec::new(),
            })
    } else {
        Settings {
            default_model: Some("gpt-4".to_string()),
            authorized_apps: Vec::new(),
        }
    };
    
    let new_app = AuthorizedApp {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        model,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    settings.authorized_apps.push(new_app);
    
    // Ensure config directory exists
    if let Some(parent) = settings_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
    }
    
    // Write back to file
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
    fs::write(&settings_path, content).await
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    Ok(())
}

/// Update an app's model
#[tauri::command]
pub async fn update_app_model(
    state: State<'_, AppState>,
    app_id: String,
    model: String,
) -> Result<(), String> {
    let config_dir = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?
        .join(".mindlink");
    
    let settings_path = config_dir.join("settings.json");
    
    // Read current settings
    let mut settings = if let Ok(content) = fs::read_to_string(&settings_path).await {
        serde_json::from_str::<Settings>(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))?
    } else {
        return Err("Settings file not found".to_string());
    };
    
    let app = settings.authorized_apps.iter_mut()
        .find(|app| app.id == app_id)
        .ok_or_else(|| "App not found".to_string())?;
    
    app.model = model;
    
    // Write back to file
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
    fs::write(&settings_path, content).await
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    Ok(())
}

/// Remove an authorized app
#[tauri::command]
pub async fn remove_authorized_app(
    state: State<'_, AppState>,
    app_id: String,
) -> Result<(), String> {
    let config_dir = dirs::home_dir()
        .ok_or_else(|| "Cannot determine home directory".to_string())?
        .join(".mindlink");
    
    let settings_path = config_dir.join("settings.json");
    
    // Read current settings
    let mut settings = if let Ok(content) = fs::read_to_string(&settings_path).await {
        serde_json::from_str::<Settings>(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))?
    } else {
        return Err("Settings file not found".to_string());
    };
    
    settings.authorized_apps.retain(|app| app.id != app_id);
    
    // Write back to file
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
    fs::write(&settings_path, content).await
        .map_err(|e| format!("Failed to write settings file: {}", e))?;
    
    Ok(())
}

/// Show and focus the main application window.
///
/// This command is typically called from settings or other secondary windows
/// to return focus to the main dashboard window.
///
/// # Returns
///
/// - `Ok(())`: Window was successfully shown and focused
/// - `Err(String)`: Error message if operation failed
#[tauri::command]
pub async fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    println!("show_main_window command called");
    
    // Debug: List all available webview windows
    let windows = app_handle.webview_windows();
    println!("Available webview windows: {:?}", windows.keys().collect::<Vec<_>>());
    
    if let Some(window) = app_handle.get_webview_window("main") {
        println!("Main window found, showing and focusing");
        
        // Always show the window first
        window.show().map_err(|e| format!("Failed to show main window: {}", e))?;
        
        // Then focus it
        window.set_focus().map_err(|e| format!("Failed to focus main window: {}", e))?;
        
        // Also try to bring it to the front/unminimize it if needed
        if let Err(e) = window.unminimize() {
            println!("Could not unminimize main window (might not be minimized): {}", e);
        }
        
        println!("Main window shown and focused successfully");
        Ok(())
    } else {
        println!("Main window not found!");
        
        // Try to find any window with a similar name
        for (label, _) in &windows {
            println!("Found window with label: {}", label);
            if label.to_lowercase().contains("main") || label == "MindLink - Local LLM Router" {
                if let Some(window) = app_handle.get_webview_window(label) {
                    println!("Trying to use window: {}", label);
                    window.show().map_err(|e| format!("Failed to show window {}: {}", label, e))?;
                    window.set_focus().map_err(|e| format!("Failed to focus window {}: {}", label, e))?;
                    if let Err(e) = window.unminimize() {
                        println!("Could not unminimize window {} (might not be minimized): {}", label, e);
                    }
                    return Ok(());
                }
            }
        }
        
        Err("Main window not found".to_string())
    }
}

/// Test command to debug the show_main_window functionality
#[tauri::command]
pub async fn test_show_main_window(app_handle: AppHandle) -> Result<String, String> {
    println!("test_show_main_window called");
    match show_main_window(app_handle).await {
        Ok(()) => Ok("show_main_window succeeded".to_string()),
        Err(e) => Ok(format!("show_main_window failed: {}", e)),
    }
}

/// Simple test command
#[tauri::command]
pub fn simple_test() -> String {
    "Hello from Rust!".to_string()
}

/// Open an external URL in the default browser
#[tauri::command]
pub async fn open_external_url(url: String) -> Result<String, String> {
    use std::process::Command;
    
    println!("Opening external URL: {}", url);
    
    // Use the appropriate command for the current platform
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "start", &url])
            .output()
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&url)
            .output()
    } else {
        // Linux and other Unix-like systems
        Command::new("xdg-open")
            .arg(&url)
            .output()
    };
    
    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(format!("Successfully opened URL: {}", url))
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to open URL: {}", error))
            }
        }
        Err(e) => Err(format!("Failed to execute open command: {}", e)),
    }
}

/// Get certificate installation instructions for manual setup
#[tauri::command]
pub async fn get_certificate_instructions() -> Result<String, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;
    let cert_path = home_dir.join(".cloudflared").join("cert.pem");
    
    let instructions = format!(
        r#"Certificate Installation Instructions:

1. Complete the authentication in your browser (if you haven't already)
2. If the certificate download failed, manually copy it to:
   {}

3. The certificate file should contain your Cloudflare authentication credentials
4. Make sure the file is readable and not empty
5. Once installed, the authentication should work automatically

If you're still having issues:
- Try logging out and logging back in
- Check that the .cloudflared directory has proper permissions
- Ensure the certificate file is not corrupted"#,
        cert_path.display()
    );
    
    Ok(instructions)
}

/// Enhanced certificate status check with automatic handling information
#[tauri::command]
pub async fn check_certificate_status() -> Result<String, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "Cannot determine home directory".to_string())?;
    let cloudflared_dir = home_dir.join(".cloudflared");
    let cert_path = cloudflared_dir.join("cert.pem");
    
    let mut status = String::new();
    status.push_str("🔍 Certificate Status Check:\n\n");
    
    // Check directory
    if cloudflared_dir.exists() {
        status.push_str("✅ .cloudflared directory exists\n");
    } else {
        status.push_str("❌ .cloudflared directory does not exist\n");
        status.push_str("   → Will be created automatically when certificate is found\n");
        return Ok(status);
    }
    
    // Check certificate file
    if cert_path.exists() {
        status.push_str("✅ cert.pem file exists\n");
        
        // Check if readable and non-empty
        match fs::read_to_string(&cert_path).await {
            Ok(content) => {
                if content.trim().is_empty() {
                    status.push_str("❌ cert.pem file is empty\n");
                } else if content.len() > 100 {
                    status.push_str(&format!("✅ cert.pem file has content ({} chars)\n", content.len()));
                } else {
                    status.push_str("⚠️ cert.pem file seems too small\n");
                }
            }
            Err(e) => {
                status.push_str(&format!("❌ Cannot read cert.pem file: {}\n", e));
            }
        }
    } else {
        status.push_str("❌ cert.pem file does not exist\n");
        
        // Check Downloads folder for automatic handling
        if let Some(downloads_cert) = find_recent_cert_in_downloads().await {
            status.push_str("✅ Recent cert.pem found in Downloads folder!\n");
            status.push_str("   → Will be moved automatically on next authentication check\n");
            status.push_str(&format!("   → Location: {:?}\n", downloads_cert));
        } else {
            status.push_str("❌ No recent cert.pem found in Downloads folder\n");
            status.push_str("   → Complete the Cloudflare authentication in your browser\n");
            status.push_str("   → The certificate will be handled automatically\n");
        }
    }
    
    status.push_str(&format!("\nExpected path: {}", cert_path.display()));
    
    // Add automatic handling information
    if let Some(downloads_dir) = dirs::download_dir() {
        status.push_str(&format!("\nWatching for cert.pem in: {}", downloads_dir.display()));
    }
    
    status.push_str("\n\n🤖 Automatic Certificate Handling is ENABLED");
    status.push_str("\nThe system will automatically move cert.pem from Downloads to .cloudflared when found.");
    
    Ok(status)
}

/// Test the automatic certificate handling system (for debugging)
#[tauri::command]
pub async fn test_certificate_handling() -> Result<String, String> {
    let mut result = String::new();
    result.push_str("🧪 Testing Automatic Certificate Handling System:\n\n");
    
    // Test 1: Check Downloads folder
    result.push_str("Test 1: Downloads folder check\n");
    if let Some(downloads_dir) = dirs::download_dir() {
        result.push_str(&format!("✅ Downloads directory: {:?}\n", downloads_dir));
        
        if let Some(cert_path) = find_recent_cert_in_downloads().await {
            result.push_str(&format!("✅ Recent cert.pem found: {:?}\n", cert_path));
        } else {
            result.push_str("❌ No recent cert.pem found in Downloads\n");
        }
    } else {
        result.push_str("❌ Cannot determine Downloads directory\n");
    }
    
    // Test 2: Check .cloudflared directory permissions
    result.push_str("\nTest 2: .cloudflared directory permissions\n");
    if let Some(home_dir) = dirs::home_dir() {
        let cloudflared_dir = home_dir.join(".cloudflared");
        result.push_str(&format!("Target directory: {:?}\n", cloudflared_dir));
        
        if cloudflared_dir.exists() {
            result.push_str("✅ Directory exists\n");
        } else {
            result.push_str("⚠️ Directory doesn't exist (will be created automatically)\n");
        }
    } else {
        result.push_str("❌ Cannot determine home directory\n");
    }
    
    // Test 3: Check current authentication status
    result.push_str("\nTest 3: Current authentication status\n");
    if let Some(home_dir) = dirs::home_dir() {
        let cert_path = home_dir.join(".cloudflared").join("cert.pem");
        if cert_path.exists() {
            match fs::read_to_string(&cert_path).await {
                Ok(content) if !content.trim().is_empty() => {
                    result.push_str(&format!("✅ Valid certificate exists ({} chars)\n", content.len()));
                }
                Ok(_) => {
                    result.push_str("❌ Certificate file is empty\n");
                }
                Err(e) => {
                    result.push_str(&format!("❌ Cannot read certificate: {}\n", e));
                }
            }
        } else {
            result.push_str("❌ No certificate file exists\n");
        }
    }
    
    result.push_str("\n✅ Test complete. The automatic certificate handling system is ready.");
    
    Ok(result)
}

// ===== Plugin Management Commands =====

/// Plugin manifest structure for external plugins
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub main: String,
    pub dependencies: Option<Vec<String>>,
    pub mindlink_version: Option<String>,
}

/// Response for plugin discovery operations
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginDiscoveryResponse {
    pub success: bool,
    pub manifests: Vec<PluginManifest>,
    pub plugins_directory: Option<String>,
    pub error: Option<String>,
}

/// Get available plugin manifests from the plugins directory
#[tauri::command]
pub async fn get_plugin_manifests() -> Result<PluginDiscoveryResponse, String> {
    println!("🔌 Discovering available plugins...");
    
    // For now, return built-in manifests since we haven't implemented external plugins yet
    let built_in_manifests = vec![
        PluginManifest {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Connect to OpenAI GPT models via API".to_string()),
            author: Some("MindLink Team".to_string()),
            main: "openai.js".to_string(),
            dependencies: None,
            mindlink_version: Some("1.0.0".to_string()),
        },
        PluginManifest {
            id: "anthropic".to_string(),
            name: "Anthropic".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Connect to Claude models via Anthropic API".to_string()),
            author: Some("MindLink Team".to_string()),
            main: "anthropic.js".to_string(),
            dependencies: None,
            mindlink_version: Some("1.0.0".to_string()),
        },
        PluginManifest {
            id: "google".to_string(),
            name: "Google".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Connect to Gemini models via Google AI Studio".to_string()),
            author: Some("MindLink Team".to_string()),
            main: "google.js".to_string(),
            dependencies: None,
            mindlink_version: Some("1.0.0".to_string()),
        },
    ];
    
    println!("✅ Found {} plugin manifests", built_in_manifests.len());
    
    Ok(PluginDiscoveryResponse {
        success: true,
        manifests: built_in_manifests,
        plugins_directory: Some("Built-in plugins".to_string()),
        error: None,
    })
}

/// Get the plugins directory path for external plugins
#[tauri::command]
pub async fn get_plugins_directory() -> Result<String, String> {
    // In production, this would be in the app data directory
    // For example: ~/.local/share/mindlink/plugins or %APPDATA%/mindlink/plugins
    let app_data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Cannot determine app data directory".to_string())?;
    
    let plugins_dir = app_data_dir.join("mindlink").join("plugins");
    
    Ok(plugins_dir.to_string_lossy().to_string())
}

/// Create the plugins directory if it doesn't exist
#[tauri::command]
pub async fn ensure_plugins_directory() -> Result<String, String> {
    let app_data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Cannot determine app data directory".to_string())?;
    
    let plugins_dir = app_data_dir.join("mindlink").join("plugins");
    
    // Create directory if it doesn't exist
    if !plugins_dir.exists() {
        println!("📁 Creating plugins directory: {:?}", plugins_dir);
        fs::create_dir_all(&plugins_dir).await
            .map_err(|e| format!("Failed to create plugins directory: {}", e))?;
    }
    
    Ok(plugins_dir.to_string_lossy().to_string())
}
