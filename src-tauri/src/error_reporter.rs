// Centralized error reporting system

#![allow(dead_code)]
#![allow(static_mut_refs)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;

use crate::dialog::DialogManager;
use crate::error::MindLinkError;
use crate::logging::{get_logger, LogCategory, LogEntry, LogLevel};

/// Error context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub component: String,
    pub operation: String,
    pub user_action: Option<String>,
    pub correlation_id: Option<String>,
    pub additional_info: HashMap<String, String>,
}

/// Error report entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub error: MindLinkError,
    pub context: ErrorContext,
    pub user_notified: bool,
    pub resolved: bool,
    pub resolution_notes: Option<String>,
}

/// Error reporting configuration
#[derive(Debug, Clone)]
pub struct ErrorReportingConfig {
    pub show_user_dialogs: bool,
    pub send_notifications: bool,
    pub auto_retry_recoverable: bool,
    pub max_retry_attempts: u32,
    pub retry_delay_seconds: u64,
}

impl Default for ErrorReportingConfig {
    fn default() -> Self {
        Self {
            show_user_dialogs: true,
            send_notifications: true,
            auto_retry_recoverable: false, // Conservative default
            max_retry_attempts: 3,
            retry_delay_seconds: 5,
        }
    }
}

/// Centralized error reporter
pub struct ErrorReporter {
    config: ErrorReportingConfig,
    error_history: std::sync::Arc<tokio::sync::RwLock<Vec<ErrorReport>>>,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new(config: ErrorReportingConfig) -> Self {
        Self {
            config,
            error_history: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Report an error with full context
    pub async fn report_error(
        &self,
        app_handle: &AppHandle,
        error: MindLinkError,
        context: ErrorContext,
    ) -> ErrorReport {
        let error_id = uuid::Uuid::new_v4().to_string();

        // Create error report
        let report = ErrorReport {
            id: error_id.clone(),
            timestamp: Utc::now(),
            error: error.clone(),
            context: context.clone(),
            user_notified: false,
            resolved: false,
            resolution_notes: None,
        };

        // Log the error
        if let Some(logger) = get_logger() {
            logger.log_error(
                &context.component,
                &error,
                context.correlation_id.as_deref(),
            );
        }

        // Store in history
        {
            let mut history = self.error_history.write().await;
            history.push(report.clone());

            // Limit history size (keep last 1000 errors)
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        // Handle user notification
        let mut updated_report = report;
        if self.config.show_user_dialogs || self.config.send_notifications {
            updated_report = self.notify_user(app_handle, updated_report, &context).await;
        }

        updated_report
    }

    /// Notify user about the error
    async fn notify_user(
        &self,
        app_handle: &AppHandle,
        mut report: ErrorReport,
        context: &ErrorContext,
    ) -> ErrorReport {
        // Determine notification strategy based on error type and context
        let should_show_dialog =
            self.config.show_user_dialogs && self.should_show_dialog(&report.error, context);

        if should_show_dialog {
            // Show blocking dialog for critical errors
            let dialog_result =
                DialogManager::show_error(app_handle, &report.error, Some(&context.component))
                    .await;

            report.user_notified = true;

            // Handle dialog response
            if dialog_result.button_id == "retry" && report.error.is_recoverable() {
                // User requested retry - this would be handled by the calling code
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Info,
                        LogCategory::UserAction,
                        format!("User requested retry for error: {}", report.id),
                    )
                    .with_component("ErrorReporter");
                    logger.log(entry);
                }
            }
        } else if self.config.send_notifications {
            // Send non-blocking notification
            DialogManager::send_error_notification(
                app_handle,
                &report.error,
                Some(&context.component),
            );

            report.user_notified = true;
        }

        report
    }

    /// Determine if a dialog should be shown for this error
    fn should_show_dialog(&self, error: &MindLinkError, context: &ErrorContext) -> bool {
        match error {
            // Always show dialogs for authentication errors (user needs to act)
            MindLinkError::Authentication { .. } => true,

            // Show dialogs for binary execution errors (user may need to install)
            MindLinkError::BinaryExecution { .. } => true,

            // Show dialogs for configuration errors (user needs to fix settings)
            MindLinkError::Configuration { .. } => true,

            // Show dialogs for network errors if user-initiated
            MindLinkError::Network { .. } => context.user_action.is_some(),

            // Don't show dialogs for routine monitoring/health checks
            MindLinkError::HealthCheck { .. } => false,
            MindLinkError::ProcessMonitoring { .. } => {
                // Only show if it's not an automatic health check
                !context.operation.contains("health_check")
            },

            // Show dialogs for file system errors that affect user operations
            MindLinkError::FileSystem { .. } => context.user_action.is_some(),

            // Show dialogs for tunnel errors (user may need to troubleshoot)
            MindLinkError::Tunnel { .. } => true,

            // Show dialogs for system resource errors
            MindLinkError::SystemResource { .. } => true,

            // Show dialogs for internal errors that affect user operations
            MindLinkError::Internal { .. } => context.user_action.is_some(),
        }
    }

    /// Report and potentially retry an operation
    pub async fn report_with_retry<F, T, E>(
        &self,
        app_handle: &AppHandle,
        context: ErrorContext,
        operation: F,
    ) -> Result<T, MindLinkError>
    where
        F: Fn() -> Result<T, E> + Clone,
        E: Into<MindLinkError>,
    {
        let mut last_error: Option<String> = None;
        let max_attempts = if self.config.auto_retry_recoverable {
            self.config.max_retry_attempts
        } else {
            1
        };

        for attempt in 1..=max_attempts {
            match operation() {
                Ok(result) => {
                    // Success - clear any previous error reports for this operation
                    if let Some(error) = last_error {
                        self.mark_resolved(&error, &format!("Resolved on attempt {}", attempt))
                            .await;
                    }
                    return Ok(result);
                },
                Err(e) => {
                    let mindlink_error = e.into();

                    // Report the error
                    let mut error_context = context.clone();
                    error_context
                        .additional_info
                        .insert("attempt".to_string(), attempt.to_string());
                    error_context
                        .additional_info
                        .insert("max_attempts".to_string(), max_attempts.to_string());

                    let report = self
                        .report_error(app_handle, mindlink_error.clone(), error_context)
                        .await;
                    last_error = Some(report.id);

                    // Check if we should retry
                    if attempt < max_attempts
                        && self.config.auto_retry_recoverable
                        && mindlink_error.is_recoverable()
                    {
                        if let Some(logger) = get_logger() {
                            let entry = LogEntry::new(
                                LogLevel::Info,
                                LogCategory::System,
                                format!(
                                    "Retrying operation (attempt {} of {})",
                                    attempt + 1,
                                    max_attempts
                                ),
                            )
                            .with_component("ErrorReporter");
                            logger.log(entry);
                        }

                        // Wait before retry
                        tokio::time::sleep(tokio::time::Duration::from_secs(
                            self.config.retry_delay_seconds,
                        ))
                        .await;
                        continue;
                    }

                    return Err(mindlink_error);
                },
            }
        }

        unreachable!()
    }

    /// Mark an error as resolved
    pub async fn mark_resolved(&self, error_id: &str, resolution_notes: &str) {
        let mut history = self.error_history.write().await;
        if let Some(report) = history.iter_mut().find(|r| r.id == error_id) {
            report.resolved = true;
            report.resolution_notes = Some(resolution_notes.to_string());
        }
    }

    /// Get error history
    pub async fn get_error_history(&self) -> Vec<ErrorReport> {
        let history = self.error_history.read().await;
        history.clone()
    }

    /// Get unresolved errors
    pub async fn get_unresolved_errors(&self) -> Vec<ErrorReport> {
        let history = self.error_history.read().await;
        history
            .iter()
            .filter(|report| !report.resolved)
            .cloned()
            .collect()
    }

    /// Clear resolved errors from history
    pub async fn clear_resolved_errors(&self) {
        let mut history = self.error_history.write().await;
        history.retain(|report| !report.resolved);
    }

    /// Get error statistics
    pub async fn get_error_statistics(&self) -> ErrorStatistics {
        let history = self.error_history.read().await;

        let total_errors = history.len();
        let resolved_errors = history.iter().filter(|r| r.resolved).count();
        let unresolved_errors = total_errors - resolved_errors;

        // Count by error type
        let mut by_type = HashMap::new();
        for report in history.iter() {
            let error_type = std::mem::discriminant(&report.error);
            *by_type.entry(format!("{:?}", error_type)).or_insert(0) += 1;
        }

        // Count by component
        let mut by_component = HashMap::new();
        for report in history.iter() {
            *by_component
                .entry(report.context.component.clone())
                .or_insert(0) += 1;
        }

        ErrorStatistics {
            total_errors,
            resolved_errors,
            unresolved_errors,
            by_type,
            by_component,
        }
    }
}

/// Error statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_errors: usize,
    pub resolved_errors: usize,
    pub unresolved_errors: usize,
    pub by_type: HashMap<String, usize>,
    pub by_component: HashMap<String, usize>,
}

