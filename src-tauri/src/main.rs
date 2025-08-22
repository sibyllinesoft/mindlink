// MindLink Tauri Application  
// Replaces the Electron main process with Rust backend
#![allow(dead_code)] // Allow unused functions during development
#![allow(unused_variables)] // Allow unused variables during development  
#![allow(missing_docs)] // Allow missing documentation during development
#![allow(missing_copy_implementations)] // Allow missing Copy implementations
#![allow(static_mut_refs)] // Allow static mut references for global state

use tauri::{
    AppHandle, Manager, Emitter,
    tray::{TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder, MenuEvent},
    WebviewWindowBuilder, WebviewUrl,
};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::RwLock;
use std::sync::Arc;

mod managers;
mod commands;
mod error;
mod logging;
mod process_monitor;
mod dialog;
mod error_reporter;
mod command_helpers;

#[cfg(test)]
mod tests;

use error::{MindLinkError, MindLinkResult};
use logging::{init_logging, get_logger, LogEntry, LogLevel, LogCategory};
use process_monitor::init_process_monitor;
use error_reporter::{init_error_reporter, ErrorReportingConfig};

use managers::{
    auth_manager::AuthManager,
    server_manager::ServerManager,
    tunnel_manager::TunnelManager,
    config_manager::ConfigManager,
    bifrost_manager::BifrostManager,
    dashboard_manager::DashboardManager,
    binary_manager::BinaryManager,
};

// Application state shared between Tauri commands
#[derive(Debug)]
pub struct AppState {
    /// Authentication and OAuth token management
    pub auth_manager: Arc<RwLock<AuthManager>>,
    /// HTTP server management for OpenAI-compatible endpoints  
    pub server_manager: Arc<RwLock<ServerManager>>,
    /// Tunneling service management
    pub tunnel_manager: Arc<RwLock<TunnelManager>>,
    /// Configuration file management
    pub config_manager: Arc<RwLock<ConfigManager>>,
    /// Bifrost UI server management
    pub bifrost_manager: Arc<RwLock<BifrostManager>>,
    /// Dashboard service management
    pub dashboard_manager: Arc<RwLock<DashboardManager>>,
    /// Binary download and management
    pub binary_manager: Arc<RwLock<BinaryManager>>,
    /// Current serving status
    pub is_serving: Arc<RwLock<bool>>,
    /// Last error message for display
    pub last_error: Arc<RwLock<Option<String>>>,
}

