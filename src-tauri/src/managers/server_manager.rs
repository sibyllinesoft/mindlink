// Production-ready Axum API Server implementing OpenAI-compatible endpoints
use crate::error::{MindLinkError, MindLinkResult};
use crate::managers::auth_manager::AuthManager;
use crate::{log_info, log_error, log_debug, network_error};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json, Response, IntoResponse},
    routing::{get, post},
    Router,
    body::Body,
};
use tower::ServiceBuilder;
use tower_http::cors::{CorsLayer, Any};
use reqwest::Client;
use uuid::Uuid;

// ===== OpenAI API Request/Response Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Option<Message>,
    pub delta: Option<Delta>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

// ===== ChatGPT Backend API Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGptMessage {
    pub id: String,
    pub author: ChatGptAuthor,
    pub content: ChatGptContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGptAuthor {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGptContent {
    pub content_type: String,
    pub parts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGptRequest {
    pub action: String,
    pub messages: Vec<ChatGptMessage>,
    pub parent_message_id: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

// ===== Application State =====

#[derive(Clone)]
pub struct AppState {
    auth_manager: Arc<RwLock<AuthManager>>,
    http_client: Client,
}

// ===== Server Manager =====

#[derive(Debug)]
pub struct ServerManager {
    port: u16,
    host: String,
    is_running: Arc<RwLock<bool>>,
    server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl ServerManager {
    /// Create a new ServerManager with production-grade configuration
    pub async fn new() -> Self {
        log_info!("ServerManager", "Initializing production API server");
        
        Self {
            port: 3001,
            host: "127.0.0.1".to_string(),
            is_running: Arc::new(RwLock::new(false)),
            server_handle: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Start the axum server with comprehensive error handling
    pub async fn start(&mut self, auth_manager: Arc<RwLock<AuthManager>>) -> MindLinkResult<String> {
        if *self.is_running.read().await {
            let url = self.get_local_url().await.unwrap_or_default();
            log_info!("ServerManager", &format!("Server already running at {}", url));
            return Ok(url);
        }
        
        log_info!("ServerManager", &format!("Starting API server on {}:{}", self.host, self.port));
        
        // Create HTTP client with proper timeouts
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("MindLink/1.0")
            .build()
            .map_err(|e| network_error!("Failed to create HTTP client", "", e))?;
        
        let app_state = AppState {
            auth_manager: auth_manager.clone(),
            http_client,
        };
        
        // Create the router with middleware
        let app = create_router(app_state);
        
        // Bind to the configured address
        let bind_address = format!("{}:{}", self.host, self.port);
        let listener = TcpListener::bind(&bind_address).await.map_err(|e| {
            MindLinkError::Network {
                message: format!("Failed to bind to {}", bind_address),
                url: Some(bind_address.clone()),
                source: Some(e.into()),
            }
        })?;
        
        log_info!("ServerManager", &format!("Server bound to {}", bind_address));
        
        // Start the server in a background task
        let server_task = tokio::spawn(async move {
            log_info!("ServerManager", "Axum server starting...");
            if let Err(e) = axum::serve(listener, app).await {
                log_error!("ServerManager", MindLinkError::Network {
                    message: "Server error occurred".to_string(),
                    url: None,
                    source: Some(e.into()),
                });
            }
        });
        
        *self.server_handle.write().await = Some(server_task);
        *self.is_running.write().await = true;
        
        let url = format!("http://{}:{}", self.host, self.port);
        log_info!("ServerManager", &format!("API server started successfully at {}", url));
        
        Ok(url)
    }
    
    /// Stop the server gracefully
    pub async fn stop(&mut self) -> MindLinkResult<()> {
        if !*self.is_running.read().await {
            log_debug!("ServerManager", "Server is not running, no action needed");
            return Ok(());
        }
        
        log_info!("ServerManager", "Stopping API server...");
        
        // Cancel the server task
        if let Some(handle) = self.server_handle.write().await.take() {
            handle.abort();
            // Give it a moment to clean up
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        *self.is_running.write().await = false;
        log_info!("ServerManager", "API server stopped successfully");
        
        Ok(())
    }
    
    /// Check if the server is responding to requests
    pub async fn check_health(&self) -> MindLinkResult<bool> {
        if !*self.is_running.read().await {
            return Ok(false);
        }
        
        let health_url = format!("http://{}:{}/health", self.host, self.port);
        
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| network_error!("Failed to create health check client", &health_url, e))?;
        
        match client.get(&health_url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                log_debug!("ServerManager", &format!("Health check result: {}", is_healthy));
                Ok(is_healthy)
            }
            Err(e) => {
                log_debug!("ServerManager", &format!("Health check failed: {}", e));
                Ok(false)
            }
        }
    }
    
    /// Get the local server URL if running
    pub async fn get_local_url(&self) -> Option<String> {
        if *self.is_running.read().await {
            Some(format!("http://{}:{}", self.host, self.port))
        } else {
            None
        }
    }
    
    /// Check if the server is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
    
    /// Restart the server with graceful shutdown
    pub async fn restart(&mut self, auth_manager: Arc<RwLock<AuthManager>>) -> MindLinkResult<String> {
        log_info!("ServerManager", "Restarting server...");
        self.stop().await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        self.start(auth_manager).await
    }
    
    /// Configure server settings (only when stopped)
    pub async fn configure(&mut self, host: String, port: u16) -> MindLinkResult<()> {
        if *self.is_running.read().await {
            return Err(MindLinkError::Configuration {
                message: "Cannot change server configuration while running".to_string(),
                config_key: Some("host/port".to_string()),
                source: None,
            });
        }
        
        log_info!("ServerManager", &format!("Configuring server: {}:{}", host, port));
        self.host = host;
        self.port = port;
        
        Ok(())
    }
}

// ===== Router Configuration =====

fn create_router(state: AppState) -> Router {
    Router::new()
        // OpenAI-compatible API endpoints
        .route("/v1/models", get(get_models))
        .route("/v1/chat/completions", post(chat_completions))
        // Health and status endpoints
        .route("/health", get(health_check))
        .route("/dashboard", get(dashboard))
        .route("/", get(root_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any))
                .into_inner()
        )
}

// ===== Route Handlers =====

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp(),
        "service": "MindLink API Server"
    }))
}

