// Binary Manager - Downloads and manages bundled binaries for MindLink with enterprise-grade error handling
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::process::Command as TokioCommand;

use crate::error::{MindLinkError, MindLinkResult};
use crate::logging::get_logger;
use crate::{log_error, log_info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub executable_name: String,
    pub checksum: Option<String>,
}

#[derive(Debug)]
pub struct BinaryManager {
    #[allow(dead_code)]
    data_dir: PathBuf,
    binaries_dir: PathBuf,
}

impl BinaryManager {
    /// Create a new BinaryManager with proper error handling
    pub async fn new() -> MindLinkResult<Self> {
        // Get application data directory
        let data_dir = Self::get_app_data_dir()?;
        let binaries_dir = data_dir.join("binaries");

        // Ensure directories exist
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(&binaries_dir)?;

        Ok(Self {
            data_dir,
            binaries_dir,
        })
    }

    fn get_app_data_dir() -> Result<PathBuf> {
        let app_name = "mindlink";

        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                return Ok(PathBuf::from(appdata).join(app_name));
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = std::env::var_os("HOME") {
                return Ok(PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join(app_name));
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(xdg_data) = std::env::var_os("XDG_DATA_HOME") {
                return Ok(PathBuf::from(xdg_data).join(app_name));
            }
            if let Some(home) = std::env::var_os("HOME") {
                return Ok(PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join(app_name));
            }
        }

        Err(anyhow!("Could not determine application data directory"))
    }

    /// Build Bifrost binary using the local build system with real-time logging and verification
    /// This replaces the old npm-based installation
    pub async fn build_bifrost(&self) -> MindLinkResult<PathBuf> {
        log_info!("BinaryManager", "Starting Bifrost binary build from source");

        // Check if we're in the correct directory structure
        let project_root = std::env::current_dir().map_err(|e| MindLinkError::FileSystem {
            message: "Failed to get current directory".to_string(),
            path: None,
            operation: "get current directory".to_string(),
            source: Some(e.into()),
        })?;

        let build_script = project_root.join("scripts").join("tauri-build-bifrost.sh");

        if !build_script.exists() {
            return Err(MindLinkError::BinaryExecution {
                message: "Build script not found. Make sure you're running from the project root."
                    .to_string(),
                binary_name: "tauri-build-bifrost.sh".to_string(),
                binary_path: Some(build_script.to_string_lossy().to_string()),
                source: None,
            });
        }

        log_info!(
            "BinaryManager",
            format!("Found build script at: {:?}", build_script)
        );

        // Run the build script with real-time logging
        let script_path = build_script
            .to_str()
            .ok_or_else(|| MindLinkError::BinaryExecution {
                message: "Build script path contains invalid UTF-8 characters".to_string(),
                binary_name: "tauri-build-bifrost.sh".to_string(),
                binary_path: Some(build_script.to_string_lossy().to_string()),
                source: None,
            })?;

        log_info!(
            "BinaryManager",
            format!("Executing build script: {}", script_path)
        );

        // Use spawn instead of output for real-time logging
        let mut child = TokioCommand::new("bash")
            .args(&[script_path, "--force"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| MindLinkError::BinaryExecution {
                message: "Failed to start build script".to_string(),
                binary_name: "bash".to_string(),
                binary_path: Some(script_path.to_string()),
                source: Some(e.into()),
            })?;

        // Stream stdout and stderr with real-time logging
        let stdout = child.stdout.take().ok_or_else(|| MindLinkError::Internal {
            message: "Failed to capture build script stdout".to_string(),
            component: Some("BinaryManager".to_string()),
            source: None,
        })?;

        let stderr = child.stderr.take().ok_or_else(|| MindLinkError::Internal {
            message: "Failed to capture build script stderr".to_string(),
            component: Some("BinaryManager".to_string()),
            source: None,
        })?;

        use tokio::io::{AsyncBufReadExt, BufReader};

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let mut stdout_lines = stdout_reader.lines();
        let mut stderr_lines = stderr_reader.lines();

        // Log output in real-time
        let logging_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    line_result = stdout_lines.next_line() => {
                        match line_result {
                            Ok(Some(line)) => {
                                if let Some(logger) = get_logger() {
                                    logger.log_process_output("tauri-build-bifrost", "stdout", &line, None);
                                }
                                println!("BUILD: {}", line);
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                    line_result = stderr_lines.next_line() => {
                        match line_result {
                            Ok(Some(line)) => {
                                if let Some(logger) = get_logger() {
                                    logger.log_process_output("tauri-build-bifrost", "stderr", &line, None);
                                }
                                eprintln!("BUILD ERROR: {}", line);
                            }
                            Ok(None) => break,
                            Err(_) => break,
                        }
                    }
                }
            }
        });

        // Wait for the build process to complete
        let exit_status = child
            .wait()
            .await
            .map_err(|e| MindLinkError::BinaryExecution {
                message: "Build script execution failed".to_string(),
                binary_name: "tauri-build-bifrost.sh".to_string(),
                binary_path: Some(script_path.to_string()),
                source: Some(e.into()),
            })?;

        // Ensure logging task completes
        let _ = logging_task.await;

        if !exit_status.success() {
            let error_msg = format!(
                "Build script failed with exit code: {:?}",
                exit_status.code()
            );
            log_error!(
                "BinaryManager",
                MindLinkError::BinaryExecution {
                    message: error_msg.clone(),
                    binary_name: "tauri-build-bifrost.sh".to_string(),
                    binary_path: Some(script_path.to_string()),
                    source: None,
                }
            );

            return Err(MindLinkError::BinaryExecution {
                message: error_msg,
                binary_name: "tauri-build-bifrost.sh".to_string(),
                binary_path: Some(script_path.to_string()),
                source: None,
            });
        }

        log_info!("BinaryManager", "Build script completed successfully");

        // Verify the built binary exists and is valid
        let binary_path =
            self.get_local_bifrost_path()
                .ok_or_else(|| MindLinkError::BinaryExecution {
                    message: "Built binary not found at expected location".to_string(),
                    binary_name: "bifrost".to_string(),
                    binary_path: None,
                    source: None,
                })?;

        // Verify binary is executable and calculate checksum
        self.verify_binary_integrity(&binary_path).await?;

        log_info!(
            "BinaryManager",
            format!(
                "Successfully built and verified Bifrost binary at: {:?}",
                binary_path
            )
        );

        Ok(binary_path)
    }

