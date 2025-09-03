// Process monitoring system for child processes
#![allow(dead_code)]
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{timeout, Duration};

use crate::error::{MindLinkError, MindLinkResult};
use crate::logging::get_logger;

/// Information about a monitored process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub pid: Option<u32>,
    #[allow(dead_code)]
    pub start_time: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub status: ProcessStatus,
    #[allow(dead_code)]
    pub restart_count: u32,
    #[allow(dead_code)]
    pub last_restart: Option<chrono::DateTime<chrono::Utc>>,
}

/// Status of a monitored process
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopped,
    Failed,
    Crashed,
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Starting => write!(f, "starting"),
            ProcessStatus::Running => write!(f, "running"),
            ProcessStatus::Stopped => write!(f, "stopped"),
            ProcessStatus::Failed => write!(f, "failed"),
            ProcessStatus::Crashed => write!(f, "crashed"),
        }
    }
}

/// Configuration for process monitoring
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    #[allow(dead_code)]
    pub capture_stdout: bool,
    #[allow(dead_code)]
    pub capture_stderr: bool,
    #[allow(dead_code)]
    pub max_restart_attempts: u32,
    #[allow(dead_code)]
    pub restart_delay: Duration,
    #[allow(dead_code)]
    pub output_buffer_size: usize,
    #[allow(dead_code)]
    pub health_check_interval: Duration,
    #[allow(dead_code)]
    pub process_timeout: Option<Duration>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            capture_stdout: true,
            capture_stderr: true,
            max_restart_attempts: 3,
            restart_delay: Duration::from_secs(5),
            output_buffer_size: 1024 * 1024, // 1MB
            health_check_interval: Duration::from_secs(30),
            process_timeout: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

/// Events that can be sent from the process monitor
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ProcessEvent {
    Started {
        process_id: String,
        pid: u32,
    },
    Stopped {
        process_id: String,
        exit_code: Option<i32>,
    },
    Crashed {
        process_id: String,
        error: String,
    },
    OutputReceived {
        process_id: String,
        output_type: String,
        content: String,
    },
    RestartAttempted {
        process_id: String,
        attempt: u32,
    },
    RestartLimitReached {
        process_id: String,
    },
    HealthCheckFailed {
        process_id: String,
        error: String,
    },
}

/// Process monitor that manages and monitors child processes
#[allow(dead_code)]
pub struct ProcessMonitor {
    processes: Arc<RwLock<HashMap<String, ProcessInfo>>>,
    child_handles: Arc<RwLock<HashMap<String, Child>>>,
    event_sender: mpsc::UnboundedSender<ProcessEvent>,
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ProcessEvent>>>>,
    configs: Arc<RwLock<HashMap<String, MonitorConfig>>>,
}

impl ProcessMonitor {
    /// Create a new process monitor
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            child_handles: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a process for monitoring
    pub async fn register_process(
        &self,
        process_id: String,
        process_name: String,
        config: MonitorConfig,
    ) -> MindLinkResult<()> {
        let mut processes = self.processes.write().await;
        let mut configs = self.configs.write().await;

        let process_info = ProcessInfo {
            name: process_name,
            pid: None,
            start_time: chrono::Utc::now(),
            status: ProcessStatus::Stopped,
            restart_count: 0,
            last_restart: None,
        };

        processes.insert(process_id.clone(), process_info);
        configs.insert(process_id, config);

        Ok(())
    }

    /// Start monitoring a child process
    pub async fn start_monitoring(
        &self,
        process_id: String,
        mut child: Child,
    ) -> MindLinkResult<()> {
        // Get the PID before taking stdout/stderr
        let pid = child.id();

        // Update process info
        {
            let mut processes = self.processes.write().await;
            if let Some(info) = processes.get_mut(&process_id) {
                info.pid = pid;
                info.status = ProcessStatus::Starting;
            } else {
                return Err(MindLinkError::ProcessMonitoring {
                    message: "Process not registered".to_string(),
                    process_name: process_id.clone(),
                    pid,
                    source: None,
                });
            }
        }

        // Get the monitoring configuration
        let config = {
            let configs = self.configs.read().await;
            configs.get(&process_id).cloned().unwrap_or_default()
        };

        // Set up output monitoring
        if config.capture_stdout {
            if let Some(stdout) = child.stdout.take() {
                self.monitor_output(
                    process_id.clone(),
                    "stdout".to_string(),
                    stdout,
                    config.output_buffer_size,
                )
                .await;
            }
        }

        if config.capture_stderr {
            if let Some(stderr) = child.stderr.take() {
                self.monitor_output(
                    process_id.clone(),
                    "stderr".to_string(),
                    stderr,
                    config.output_buffer_size,
                )
                .await;
            }
        }

        // Store the child process
        {
            let mut handles = self.child_handles.write().await;
            handles.insert(process_id.clone(), child);
        }

        // Send started event
        if let Some(pid) = pid {
            let _ = self.event_sender.send(ProcessEvent::Started {
                process_id: process_id.clone(),
                pid,
            });
        }

        // Update status to running
        {
            let mut processes = self.processes.write().await;
            if let Some(info) = processes.get_mut(&process_id) {
                info.status = ProcessStatus::Running;
            }
        }

        // Start process monitoring task
        self.start_process_monitoring_task(process_id.clone()).await;

        Ok(())
    }

