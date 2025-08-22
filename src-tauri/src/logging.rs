// Comprehensive logging system for MindLink application
use log::{error, warn, info, debug, trace};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::error::MindLinkError;

/// Log levels for the application
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

/// Categories for different types of log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogCategory {
    System,
    Authentication,
    Network,
    Process,
    HealthCheck,
    Configuration,
    UserAction,
    Error,
}

impl std::fmt::Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogCategory::System => write!(f, "SYSTEM"),
            LogCategory::Authentication => write!(f, "AUTH"),
            LogCategory::Network => write!(f, "NET"),
            LogCategory::Process => write!(f, "PROC"),
            LogCategory::HealthCheck => write!(f, "HEALTH"),
            LogCategory::Configuration => write!(f, "CONFIG"),
            LogCategory::UserAction => write!(f, "USER"),
            LogCategory::Error => write!(f, "ERROR"),
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub category: LogCategory,
    pub component: Option<String>,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub correlation_id: Option<String>,
}

impl LogEntry {
    pub fn new(
        level: LogLevel,
        category: LogCategory,
        message: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            category,
            component: None,
            message,
            details: None,
            correlation_id: None,
        }
    }
    
    pub fn with_component(mut self, component: &str) -> Self {
        self.component = Some(component.to_string());
        self
    }
    
    pub fn with_details<T: Serialize>(mut self, details: &T) -> Self {
        if let Ok(json_value) = serde_json::to_value(details) {
            self.details = Some(json_value);
        }
        self
    }
    
    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }
    
    /// Format the log entry for file output
    pub fn format_for_file(&self) -> String {
        let component_str = match &self.component {
            Some(comp) => format!("[{}]", comp),
            None => String::new(),
        };
        
        let correlation_str = match &self.correlation_id {
            Some(id) => format!(" [{}]", &id[..8]), // Short correlation ID
            None => String::new(),
        };
        
        let details_str = match &self.details {
            Some(details) => {
                match serde_json::to_string(details) {
                    Ok(json) => format!(" - {}", json),
                    Err(_) => String::new(),
                }
            }
            None => String::new(),
        };
        
        format!(
            "{} [{}] [{}]{}{} {}{}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC"),
            self.level,
            self.category,
            component_str,
            correlation_str,
            self.message,
            details_str
        )
    }
    
    /// Format the log entry for console output (more colorful/readable)
    pub fn format_for_console(&self) -> String {
        let component_str = match &self.component {
            Some(comp) => format!(" {}", comp),
            None => String::new(),
        };
        
        let level_color = match self.level {
            LogLevel::Error => "\x1b[31m", // Red
            LogLevel::Warn => "\x1b[33m",  // Yellow
            LogLevel::Info => "\x1b[32m",  // Green
            LogLevel::Debug => "\x1b[36m", // Cyan
            LogLevel::Trace => "\x1b[90m", // Gray
        };
        
        format!(
            "{}[{}]\x1b[0m [{}]{} {}",
            level_color,
            self.level,
            self.category,
            component_str,
            self.message
        )
    }
}

/// Main logging manager
pub struct LogManager {
    log_file_path: PathBuf,
    file_writer: Arc<Mutex<BufWriter<File>>>,
    max_file_size: u64,
    max_files: usize,
    console_enabled: bool,
}

impl LogManager {
    /// Create a new log manager
    pub fn new() -> Result<Self, MindLinkError> {
        // Determine log directory
        let log_dir = Self::get_log_directory()?;
        std::fs::create_dir_all(&log_dir).map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to create log directory".to_string(),
                path: Some(log_dir.to_string_lossy().to_string()),
                operation: "create directory".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        let log_file_path = log_dir.join("mindlink.log");
        
        // Open log file for appending
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .map_err(|e| {
                MindLinkError::FileSystem {
                    message: "Failed to open log file".to_string(),
                    path: Some(log_file_path.to_string_lossy().to_string()),
                    operation: "open file".to_string(),
                    source: Some(e.into()),
                }
            })?;
        
        let file_writer = Arc::new(Mutex::new(BufWriter::new(log_file)));
        
        Ok(Self {
            log_file_path,
            file_writer,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
            console_enabled: true,
        })
    }
    
