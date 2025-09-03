//! E2E Test Configuration and Utilities
//!
//! This module provides configuration and utility functions for running
//! end-to-end tests with proper setup and teardown.

use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for E2E test environment
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    /// Maximum time to wait for application operations
    pub app_timeout: Duration,
    /// Delay to wait for server startup
    pub server_startup_delay: Duration,
    /// Base URL for API testing
    pub api_base_url: String,
    /// Test token for authentication
    pub test_user_token: String,
    /// Maximum number of retries for operations
    pub max_retries: usize,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            app_timeout: Duration::from_secs(30),
            server_startup_delay: Duration::from_secs(10),
            api_base_url: "http://127.0.0.1:3001".to_string(),
            test_user_token: "test-token-e2e".to_string(),
            max_retries: 3,
        }
    }
}

/// Test environment manager for E2E tests
#[derive(Debug)]
pub struct E2ETestEnvironment {
    /// Configuration for the test environment
    pub config: E2ETestConfig,
    /// Handle to the running application process
    app_process: Option<std::process::Child>,
}

impl E2ETestEnvironment {
    /// Create a new test environment with default config
    pub fn new() -> Self {
        Self {
            config: E2ETestConfig::default(),
            app_process: None,
        }
    }

    /// Create a new test environment with custom config
    pub fn with_config(config: E2ETestConfig) -> Self {
        Self {
            config,
            app_process: None,
        }
    }

    /// Start the MindLink application for testing
    pub async fn start_app(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting MindLink application for E2E testing...");

        // Build the application first
        let build_output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir("../") // Go up one level to src-tauri directory
            .output()
            .expect("Failed to build application");

        if !build_output.status.success() {
            let error_msg = String::from_utf8_lossy(&build_output.stderr);
            return Err(format!("Failed to build application: {}", error_msg).into());
        }

        println!("✅ Application built successfully");

        // Start the application
        let mut app = Command::new("cargo")
            .args(["run", "--release"])
            .current_dir("../")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start application");

        println!("⏳ Waiting for application startup...");
        sleep(self.config.server_startup_delay).await;

        // Check if process is still running
        match app.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("Application exited early with status: {}", status).into());
            },
            Ok(None) => {
                println!("✅ Application started successfully");
                self.app_process = Some(app);
            },
            Err(e) => {
                return Err(format!("Failed to check application status: {}", e).into());
            },
        }

        Ok(())
    }

    /// Stop the MindLink application
    pub fn stop_app(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut process) = self.app_process.take() {
            println!("🛑 Stopping MindLink application...");

            // Try graceful shutdown first
            process.kill()?;

            // Wait for process to exit
            match process.wait() {
                Ok(_) => println!("✅ Application stopped successfully"),
                Err(e) => println!("⚠️  Warning: {}", e),
            }
        }

        Ok(())
    }

    /// Check if the API server is responding
    pub async fn wait_for_api(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("⏳ Waiting for API server to respond...");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        for attempt in 1..=self.config.max_retries {
            println!(
                "🔍 API health check attempt {} of {}",
                attempt, self.config.max_retries
            );

            match client
                .get(&format!("{}/health", self.config.api_base_url))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("✅ API server is responding");
                        return Ok(());
                    }
                    println!("⚠️  API returned status: {}", response.status());
                },
                Err(e) => {
                    println!("⚠️  API check failed: {}", e);
                },
            }

            if attempt < self.config.max_retries {
                sleep(Duration::from_secs(2)).await;
            }
        }

        Err("API server did not respond after maximum retries".into())
    }

    /// Perform a complete health check of the test environment
    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Performing E2E environment health check...");

        // Check if application process is running
        if let Some(ref _process) = &self.app_process {
            // Note: We can't easily check if process is running without mutable access
            println!("✅ Application process exists");
        } else {
            return Err("Application process not started".into());
        }

        // Check API availability
        self.wait_for_api().await?;

        println!("✅ E2E environment health check passed");
        Ok(())
    }
}

impl Drop for E2ETestEnvironment {
    fn drop(&mut self) {
        if let Err(e) = self.stop_app() {
            eprintln!("⚠️  Failed to stop application in Drop: {}", e);
        }
    }
}

/// Utility functions for E2E tests
pub mod test_utils {
    use super::*;
    use serde_json::{json, Value};
    use uuid::Uuid;

    /// Generate a test request ID for tracing
    pub fn generate_test_id() -> String {
        format!("e2e-test-{}", Uuid::new_v4())
    }

    /// Create a standard test chat completion request
    pub fn create_test_chat_request(message: &str, stream: bool) -> Value {
        json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a test assistant. Keep responses very brief."
                },
                {
                    "role": "user",
                    "content": message
                }
            ],
            "max_tokens": 50,
            "temperature": 0.1,
            "stream": stream
        })
    }

    /// Create test headers for API requests
    pub fn create_test_headers(config: &E2ETestConfig) -> Vec<(&'static str, String)> {
        vec![
            ("Content-Type", "application/json".to_string()),
            (
                "Authorization",
                format!("Bearer {}", config.test_user_token),
            ),
            ("User-Agent", "MindLink-E2E-Tests/1.0".to_string()),
        ]
    }

    /// Validate OpenAI API response structure
    pub fn validate_openai_response(response: &Value) -> Result<(), String> {
        // Check required fields
        if response.get("id").is_none() {
            return Err("Missing 'id' field".to_string());
        }

        if response.get("object").is_none() {
            return Err("Missing 'object' field".to_string());
        }

        if response.get("choices").is_none() {
            return Err("Missing 'choices' field".to_string());
        }

        // Validate choices array
        if let Some(choices) = response.get("choices") {
            if !choices.is_array() {
                return Err("'choices' should be an array".to_string());
            }

            let choices_array = choices.as_array().unwrap();
            if choices_array.is_empty() {
                return Err("'choices' array should not be empty".to_string());
            }

            // Check first choice structure
            if let Some(first_choice) = choices_array.first() {
                if first_choice.get("message").is_none() && first_choice.get("delta").is_none() {
                    return Err("Choice should have 'message' or 'delta' field".to_string());
                }
            }
        }

        Ok(())
    }

    /// Wait for a condition with timeout
    pub async fn wait_for_condition<F, Fut>(
        mut condition: F,
        timeout: Duration,
        check_interval: Duration,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if condition().await {
                return Ok(());
            }

            sleep(check_interval).await;
        }

        Err("Condition not met within timeout".into())
    }

    /// Log test step with formatting
    pub fn log_test_step(step: usize, description: &str) {
        println!("📋 Step {}: {}", step, description);
    }

    /// Log test result
    pub fn log_test_result(test_name: &str, success: bool, details: Option<&str>) {
        let emoji = if success { "✅" } else { "❌" };
        let status = if success { "PASS" } else { "FAIL" };

        if let Some(details) = details {
            println!("{} {} {}: {}", emoji, test_name, status, details);
        } else {
            println!("{} {} {}", emoji, test_name, status);
        }
    }
}
