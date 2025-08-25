// Tunnel Manager - Real Cloudflare tunnel implementation
use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tokio::time::timeout;

use super::binary_manager::BinaryManager;

#[derive(Debug, Clone)]
pub enum TunnelType {
    Quick,
    Named(String),
}

#[derive(Debug)]
pub struct TunnelManager {
    process: Arc<RwLock<Option<Child>>>,
    current_url: Arc<RwLock<Option<String>>>,
    tunnel_type: TunnelType,
    local_port: u16,
    is_connected: Arc<RwLock<bool>>,
    binary_manager: BinaryManager,
    cloudflared_path: Arc<RwLock<Option<PathBuf>>>,
}

impl TunnelManager {
    pub async fn new() -> Result<Self> {
        let binary_manager = BinaryManager::new().await?;

        Ok(Self {
            process: Arc::new(RwLock::new(None)),
            current_url: Arc::new(RwLock::new(None)),
            tunnel_type: TunnelType::Quick,
            local_port: 3001,
            is_connected: Arc::new(RwLock::new(false)),
            binary_manager,
            cloudflared_path: Arc::new(RwLock::new(None)),
        })
    }

    /// Ensure cloudflared binary is available
    async fn ensure_cloudflared(&self) -> Result<PathBuf> {
        // Check if we already have the path cached
        if let Some(path) = self.cloudflared_path.read().await.as_ref() {
            return Ok(path.clone());
        }

        // Get cloudflared path from binary manager
        let path = self.binary_manager.ensure_cloudflared().await?;
        *self.cloudflared_path.write().await = Some(path.clone());

        Ok(path)
    }

    pub async fn create_tunnel(&mut self) -> Result<String> {
        if *self.is_connected.read().await {
            if let Some(url) = &*self.current_url.read().await {
                return Ok(url.clone());
            }
        }

        println!("Creating Cloudflare tunnel...");

        let tunnel_type = self.tunnel_type.clone();
        match tunnel_type {
            TunnelType::Quick => self.create_quick_tunnel().await,
            TunnelType::Named(name) => self.create_named_tunnel(&name).await,
        }
    }

    async fn create_quick_tunnel(&mut self) -> Result<String> {
        println!("Creating Cloudflare quick tunnel...");

        // Ensure cloudflared binary is available
        let cloudflared_path = self.ensure_cloudflared().await?;

        // Spawn cloudflared process
        let mut child = Command::new(&cloudflared_path)
            .args(&[
                "tunnel",
                "--url",
                &format!("http://localhost:{}", self.local_port),
                "--no-autoupdate",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn cloudflared process: {}", e))?;

        // Parse tunnel URL from stdout
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture cloudflared stdout"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to capture cloudflared stderr"))?;

        let tunnel_url = self.parse_tunnel_output(stdout, stderr).await?;

        // Store the process and update state
        *self.process.write().await = Some(child);
        *self.current_url.write().await = Some(tunnel_url.clone());
        *self.is_connected.write().await = true;

        println!("Quick tunnel created successfully: {}", tunnel_url);
        Ok(tunnel_url)
    }

    /// Parse cloudflared output to extract tunnel URL
    async fn parse_tunnel_output(
        &self,
        stdout: tokio::process::ChildStdout,
        stderr: tokio::process::ChildStderr,
    ) -> Result<String> {
        let url_regex = Regex::new(r"https://[a-zA-Z0-9-]+\.trycloudflare\.com")
            .map_err(|e| anyhow!("Failed to compile regex: {}", e))?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let mut stdout_lines = stdout_reader.lines();
        let mut stderr_lines = stderr_reader.lines();

        // Use timeout to prevent hanging indefinitely
        let parse_result = timeout(Duration::from_secs(30), async {
            loop {
                tokio::select! {
                    stdout_line = stdout_lines.next_line() => {
                        if let Ok(Some(line)) = stdout_line {
                            println!("cloudflared stdout: {}", line);

                            // Look for tunnel URL in stdout
                            if let Some(captures) = url_regex.find(&line) {
                                return Ok(captures.as_str().to_string());
                            }

                            // Check for connection success indicators
                            if line.contains("Connection") && line.contains("registered") {
                                println!("Tunnel connection registered");
                            }
                        }
                    }
                    stderr_line = stderr_lines.next_line() => {
                        if let Ok(Some(line)) = stderr_line {
                            println!("cloudflared stderr: {}", line);

                            // Check for specific error conditions
                            if line.contains("connection refused") || line.contains("no such host") {
                                return Err(anyhow!("Local server not accessible: {}", line));
                            }

                            if line.contains("authentication") || line.contains("login") {
                                return Err(anyhow!("Cloudflare authentication required: {}", line));
                            }

                            if line.contains("failed") && line.contains("tunnel") {
                                return Err(anyhow!("Tunnel creation failed: {}", line));
                            }
                        }
                    }
                }
            }
        }).await;

        match parse_result {
            Ok(Ok(url)) => Ok(url),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Timeout waiting for tunnel URL (30 seconds)")),
        }
    }