impl AppState {
    /// Create new application state with all managers initialized
    pub async fn new() -> MindLinkResult<Self> {
        let config_manager = Arc::new(RwLock::new(ConfigManager::new().await?));
        let auth_manager = Arc::new(RwLock::new(AuthManager::new().await?));
        let server_manager = Arc::new(RwLock::new(ServerManager::new().await));
        
        let tunnel_manager = Arc::new(RwLock::new(
            TunnelManager::new().await.map_err(|e| {
                MindLinkError::Internal {
                    message: "Failed to initialize tunnel manager".to_string(),
                    component: Some("AppState".to_string()),
                    source: Some(e.into()),
                }
            })?
        ));
        
        let binary_manager = Arc::new(RwLock::new(
            BinaryManager::new().await.map_err(|e| {
                MindLinkError::Internal {
                    message: "Failed to initialize binary manager".to_string(),
                    component: Some("AppState".to_string()),
                    source: Some(e.into()),
                }
            })?
        ));
        
        let bifrost_manager = Arc::new(RwLock::new(BifrostManager::new().await));
        let dashboard_manager = Arc::new(RwLock::new(DashboardManager::new().await));

        Ok(Self {
            auth_manager,
            server_manager,
            tunnel_manager,
            config_manager,
            bifrost_manager,
            dashboard_manager,
            binary_manager,
            is_serving: Arc::new(RwLock::new(false)),
            last_error: Arc::new(RwLock::new(None)),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize comprehensive logging system
    env_logger::init();
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging system: {}", e);
        // Continue without comprehensive logging but with basic env_logger
    }
    
    // Initialize process monitor
    let _process_monitor = init_process_monitor();
    
    // Initialize error reporting system
    let error_config = ErrorReportingConfig {
        show_user_dialogs: true,
        send_notifications: true,
        auto_retry_recoverable: false, // Manual retry for better user control
        max_retry_attempts: 3,
        retry_delay_seconds: 5,
    };
    init_error_reporter(error_config);
    
    // Initialize application state
    let app_state = match AppState::new().await {
        Ok(state) => {
            if let Some(logger) = get_logger() {
                let entry = LogEntry::new(
                    LogLevel::Info,
                    LogCategory::System,
                    "Application state initialized successfully".to_string(),
                ).with_component("Main");
                logger.log(entry);
            }
            state
        },
        Err(e) => {
            eprintln!("Failed to initialize application state: {}", e.user_message());
            if let Some(logger) = get_logger() {
                logger.log_error("Main", &e, None);
            }
            return Err(e.into());
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .setup(move |app| {
            // Create system tray menu
            let login_serve = MenuItemBuilder::new("Login & Serve").id("login_serve").build(app)?;
            let stop_serving = MenuItemBuilder::new("Stop Serving").id("stop_serving").enabled(false).build(app)?;
            let bifrost_dashboard = MenuItemBuilder::new("Bifrost Dashboard").id("bifrost_dashboard").build(app)?;
            let connection_status = MenuItemBuilder::new("Connection Status").id("connection_status").build(app)?;
            let settings = MenuItemBuilder::new("Settings").id("settings").build(app)?;
            let open_api_dashboard = MenuItemBuilder::new("Open API Dashboard").id("open_api_dashboard").enabled(false).build(app)?;
            let copy_api_url = MenuItemBuilder::new("Copy API URL").id("copy_api_url").enabled(false).build(app)?;
            let help = MenuItemBuilder::new("Help").id("help").build(app)?;
            let quit = MenuItemBuilder::new("Quit").id("quit").build(app)?;
            
            let tray_menu = MenuBuilder::new(app)
                .item(&login_serve)
                .item(&stop_serving)
                .separator()
                .item(&bifrost_dashboard)
                .item(&connection_status)
                .item(&settings)
                .separator()
                .item(&open_api_dashboard)
                .item(&copy_api_url)
                .separator()
                .item(&help)
                .item(&quit)
                .build()?;
            
            let _tray = TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(handle_menu_event)
                .build(app)?;

            // Start dashboard automatically
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_dashboard(app_handle).await {
                    if let Some(logger) = get_logger() {
                        let entry = LogEntry::new(
                            LogLevel::Error,
                            LogCategory::System,
                            format!("Failed to start dashboard: {}", e),
                        ).with_component("Main");
                        logger.log(entry);
                    } else {
                        eprintln!("Failed to start dashboard: {}", e);
                    }
                }
            });

            // Show the main window on startup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            // Start Bifrost automatically (if binary available)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_bifrost_service(app_handle).await {
                    eprintln!("Bifrost auto-start failed: {}", e);
                }
            });

            // Start health monitoring
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                start_health_monitoring(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::login_and_serve,
            commands::stop_serving,
            commands::logout,
            commands::get_config,
            commands::save_config,
            commands::show_notification,
            commands::open_bifrost_dashboard,
            commands::copy_api_url,
            commands::test_completion,
            commands::start_bifrost,
            commands::stop_bifrost,
            commands::install_bifrost_binary,
            commands::get_bifrost_installation_status,
            commands::reinstall_bifrost_binary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

async fn start_dashboard(app_handle: AppHandle) -> MindLinkResult<()> {
    let state = app_handle.state::<AppState>();
    let mut dashboard_manager = state.dashboard_manager.write().await;
    
    match dashboard_manager.start().await {
        Ok(_) => {
            if let Some(logger) = get_logger() {
                let entry = LogEntry::new(
                    LogLevel::Info,
                    LogCategory::System,
                    "MindLink dashboard started successfully".to_string(),
                ).with_component("Dashboard");
                logger.log(entry);
            }
        }
        Err(e) => {
            let mindlink_error = MindLinkError::Internal {
                message: "Failed to start MindLink dashboard".to_string(),
                component: Some("DashboardManager".to_string()),
                source: Some(e.into()),
            };
            
            if let Some(logger) = get_logger() {
                logger.log_error("Dashboard", &mindlink_error, None);
            }
            
            // Show user-friendly notification
            let _ = app_handle.emit("notification", 
                format!("Dashboard Warning: {}", mindlink_error.user_message()));
            
            return Err(mindlink_error);
        }
    }
    
    Ok(())
}

async fn start_bifrost_service(app_handle: AppHandle) -> MindLinkResult<()> {
    let state = app_handle.state::<AppState>();
    let mut bifrost_manager = state.bifrost_manager.write().await;
    
    // Check if binary is available
    if !bifrost_manager.is_binary_available().await {
        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Warn,
                LogCategory::System,
                "Bifrost binary not available - skipping auto-start".to_string(),
            ).with_component("Bifrost");
            logger.log(entry);
        }
        return Ok(());
    }
    
    // Wait a moment for system to stabilize
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    match bifrost_manager.start().await {
        Ok(_) => {
            if let Some(logger) = get_logger() {
                let entry = LogEntry::new(
                    LogLevel::Info,
                    LogCategory::System,
                    "Bifrost LLM Router auto-started successfully".to_string(),
                ).with_component("Bifrost");
                logger.log(entry);
            }
            let _ = app_handle.emit("notification", "Bifrost LLM Router started successfully");
        }
        Err(e) => {
            let mindlink_error = MindLinkError::BinaryExecution {
                message: "Auto-start failed - binary may not be installed correctly".to_string(),
                binary_name: "bifrost".to_string(),
                binary_path: bifrost_manager.get_binary_path().await.map(|p| p.to_string_lossy().to_string()),
                source: Some(e.into()),
            };
            
            if let Some(logger) = get_logger() {
                logger.log_error("Bifrost", &mindlink_error, None);
            }
            
            // Show user-friendly notification
            let _ = app_handle.emit("notification", 
                format!("Bifrost Warning: {}", mindlink_error.user_message()));
            
            // Don't return error for auto-start failures - they're non-critical
        }
    }
    
    Ok(())
}

async fn start_health_monitoring(app_handle: AppHandle) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        // Perform health check
        if let Err(e) = perform_health_check(&app_handle).await {
            eprintln!("Health check failed: {}", e);
        }
    }
}