    /// Get the appropriate log directory for the platform
    fn get_log_directory() -> Result<PathBuf, MindLinkError> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| {
                MindLinkError::SystemResource {
                    message: "Cannot determine application data directory".to_string(),
                    resource_type: "data directory".to_string(),
                    source: None,
                }
            })?;
        
        Ok(app_data_dir.join("MindLink").join("logs"))
    }
    
    /// Log a structured entry
    pub fn log(&self, entry: LogEntry) {
        // Write to console if enabled
        if self.console_enabled {
            match entry.level {
                LogLevel::Error => error!("{}", entry.format_for_console()),
                LogLevel::Warn => warn!("{}", entry.format_for_console()),
                LogLevel::Info => info!("{}", entry.format_for_console()),
                LogLevel::Debug => debug!("{}", entry.format_for_console()),
                LogLevel::Trace => trace!("{}", entry.format_for_console()),
            }
        }
        
        // Write to file
        if let Err(e) = self.write_to_file(&entry) {
            eprintln!("Failed to write to log file: {}", e);
        }
    }
    
    /// Write entry to log file
    fn write_to_file(&self, entry: &LogEntry) -> Result<(), MindLinkError> {
        let formatted_entry = entry.format_for_file();
        
        if let Ok(mut writer) = self.file_writer.lock() {
            writeln!(writer, "{}", formatted_entry).map_err(|e| {
                MindLinkError::FileSystem {
                    message: "Failed to write log entry".to_string(),
                    path: Some(self.log_file_path.to_string_lossy().to_string()),
                    operation: "write".to_string(),
                    source: Some(e.into()),
                }
            })?;
            
            writer.flush().map_err(|e| {
                MindLinkError::FileSystem {
                    message: "Failed to flush log buffer".to_string(),
                    path: Some(self.log_file_path.to_string_lossy().to_string()),
                    operation: "flush".to_string(),
                    source: Some(e.into()),
                }
            })?;
        }
        
        // Check if we need to rotate logs
        if let Err(e) = self.check_and_rotate_logs() {
            eprintln!("Failed to rotate logs: {}", e);
        }
        
        Ok(())
    }
    
    /// Check file size and rotate logs if necessary
    fn check_and_rotate_logs(&self) -> Result<(), MindLinkError> {
        let metadata = std::fs::metadata(&self.log_file_path).map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to read log file metadata".to_string(),
                path: Some(self.log_file_path.to_string_lossy().to_string()),
                operation: "read metadata".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        if metadata.len() > self.max_file_size {
            self.rotate_logs()?;
        }
        
        Ok(())
    }
    
    /// Rotate log files
    fn rotate_logs(&self) -> Result<(), MindLinkError> {
        let log_dir = self.log_file_path.parent().ok_or_else(|| {
            MindLinkError::FileSystem {
                message: "Cannot determine log directory".to_string(),
                path: Some(self.log_file_path.to_string_lossy().to_string()),
                operation: "get parent directory".to_string(),
                source: None,
            }
        })?;
        
        // Rename existing files
        for i in (1..self.max_files).rev() {
            let from = log_dir.join(format!("mindlink.log.{}", i));
            let to = log_dir.join(format!("mindlink.log.{}", i + 1));
            
            if from.exists() {
                if i == self.max_files - 1 {
                    // Delete the oldest file
                    std::fs::remove_file(&from).map_err(|e| {
                        MindLinkError::FileSystem {
                            message: "Failed to delete old log file".to_string(),
                            path: Some(from.to_string_lossy().to_string()),
                            operation: "delete".to_string(),
                            source: Some(e.into()),
                        }
                    })?;
                } else {
                    std::fs::rename(&from, &to).map_err(|e| {
                        MindLinkError::FileSystem {
                            message: "Failed to rotate log file".to_string(),
                            path: Some(from.to_string_lossy().to_string()),
                            operation: "rename".to_string(),
                            source: Some(e.into()),
                        }
                    })?;
                }
            }
        }
        
        // Move current log to .1
        let rotated_path = log_dir.join("mindlink.log.1");
        std::fs::rename(&self.log_file_path, &rotated_path).map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to rotate current log file".to_string(),
                path: Some(self.log_file_path.to_string_lossy().to_string()),
                operation: "rename".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        // Create new log file
        let new_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_file_path)
            .map_err(|e| {
                MindLinkError::FileSystem {
                    message: "Failed to create new log file".to_string(),
                    path: Some(self.log_file_path.to_string_lossy().to_string()),
                    operation: "create".to_string(),
                    source: Some(e.into()),
                }
            })?;
        
        // Replace the writer
        if let Ok(mut writer) = self.file_writer.lock() {
            *writer = BufWriter::new(new_file);
        }
        
        Ok(())
    }
    
    /// Configure console logging
    pub fn set_console_enabled(&mut self, enabled: bool) {
        self.console_enabled = enabled;
    }
    
    /// Configure maximum file size before rotation
    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }
    
    /// Configure maximum number of rotated files to keep
    pub fn set_max_files(&mut self, count: usize) {
        self.max_files = count;
    }
    
    /// Get the current log file path
    pub fn get_log_file_path(&self) -> &Path {
        &self.log_file_path
    }
    
    /// Log an error with full details
    pub fn log_error(&self, component: &str, error: &MindLinkError, correlation_id: Option<&str>) {
        let mut entry = LogEntry::new(
            LogLevel::Error,
            LogCategory::Error,
            error.user_message(),
        ).with_component(component);
        
        if let Some(id) = correlation_id {
            entry = entry.with_correlation_id(id);
        }
        
        // Add technical details
        let error_type = match error {
            crate::error::MindLinkError::Authentication { .. } => "Authentication",
            crate::error::MindLinkError::Network { .. } => "Network",
            crate::error::MindLinkError::BinaryExecution { .. } => "BinaryExecution",
            crate::error::MindLinkError::Configuration { .. } => "Configuration",
            crate::error::MindLinkError::FileSystem { .. } => "FileSystem",
            crate::error::MindLinkError::ProcessMonitoring { .. } => "ProcessMonitoring",
            crate::error::MindLinkError::HealthCheck { .. } => "HealthCheck",
            crate::error::MindLinkError::Tunnel { .. } => "Tunnel",
            crate::error::MindLinkError::SystemResource { .. } => "SystemResource",
            crate::error::MindLinkError::Internal { .. } => "Internal",
        };
        
        let details = serde_json::json!({
            "error_type": error_type,
            "technical_details": error.technical_details(),
            "recoverable": error.is_recoverable(),
            "suggested_action": error.suggested_action(),
        });
        entry = entry.with_details(&details);
        
        self.log(entry);
    }
    
    /// Log process stdout/stderr output
    pub fn log_process_output(&self, process_name: &str, output_type: &str, content: &str, pid: Option<u32>) {
        let details = serde_json::json!({
            "process_name": process_name,
            "output_type": output_type,
            "content": content,
            "pid": pid,
        });
        
        let entry = LogEntry::new(
            LogLevel::Debug,
            LogCategory::Process,
            format!("{} {}: {}", process_name, output_type, content.trim()),
        ).with_component("ProcessMonitor")
        .with_details(&details);
        
        self.log(entry);
    }
    
    /// Log user actions
    pub fn log_user_action(&self, action: &str, details: Option<&serde_json::Value>) {
        let mut entry = LogEntry::new(
            LogLevel::Info,
            LogCategory::UserAction,
            format!("User action: {}", action),
        ).with_component("UserInterface");
        
        if let Some(details) = details {
            entry = entry.with_details(details);
        }
        
        self.log(entry);
    }
    
    /// Log health check results
    pub fn log_health_check(&self, service: &str, healthy: bool, url: Option<&str>, response_time: Option<u64>) {
        let details = serde_json::json!({
            "service": service,
            "healthy": healthy,
            "url": url,
            "response_time_ms": response_time,
        });
        
        let level = if healthy { LogLevel::Debug } else { LogLevel::Warn };
        let message = if healthy {
            format!("{} health check passed", service)
        } else {
            format!("{} health check failed", service)
        };
        
        let entry = LogEntry::new(level, LogCategory::HealthCheck, message)
            .with_component("HealthMonitor")
            .with_details(&details);
        
        self.log(entry);
    }
}