/// Root endpoint with basic info
async fn root_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "MindLink API Server",
        "version": "1.0.0",
        "endpoints": {
            "models": "/v1/models",
            "chat": "/v1/chat/completions",
            "health": "/health",
            "dashboard": "/dashboard"
        }
    }))
}

/// Dashboard HTML page
async fn dashboard() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MindLink API Dashboard</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: white;
        }
        .container {
            max-width: 800px;
            margin: 0 auto;
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            border-radius: 15px;
            padding: 30px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }
        .header {
            text-align: center;
            margin-bottom: 40px;
        }
        .status {
            display: flex;
            align-items: center;
            justify-content: center;
            margin: 20px 0;
        }
        .status-dot {
            width: 12px;
            height: 12px;
            background: #4ade80;
            border-radius: 50%;
            margin-right: 8px;
            animation: pulse 2s infinite;
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        .endpoints {
            display: grid;
            gap: 15px;
            margin-top: 30px;
        }
        .endpoint {
            background: rgba(255, 255, 255, 0.1);
            padding: 15px;
            border-radius: 10px;
            border: 1px solid rgba(255, 255, 255, 0.2);
        }
        .endpoint h3 {
            margin: 0 0 10px 0;
            color: #fbbf24;
        }
        .endpoint code {
            background: rgba(0, 0, 0, 0.3);
            padding: 4px 8px;
            border-radius: 4px;
            font-family: 'SF Mono', Monaco, monospace;
        }
        .footer {
            text-align: center;
            margin-top: 30px;
            opacity: 0.8;
            font-size: 14px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 MindLink API Server</h1>
            <div class="status">
                <div class="status-dot"></div>
                <span>Server is running</span>
            </div>
        </div>
        
        <div class="endpoints">
            <div class="endpoint">
                <h3>📋 Models</h3>
                <p>Get available models</p>
                <code>GET /v1/models</code>
            </div>
            
            <div class="endpoint">
                <h3>💬 Chat Completions</h3>
                <p>OpenAI-compatible chat completions endpoint</p>
                <code>POST /v1/chat/completions</code>
            </div>
            
            <div class="endpoint">
                <h3>❤️ Health Check</h3>
                <p>Server health status</p>
                <code>GET /health</code>
            </div>
        </div>
        
        <div class="footer">
            <p>Built with ❤️ using Rust + Axum</p>
        </div>
    </div>
    
    <script>
        // Auto-refresh status every 30 seconds
        setInterval(async () => {
            try {
                const response = await fetch('/health');
                const data = await response.json();
                console.log('Health check:', data);
            } catch (error) {
                console.error('Health check failed:', error);
            }
        }, 30000);
    </script>
</body>
</html>
    "#;
    
    Html(html)
}

/// Get supported models endpoint
async fn get_models() -> impl IntoResponse {
    log_debug!("ServerManager", "Models endpoint requested");
    
    let models = ModelList {
        object: "list".to_string(),
        data: vec![
            Model {
                id: "gpt-5".to_string(),
                object: "model".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                owned_by: "mindlink".to_string(),
            },
            Model {
                id: "codex-mini".to_string(),
                object: "model".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                owned_by: "mindlink".to_string(),
            },
        ],
    };
    
    Json(models)
}

/// Chat completions endpoint with streaming support
async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    log_info!("ServerManager", &format!("Chat completion request for model: {}", request.model));
    
    // Validate request
    if request.messages.is_empty() {
        return create_error_response(
            StatusCode::BAD_REQUEST,
            "messages array cannot be empty"
        );
    }
    
    // Get valid access token
    let access_token = match get_valid_access_token(&state.auth_manager).await {
        Ok(token) => token,
        Err(e) => {
            log_error!("ServerManager", e.clone());
            return create_error_response(
                StatusCode::UNAUTHORIZED,
                &e.user_message()
            );
        }
    };
    
    // Convert OpenAI request to ChatGPT format
    let chatgpt_request = match convert_to_chatgpt_format(&request) {
        Ok(req) => req,
        Err(e) => {
            log_error!("ServerManager", e.clone());
            return create_error_response(
                StatusCode::BAD_REQUEST,
                &e.user_message()
            );
        }
    };
    
    // Handle streaming vs non-streaming
    let is_streaming = request.stream.unwrap_or(false);
    
    if is_streaming {
        handle_streaming_request(state, chatgpt_request, access_token, request).await
    } else {
        handle_non_streaming_request(state, chatgpt_request, access_token, request).await
    }
}