async fn perform_health_check(app_handle: &AppHandle) -> MindLinkResult<()> {
    let state = app_handle.state::<AppState>();
    let is_serving = *state.is_serving.read().await;
    
    if !is_serving {
        return Ok(());
    }

    // Check all managers' health with proper error handling
    let server_healthy = {
        let server_manager = state.server_manager.read().await;
        match server_manager.check_health().await {
            Ok(healthy) => healthy,
            Err(e) => {
                if let Some(logger) = get_logger() {
                    logger.log_health_check("Server", false, None, None);
                    let entry = LogEntry::new(
                        LogLevel::Warn,
                        LogCategory::HealthCheck,
                        format!("Server health check failed: {}", e),
                    ).with_component("HealthMonitor");
                    logger.log(entry);
                }
                false
            }
        }
    };

    let tunnel_healthy = {
        let tunnel_manager = state.tunnel_manager.read().await;
        match tunnel_manager.check_health().await {
            Ok(healthy) => healthy,
            Err(e) => {
                if let Some(logger) = get_logger() {
                    logger.log_health_check("Tunnel", false, None, None);
                    let entry = LogEntry::new(
                        LogLevel::Warn,
                        LogCategory::HealthCheck,
                        format!("Tunnel health check failed: {}", e),
                    ).with_component("HealthMonitor");
                    logger.log(entry);
                }
                false
            }
        }
    };

    let bifrost_healthy = {
        let bifrost_manager = state.bifrost_manager.read().await;
        match bifrost_manager.check_health().await {
            Ok(healthy) => {
                if let Some(logger) = get_logger() {
                    logger.log_health_check("Bifrost", healthy, 
                        bifrost_manager.get_local_url().await.as_deref(), None);
                }
                healthy
            }
            Err(e) => {
                if let Some(logger) = get_logger() {
                    logger.log_health_check("Bifrost", false, None, None);
                    let entry = LogEntry::new(
                        LogLevel::Warn,
                        LogCategory::HealthCheck,
                        format!("Bifrost health check failed: {}", e),
                    ).with_component("HealthMonitor");
                    logger.log(entry);
                }
                false
            }
        }
    };

    let dashboard_healthy = {
        let dashboard_manager = state.dashboard_manager.read().await;
        match dashboard_manager.check_health().await {
            Ok(healthy) => {
                if let Some(logger) = get_logger() {
                    logger.log_health_check("Dashboard", healthy, 
                        dashboard_manager.get_local_url().await.as_deref(), None);
                }
                healthy
            }
            Err(e) => {
                if let Some(logger) = get_logger() {
                    logger.log_health_check("Dashboard", false, None, None);
                    let entry = LogEntry::new(
                        LogLevel::Warn,
                        LogCategory::HealthCheck,
                        format!("Dashboard health check failed: {}", e),
                    ).with_component("HealthMonitor");
                    logger.log(entry);
                }
                false
            }
        }
    };

    if !server_healthy || !tunnel_healthy || !bifrost_healthy || !dashboard_healthy {
        let error_msg = format!(
            "Health check failed - Server: {}, Tunnel: {}, Bifrost: {}, Dashboard: {}",
            server_healthy, tunnel_healthy, bifrost_healthy, dashboard_healthy
        );
        
        *state.last_error.write().await = Some(error_msg.clone());
        
        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Error,
                LogCategory::HealthCheck,
                error_msg.clone(),
            ).with_component("HealthMonitor");
            logger.log(entry);
        }
        
        // Try to restart Bifrost if it's unhealthy
        if !bifrost_healthy {
            let mut bifrost_manager = state.bifrost_manager.write().await;
            if let Err(e) = bifrost_manager.restart().await {
                let restart_error = MindLinkError::ProcessMonitoring {
                    message: "Failed to restart Bifrost service".to_string(),
                    process_name: "Bifrost".to_string(),
                    pid: None,
                    source: Some(e.into()),
                };
                
                if let Some(logger) = get_logger() {
                    logger.log_error("HealthMonitor", &restart_error, None);
                }
            }
        }
        
        // Try to restart dashboard if it's unhealthy
        if !dashboard_healthy {
            let mut dashboard_manager = state.dashboard_manager.write().await;
            if let Err(e) = dashboard_manager.start().await {
                let restart_error = MindLinkError::ProcessMonitoring {
                    message: "Failed to restart Dashboard service".to_string(),
                    process_name: "Dashboard".to_string(),
                    pid: None,
                    source: Some(e.into()),
                };
                
                if let Some(logger) = get_logger() {
                    logger.log_error("HealthMonitor", &restart_error, None);
                }
            }
        }
    }

    Ok(())
}