/// Global log manager instance
static mut LOG_MANAGER: Option<Arc<LogManager>> = None;
static LOG_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global log manager
pub fn init_logging() -> Result<(), MindLinkError> {
    LOG_MANAGER_INIT.call_once(|| {
        match LogManager::new() {
            Ok(manager) => {
                #[allow(unsafe_code)]
                unsafe {
                    LOG_MANAGER = Some(Arc::new(manager));
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize log manager: {}", e);
            }
        }
    });
    
    Ok(())
}

/// Get the global log manager
pub fn get_logger() -> Option<Arc<LogManager>> {
    #[allow(unsafe_code)]
    unsafe {
        LOG_MANAGER.as_ref().cloned()
    }
}

/// Convenience macro for logging errors
#[macro_export]
macro_rules! log_error {
    ($component:expr, $error:expr) => {
        if let Some(logger) = crate::logging::get_logger() {
            logger.log_error($component, &$error, None);
        }
    };
    ($component:expr, $error:expr, $correlation_id:expr) => {
        if let Some(logger) = crate::logging::get_logger() {
            logger.log_error($component, &$error, Some($correlation_id));
        }
    };
}

/// Convenience macro for logging user actions
#[macro_export]
macro_rules! log_user_action {
    ($action:expr) => {
        if let Some(logger) = crate::logging::get_logger() {
            logger.log_user_action($action, None);
        }
    };
    ($action:expr, $details:expr) => {
        if let Some(logger) = crate::logging::get_logger() {
            logger.log_user_action($action, Some(&$details));
        }
    };
}

/// Convenience macro for logging info messages
#[macro_export]
macro_rules! log_info {
    ($component:expr, $message:expr) => {
        if let Some(logger) = crate::logging::get_logger() {
            let entry = crate::logging::LogEntry::new(
                crate::logging::LogLevel::Info,
                crate::logging::LogCategory::System,
                $message.to_string(),
            ).with_component($component);
            logger.log(entry);
        }
    };
}

/// Convenience macro for logging warnings
#[macro_export]
macro_rules! log_warn {
    ($component:expr, $message:expr) => {
        if let Some(logger) = crate::logging::get_logger() {
            let entry = crate::logging::LogEntry::new(
                crate::logging::LogLevel::Warn,
                crate::logging::LogCategory::System,
                $message.to_string(),
            ).with_component($component);
            logger.log(entry);
        }
    };
}

/// Convenience macro for debug logging
#[macro_export]
macro_rules! log_debug {
    ($component:expr, $message:expr) => {
        if let Some(logger) = crate::logging::get_logger() {
            let entry = crate::logging::LogEntry::new(
                crate::logging::LogLevel::Debug,
                crate::logging::LogCategory::System,
                $message.to_string(),
            ).with_component($component);
            logger.log(entry);
        }
    };
}