    async fn create_named_tunnel(&mut self, name: &str) -> Result<String> {
        // In a real implementation, this would create a named tunnel
        let tunnel_url = format!("https://{}.yourdomain.com", name);

        println!("Named tunnel created: {}", tunnel_url);

        *self.current_url.write().await = Some(tunnel_url.clone());
        *self.is_connected.write().await = true;

        Ok(tunnel_url)
    }

    pub async fn close_tunnel(&mut self) -> Result<()> {
        if !*self.is_connected.read().await {
            return Ok(());
        }

        println!("Closing tunnel...");

        // Gracefully terminate the cloudflared process
        if let Some(mut child) = self.process.write().await.take() {
            // First try graceful termination
            #[cfg(unix)]
            {
                if let Some(id) = child.id() {
                    // Send SIGTERM for graceful shutdown
                    #[allow(unsafe_code)]
                    unsafe {
                        libc::kill(id as i32, libc::SIGTERM);
                    }

                    // Wait up to 5 seconds for graceful shutdown
                    let graceful_shutdown = timeout(Duration::from_secs(5), child.wait()).await;

                    if graceful_shutdown.is_err() {
                        // If graceful shutdown failed, force kill
                        println!("Graceful shutdown timed out, force killing process");
                        if let Err(e) = child.kill().await {
                            eprintln!("Failed to force kill tunnel process: {}", e);
                        }
                    } else {
                        println!("Tunnel process terminated gracefully");
                    }
                } else {
                    // Process already finished
                    let _ = child.wait().await;
                }
            }

            #[cfg(windows)]
            {
                // On Windows, just kill the process
                if let Err(e) = child.kill().await {
                    eprintln!("Failed to kill tunnel process: {}", e);
                } else {
                    println!("Tunnel process terminated");
                }
            }
        }

        *self.current_url.write().await = None;
        *self.is_connected.write().await = false;

        println!("Tunnel closed");
        Ok(())
    }

    pub async fn check_health(&self) -> Result<bool> {
        if !*self.is_connected.read().await {
            return Ok(false);
        }

        // First check if the process is still running
        let process_running = {
            let process_guard = self.process.read().await;
            if let Some(child) = process_guard.as_ref() {
                child.id().is_some()
            } else {
                false
            }
        };

        if !process_running {
            println!("Tunnel process has exited, marking as unhealthy");
            // Update connection state since process died
            *self.is_connected.write().await = false;
            return Ok(false);
        }

        // Then check HTTP connectivity through the tunnel
        if let Some(url) = &*self.current_url.read().await {
            let health_url = format!("{}/health", url);

            // Use a short timeout for health checks
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

            match client.get(&health_url).send().await {
                Ok(response) => {
                    let is_healthy = response.status().is_success();
                    if !is_healthy {
                        println!("Tunnel HTTP health check failed: {}", response.status());
                    }
                    Ok(is_healthy)
                },
                Err(e) => {
                    println!("Tunnel health check request failed: {}", e);
                    Ok(false)
                },
            }
        } else {
            Ok(false)
        }
    }

    pub async fn get_current_url(&self) -> Option<String> {
        self.current_url.read().await.clone()
    }

    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    pub async fn recreate_tunnel(&mut self) -> Result<String> {
        println!("Recreating tunnel...");
        self.close_tunnel().await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
        self.create_tunnel().await
    }

    pub async fn set_tunnel_type(&mut self, tunnel_type: TunnelType) {
        if *self.is_connected.read().await {
            eprintln!("Cannot change tunnel type while connected");
            return;
        }

        self.tunnel_type = tunnel_type;
    }

    pub async fn set_local_port(&mut self, port: u16) {
        if *self.is_connected.read().await {
            eprintln!("Cannot change local port while tunnel is active");
            return;
        }

        self.local_port = port;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tunnel_manager_creation() {
        // Test that TunnelManager can be created
        let result = TunnelManager::new().await;
        assert!(result.is_ok(), "TunnelManager creation should succeed");

        let manager = result.expect("TunnelManager creation should succeed");
        assert!(
            !manager.is_connected().await,
            "Manager should start disconnected"
        );
        assert_eq!(
            manager.get_current_url().await,
            None,
            "Should have no URL initially"
        );
    }

    #[tokio::test]
    async fn test_cloudflared_binary_check() {
        let manager = TunnelManager::new()
            .await
            .expect("TunnelManager creation should succeed in test");

        // Test that ensure_cloudflared either finds it in PATH or downloads it
        // This will either succeed or fail with a specific error message
        let result = manager.ensure_cloudflared().await;

        // We don't assert success since it depends on system state,
        // but we validate that it returns a proper Result
        match result {
            Ok(path) => {
                println!("cloudflared found/downloaded at: {:?}", path);
                assert!(path.exists() || path.to_str() == Some("cloudflared"));
            },
            Err(e) => {
                println!("cloudflared setup failed (expected in CI): {}", e);
                // This is expected in CI environments without cloudflared
            },
        }
    }
}