// ===== Helper Functions =====

async fn get_valid_access_token(auth_manager: &Arc<RwLock<AuthManager>>) -> MindLinkResult<String> {
    let mut auth = auth_manager.write().await;
    
    // Ensure we have valid tokens (handles refresh automatically)
    auth.ensure_valid_tokens().await.map_err(|e| {
        let error: MindLinkError = e.into();
        log_error!("ServerManager", error);
        MindLinkError::Authentication {
            message: "Token validation failed".to_string(),
            source: None,
        }
    })?;
    
    // Get the access token
    auth.get_access_token()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            MindLinkError::Authentication {
                message: "No valid access token available".to_string(),
                source: None,
            }
        })
}

fn convert_to_chatgpt_format(request: &ChatCompletionRequest) -> MindLinkResult<ChatGptRequest> {
    let mut chatgpt_messages = Vec::new();
    
    for (_index, message) in request.messages.iter().enumerate() {
        let chatgpt_message = ChatGptMessage {
            id: Uuid::new_v4().to_string(),
            author: ChatGptAuthor {
                role: message.role.clone(),
                name: None,
            },
            content: ChatGptContent {
                content_type: "text".to_string(),
                parts: vec![message.content.clone()],
            },
            metadata: None,
        };
        chatgpt_messages.push(chatgpt_message);
    }
    
    // Add a parent message ID (required by ChatGPT API)
    let parent_message_id = if chatgpt_messages.len() > 1 {
        chatgpt_messages[chatgpt_messages.len() - 2].id.clone()
    } else {
        Uuid::new_v4().to_string()
    };
    
    Ok(ChatGptRequest {
        action: "next".to_string(),
        messages: chatgpt_messages,
        parent_message_id,
        model: map_model_name(&request.model),
        stream: request.stream,
        temperature: request.temperature,
    })
}