/// Convenience macros for error reporting
#[macro_export]
macro_rules! report_error {
    ($reporter:expr, $app:expr, $error:expr, $component:expr, $operation:expr) => {{
        let context = crate::error_reporter::ErrorContext {
            component: $component.to_string(),
            operation: $operation.to_string(),
            user_action: None,
            correlation_id: None,
            additional_info: std::collections::HashMap::new(),
        };
        $reporter.report_error($app, $error, context).await
    }};
    ($reporter:expr, $app:expr, $error:expr, $component:expr, $operation:expr, $user_action:expr) => {{
        let context = crate::error_reporter::ErrorContext {
            component: $component.to_string(),
            operation: $operation.to_string(),
            user_action: Some($user_action.to_string()),
            correlation_id: None,
            additional_info: std::collections::HashMap::new(),
        };
        $reporter.report_error($app, $error, context).await
    }};
}

/// Global error reporter instance
static mut ERROR_REPORTER: Option<std::sync::Arc<ErrorReporter>> = None;
static ERROR_REPORTER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global error reporter
pub fn init_error_reporter(config: ErrorReportingConfig) {
    ERROR_REPORTER_INIT.call_once(|| {
        #[allow(unsafe_code)]
        unsafe {
            ERROR_REPORTER = Some(std::sync::Arc::new(ErrorReporter::new(config)));
        }
    });
}

/// Get the global error reporter
pub fn get_error_reporter() -> Option<std::sync::Arc<ErrorReporter>> {
    #[allow(unsafe_code)]
    unsafe {
        ERROR_REPORTER.as_ref().cloned()
    }
}
