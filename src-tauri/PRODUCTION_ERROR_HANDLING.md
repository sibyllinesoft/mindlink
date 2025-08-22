# MindLink Production Error Handling System

This document describes the comprehensive error handling and logging system implemented for the MindLink Tauri application to ensure production readiness.

## System Overview

The error handling system consists of several integrated components:

1. **Structured Error Types** (`src/error.rs`) - Type-safe error definitions with user-friendly messages
2. **Comprehensive Logging** (`src/logging.rs`) - File-based logging with rotation and structured output
3. **Process Monitoring** (`src/process_monitor.rs`) - Child process supervision with stdout/stderr capture
4. **User Dialog System** (`src/dialog.rs`) - User-friendly error dialogs and notifications
5. **Error Reporter** (`src/error_reporter.rs`) - Centralized error reporting with context tracking
6. **Command Helpers** (`src/command_helpers.rs`) - Convenience functions for command error handling

## Key Features Implemented

### ✅ 1. Rust Error Propagation
- **Eliminated all `.unwrap()` and `.expect()` calls** in production code
- **Consistent `Result` types** throughout all manager functions
- **Proper error chaining** with `anyhow::Error` and custom `MindLinkError`
- **Type-safe error handling** with pattern matching

**Example:**
```rust
pub async fn start(&mut self) -> MindLinkResult<()> {
    let binary_path = self.binary_path.as_ref()
        .ok_or_else(|| MindLinkError::BinaryExecution {
            message: "Binary not found - please run the build system".to_string(),
            binary_name: "bifrost-http".to_string(),
            binary_path: None,
            source: None,
        })?;
    // ... rest of implementation
}
```

### ✅ 2. User-Facing Dialogs
- **Contextual error messages** that don't expose internal details
- **Actionable suggestions** for common error scenarios
- **Retry mechanisms** for recoverable errors
- **Non-blocking notifications** for background operations

**User-Friendly Error Categories:**
- `Authentication Error: Login failed - Please try logging in again`
- `Connection failed to cloudflare.com - Check your internet connection`
- `Program Error: bifrost failed to start - Please reinstall bifrost`
- `Service Health: Bifrost is not responding - Restart the Bifrost service`

### ✅ 3. Comprehensive Logging
- **Structured log entries** with timestamps, categories, and components
- **Automatic log rotation** (10MB max, 5 files retained)
- **Multiple log levels** (Error, Warn, Info, Debug, Trace)
- **Log file location**: `~/.local/share/MindLink/logs/mindlink.log` (Linux)

**Log Entry Example:**
```
2024-01-15 14:32:15.123 UTC [ERROR] [PROCESS][BifrostManager] Process exited immediately with status: 1 - {"error_type": "ProcessMonitoring", "technical_details": "Process exited with non-zero status", "recoverable": true}
```

### ✅ 4. Process Monitoring
- **Child process supervision** with automatic restart capabilities
- **stdout/stderr capture** with real-time logging
- **Process health monitoring** with configurable intervals
- **Graceful shutdown** with SIGTERM followed by SIGKILL if needed

**Process Monitoring Features:**
```rust
let config = MonitorConfig {
    capture_stdout: true,
    capture_stderr: true,
    max_restart_attempts: 3,
    restart_delay: Duration::from_secs(5),
    output_buffer_size: 1024 * 1024, // 1MB
    health_check_interval: Duration::from_secs(30),
    process_timeout: Some(Duration::from_secs(300)), // 5 minutes
};
```

## Implementation Details

### Error Type Hierarchy

```rust
pub enum MindLinkError {
    Authentication { message: String, source: Option<anyhow::Error> },
    Network { message: String, url: Option<String>, source: Option<anyhow::Error> },
    BinaryExecution { message: String, binary_name: String, binary_path: Option<String>, source: Option<anyhow::Error> },
    Configuration { message: String, config_key: Option<String>, source: Option<anyhow::Error> },
    FileSystem { message: String, path: Option<String>, operation: String, source: Option<anyhow::Error> },
    ProcessMonitoring { message: String, process_name: String, pid: Option<u32>, source: Option<anyhow::Error> },
    HealthCheck { message: String, service_name: String, url: Option<String>, source: Option<anyhow::Error> },
    Tunnel { message: String, tunnel_type: Option<String>, local_port: Option<u16>, source: Option<anyhow::Error> },
    SystemResource { message: String, resource_type: String, source: Option<anyhow::Error> },
    Internal { message: String, component: Option<String>, source: Option<anyhow::Error> },
}
```

### Error Context Tracking

```rust
pub struct ErrorContext {
    pub component: String,        // Which component failed (e.g., "BifrostManager")
    pub operation: String,        // What operation was attempted (e.g., "start_service")
    pub user_action: Option<String>, // Was this user-initiated? (e.g., "login_and_serve")
    pub correlation_id: Option<String>, // For tracing related errors
    pub additional_info: HashMap<String, String>, // Extra context
}
```

### Process Output Monitoring

The system captures and logs all output from child processes:

```rust
// BifrostManager spawns bifrost-http with captured stdio
let mut cmd = Command::new(&binary_path);
cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

let child = cmd.spawn()?;

// Process monitor captures and logs the output
monitor.start_monitoring("bifrost".to_string(), child).await?;
```

