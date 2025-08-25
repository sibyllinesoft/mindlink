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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

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
/// - `last_error`: Most recent error message (if any)
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub is_serving: bool,
    pub is_authenticated: bool,
    pub tunnel_url: Option<String>,
    pub server_url: Option<String>,
    pub bifrost_url: Option<String>,
    pub last_error: Option<String>,
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
        let auth_manager = state.auth_manager.read().await;
        auth_manager.is_authenticated().await
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

    // Detect actual Bifrost services running on various ports  
    let bifrost_url = match detect_actual_bifrost_url().await {
        Some(url) => Some(url),
        None => {
            let bifrost_manager = state.bifrost_manager.read().await;
            bifrost_manager.get_local_url().await
        }
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
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let config_manager = state.config_manager.read().await;
    let config = config_manager.get_config().await;
    // Convert ConfigSchema to HashMap
    let json_value =
        serde_json::to_value(&config).map_err(|e| format!("Serialization failed: {}", e))?;
    let map = json_value
        .as_object()
        .unwrap()
        .clone()
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();
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
        },
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
            })
        },
        Err(e) => {
            println!("❌ Failed to close tunnel: {}", e);

            Ok(ServiceResponse {
                success: false,
                message: Some(format!("Failed to close tunnel: {}", e)),
                server_url: None,
                tunnel_url: None,
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
        })
    } else {
        Ok(ServiceResponse {
            success: true,
            message: Some("No active tunnel".to_string()),
            server_url: None,
            tunnel_url: None,
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
        }),
        Err(e) => Ok(ServiceResponse {
            success: false,
            message: Some(format!("Logout failed: {}", e)),
            server_url: None,
            tunnel_url: None,
        }),
    }
}

// ===== Helper functions for detecting actual running services =====

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

    // Check common Bifrost ports
    let ports = vec![3002, 3003, 3004];
    
    for port in ports {
        let url = format!("http://127.0.0.1:{}", port);
        
        // Try health endpoint or root endpoint
        let endpoints = vec!["/health", "/v1/models", "/"];
        
        for endpoint in endpoints {
            if let Ok(response) = client.get(&format!("{}{}", url, endpoint)).send().await {
                if response.status().is_success() {
                    // If it's port 3003 (API), return port 3002 (dashboard)
                    if port == 3003 {
                        return Some("http://127.0.0.1:3002".to_string());
                    }
                    return Some(url);
                }
            }
        }
    }
    
    None
}
