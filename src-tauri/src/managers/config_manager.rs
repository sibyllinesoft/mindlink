// Configuration Manager - Rust implementation with enterprise-grade error handling
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

use crate::error::{MindLinkError, MindLinkResult};
use crate::{log_info, log_error};

/// Current configuration schema version for migration support
const CONFIG_VERSION: u32 = 1;

/// Configuration schema with version and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub version: u32,
    pub server: ServerConfig,
    pub bifrost: BifrostConfig,
    pub tunnel: TunnelConfig,
    pub features: FeatureConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BifrostConfig {
    pub port: u16,
    pub host: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub enabled: bool,
    pub tunnel_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub reasoning_effort: String,
    pub reasoning_summaries: String,
    pub reasoning_compatibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub health_check_interval: u64,
    pub error_threshold: u32,
    pub notifications: bool,
}

/// Enterprise-grade configuration manager with validation and migration support
#[derive(Debug)]
pub struct ConfigManager {
    config_path: PathBuf,
    backup_path: PathBuf,
    config: RwLock<ConfigSchema>,
}

impl ConfigManager {
    /// Create a new ConfigManager with proper error handling and validation
    pub async fn new() -> MindLinkResult<Self> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| MindLinkError::SystemResource {
                message: "Cannot determine home directory".to_string(),
                resource_type: "home directory".to_string(),
                source: None,
            })?
            .join(".mindlink");
        
        let config_path = config_dir.join("config.json");
        let backup_path = config_dir.join("config.json.backup");
        
        // Ensure directory exists
        fs::create_dir_all(&config_dir).await.map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to create config directory".to_string(),
                path: Some(config_dir.to_string_lossy().to_string()),
                operation: "create directory".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        log_info!("ConfigManager", "Initializing configuration system");
        
        let config = Self::load_or_create_config(&config_path, &backup_path).await?;
        
        let manager = Self {
            config_path,
            backup_path,
            config: RwLock::new(config),
        };
        
        log_info!("ConfigManager", "Configuration system initialized successfully");
        
        Ok(manager)
    }
    
    /// Load existing config or create default with proper migration support
    async fn load_or_create_config(config_path: &PathBuf, backup_path: &PathBuf) -> MindLinkResult<ConfigSchema> {
        match fs::read_to_string(config_path).await {
            Ok(content) => {
                log_info!("ConfigManager", "Loading existing configuration");
                
                match serde_json::from_str::<ConfigSchema>(&content) {
                    Ok(config) => {
                        Self::validate_config(&config)?;
                        Self::migrate_config_if_needed(config, config_path, backup_path).await
                    }
                    Err(e) => {
                        log_error!("ConfigManager", MindLinkError::Configuration {
                            message: "Failed to parse configuration, creating backup and using defaults".to_string(),
                            config_key: None,
                            source: Some(e.into()),
                        });
                        
                        // Backup corrupted config
                        if let Err(backup_err) = fs::copy(config_path, backup_path).await {
                            log_error!("ConfigManager", MindLinkError::FileSystem {
                                message: "Failed to backup corrupted config".to_string(),
                                path: Some(backup_path.to_string_lossy().to_string()),
                                operation: "backup".to_string(),
                                source: Some(backup_err.into()),
                            });
                        }
                        
                        Self::create_default_config(config_path).await
                    }
                }
            }
            Err(_) => {
                log_info!("ConfigManager", "No existing configuration found, creating default");
                Self::create_default_config(config_path).await
            }
        }
    }
    
    /// Create default configuration with validation
    async fn create_default_config(config_path: &PathBuf) -> MindLinkResult<ConfigSchema> {
        let default_config = ConfigSchema {
            version: CONFIG_VERSION,
            server: ServerConfig {
                port: 3001,
                host: "127.0.0.1".to_string(),
            },
            bifrost: BifrostConfig {
                port: 3002,
                host: "127.0.0.1".to_string(),
                enabled: true,
            },
            tunnel: TunnelConfig {
                enabled: true,
                tunnel_type: "quick".to_string(),
            },
            features: FeatureConfig {
                reasoning_effort: "medium".to_string(),
                reasoning_summaries: "auto".to_string(),
                reasoning_compatibility: "think-tags".to_string(),
            },
            monitoring: MonitoringConfig {
                health_check_interval: 30,
                error_threshold: 5,
                notifications: true,
            },
        };
        
        Self::validate_config(&default_config)?;
        
        // Save default config
        let json = serde_json::to_string_pretty(&default_config).map_err(|e| {
            MindLinkError::Configuration {
                message: "Failed to serialize default configuration".to_string(),
                config_key: None,
                source: Some(e.into()),
            }
        })?;
        
        fs::write(config_path, json).await.map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to save default configuration".to_string(),
                path: Some(config_path.to_string_lossy().to_string()),
                operation: "write".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        log_info!("ConfigManager", "Default configuration created and saved");
        
        Ok(default_config)
    }
    
    /// Validate configuration values
    fn validate_config(config: &ConfigSchema) -> MindLinkResult<()> {
        // Validate server config
        if config.server.port == 0 {
            return Err(MindLinkError::Configuration {
                message: "Server port cannot be 0".to_string(),
                config_key: Some("server.port".to_string()),
                source: None,
            });
        }
        
        if config.server.host.is_empty() {
            return Err(MindLinkError::Configuration {
                message: "Server host cannot be empty".to_string(),
                config_key: Some("server.host".to_string()),
                source: None,
            });
        }
        
        // Validate bifrost config
        if config.bifrost.port == 0 {
            return Err(MindLinkError::Configuration {
                message: "Bifrost port cannot be 0".to_string(),
                config_key: Some("bifrost.port".to_string()),
                source: None,
            });
        }
        
        // Validate reasoning effort values
        let valid_efforts = ["low", "medium", "high"];
        if !valid_efforts.contains(&config.features.reasoning_effort.as_str()) {
            return Err(MindLinkError::Configuration {
                message: format!("Invalid reasoning effort: {}. Must be one of: {:?}", 
                               config.features.reasoning_effort, valid_efforts),
                config_key: Some("features.reasoning_effort".to_string()),
                source: None,
            });
        }
        
        // Validate tunnel type
        let valid_types = ["quick", "named"];
        if !valid_types.contains(&config.tunnel.tunnel_type.as_str()) {
            return Err(MindLinkError::Configuration {
                message: format!("Invalid tunnel type: {}. Must be one of: {:?}", 
                               config.tunnel.tunnel_type, valid_types),
                config_key: Some("tunnel.tunnel_type".to_string()),
                source: None,
            });
        }
        
        Ok(())
    }
    
    /// Handle configuration migration if needed
    async fn migrate_config_if_needed(
        mut config: ConfigSchema,
        config_path: &PathBuf,
        backup_path: &PathBuf,
    ) -> MindLinkResult<ConfigSchema> {
        if config.version < CONFIG_VERSION {
            log_info!("ConfigManager", 
                     format!("Migrating configuration from version {} to {}", 
                            config.version, CONFIG_VERSION));
            
            // Backup current config before migration
            let backup_content = serde_json::to_string_pretty(&config).map_err(|e| {
                MindLinkError::Configuration {
                    message: "Failed to serialize config for backup".to_string(),
                    config_key: None,
                    source: Some(e.into()),
                }
            })?;
            
            fs::write(backup_path, backup_content).await.map_err(|e| {
                MindLinkError::FileSystem {
                    message: "Failed to create config backup before migration".to_string(),
                    path: Some(backup_path.to_string_lossy().to_string()),
                    operation: "write backup".to_string(),
                    source: Some(e.into()),
                }
            })?;
            
            // Perform migration steps
            config = Self::migrate_config(config)?;
            config.version = CONFIG_VERSION;
            
            // Save migrated config
            let json = serde_json::to_string_pretty(&config).map_err(|e| {
                MindLinkError::Configuration {
                    message: "Failed to serialize migrated configuration".to_string(),
                    config_key: None,
                    source: Some(e.into()),
                }
            })?;
            
            fs::write(config_path, json).await.map_err(|e| {
                MindLinkError::FileSystem {
                    message: "Failed to save migrated configuration".to_string(),
                    path: Some(config_path.to_string_lossy().to_string()),
                    operation: "write migrated config".to_string(),
                    source: Some(e.into()),
                }
            })?;
            
            log_info!("ConfigManager", "Configuration migration completed successfully");
        }
        
        Ok(config)
    }
    
    /// Migrate configuration between versions
    fn migrate_config(config: ConfigSchema) -> MindLinkResult<ConfigSchema> {
        // For now, no migration logic needed since this is version 1
        // Future migrations would be implemented here
        Ok(config)
    }
    
    /// Get a read-only copy of the configuration
    pub async fn get_config(&self) -> ConfigSchema {
        self.config.read().await.clone()
    }
    
    /// Update the entire configuration with validation
    pub async fn update_config(&self, new_config: ConfigSchema) -> MindLinkResult<()> {
        Self::validate_config(&new_config)?;
        
        // Create backup before update
        let current_config = self.config.read().await.clone();
        let backup_content = serde_json::to_string_pretty(&current_config).map_err(|e| {
            MindLinkError::Configuration {
                message: "Failed to serialize current config for backup".to_string(),
                config_key: None,
                source: Some(e.into()),
            }
        })?;
        
        fs::write(&self.backup_path, backup_content).await.map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to create config backup before update".to_string(),
                path: Some(self.backup_path.to_string_lossy().to_string()),
                operation: "write backup".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        // Save new config
        let json = serde_json::to_string_pretty(&new_config).map_err(|e| {
            MindLinkError::Configuration {
                message: "Failed to serialize new configuration".to_string(),
                config_key: None,
                source: Some(e.into()),
            }
        })?;
        
        fs::write(&self.config_path, json).await.map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to save new configuration".to_string(),
                path: Some(self.config_path.to_string_lossy().to_string()),
                operation: "write config".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        // Update in-memory config
        *self.config.write().await = new_config;
        
        log_info!("ConfigManager", "Configuration updated successfully");
        
        Ok(())
    }
    
    /// Get specific configuration section
    pub async fn get_server_config(&self) -> ServerConfig {
        self.config.read().await.server.clone()
    }
    
    pub async fn get_bifrost_config(&self) -> BifrostConfig {
        self.config.read().await.bifrost.clone()
    }
    
    pub async fn get_tunnel_config(&self) -> TunnelConfig {
        self.config.read().await.tunnel.clone()
    }
    
    pub async fn get_feature_config(&self) -> FeatureConfig {
        self.config.read().await.features.clone()
    }
    
    pub async fn get_monitoring_config(&self) -> MonitoringConfig {
        self.config.read().await.monitoring.clone()
    }
    
    /// Restore configuration from backup
    pub async fn restore_from_backup(&self) -> MindLinkResult<()> {
        if !self.backup_path.exists() {
            return Err(MindLinkError::FileSystem {
                message: "No backup configuration file found".to_string(),
                path: Some(self.backup_path.to_string_lossy().to_string()),
                operation: "check backup existence".to_string(),
                source: None,
            });
        }
        
        let content = fs::read_to_string(&self.backup_path).await.map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to read backup configuration".to_string(),
                path: Some(self.backup_path.to_string_lossy().to_string()),
                operation: "read backup".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        let backup_config: ConfigSchema = serde_json::from_str(&content).map_err(|e| {
            MindLinkError::Configuration {
                message: "Failed to parse backup configuration".to_string(),
                config_key: None,
                source: Some(e.into()),
            }
        })?;
        
        Self::validate_config(&backup_config)?;
        
        // Save restored config
        let json = serde_json::to_string_pretty(&backup_config).map_err(|e| {
            MindLinkError::Configuration {
                message: "Failed to serialize restored configuration".to_string(),
                config_key: None,
                source: Some(e.into()),
            }
        })?;
        
        fs::write(&self.config_path, json).await.map_err(|e| {
            MindLinkError::FileSystem {
                message: "Failed to save restored configuration".to_string(),
                path: Some(self.config_path.to_string_lossy().to_string()),
                operation: "write restored config".to_string(),
                source: Some(e.into()),
            }
        })?;
        
        // Update in-memory config
        *self.config.write().await = backup_config;
        
        log_info!("ConfigManager", "Configuration restored from backup successfully");
        
        Ok(())
    }
}