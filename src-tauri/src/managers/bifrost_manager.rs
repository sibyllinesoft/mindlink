// Bifrost Manager - Manages the actual Bifrost LLM Router process
use crate::error::{MindLinkError, MindLinkResult};
use crate::logging::{get_logger, LogCategory, LogEntry, LogLevel};
use crate::managers::binary_manager::BinaryManager;
use crate::process_monitor::{get_process_monitor, MonitorConfig};
use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct BifrostManager {
    process: Arc<RwLock<Option<Child>>>,
    port: u16,
    host: String,
    is_running: Arc<RwLock<bool>>,
    config_path: Option<PathBuf>,
    binary_path: Option<PathBuf>,
    binary_manager: Arc<RwLock<BinaryManager>>,
}

impl BifrostManager {
    pub async fn new() -> Self {
        let binary_manager = Arc::new(RwLock::new(BinaryManager::new().await.unwrap_or_else(
            |e| {
                eprintln!("Failed to initialize binary manager: {}", e);
                // This should not fail in production, but we'll handle it gracefully
                panic!("Binary manager initialization failed: {}", e);
            },
        )));

        let binary_path = Self::find_local_bifrost_binary(&binary_manager).await;

        let available_port = Self::find_available_port("127.0.0.1", 3003)
            .await
            .unwrap_or(3003); // Fallback to 3003 if detection fails

        println!("Using port {} for Bifrost", available_port);

        Self {
            process: Arc::new(RwLock::new(None)),
            port: available_port,
            host: "127.0.0.1".to_string(),
            is_running: Arc::new(RwLock::new(false)),
            config_path: None,
            binary_path,
            binary_manager,
        }
    }

    // Find the first available port starting from the given port
    async fn find_available_port(host: &str, start_port: u16) -> Option<u16> {
        for port in start_port..start_port + 100 {
            // Check up to 100 ports
            let addr: SocketAddr = format!("{}:{}", host, port).parse().ok()?;

            if TcpListener::bind(&addr).await.is_ok() {
                return Some(port);
            }
        }
        None
    }

    // Find the locally-built Bifrost binary
    async fn find_local_bifrost_binary(
        _binary_manager: &Arc<RwLock<BinaryManager>>,
    ) -> Option<PathBuf> {
        // Determine the correct binary name based on platform
        let binary_name = if cfg!(windows) {
            "bifrost-http.exe"
        } else {
            "bifrost-http"
        };
        let platform_specific_name = format!(
            "{}-{}",
            binary_name.trim_end_matches(".exe"),
            Self::get_platform_target()
        );

        // List of possible binary names to check (in priority order)
        let binary_names = vec![
            binary_name.to_string(),
            platform_specific_name,
            "bifrost-http".to_string(), // fallback without extension
        ];

        // Check locations for each binary name
        for name in &binary_names {
            // First check for the locally-built binary in src-tauri/binaries/
            let local_binary_path = PathBuf::from("binaries").join(name);

            if local_binary_path.exists() && local_binary_path.is_file() {
                println!(
                    "Found locally-built Bifrost binary at: {:?}",
                    local_binary_path
                );

                // Verify it's executable and works
                if Self::verify_local_binary(&local_binary_path).await {
                    return Some(local_binary_path);
                } else {
                    println!("Local binary exists but failed verification");
                }
            }

            // Try absolute path from current exe directory
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let abs_binary_path = exe_dir.join("binaries").join(name);
                    if abs_binary_path.exists() && abs_binary_path.is_file() {
                        println!("Found Bifrost binary at: {:?}", abs_binary_path);
                        if Self::verify_local_binary(&abs_binary_path).await {
                            return Some(abs_binary_path);
                        }
                    }
                }
            }

