// Helper functions for command error handling and user interaction

#![allow(dead_code)]
use crate::commands::ServiceResponse;
use crate::dialog::DialogManager;
use crate::error::{MindLinkError, MindLinkResult};
use crate::error_reporter::{get_error_reporter, ErrorContext};
use std::collections::HashMap;
use tauri::AppHandle;

/// Helper for handling command errors with proper user feedback
pub struct CommandErrorHandler;

impl CommandErrorHandler {
    /// Handle an error in a command with full reporting and user notification
    pub async fn handle_command_error(
        app_handle: &AppHandle,
        error: MindLinkError,
        component: &str,
        operation: &str,
        user_action: Option<&str>,
    ) -> ServiceResponse {
        // Create error context
        let context = ErrorContext {
            component: component.to_string(),
            operation: operation.to_string(),
            user_action: user_action.map(|s| s.to_string()),
            correlation_id: Some(uuid::Uuid::new_v4().to_string()),
            additional_info: HashMap::new(),
        };

        // Report the error
        if let Some(reporter) = get_error_reporter() {
            let _report = reporter
                .report_error(app_handle, error.clone(), context)
                .await;
        } else {
            // Fallback: show dialog directly
            let _ = DialogManager::show_error(app_handle, &error, Some(component)).await;
        }

        // Return service response with error details
        ServiceResponse {
            success: false,
            message: Some(error.user_message()),
            server_url: None,
            tunnel_url: None,
            auth_url: None,
        }
    }

    /// Handle authentication errors specifically
    pub async fn handle_auth_error(
        app_handle: &AppHandle,
        error: anyhow::Error,
        operation: &str,
    ) -> ServiceResponse {
        let auth_error = MindLinkError::Authentication {
            message: "Authentication failed".to_string(),
            source: Some(error),
        };

        Self::handle_command_error(
            app_handle,
            auth_error,
            "Authentication",
            operation,
            Some("login"),
        )
        .await
    }

    /// Handle network errors specifically
    pub async fn handle_network_error(
        app_handle: &AppHandle,
        error: anyhow::Error,
        url: Option<&str>,
        operation: &str,
    ) -> ServiceResponse {
        let network_error = MindLinkError::Network {
            message: "Network connection failed".to_string(),
            url: url.map(|u| u.to_string()),
            source: Some(error),
        };

        Self::handle_command_error(
            app_handle,
            network_error,
            "Network",
            operation,
            Some("network_operation"),
        )
        .await
    }

    /// Handle binary execution errors specifically
    pub async fn handle_binary_error(
        app_handle: &AppHandle,
        error: anyhow::Error,
        binary_name: &str,
        binary_path: Option<&str>,
        operation: &str,
    ) -> ServiceResponse {
        let binary_error = MindLinkError::BinaryExecution {
            message: format!("Failed to execute {}", binary_name),
            binary_name: binary_name.to_string(),
            binary_path: binary_path.map(|p| p.to_string()),
            source: Some(error),
        };

        Self::handle_command_error(
            app_handle,
            binary_error,
            binary_name,
            operation,
            Some("start_service"),
        )
        .await
    }

    /// Send success notification
    pub fn send_success_notification(app_handle: &AppHandle, title: &str, message: &str) {
        DialogManager::send_success_notification(app_handle, title, message);
    }

    /// Send warning notification
    pub fn send_warning_notification(app_handle: &AppHandle, title: &str, message: &str) {
        DialogManager::send_warning_notification(app_handle, title, message);
    }
}

/// Wrapper for async operations that may fail and need error handling
pub async fn with_error_handling<F, T>(
    app_handle: &AppHandle,
    component: &str,
    operation: &str,
    user_action: Option<&str>,
    action: F,
) -> Result<T, ServiceResponse>
where
    F: std::future::Future<Output = MindLinkResult<T>>,
{
    match action.await {
        Ok(result) => Ok(result),
        Err(error) => {
            let service_response = CommandErrorHandler::handle_command_error(
                app_handle,
                error,
                component,
                operation,
                user_action,
            )
            .await;
            Err(service_response)
        },
    }
}

/// Helper macro for error handling in commands
#[macro_export]
macro_rules! handle_command_result {
    ($app:expr, $result:expr, $component:expr, $operation:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                return Ok(
                    crate::command_helpers::CommandErrorHandler::handle_command_error(
                        &$app,
                        error.into(),
                        $component,
                        $operation,
                        None,
                    )
                    .await,
                );
            },
        }
    };
    ($app:expr, $result:expr, $component:expr, $operation:expr, $user_action:expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => {
                return Ok(
                    crate::command_helpers::CommandErrorHandler::handle_command_error(
                        &$app,
                        error.into(),
                        $component,
                        $operation,
                        Some($user_action),
                    )
                    .await,
                );
            },
        }
    };
}

/// Helper for creating success responses
pub fn success_response(
    message: &str,
    server_url: Option<String>,
    tunnel_url: Option<String>,
) -> ServiceResponse {
    ServiceResponse {
        success: true,
        message: Some(message.to_string()),
        server_url,
        tunnel_url,
        auth_url: None,
    }
}

/// Helper for creating error responses
pub fn error_response(message: &str) -> ServiceResponse {
    ServiceResponse {
        success: false,
        message: Some(message.to_string()),
        server_url: None,
        tunnel_url: None,
        auth_url: None,
    }
}

/// Validate input and return error response if invalid
pub fn validate_input<T>(input: Option<T>, field_name: &str) -> Result<T, ServiceResponse> {
    match input {
        Some(value) => Ok(value),
        None => Err(error_response(&format!(
            "Missing required field: {}",
            field_name
        ))),
    }
}

/// Check if a service is healthy and return appropriate response
pub async fn check_service_health(
    service_name: &str,
    health_check: impl std::future::Future<Output = MindLinkResult<bool>>,
) -> Result<bool, ServiceResponse> {
    match health_check.await {
        Ok(healthy) => {
            if !healthy {
                Err(error_response(&format!(
                    "{} service is not healthy",
                    service_name
                )))
            } else {
                Ok(true)
            }
        },
        Err(e) => Err(error_response(&format!(
            "{} health check failed: {}",
            service_name,
            e.user_message()
        ))),
    }
}