    /// Monitor output from a process
    async fn monitor_output<T>(
        &self,
        process_id: String,
        output_type: String,
        stream: T,
        buffer_size: usize,
    ) where
        T: tokio::io::AsyncRead + Unpin + Send + 'static,
    {
        let event_sender = self.event_sender.clone();
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                // Check length before processing to prevent memory issues
                if line.len() > buffer_size {
                    break;
                }

                // Log to file through our logging system
                if let Some(logger) = get_logger() {
                    logger.log_process_output(&process_id, &output_type, &line, None);
                }

                // Send event for real-time monitoring
                let _ = event_sender.send(ProcessEvent::OutputReceived {
                    process_id: process_id.clone(),
                    output_type: output_type.clone(),
                    content: line,
                });
            }
        });
    }

    /// Start the process monitoring task
    async fn start_process_monitoring_task(&self, process_id: String) {
        let processes = self.processes.clone();
        let child_handles = self.child_handles.clone();
        let event_sender = self.event_sender.clone();
        let configs = self.configs.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                let config = {
                    let configs_guard = configs.read().await;
                    configs_guard.get(&process_id).cloned().unwrap_or_default()
                };

                // Check if the process is still running
                let process_exited = {
                    let mut handles = child_handles.write().await;
                    if let Some(child) = handles.get_mut(&process_id) {
                        match child.try_wait() {
                            Ok(Some(exit_status)) => {
                                // Process has exited
                                let _ = event_sender.send(ProcessEvent::Stopped {
                                    process_id: process_id.clone(),
                                    exit_code: exit_status.code(),
                                });

                                // Update process status
                                {
                                    let mut processes_guard = processes.write().await;
                                    if let Some(info) = processes_guard.get_mut(&process_id) {
                                        info.status = if exit_status.success() {
                                            ProcessStatus::Stopped
                                        } else {
                                            ProcessStatus::Crashed
                                        };
                                        info.pid = None;
                                    }
                                }

                                handles.remove(&process_id);
                                true
                            },
                            Ok(None) => {
                                // Process is still running
                                false
                            },
                            Err(e) => {
                                // Error checking process status
                                let _ = event_sender.send(ProcessEvent::Crashed {
                                    process_id: process_id.clone(),
                                    error: format!("Failed to check process status: {}", e),
                                });

                                // Update process status
                                {
                                    let mut processes_guard = processes.write().await;
                                    if let Some(info) = processes_guard.get_mut(&process_id) {
                                        info.status = ProcessStatus::Failed;
                                        info.pid = None;
                                    }
                                }

                                handles.remove(&process_id);
                                true
                            },
                        }
                    } else {
                        // No child handle found
                        true
                    }
                };

                if process_exited {
                    break;
                }

                // Sleep for the health check interval
                tokio::time::sleep(config.health_check_interval).await;
            }
        });
    }

    /// Stop monitoring a process
    pub async fn stop_process(&self, process_id: &str) -> MindLinkResult<()> {
        let mut handles = self.child_handles.write().await;

        if let Some(mut child) = handles.remove(process_id) {
            // Try graceful shutdown first
            #[cfg(unix)]
            {
                if let Some(pid) = child.id() {
                    // Send SIGTERM
                    #[allow(unsafe_code)]
                    unsafe {
                        libc::kill(pid as i32, libc::SIGTERM);
                    }

                    // Wait for graceful shutdown
                    let graceful_result = timeout(Duration::from_secs(10), child.wait()).await;

                    if graceful_result.is_err() {
                        // Force kill if graceful shutdown failed
                        let _ = child.kill().await;
                    }
                } else {
                    let _ = child.wait().await;
                }
            }

            #[cfg(windows)]
            {
                let _ = child.kill().await;
            }

            // Update process status
            {
                let mut processes = self.processes.write().await;
                if let Some(info) = processes.get_mut(process_id) {
                    info.status = ProcessStatus::Stopped;
                    info.pid = None;
                }
            }

            let _ = self.event_sender.send(ProcessEvent::Stopped {
                process_id: process_id.to_string(),
                exit_code: None,
            });
        }

        Ok(())
    }

    /// Get information about a monitored process
    pub async fn get_process_info(&self, process_id: &str) -> Option<ProcessInfo> {
        let processes = self.processes.read().await;
        processes.get(process_id).cloned()
    }

    /// Get information about all monitored processes
    pub async fn get_all_processes(&self) -> HashMap<String, ProcessInfo> {
        let processes = self.processes.read().await;
        processes.clone()
    }

    /// Check if a process is running
    pub async fn is_process_running(&self, process_id: &str) -> bool {
        let processes = self.processes.read().await;
        if let Some(info) = processes.get(process_id) {
            matches!(
                info.status,
                ProcessStatus::Running | ProcessStatus::Starting
            )
        } else {
            false
        }
    }

    /// Get the event receiver for process events
    pub async fn get_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<ProcessEvent>> {
        let mut receiver = self.event_receiver.write().await;
        receiver.take()
    }

    /// Restart a process with backoff
    pub async fn restart_process(&self, process_id: &str) -> MindLinkResult<()> {
        // Check restart limits
        let should_restart = {
            let mut processes = self.processes.write().await;
            if let Some(info) = processes.get_mut(process_id) {
                let config = {
                    let configs = self.configs.read().await;
                    configs.get(process_id).cloned().unwrap_or_default()
                };

                if info.restart_count >= config.max_restart_attempts {
                    let _ = self.event_sender.send(ProcessEvent::RestartLimitReached {
                        process_id: process_id.to_string(),
                    });
                    return Err(MindLinkError::ProcessMonitoring {
                        message: format!(
                            "Restart limit reached ({} attempts)",
                            config.max_restart_attempts
                        ),
                        process_name: process_id.to_string(),
                        pid: None,
                        source: None,
                    });
                }

                info.restart_count += 1;
                info.last_restart = Some(chrono::Utc::now());

                let _ = self.event_sender.send(ProcessEvent::RestartAttempted {
                    process_id: process_id.to_string(),
                    attempt: info.restart_count,
                });

                true
            } else {
                false
            }
        };

        if !should_restart {
            return Err(MindLinkError::ProcessMonitoring {
                message: "Process not found for restart".to_string(),
                process_name: process_id.to_string(),
                pid: None,
                source: None,
            });
        }

        // Stop the current process
        self.stop_process(process_id).await?;

        // Wait for restart delay
        let config = {
            let configs = self.configs.read().await;
            configs.get(process_id).cloned().unwrap_or_default()
        };
        tokio::time::sleep(config.restart_delay).await;

        // The actual restart would be handled by the specific manager
        // This just sets up the monitoring infrastructure

        Ok(())
    }

    /// Clean up resources for a process
    pub async fn unregister_process(&self, process_id: &str) -> MindLinkResult<()> {
        // Stop the process first
        let _ = self.stop_process(process_id).await;

        // Remove from tracking
        {
            let mut processes = self.processes.write().await;
            processes.remove(process_id);
        }

        {
            let mut configs = self.configs.write().await;
            configs.remove(process_id);
        }

        Ok(())
    }
}

/// Global process monitor instance
static mut PROCESS_MONITOR: Option<Arc<ProcessMonitor>> = None;
static PROCESS_MONITOR_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global process monitor
pub fn init_process_monitor() -> Arc<ProcessMonitor> {
    PROCESS_MONITOR_INIT.call_once(|| {
        #[allow(unsafe_code)]
        unsafe {
            PROCESS_MONITOR = Some(Arc::new(ProcessMonitor::new()));
        }
    });

    #[allow(unsafe_code)]
    unsafe {
        PROCESS_MONITOR
            .as_ref()
            .expect("ProcessMonitor should be initialized by call_once")
            .clone()
    }
}

/// Get the global process monitor
pub fn get_process_monitor() -> Option<Arc<ProcessMonitor>> {
    #[allow(unsafe_code)]
    unsafe {
        PROCESS_MONITOR.as_ref().cloned()
    }
}
