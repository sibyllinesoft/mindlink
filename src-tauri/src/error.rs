// Comprehensive error handling types for MindLink application
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main application error type that provides user-friendly messages
/// and detailed technical information for logging
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum MindLinkError {
    #[error("Authentication failed: {message}")]
    Authentication { 
        message: String,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("Network connection failed: {message}")]
    Network { 
        message: String,
        url: Option<String>,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("Binary not found or failed to start: {message}")]
    BinaryExecution { 
        message: String,
        binary_name: String,
        binary_path: Option<String>,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("Configuration error: {message}")]
    Configuration { 
        message: String,
        config_key: Option<String>,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("File system operation failed: {message}")]
    FileSystem { 
        message: String,
        path: Option<String>,
        operation: String,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("Process monitoring failed: {message}")]
    ProcessMonitoring { 
        message: String,
        process_name: String,
        pid: Option<u32>,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("Service health check failed: {message}")]
    HealthCheck { 
        message: String,
        service_name: String,
        url: Option<String>,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("Tunnel operation failed: {message}")]
    Tunnel { 
        message: String,
        tunnel_type: Option<String>,
        local_port: Option<u16>,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("System resource unavailable: {message}")]
    SystemResource { 
        message: String,
        resource_type: String,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
    
    #[error("Internal application error: {message}")]
    Internal { 
        message: String,
        component: Option<String>,
        #[serde(skip)]
        source: Option<anyhow::Error>,
    },
}

impl Clone for MindLinkError {
    fn clone(&self) -> Self {
        match self {
            MindLinkError::Authentication { message, source: _ } => {
                MindLinkError::Authentication {
                    message: message.clone(),
                    source: None, // Don't clone the source as anyhow::Error doesn't implement Clone
                }
            },
            MindLinkError::Network { message, url, source: _ } => {
                MindLinkError::Network {
                    message: message.clone(),
                    url: url.clone(),
                    source: None,
                }
            },
            MindLinkError::BinaryExecution { message, binary_name, binary_path, source: _ } => {
                MindLinkError::BinaryExecution {
                    message: message.clone(),
                    binary_name: binary_name.clone(),
                    binary_path: binary_path.clone(),
                    source: None,
                }
            },
            MindLinkError::Configuration { message, config_key, source: _ } => {
                MindLinkError::Configuration {
                    message: message.clone(),
                    config_key: config_key.clone(),
                    source: None,
                }
            },
            MindLinkError::FileSystem { message, path, operation, source: _ } => {
                MindLinkError::FileSystem {
                    message: message.clone(),
                    path: path.clone(),
                    operation: operation.clone(),
                    source: None,
                }
            },
            MindLinkError::ProcessMonitoring { message, process_name, pid, source: _ } => {
                MindLinkError::ProcessMonitoring {
                    message: message.clone(),
                    process_name: process_name.clone(),
                    pid: *pid,
                    source: None,
                }
            },
            MindLinkError::HealthCheck { message, service_name, url, source: _ } => {
                MindLinkError::HealthCheck {
                    message: message.clone(),
                    service_name: service_name.clone(),
                    url: url.clone(),
                    source: None,
                }
            },
            MindLinkError::Tunnel { message, tunnel_type, local_port, source: _ } => {
                MindLinkError::Tunnel {
                    message: message.clone(),
                    tunnel_type: tunnel_type.clone(),
                    local_port: *local_port,
                    source: None,
                }
            },
            MindLinkError::SystemResource { message, resource_type, source: _ } => {
                MindLinkError::SystemResource {
                    message: message.clone(),
                    resource_type: resource_type.clone(),
                    source: None,
                }
            },
            MindLinkError::Internal { message, component, source: _ } => {
                MindLinkError::Internal {
                    message: message.clone(),
                    component: component.clone(),
                    source: None,
                }
            },
        }
    }
}

impl MindLinkError {
    /// Get a user-friendly error message that can be displayed in dialogs
    pub fn user_message(&self) -> String {
        match self {
            MindLinkError::Authentication { message, .. } => {
                format!("Authentication Error: {}", message)
            }
            MindLinkError::Network { message, url, .. } => {
                match url {
                    Some(url) => format!("Connection failed to {}: {}", url, message),
                    None => format!("Network Error: {}", message),
                }
            }
            MindLinkError::BinaryExecution { message, binary_name, .. } => {
                format!("Program Error: {} failed to start - {}", binary_name, message)
            }
            MindLinkError::Configuration { message, config_key, .. } => {
                match config_key {
                    Some(key) => format!("Configuration Error: Issue with '{}' - {}", key, message),
                    None => format!("Configuration Error: {}", message),
                }
            }
            MindLinkError::FileSystem { message, operation, .. } => {
                format!("File Error: Failed to {} - {}", operation, message)
            }
            MindLinkError::ProcessMonitoring { message, process_name, .. } => {
                format!("Service Error: {} monitoring failed - {}", process_name, message)
            }
            MindLinkError::HealthCheck { message, service_name, .. } => {
                format!("Service Health: {} is not responding - {}", service_name, message)
            }
            MindLinkError::Tunnel { message, .. } => {
                format!("Tunnel Error: {}", message)
            }
            MindLinkError::SystemResource { message, resource_type, .. } => {
                format!("System Error: {} unavailable - {}", resource_type, message)
            }
            MindLinkError::Internal { message, component, .. } => {
                match component {
                    Some(comp) => format!("Internal Error in {}: {}", comp, message),
                    None => format!("Internal Error: {}", message),
                }
            }
        }
    }
    
    /// Get technical details for logging (includes source error chain)
    pub fn technical_details(&self) -> String {
        let base_message = self.to_string();
        
        // For now, just return the base message to avoid type complexity
        // The anyhow source chain is complex to traverse due to type issues
        base_message
    }
    
    /// Get the source error if available
    pub fn source(&self) -> Option<&anyhow::Error> {
        match self {
            MindLinkError::Authentication { source, .. } |
            MindLinkError::Network { source, .. } |
            MindLinkError::BinaryExecution { source, .. } |
            MindLinkError::Configuration { source, .. } |
            MindLinkError::FileSystem { source, .. } |
            MindLinkError::ProcessMonitoring { source, .. } |
            MindLinkError::HealthCheck { source, .. } |
            MindLinkError::Tunnel { source, .. } |
            MindLinkError::SystemResource { source, .. } |
            MindLinkError::Internal { source, .. } => source.as_ref(),
        }
    }
    
    /// Check if this error is recoverable (user can retry)
    pub fn is_recoverable(&self) -> bool {
        match self {
            MindLinkError::Authentication { .. } => true,  // User can re-login
            MindLinkError::Network { .. } => true,         // Network may recover
            MindLinkError::BinaryExecution { .. } => false, // Binary issues need fixing
            MindLinkError::Configuration { .. } => false,   // Config needs to be fixed
            MindLinkError::FileSystem { .. } => true,       // File operations can be retried
            MindLinkError::ProcessMonitoring { .. } => true, // Process can be restarted
            MindLinkError::HealthCheck { .. } => true,      // Service may recover
            MindLinkError::Tunnel { .. } => true,           // Tunnels can be recreated
            MindLinkError::SystemResource { .. } => true,   // Resources may become available
            MindLinkError::Internal { .. } => false,        // Internal errors need investigation
        }
    }
    
    /// Get suggested action for the user
    pub fn suggested_action(&self) -> Option<String> {
        match self {
            MindLinkError::Authentication { .. } => {
                Some("Please try logging in again or check your credentials.".to_string())
            }
            MindLinkError::Network { .. } => {
                Some("Check your internet connection and try again.".to_string())
            }
            MindLinkError::BinaryExecution { binary_name, .. } => {
                Some(format!("Please reinstall {} or contact support.", binary_name))
            }
            MindLinkError::Configuration { config_key, .. } => {
                match config_key {
                    Some(key) => Some(format!("Please check your {} configuration setting.", key)),
                    None => Some("Please check your application settings.".to_string()),
                }
            }
            MindLinkError::FileSystem { operation, .. } => {
                Some(format!("Please ensure you have permission to {} files and try again.", operation))
            }
            MindLinkError::ProcessMonitoring { process_name, .. } => {
                Some(format!("Restart {} service or contact support if the problem persists.", process_name))
            }
            MindLinkError::HealthCheck { service_name, .. } => {
                Some(format!("Restart the {} service and try again.", service_name))
            }
            MindLinkError::Tunnel { .. } => {
                Some("Check your network connection and try creating the tunnel again.".to_string())
            }
            MindLinkError::SystemResource { resource_type, .. } => {
                Some(format!("Ensure {} is available and try again.", resource_type))
            }
            MindLinkError::Internal { .. } => {
                Some("Please restart the application or contact support.".to_string())
            }
        }
    }
}

/// Result type alias for convenience
pub type MindLinkResult<T> = Result<T, MindLinkError>;

/// Convert from anyhow::Error to MindLinkError
impl From<anyhow::Error> for MindLinkError {
    fn from(err: anyhow::Error) -> Self {
        // Try to categorize the error based on its message
        let err_msg = err.to_string().to_lowercase();
        
        if err_msg.contains("auth") || err_msg.contains("login") || err_msg.contains("credential") {
            MindLinkError::Authentication {
                message: "Authentication system error".to_string(),
                source: Some(err),
            }
        } else if err_msg.contains("network") || err_msg.contains("connection") || err_msg.contains("timeout") {
            MindLinkError::Network {
                message: "Network communication error".to_string(),
                url: None,
                source: Some(err),
            }
        } else if err_msg.contains("binary") || err_msg.contains("spawn") || err_msg.contains("process") {
            MindLinkError::BinaryExecution {
                message: "Process execution error".to_string(),
                binary_name: "unknown".to_string(),
                binary_path: None,
                source: Some(err),
            }
        } else if err_msg.contains("config") || err_msg.contains("setting") {
            MindLinkError::Configuration {
                message: "Configuration system error".to_string(),
                config_key: None,
                source: Some(err),
            }
        } else if err_msg.contains("file") || err_msg.contains("directory") || err_msg.contains("path") {
            MindLinkError::FileSystem {
                message: "File system operation error".to_string(),
                path: None,
                operation: "unknown".to_string(),
                source: Some(err),
            }
        } else {
            MindLinkError::Internal {
                message: "Unexpected error occurred".to_string(),
                component: None,
                source: Some(err),
            }
        }
    }
}

/// Convert from std::io::Error to MindLinkError
impl From<std::io::Error> for MindLinkError {
    fn from(err: std::io::Error) -> Self {
        let anyhow_err = anyhow::Error::from(err);
        anyhow_err.into()
    }
}

/// Convert from reqwest::Error to MindLinkError
impl From<reqwest::Error> for MindLinkError {
    fn from(err: reqwest::Error) -> Self {
        MindLinkError::Network {
            message: "HTTP request failed".to_string(),
            url: err.url().map(|u| u.to_string()),
            source: Some(anyhow::Error::from(err)),
        }
    }
}

/// Convert from serde_json::Error to MindLinkError
impl From<serde_json::Error> for MindLinkError {
    fn from(err: serde_json::Error) -> Self {
        MindLinkError::Configuration {
            message: "JSON parsing failed".to_string(),
            config_key: None,
            source: Some(anyhow::Error::from(err)),
        }
    }
}

/// Helper macros for creating specific error types
#[macro_export]
macro_rules! auth_error {
    ($msg:expr) => {
        MindLinkError::Authentication {
            message: $msg.to_string(),
            source: None,
        }
    };
    ($msg:expr, $source:expr) => {
        MindLinkError::Authentication {
            message: $msg.to_string(),
            source: Some(anyhow::Error::from($source)),
        }
    };
}

#[macro_export]
macro_rules! network_error {
    ($msg:expr) => {
        MindLinkError::Network {
            message: $msg.to_string(),
            url: None,
            source: None,
        }
    };
    ($msg:expr, $url:expr) => {
        MindLinkError::Network {
            message: $msg.to_string(),
            url: Some($url.to_string()),
            source: None,
        }
    };
    ($msg:expr, $url:expr, $source:expr) => {
        MindLinkError::Network {
            message: $msg.to_string(),
            url: Some($url.to_string()),
            source: Some(anyhow::Error::from($source)),
        }
    };
}

#[macro_export]
macro_rules! binary_error {
    ($msg:expr, $binary:expr) => {
        MindLinkError::BinaryExecution {
            message: $msg.to_string(),
            binary_name: $binary.to_string(),
            binary_path: None,
            source: None,
        }
    };
    ($msg:expr, $binary:expr, $path:expr) => {
        MindLinkError::BinaryExecution {
            message: $msg.to_string(),
            binary_name: $binary.to_string(),
            binary_path: Some($path.to_string()),
            source: None,
        }
    };
    ($msg:expr, $binary:expr, $path:expr, $source:expr) => {
        MindLinkError::BinaryExecution {
            message: $msg.to_string(),
            binary_name: $binary.to_string(),
            binary_path: Some($path.to_string()),
            source: Some(anyhow::Error::from($source)),
        }
    };
}