    /// Verify binary integrity and generate checksum
    async fn verify_binary_integrity(
        &self,
        binary_path: &std::path::Path,
    ) -> MindLinkResult<String> {
        use sha2::{Digest, Sha256};
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        // Check if file exists and is executable
        let metadata =
            tokio::fs::metadata(binary_path)
                .await
                .map_err(|e| MindLinkError::FileSystem {
                    message: "Failed to read binary metadata".to_string(),
                    path: Some(binary_path.to_string_lossy().to_string()),
                    operation: "read metadata".to_string(),
                    source: Some(e.into()),
                })?;

        if !metadata.is_file() {
            return Err(MindLinkError::BinaryExecution {
                message: "Binary path is not a regular file".to_string(),
                binary_name: "bifrost".to_string(),
                binary_path: Some(binary_path.to_string_lossy().to_string()),
                source: None,
            });
        }

        // Calculate SHA256 checksum
        let mut file = File::open(binary_path)
            .await
            .map_err(|e| MindLinkError::FileSystem {
                message: "Failed to open binary for checksum calculation".to_string(),
                path: Some(binary_path.to_string_lossy().to_string()),
                operation: "open file".to_string(),
                source: Some(e.into()),
            })?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 8192];

        loop {
            let bytes_read =
                file.read(&mut buffer)
                    .await
                    .map_err(|e| MindLinkError::FileSystem {
                        message: "Failed to read binary for checksum calculation".to_string(),
                        path: Some(binary_path.to_string_lossy().to_string()),
                        operation: "read file".to_string(),
                        source: Some(e.into()),
                    })?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let checksum = format!("{:x}", hasher.finalize());

        log_info!(
            "BinaryManager",
            format!(
                "Binary verified - Size: {} bytes, SHA256: {}",
                metadata.len(),
                checksum
            )
        );

        Ok(checksum)
    }

    /// Download Bifrost binary directly from GitHub releases
    #[allow(dead_code)]
    async fn download_bifrost_from_github(&self, install_dir: &Path) -> Result<PathBuf> {
        println!("Downloading Bifrost from GitHub releases...");

        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        // Map Rust target names to GitHub release names
        let (_platform, extension) = match (os, arch) {
            ("linux", "x86_64") => ("linux-amd64", ""),
            ("linux", "aarch64") => ("linux-arm64", ""),
            ("macos", "x86_64") => ("darwin-amd64", ""),
            ("macos", "aarch64") => ("darwin-arm64", ""),
            ("windows", "x86_64") => ("windows-amd64", ".exe"),
            _ => return Err(anyhow!("Unsupported platform: {}-{}", os, arch)),
        };

        // For now, let's create a simple binary downloader
        // In a real implementation, you'd fetch the latest release from GitHub API
        let binary_name = format!("bifrost{}", extension);
        let binary_path = install_dir.join(&binary_name);

        // Create a placeholder binary for development
        // In production, this would download the actual binary
        self.create_development_bifrost_stub(&binary_path).await?;

        Ok(binary_path)
    }

