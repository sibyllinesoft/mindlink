// User-friendly dialog system for error messages and notifications
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;
use serde::{Serialize, Deserialize};

use crate::error::MindLinkError;
use crate::logging::{get_logger, LogEntry, LogLevel, LogCategory};

/// Types of dialogs that can be shown to users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DialogType {
    Error,
    Warning,
    Info,
    Question,
}

/// Dialog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogConfig {
    pub title: String,
    pub message: String,
    pub dialog_type: DialogType,
    pub show_details: bool,
    pub technical_details: Option<String>,
    pub suggested_action: Option<String>,
    pub buttons: Vec<DialogButton>,
}

/// Dialog button configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogButton {
    pub id: String,
    pub label: String,
    pub is_default: bool,
    pub is_cancel: bool,
}

impl Default for DialogButton {
    fn default() -> Self {
        Self {
            id: "ok".to_string(),
            label: "OK".to_string(),
            is_default: true,
            is_cancel: false,
        }
    }
}

/// Dialog result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogResult {
    pub button_id: String,
    pub cancelled: bool,
}

/// Dialog manager for showing user-friendly error messages
pub struct DialogManager;

impl DialogManager {
    /// Show an error dialog based on a MindLinkError
    pub async fn show_error(app_handle: &AppHandle, error: &MindLinkError, context: Option<&str>) -> DialogResult {
        // Log the error display action
        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Info,
                LogCategory::UserAction,
                format!("Showing error dialog: {}", error.user_message()),
            ).with_component("Dialog");
            logger.log(entry);
        }
        
        let title = match context {
            Some(ctx) => format!("{} Error", ctx),
            None => "Application Error".to_string(),
        };
        
        let config = DialogConfig {
            title,
            message: error.user_message(),
            dialog_type: DialogType::Error,
            show_details: true,
            technical_details: Some(error.technical_details()),
            suggested_action: error.suggested_action(),
            buttons: if error.is_recoverable() {
                vec![
                    DialogButton {
                        id: "retry".to_string(),
                        label: "Retry".to_string(),
                        is_default: true,
                        is_cancel: false,
                    },
                    DialogButton {
                        id: "cancel".to_string(),
                        label: "Cancel".to_string(),
                        is_default: false,
                        is_cancel: true,
                    },
                ]
            } else {
                vec![DialogButton::default()]
            },
        };
        
        Self::show_dialog(app_handle, config).await
    }
    
    /// Show a warning dialog
    pub async fn show_warning(app_handle: &AppHandle, title: &str, message: &str, details: Option<&str>) -> DialogResult {
        let config = DialogConfig {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: DialogType::Warning,
            show_details: details.is_some(),
            technical_details: details.map(|d| d.to_string()),
            suggested_action: None,
            buttons: vec![DialogButton::default()],
        };
        
        Self::show_dialog(app_handle, config).await
    }
    
    /// Show an info dialog
    pub async fn show_info(app_handle: &AppHandle, title: &str, message: &str) -> DialogResult {
        let config = DialogConfig {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: DialogType::Info,
            show_details: false,
            technical_details: None,
            suggested_action: None,
            buttons: vec![DialogButton::default()],
        };
        
        Self::show_dialog(app_handle, config).await
    }
    
    /// Show a question dialog with custom buttons
    pub async fn show_question(
        app_handle: &AppHandle,
        title: &str,
        message: &str,
        buttons: Vec<DialogButton>,
    ) -> DialogResult {
        let config = DialogConfig {
            title: title.to_string(),
            message: message.to_string(),
            dialog_type: DialogType::Question,
            show_details: false,
            technical_details: None,
            suggested_action: None,
            buttons,
        };
        
        Self::show_dialog(app_handle, config).await
    }
    
    /// Show a generic dialog
    pub async fn show_dialog(app_handle: &AppHandle, config: DialogConfig) -> DialogResult {
        // Log the dialog display
        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Debug,
                LogCategory::UserAction,
                format!("Showing dialog: {} - {}", config.title, config.message),
            ).with_component("Dialog");
            logger.log(entry);
        }
        
        // For now, use Tauri's built-in message dialog
        // In a full implementation, this would show a custom dialog with all the features
        let full_message = Self::format_dialog_message(&config);
        
        // Show message dialog using the callback-based API
        // For now, we'll just simulate the dialog since the actual dialog is async with callbacks
        
        // Show dialog in a fire-and-forget manner (non-blocking)
        app_handle.dialog().message(full_message).show(|_response| {
            // Dialog was shown, response received
        });
        
        // Return immediately since we can't await the callback-based API
        DialogResult {
            button_id: "ok".to_string(),
            cancelled: false,
        }
    }
    
    /// Format a dialog message with all details
    fn format_dialog_message(config: &DialogConfig) -> String {
        let mut message = config.message.clone();
        
        if let Some(action) = &config.suggested_action {
            message.push_str(&format!("\n\nSuggested Action: {}", action));
        }
        
        if config.show_details {
            if let Some(details) = &config.technical_details {
                message.push_str(&format!("\n\nTechnical Details:\n{}", details));
            }
        }
        
        message
    }
    
    /// Show a network error dialog with specific suggestions
    pub async fn show_network_error(app_handle: &AppHandle, url: Option<&str>, error: &str) -> DialogResult {
        let title = "Network Connection Error";
        let message = match url {
            Some(u) => format!("Failed to connect to {}:\n{}", u, error),
            None => format!("Network connection failed:\n{}", error),
        };
        
        let details = "This error usually indicates:\n\
            • Internet connection is down\n\
            • Firewall is blocking the connection\n\
            • The remote service is temporarily unavailable\n\
            • Proxy settings need to be configured";
        
        let config = DialogConfig {
            title: title.to_string(),
            message,
            dialog_type: DialogType::Error,
            show_details: true,
            technical_details: Some(details.to_string()),
            suggested_action: Some("Check your internet connection and try again. If the problem persists, check your firewall settings.".to_string()),
            buttons: vec![
                DialogButton {
                    id: "retry".to_string(),
                    label: "Retry".to_string(),
                    is_default: true,
                    is_cancel: false,
                },
                DialogButton {
                    id: "cancel".to_string(),
                    label: "Cancel".to_string(),
                    is_default: false,
                    is_cancel: true,
                },
            ],
        };
        
        Self::show_dialog(app_handle, config).await
    }
    
    /// Show a binary execution error dialog
    pub async fn show_binary_error(
        app_handle: &AppHandle,
        binary_name: &str,
        binary_path: Option<&str>,
        error: &str,
    ) -> DialogResult {
        let title = format!("{} Error", binary_name);
        let message = format!("Failed to run {}:\n{}", binary_name, error);
        
        let details = match binary_path {
            Some(path) => format!(
                "Binary location: {}\n\nThis error usually indicates:\n\
                • The binary file is missing or corrupted\n\
                • Insufficient permissions to execute the file\n\
                • Missing system dependencies\n\
                • Incompatible binary version",
                path
            ),
            None => "Binary location: Not found\n\nThis error usually indicates:\n\
                • The binary has not been installed\n\
                • The binary is not in the expected location\n\
                • The installation process was interrupted".to_string(),
        };
        
        let suggested_action = if binary_path.is_some() {
            "Try reinstalling the binary or check file permissions."
        } else {
            "Please install the required binary through the application settings."
        };
        
        let config = DialogConfig {
            title,
            message,
            dialog_type: DialogType::Error,
            show_details: true,
            technical_details: Some(details),
            suggested_action: Some(suggested_action.to_string()),
            buttons: vec![
                DialogButton {
                    id: "install".to_string(),
                    label: "Install/Reinstall".to_string(),
                    is_default: true,
                    is_cancel: false,
                },
                DialogButton {
                    id: "cancel".to_string(),
                    label: "Cancel".to_string(),
                    is_default: false,
                    is_cancel: true,
                },
            ],
        };
        
        Self::show_dialog(app_handle, config).await
    }
    
    /// Send a notification to the frontend (for non-blocking notifications)
    pub fn send_notification(app_handle: &AppHandle, title: &str, message: &str, notification_type: &str) {
        let notification_data = serde_json::json!({
            "title": title,
            "message": message,
            "type": notification_type,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // Emit to frontend for display in notification system
        if let Err(e) = app_handle.emit("notification", notification_data) {
            eprintln!("Failed to send notification to frontend: {}", e);
        }
        
        // Log the notification
        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Info,
                LogCategory::UserAction,
                format!("Sent notification: {} - {}", title, message),
            ).with_component("Dialog");
            logger.log(entry);
        }
    }
    
    /// Send an error notification (non-blocking)
    pub fn send_error_notification(app_handle: &AppHandle, error: &MindLinkError, context: Option<&str>) {
        let title = match context {
            Some(ctx) => format!("{} Error", ctx),
            None => "Application Error".to_string(),
        };
        
        Self::send_notification(app_handle, &title, &error.user_message(), "error");
    }
    
    /// Send a success notification
    pub fn send_success_notification(app_handle: &AppHandle, title: &str, message: &str) {
        Self::send_notification(app_handle, title, message, "success");
    }
    
    /// Send a warning notification
    pub fn send_warning_notification(app_handle: &AppHandle, title: &str, message: &str) {
        Self::send_notification(app_handle, title, message, "warning");
    }
    
    /// Send an info notification
    pub fn send_info_notification(app_handle: &AppHandle, title: &str, message: &str) {
        Self::send_notification(app_handle, title, message, "info");
    }
}