fn handle_tray_event(_app: &AppHandle, event: TrayIconEvent) {
    println!("Tray event received: {:?}", event);
    
    // For now, we'll implement menu handling through proper menu event system
    // The exact event structure will be updated when we get the proper API docs
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "login_serve" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::login_and_serve(app_handle.state()).await {
                    eprintln!("Login and serve failed: {}", e);
                }
            });
        }
        "stop_serving" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = commands::stop_serving(app_handle.state()).await {
                    eprintln!("Stop serving failed: {}", e);
                }
            });
        }
        "bifrost_dashboard" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();
                let bifrost_manager = state.bifrost_manager.read().await;
                if let Some(url) = bifrost_manager.get_local_url().await {
                    if bifrost_manager.is_running().await {
                        println!("Opening Bifrost dashboard: {}", url);
                        #[allow(deprecated)]
                        if let Err(e) = app_handle.shell().open(&url, None) {
                            eprintln!("Failed to open Bifrost dashboard in browser: {}", e);
                        }
                    } else {
                        eprintln!("Bifrost dashboard is not running");
                    }
                } else {
                    eprintln!("Bifrost dashboard URL not available");
                }
            });
        }
        "connection_status" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                show_connection_status(&app_handle).await;
            });
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("settings") {
                let _ = window.show();
                let _ = window.set_focus();
            } else {
                create_settings_window(app);
            }
        }
        "copy_api_url" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                match commands::copy_api_url(app_handle.state()).await {
                    Ok(api_url) => {
                        // Note: Direct clipboard access from tray menu is limited
                        // This will print the URL and could be enhanced with notification
                        println!("API URL to copy: {}", api_url);
                        // Could add a notification or show in a dialog
                        let _ = app_handle.dialog().message(&format!("API URL: {}", api_url));
                    }
                    Err(e) => {
                        eprintln!("Failed to get API URL: {}", e);
                    }
                }
            });
        }
        "help" => {
            #[allow(deprecated)]
            if let Err(e) = app.shell().open("https://github.com/mindlink/docs", None) {
                eprintln!("Failed to open help URL: {}", e);
            }
        }
        "open_api_dashboard" => {
            // Open the main MindLink dashboard window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {
            println!("Unhandled menu item: {}", event.id.as_ref());
        }
    }
}

fn create_settings_window(app: &AppHandle) {
    let _window = WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("settings.html".into())
    )
    .title("MindLink Settings")
    .inner_size(600.0, 500.0)
    .resizable(true)
    .build();
}

async fn show_connection_status(app: &AppHandle) {
    let state = app.state::<AppState>();
    let is_serving = *state.is_serving.read().await;
    let last_error = state.last_error.read().await.clone();
    
    let status = if is_serving { "Connected" } else { "Disconnected" };
    
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
    
    let dashboard_url = {
        let dashboard_manager = state.dashboard_manager.read().await;
        dashboard_manager.get_local_url().await
    };
    
    let mut message = format!("Status: {}\n", status);
    if is_serving {
        if let Some(server) = server_url {
            message.push_str(&format!("Local: {}\n", server));
        }
        if let Some(tunnel) = tunnel_url {
            message.push_str(&format!("Public: {}\n", tunnel));
        }
    }
    
    if let Some(dashboard) = dashboard_url {
        message.push_str(&format!("Dashboard: Running ({})\n", dashboard));
    } else {
        message.push_str("Dashboard: Stopped\n");
    }
    
    if let Some(bifrost) = bifrost_url {
        message.push_str(&format!("Bifrost LLM: Running ({})", bifrost));
    } else {
        message.push_str("Bifrost LLM: Stopped");
    }
    
    if let Some(error) = last_error {
        message.push_str(&format!("\nLast Error: {}", error));
    }
    
    // Show dialog using Tauri's dialog API
    let _ = app.dialog().message(&message);
}