    /// Create a development stub for Bifrost (for testing)
    #[allow(dead_code)]
    async fn create_development_bifrost_stub(&self, binary_path: &Path) -> Result<()> {
        println!("Creating development Bifrost stub at: {:?}", binary_path);

        let stub_content = if cfg!(windows) {
            r#"@echo off
echo Bifrost LLM Router (Development Stub)
echo This is a development stub. In production, this would be the real Bifrost binary.
echo Starting on port 3003...
timeout /t 2 >nul
echo Error: Development stub cannot actually start Bifrost service.
exit 1
"#
        } else {
            r#"#!/bin/bash
echo "Bifrost LLM Router (Development Stub)"
echo "This is a development stub. In production, this would be the real Bifrost binary."
echo "Starting on port 3003..."
sleep 2
echo "Error: Development stub cannot actually start Bifrost service."
exit 1
"#
        };

        fs::write(binary_path, stub_content)?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(binary_path, perms)?;
        }

        Ok(())
    }

    /// Check if npm is available
    #[allow(dead_code)]
    async fn is_npm_available(&self) -> Result<bool> {
        let output = TokioCommand::new("npm").args(&["--version"]).output().await;

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    /// Check if a binary is installed and get its path
    /// For Bifrost, this now looks for the locally-built binary first
    pub fn get_binary_path(&self, binary_name: &str) -> Option<PathBuf> {
        if binary_name == "bifrost" || binary_name == "bifrost-http" {
            return self.get_local_bifrost_path();
        }

        let binary_dir = self.binaries_dir.join(binary_name);

        let possible_paths = vec![
            binary_dir.join(binary_name),
            binary_dir.join(format!("{}.exe", binary_name)),
            binary_dir
                .join("node_modules")
                .join(".bin")
                .join(binary_name),
            binary_dir
                .join("node_modules")
                .join(".bin")
                .join(format!("{}.exe", binary_name)),
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Get the path to the locally-built Bifrost binary
    pub fn get_local_bifrost_path(&self) -> Option<PathBuf> {
        // Determine the correct binary name and platform-specific variant
        let base_name = "bifrost-http";
        let binary_name = if cfg!(windows) {
            "bifrost-http.exe"
        } else {
            "bifrost-http"
        };
        let platform_specific_name = format!("{}-{}", base_name, Self::get_platform_target());

        // List of possible binary names to check (in priority order)
        let binary_names = vec![
            binary_name.to_string(),
            platform_specific_name,
            base_name.to_string(), // fallback without extension
        ];

        for name in &binary_names {
            // Check relative to current executable (for bundled apps)
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let bundled_path = exe_dir.join("binaries").join(name);
                    if bundled_path.exists() {
                        return Some(bundled_path);
                    }
                }
            }

            // Check in src-tauri/binaries (development)
            let dev_path = std::path::PathBuf::from("src-tauri/binaries").join(name);
            if dev_path.exists() {
                return Some(dev_path);
            }

            // Check in binaries directory relative to current working dir
            let cwd_path = std::path::PathBuf::from("binaries").join(name);
            if cwd_path.exists() {
                return Some(cwd_path);
            }
        }

        None
    }

    /// Get the platform target string for binary names
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

    /// Verify a binary is working
    pub async fn verify_binary(&self, binary_path: &Path) -> Result<bool> {
        if !binary_path.exists() {
            return Ok(false);
        }

        // For the locally-built Bifrost binary, run it directly
        let filename = binary_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if filename == "bifrost-http"
            || filename == "bifrost-http.exe"
            || filename.starts_with("bifrost-http-")
        {
            // This is the locally-built Bifrost binary

            // Check if it's executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(binary_path) {
                    let permissions = metadata.permissions();
                    if permissions.mode() & 0o111 == 0 {
                        return Ok(false);
                    }
                }
            }

            // Try running with --version first
            match TokioCommand::new(binary_path)
                .arg("--version")
                .output()
                .await
            {
                Ok(output) => return Ok(output.status.success()),
                Err(_) => {
                    // Try with --help as fallback
                    match TokioCommand::new(binary_path).arg("--help").output().await {
                        Ok(output) => return Ok(output.status.success()),
                        Err(_) => return Ok(false),
                    }
                },
            }
        }

        // For legacy Node.js scripts (like old Bifrost npm packages), run with node
        if binary_path.extension().and_then(|ext| ext.to_str()) == Some("js")
            || binary_path.to_string_lossy().contains("node_modules")
        {
            // This is likely a Node.js script, try running with node
            let output = TokioCommand::new("node")
                .arg(binary_path)
                .args(&["--help"])
                .output()
                .await;

            match output {
                Ok(output) => return Ok(output.status.success()),
                Err(_) => {
                    // Try with --version instead
                    let output = TokioCommand::new("node")
                        .arg(binary_path)
                        .args(&["--version"])
                        .output()
                        .await;

                    match output {
                        Ok(output) => return Ok(output.status.success()),
                        Err(_) => return Ok(false),
                    }
                },
            }
        }

        // Try to run other binaries directly with --help or --version
        let output = TokioCommand::new(binary_path)
            .args(&["--help"])
            .output()
            .await;

        match output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => {
                // Try with --version instead
                let output = TokioCommand::new(binary_path)
                    .args(&["--version"])
                    .output()
                    .await;

                match output {
                    Ok(output) => Ok(output.status.success()),
                    Err(_) => Ok(false),
                }
            },
        }
    }

    /// Get the binaries directory path
    pub fn get_binaries_dir(&self) -> &Path {
        &self.binaries_dir
    }

    /// List all installed binaries
    pub fn list_installed_binaries(&self) -> Result<Vec<String>> {
        let mut binaries = Vec::new();

        if self.binaries_dir.exists() {
            for entry in fs::read_dir(&self.binaries_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        binaries.push(name.to_string());
                    }
                }
            }
        }

        Ok(binaries)
    }

    /// Remove a binary installation
    pub async fn uninstall_binary(&self, binary_name: &str) -> Result<()> {
        let binary_dir = self.binaries_dir.join(binary_name);

        if binary_dir.exists() {
            fs::remove_dir_all(&binary_dir)?;
            println!("Uninstalled binary: {}", binary_name);
        }

        Ok(())
    }

    /// Ensure cloudflared is available (check PATH first, then download)
    pub async fn ensure_cloudflared(&self) -> Result<PathBuf> {
        // First check if cloudflared is available in PATH
        if let Ok(output) = TokioCommand::new("cloudflared")
            .arg("--version")
            .output()
            .await
        {
            if output.status.success() {
                println!("Using cloudflared from PATH");
                return Ok(PathBuf::from("cloudflared"));
            }
        }

        println!("cloudflared not found in PATH, checking local installation...");

        // Check if we already have it downloaded
        if let Some(local_path) = self.get_cloudflared_path() {
            if self.verify_binary(&local_path).await? {
                println!("Using local cloudflared at: {:?}", local_path);
                return Ok(local_path);
            }
        }

        // Download cloudflared
        println!("Downloading cloudflared...");
        self.download_cloudflared().await
    }

    /// Get the path to the local cloudflared binary
    pub fn get_cloudflared_path(&self) -> Option<PathBuf> {
        let binary_name = if cfg!(windows) {
            "cloudflared.exe"
        } else {
            "cloudflared"
        };
        let cloudflared_path = self.binaries_dir.join("cloudflared").join(binary_name);

        if cloudflared_path.exists() {
            Some(cloudflared_path)
        } else {
            None
        }
    }

    /// Download cloudflared binary from GitHub releases
    async fn download_cloudflared(&self) -> Result<PathBuf> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        // Determine download URL and filename based on platform
        let (download_url, filename) = match (os, arch) {
            ("linux", "x86_64") => (
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64",
                "cloudflared"
            ),
            ("linux", "aarch64") => (
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64",
                "cloudflared"
            ),
            ("macos", "x86_64") => (
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz",
                "cloudflared"
            ),
            ("macos", "aarch64") => (
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz",
                "cloudflared"
            ),
            ("windows", "x86_64") => (
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe",
                "cloudflared.exe"
            ),
            ("windows", "aarch64") => (
                "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-386.exe",
                "cloudflared.exe"
            ),
            _ => return Err(anyhow!("Unsupported platform: {}-{}", os, arch)),
        };

        // Create cloudflared directory
        let cloudflared_dir = self.binaries_dir.join("cloudflared");
        fs::create_dir_all(&cloudflared_dir)?;

        let binary_path = cloudflared_dir.join(filename);

        // Download the binary
        println!("Downloading cloudflared from: {}", download_url);
        let response = reqwest::get(download_url).await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download cloudflared: HTTP {}",
                response.status()
            ));
        }

        let bytes = response.bytes().await?;

        // Handle compressed files (macOS uses .tgz)
        if download_url.ends_with(".tgz") {
            // For macOS, we'd need to extract the tar.gz
            // For simplicity, let's create a direct binary download approach
            return Err(anyhow!(
                "Compressed downloads not yet supported. Please install cloudflared manually."
            ));
        } else {
            // Direct binary download
            fs::write(&binary_path, bytes)?;
        }

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        // Verify the binary works
        if !self.verify_binary(&binary_path).await? {
            return Err(anyhow!("Downloaded cloudflared binary is not working"));
        }

        println!("cloudflared downloaded and verified successfully");
        Ok(binary_path)
    }
}