            // Check if it's in the current working directory
            let cwd_binary_path = PathBuf::from("src-tauri/binaries").join(name);
            if cwd_binary_path.exists() && cwd_binary_path.is_file() {
                println!(
                    "Found Bifrost binary in src-tauri directory: {:?}",
                    cwd_binary_path
                );
                if Self::verify_local_binary(&cwd_binary_path).await {
                    return Some(cwd_binary_path);
                }
            }
        }

        println!("Locally-built Bifrost binary not found. Please run the build system to create the binary.");
        println!("Expected locations:");
        println!("  - binaries/bifrost-http (relative to executable)");
        println!("  - src-tauri/binaries/bifrost-http (relative to project root)");
        println!(
            "  - binaries/bifrost-http-{} (platform-specific)",
            Self::get_platform_target()
        );
        None
    }

    // Get the platform target string for binary names
    fn get_platform_target() -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
            ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
            ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
            ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
            _ => format!("{}-{}", arch, os),
        }
    }

    // Verify the local binary works by checking version or help
    async fn verify_local_binary(binary_path: &PathBuf) -> bool {
        // Check if the file is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(binary_path) {
                let permissions = metadata.permissions();
                if permissions.mode() & 0o111 == 0 {
                    println!("Binary is not executable: {:?}", binary_path);
                    return false;
                }
            }
        }

        // Try running the binary with --version
        match Command::new(binary_path).arg("--version").output().await {
            Ok(output) => {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    println!(
                        "Bifrost binary version check passed: {}",
                        version_str.trim()
                    );
                    return true;
                }
            },
            Err(e) => {
                println!("Failed to run binary version check: {}", e);
            },
        }

        // Try running with --help as fallback
        match Command::new(binary_path).arg("--help").output().await {
            Ok(output) => {
                if output.status.success() {
                    println!("Bifrost binary help check passed");
                    return true;
                }
            },
            Err(e) => {
                println!("Failed to run binary help check: {}", e);
            },
        }

        false
    }

    pub async fn start(&mut self) -> MindLinkResult<()> {
        if *self.is_running.read().await {
            return Ok(());
        }

        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Info,
                LogCategory::System,
                "Starting Bifrost LLM Router...".to_string(),
            )
            .with_component("BifrostManager");
            logger.log(entry);
        }

        // Ensure we have a binary path
        let binary_path = if let Some(ref path) = self.binary_path {
            path.clone()
        } else {
            return Err(MindLinkError::BinaryExecution {
                message: "Binary not found - please run the build system".to_string(),
                binary_name: "bifrost-http".to_string(),
                binary_path: None,
                source: None,
            });
        };

        // Make sure the binary exists
        if !binary_path.exists() {
            return Err(MindLinkError::BinaryExecution {
                message: "Binary file does not exist".to_string(),
                binary_name: "bifrost-http".to_string(),
                binary_path: Some(binary_path.to_string_lossy().to_string()),
                source: None,
            });
        }

        // Register with process monitor
        if let Some(monitor) = get_process_monitor() {
            let config = MonitorConfig {
                capture_stdout: true,
                capture_stderr: true,
                max_restart_attempts: 3,
                restart_delay: tokio::time::Duration::from_secs(5),
                output_buffer_size: 1024 * 1024,
                health_check_interval: tokio::time::Duration::from_secs(30),
                process_timeout: Some(tokio::time::Duration::from_secs(300)),
            };

            if let Err(e) = monitor
                .register_process(
                    "bifrost".to_string(),
                    "Bifrost LLM Router".to_string(),
                    config,
                )
                .await
            {
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Warn,
                        LogCategory::Process,
                        format!("Failed to register Bifrost with process monitor: {}", e),
                    )
                    .with_component("BifrostManager");
                    logger.log(entry);
                }
            }
        }

        // Run the locally-built Bifrost binary directly
        let mut cmd = Command::new(&binary_path);

        // Basic Bifrost configuration - adjust arguments based on actual bifrost-http binary
        cmd.arg("--host")
            .arg(&self.host)
            .arg("--port")
            .arg(self.port.to_string());

        // Add additional arguments that the bifrost-http binary might expect
        if std::env::var("BIFROST_LOG_LEVEL")
            .unwrap_or_default()
            .is_empty()
        {
            cmd.arg("--log-level").arg("info");
        }

        // Add config file if available
        if let Some(config_path) = &self.config_path {
            cmd.arg("--config").arg(config_path);
        }

        // Set up stdio to capture output for monitoring
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Debug,
                LogCategory::Process,
                format!("Executing Bifrost binary: {:?}", cmd),
            )
            .with_component("BifrostManager");
            logger.log(entry);
        }

        // Spawn the process
        let child = cmd.spawn().map_err(|e| MindLinkError::BinaryExecution {
            message: "Failed to spawn process".to_string(),
            binary_name: "bifrost-http".to_string(),
            binary_path: Some(binary_path.to_string_lossy().to_string()),
            source: Some(e.into()),
        })?;

        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Info,
                LogCategory::Process,
                format!("Bifrost LLM Router starting on {}:{}", self.host, self.port),
            )
            .with_component("BifrostManager");
            logger.log(entry);
        }

        // Start process monitoring
        if let Some(monitor) = get_process_monitor() {
            if let Err(e) = monitor.start_monitoring("bifrost".to_string(), child).await {
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Error,
                        LogCategory::Process,
                        format!("Failed to start process monitoring: {}", e),
                    )
                    .with_component("BifrostManager");
                    logger.log(entry);
                }
                return Err(e);
            }
        } else {
            // Fallback: store the child directly (old behavior)
            *self.process.write().await = Some(child);
        }

        *self.is_running.write().await = true;

        // Wait a moment for startup
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

        // Check if process is still running (if we have direct access)
        if let Some(process) = self.process.write().await.as_mut() {
            match process.try_wait() {
                Ok(Some(status)) => {
                    *self.is_running.write().await = false;
                    return Err(MindLinkError::ProcessMonitoring {
                        message: format!("Process exited immediately with status: {}. Check the binary arguments.", status),
                        process_name: "Bifrost".to_string(),
                        pid: None,
                        source: None,
                    });
                },
                Ok(None) => {
                    if let Some(logger) = get_logger() {
                        let entry = LogEntry::new(
                            LogLevel::Info,
                            LogCategory::System,
                            format!(
                                "Bifrost LLM Router started successfully on {}:{}",
                                self.host, self.port
                            ),
                        )
                        .with_component("BifrostManager");
                        logger.log(entry);
                    }
                },
                Err(e) => {
                    *self.is_running.write().await = false;
                    return Err(MindLinkError::ProcessMonitoring {
                        message: "Failed to check process status".to_string(),
                        process_name: "Bifrost".to_string(),
                        pid: None,
                        source: Some(e.into()),
                    });
                },
            }
        }

        Ok(())
    }

    pub async fn stop(&mut self) -> MindLinkResult<()> {
        if !*self.is_running.read().await {
            return Ok(());
        }

        if let Some(logger) = get_logger() {
            let entry = LogEntry::new(
                LogLevel::Info,
                LogCategory::System,
                "Stopping Bifrost LLM Router...".to_string(),
            )
            .with_component("BifrostManager");
            logger.log(entry);
        }

        // First try to stop through process monitor
        if let Some(monitor) = get_process_monitor() {
            if let Err(e) = monitor.stop_process("bifrost").await {
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Warn,
                        LogCategory::Process,
                        format!("Process monitor stop failed: {}", e),
                    )
                    .with_component("BifrostManager");
                    logger.log(entry);
                }
            }
        }

        // Fallback: direct process termination
        if let Some(mut child) = self.process.write().await.take() {
            match child.kill().await {
                Ok(_) => {
                    // Wait for the process to exit
                    match child.wait().await {
                        Ok(_) => {
                            if let Some(logger) = get_logger() {
                                let entry = LogEntry::new(
                                    LogLevel::Info,
                                    LogCategory::System,
                                    "Bifrost LLM Router stopped successfully".to_string(),
                                )
                                .with_component("BifrostManager");
                                logger.log(entry);
                            }
                        },
                        Err(e) => {
                            let wait_error = MindLinkError::ProcessMonitoring {
                                message: "Error waiting for process to exit".to_string(),
                                process_name: "Bifrost".to_string(),
                                pid: None,
                                source: Some(e.into()),
                            };

                            if let Some(logger) = get_logger() {
                                logger.log_error("BifrostManager", &wait_error, None);
                            }
                        },
                    }
                },
                Err(e) => {
                    let kill_error = MindLinkError::ProcessMonitoring {
                        message: "Failed to terminate process".to_string(),
                        process_name: "Bifrost".to_string(),
                        pid: None,
                        source: Some(e.into()),
                    };

                    if let Some(logger) = get_logger() {
                        logger.log_error("BifrostManager", &kill_error, None);
                    }

                    return Err(kill_error);
                },
            }
        }

        *self.is_running.write().await = false;

        // Unregister from process monitor
        if let Some(monitor) = get_process_monitor() {
            if let Err(e) = monitor.unregister_process("bifrost").await {
                if let Some(logger) = get_logger() {
                    let entry = LogEntry::new(
                        LogLevel::Warn,
                        LogCategory::Process,
                        format!("Failed to unregister process: {}", e),
                    )
                    .with_component("BifrostManager");
                    logger.log(entry);
                }
            }
        }

        Ok(())
    }

    pub async fn restart(&mut self) -> MindLinkResult<()> {
        self.stop().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        self.start().await
    }

    pub async fn check_health(&self) -> Result<bool> {
        if !*self.is_running.read().await {
            return Ok(false);
        }

        // Check if process is still running
        if let Some(process) = self.process.write().await.as_mut() {
            match process.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    return Ok(false);
                },
                Ok(None) => {
                    // Process is still running, check HTTP health
                },
                Err(_) => return Ok(false),
            }
        } else {
            return Ok(false);
        }

        // Make HTTP health check request to Bifrost
        let url = format!("http://{}:{}/health", self.host, self.port);

        match reqwest::get(&url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => {
                // Try alternative health endpoints
                let alt_url = format!("http://{}:{}/v1/models", self.host, self.port);
                match reqwest::get(&alt_url).await {
                    Ok(response) => Ok(response.status().is_success()),
                    Err(_) => Ok(false),
                }
            },
        }
    }

    pub async fn get_local_url(&self) -> Option<String> {
        if *self.is_running.read().await {
            Some(format!("http://{}:{}", self.host, self.port))
        } else {
            None
        }
    }

    pub async fn get_api_url(&self) -> Option<String> {
        if *self.is_running.read().await {
            Some(format!("http://{}:{}/v1", self.host, self.port))
        } else {
            None
        }
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    pub async fn configure(&mut self, host: String, port: u16) {
        if *self.is_running.read().await {
            eprintln!("Cannot change configuration while Bifrost is running");
            return;
        }

        self.host = host;
        self.port = port;
    }

    pub async fn set_config_path(&mut self, config_path: PathBuf) {
        if *self.is_running.read().await {
            eprintln!("Cannot change config path while Bifrost is running");
            return;
        }

        self.config_path = Some(config_path);
    }

    pub async fn set_binary_path(&mut self, binary_path: PathBuf) {
        if *self.is_running.read().await {
            eprintln!("Cannot change binary path while Bifrost is running");
            return;
        }

        self.binary_path = Some(binary_path);
    }

    // Get available models from Bifrost
    pub async fn get_models(&self) -> Result<Vec<String>> {
        if !*self.is_running.read().await {
            return Err(anyhow!("Bifrost is not running"));
        }

        let url = format!("http://{}:{}/v1/models", self.host, self.port);

        match reqwest::get(&url).await {
            Ok(response) => {
                if response.status().is_success() {
                    let json: serde_json::Value = response.json().await?;
                    let mut models = Vec::new();

                    if let Some(data) = json.get("data") {
                        if let Some(models_array) = data.as_array() {
                            for model in models_array {
                                if let Some(id) = model.get("id") {
                                    if let Some(id_str) = id.as_str() {
                                        models.push(id_str.to_string());
                                    }
                                }
                            }
                        }
                    }

                    Ok(models)
                } else {
                    Err(anyhow!("Failed to get models: HTTP {}", response.status()))
                }
            },
            Err(e) => Err(anyhow!("Failed to connect to Bifrost: {}", e)),
        }
    }

    // Get Bifrost status info
    pub async fn get_status_info(&self) -> (bool, Option<String>, Option<String>) {
        let running = *self.is_running.read().await;
        let url = if running {
            Some(format!("http://{}:{}", self.host, self.port))
        } else {
            None
        };
        let api_url = if running {
            Some(format!("http://{}:{}/v1", self.host, self.port))
        } else {
            None
        };

        (running, url, api_url)
    }

    // Check if Bifrost binary is available
    pub async fn is_binary_available(&self) -> bool {
        self.binary_path.is_some()
    }

    // Get the path to Bifrost binary
    pub async fn get_binary_path(&self) -> Option<PathBuf> {
        self.binary_path.clone()
    }

    // Refresh binary path - check for locally-built binary
    pub async fn refresh_binary_path(&mut self) -> Result<PathBuf> {
        // Clear the current binary path and re-scan
        self.binary_path = None;

        // Re-scan for locally-built binary
        if let Some(path) = Self::find_local_bifrost_binary(&self.binary_manager).await {
            self.binary_path = Some(path.clone());
            println!("Refreshed Bifrost binary path: {:?}", path);
            Ok(path)
        } else {
            Err(anyhow!(
                "No locally-built Bifrost binary found. Please run the build system to create it."
            ))
        }
    }

    // Trigger binary rebuild using BinaryManager
    pub async fn rebuild_bifrost(&mut self) -> Result<PathBuf> {
        println!("Triggering Bifrost binary rebuild...");

        // Stop the current process if running
        if *self.is_running.read().await {
            self.stop().await?;
        }

        // Use BinaryManager to build the binary
        let path = {
            let manager = self.binary_manager.read().await;
            manager.build_bifrost().await?
        };

        // Update our binary path
        self.binary_path = Some(path.clone());

        println!("Bifrost binary rebuild completed: {:?}", path);
        Ok(path)
    }

    // Get installation status and info
    pub async fn get_installation_info(&self) -> (bool, Option<PathBuf>, Option<String>) {
        let binary_available = self.binary_path.is_some();
        let binary_path = self.binary_path.clone();

        let status_message = if binary_available {
            Some("Locally-built Bifrost binary is ready".to_string())
        } else {
            Some(
                "Locally-built Bifrost binary not found - build system needs to be run".to_string(),
            )
        };

        (binary_available, binary_path, status_message)
    }

    // Check if build is recommended
    pub async fn should_build(&self) -> bool {
        if self.binary_path.is_none() {
            return true;
        }

        // Check if the binary still exists and is valid
        if let Some(ref path) = self.binary_path {
            if !path.exists() || !Self::verify_local_binary(path).await {
                return true;
            }
        }

        false
    }
}