Output is logged as:
```
2024-01-15 14:32:16.456 UTC [DEBUG] [PROCESS][ProcessMonitor] bifrost stdout: Starting HTTP server on 127.0.0.1:3003
2024-01-15 14:32:16.457 UTC [DEBUG] [PROCESS][ProcessMonitor] bifrost stderr: Loading configuration from default settings
```

## Production Guarantees

### ✅ Application Never Crashes
- All potentially panicking operations are wrapped in `Result` types
- Initialization failures are gracefully handled and reported
- Manager failures are isolated and don't bring down the entire application

### ✅ Clear User Messages
- Technical errors are translated to user-friendly language
- Actionable suggestions are provided for common scenarios
- Error dialogs include "what to do next" guidance

### ✅ Comprehensive Logging
- All errors are logged with full technical details
- Process output is captured for debugging
- Log files are automatically rotated to prevent disk space issues
- Sensitive information (auth tokens) is never logged

### ✅ Process Reliability
- Child processes are monitored and can be automatically restarted
- Process failures are detected within seconds
- Graceful shutdown prevents orphaned processes

## Usage Examples

### In Manager Code
```rust
impl BifrostManager {
    pub async fn start(&mut self) -> MindLinkResult<()> {
        // Proper error propagation with context
        let binary_path = self.binary_path.as_ref()
            .ok_or_else(|| MindLinkError::BinaryExecution {
                message: "Binary not found - please run the build system".to_string(),
                binary_name: "bifrost-http".to_string(),
                binary_path: None,
                source: None,
            })?;
        
        // Process monitoring integration
        if let Some(monitor) = get_process_monitor() {
            monitor.start_monitoring("bifrost".to_string(), child).await?;
        }
        
        Ok(())
    }
}
```

### In Command Handlers
```rust
#[tauri::command]
pub async fn start_bifrost(state: State<'_, AppState>) -> Result<ServiceResponse, String> {
    // User action logging
    if let Some(logger) = get_logger() {
        logger.log_user_action("start_bifrost", None);
    }
    
    let mut bifrost_manager = state.bifrost_manager.write().await;
    
    // Handle errors with proper user feedback
    match bifrost_manager.start().await {
        Ok(_) => {
            CommandErrorHandler::send_success_notification(
                &app_handle, 
                "Bifrost Started", 
                "Bifrost LLM Router is now running"
            );
            Ok(success_response("Bifrost started successfully", None, None))
        }
        Err(e) => {
            Ok(CommandErrorHandler::handle_command_error(
                &app_handle,
                e,
                "Bifrost",
                "start_service",
                Some("start_bifrost"),
            ).await)
        }
    }
}
```

### Health Monitoring
```rust
async fn perform_health_check(app_handle: &AppHandle) -> MindLinkResult<()> {
    let bifrost_healthy = {
        let bifrost_manager = state.bifrost_manager.read().await;
        match bifrost_manager.check_health().await {
            Ok(healthy) => {
                // Log health check results
                if let Some(logger) = get_logger() {
                    logger.log_health_check("Bifrost", healthy, 
                        bifrost_manager.get_local_url().await.as_deref(), None);
                }
                healthy
            }
            Err(e) => {
                // Log health check failure with full error context
                if let Some(logger) = get_logger() {
                    logger.log_error("HealthMonitor", &e, None);
                }
                false
            }
        }
    };
    
    // Automatic service restart on health failure
    if !bifrost_healthy {
        let mut bifrost_manager = state.bifrost_manager.write().await;
        if let Err(e) = bifrost_manager.restart().await {
            // Log restart failure
            if let Some(logger) = get_logger() {
                logger.log_error("HealthMonitor", &e, None);
            }
        }
    }
    
    Ok(())
}
```

## Testing Error Handling

The system includes test utilities for verifying error handling:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_binary_not_found_error() {
        let mut manager = BifrostManager::new().await;
        
        // Clear binary path to simulate missing binary
        manager.binary_path = None;
        
        let result = manager.start().await;
        
        assert!(result.is_err());
        if let Err(MindLinkError::BinaryExecution { message, binary_name, .. }) = result {
            assert!(message.contains("Binary not found"));
            assert_eq!(binary_name, "bifrost-http");
        }
    }
}
```

## Configuration

Error handling behavior can be configured:

```rust
let error_config = ErrorReportingConfig {
    show_user_dialogs: true,        // Show error dialogs to users
    send_notifications: true,       // Send non-blocking notifications
    auto_retry_recoverable: false,  // Manual retry for better user control
    max_retry_attempts: 3,          // Maximum automatic retry attempts
    retry_delay_seconds: 5,         // Delay between retry attempts
};
```

## File Locations

- **Log Files**: `~/.local/share/MindLink/logs/` (Linux), `%APPDATA%\MindLink\logs\` (Windows)
- **Configuration**: Error handling is configured at application startup
- **Error History**: In-memory storage with optional persistence

This comprehensive error handling system ensures that the MindLink application is robust, user-friendly, and maintainable in production environments.