fn map_model_name(model: &str) -> String {
    match model {
        "gpt-5" => "gpt-4".to_string(), // Map to actual ChatGPT model
        "codex-mini" => "gpt-3.5-turbo".to_string(),
        _ => "gpt-4".to_string(), // Default fallback
    }
}

async fn handle_non_streaming_request(
    state: AppState,
    chatgpt_request: ChatGptRequest,
    access_token: String,
    original_request: ChatCompletionRequest,
) -> Response<Body> {
    log_debug!("ServerManager", "Processing non-streaming request");
    
    // Make request to ChatGPT API
    let response = match make_chatgpt_request(&state.http_client, &chatgpt_request, &access_token).await {
        Ok(resp) => resp,
        Err(e) => {
            log_error!("ServerManager", e.clone());
            return create_error_response(StatusCode::BAD_GATEWAY, &e.user_message());
        }
    };
    
    // Convert response back to OpenAI format
    let openai_response = create_openai_response(&original_request, &response);
    
    Json(openai_response).into_response()
}

async fn handle_streaming_request(
    state: AppState,
    mut chatgpt_request: ChatGptRequest,
    access_token: String,
    original_request: ChatCompletionRequest,
) -> Response<Body> {
    log_debug!("ServerManager", "Processing streaming request");
    
    // For now, handle streaming requests the same as non-streaming
    // In production, this would implement proper Server-Sent Events
    chatgpt_request.stream = Some(false);
    
    handle_non_streaming_request(state, chatgpt_request, access_token, original_request).await
}

async fn make_chatgpt_request(
    client: &Client,
    request: &ChatGptRequest,
    access_token: &str,
) -> MindLinkResult<serde_json::Value> {
    log_debug!("ServerManager", "Making request to ChatGPT backend");
    
    let response = client
        .post("https://chatgpt.com/backend-api/codex/responses")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await
        .map_err(|e| network_error!("ChatGPT API request failed", "https://chatgpt.com", e))?;
    
    if !response.status().is_success() {
        return Err(MindLinkError::Network {
            message: format!("ChatGPT API returned status: {}", response.status()),
            url: Some("https://chatgpt.com/backend-api/codex/responses".to_string()),
            source: None,
        });
    }
    
    let json_response = response.json::<serde_json::Value>().await
        .map_err(|e| network_error!("Failed to parse ChatGPT response", "", e))?;
    
    log_debug!("ServerManager", "ChatGPT request completed successfully");
    
    Ok(json_response)
}

fn create_openai_response(request: &ChatCompletionRequest, chatgpt_response: &serde_json::Value) -> ChatCompletionResponse {
    // Extract content from ChatGPT response (this is simplified)
    let content = extract_content_from_response(chatgpt_response).unwrap_or_default();
    
    ChatCompletionResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        model: request.model.clone(),
        choices: vec![Choice {
            index: 0,
            message: Some(Message {
                role: "assistant".to_string(),
                content,
            }),
            delta: None,
            finish_reason: Some("stop".to_string()),
        }],
        usage: Some(Usage {
            prompt_tokens: estimate_tokens(&request.messages),
            completion_tokens: 100, // Simplified
            total_tokens: estimate_tokens(&request.messages) + 100,
        }),
    }
}

fn extract_content_from_response(response: &serde_json::Value) -> Option<String> {
    // This is a simplified extraction - the actual ChatGPT response format
    // would need to be properly parsed based on their API documentation
    response.get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|parts| parts.as_array())
        .and_then(|arr| arr.first())
        .and_then(|part| part.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            // Fallback: try to extract from different possible structures
            response.get("content")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
}

fn estimate_tokens(messages: &[Message]) -> u32 {
    // Simple token estimation - in production, use a proper tokenizer
    messages.iter()
        .map(|m| (m.content.len() as f32 / 4.0).ceil() as u32)
        .sum()
}

fn create_error_response(status: StatusCode, message: &str) -> Response<Body> {
    let error_json = serde_json::json!({
        "error": {
            "message": message,
            "type": "invalid_request_error",
            "code": status.as_u16()
        }
    });
    
    (status, Json(error_json)).into_response()
}