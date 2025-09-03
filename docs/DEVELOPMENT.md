# MindLink Development Guide

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Project Structure](#project-structure)
3. [Architecture Deep Dive](#architecture-deep-dive)
4. [Development Workflow](#development-workflow)
5. [Testing Strategy](#testing-strategy)
6. [Performance Optimization](#performance-optimization)
7. [Debugging and Profiling](#debugging-and-profiling)
8. [Build and Deployment](#build-and-deployment)
9. [Contributing Guidelines](#contributing-guidelines)

## Development Environment Setup

### Prerequisites

#### Required Tools

**1. Rust Toolchain**
```bash
# Install rustup (Rust version manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install latest stable Rust
rustup update stable
rustup default stable

# Verify installation
rustc --version
cargo --version
```

**2. Node.js and Package Manager**
```bash
# Using Node Version Manager (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Or download directly from https://nodejs.org/
node --version
npm --version
```

**3. Tauri CLI**
```bash
# Install Tauri CLI globally
npm install -g @tauri-apps/cli

# Verify installation
tauri --version
```

#### Platform-Specific Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt update && sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libsoup2.4-dev \
    libjavascriptcoregtk-4.0-dev
```

**Linux (Fedora/CentOS/RHEL):**
```bash
sudo dnf install -y \
    webkit2gtk3-devel \
    openssl-devel \
    curl \
    wget \
    file \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Windows:**
```powershell
# Install Visual Studio Build Tools or Visual Studio Community
# Download from: https://visualstudio.microsoft.com/downloads/

# Install WebView2 (usually pre-installed on Windows 11)
# Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

#### Development Tools

**Code Editor Setup (VS Code):**
```bash
# Install recommended extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension tauri-apps.tauri-vscode
code --install-extension vadimcn.vscode-lldb
code --install-extension ms-vscode.vscode-typescript-next
```

**Additional Rust Tools:**
```bash
# Code formatting
rustup component add rustfmt

# Linting
rustup component add clippy

# Coverage reporting
cargo install cargo-tarpaulin

# Security auditing  
cargo install cargo-audit

# Dependency checking
cargo install cargo-udeps

# Performance profiling
cargo install flamegraph
```

### Repository Setup

**1. Clone and Setup**
```bash
# Clone the repository
git clone https://github.com/yourusername/mindlink.git
cd mindlink

# Install Node.js dependencies
npm install

# Install Rust dependencies (automatic on first build)
cd src-tauri
cargo check
cd ..
```

**2. Environment Configuration**
```bash
# Copy development configuration
cp .env.example .env

# Set development environment variables
echo "RUST_LOG=debug" >> .env
echo "MINDLINK_ENV=development" >> .env
```

**3. Pre-commit Hooks (Optional but Recommended)**
```bash
# Install pre-commit framework
pip install pre-commit

# Install pre-commit hooks
pre-commit install

# Test hooks
pre-commit run --all-files
```

## Project Structure

### High-Level Overview

```
mindlink/
├── 📁 src-tauri/                    # Rust backend (main application)
│   ├── 📁 src/
│   │   ├── 📄 main.rs              # Application entry point and system tray
│   │   ├── 📁 managers/            # Business logic modules
│   │   │   ├── 📄 auth_manager.rs  # OAuth2 authentication with ChatGPT
│   │   │   ├── 📄 server_manager.rs # HTTP API server (Axum)
│   │   │   ├── 📄 tunnel_manager.rs # Cloudflare tunnel management
│   │   │   ├── 📄 config_manager.rs # Configuration and persistence  
│   │   │   ├── 📄 bifrost_manager.rs # Dashboard backend
│   │   │   └── 📄 mod.rs           # Manager module exports
│   │   ├── 📁 commands/            # Tauri IPC command handlers
│   │   │   └── 📄 mod.rs           # Command exports and routing
│   │   ├── 📁 tests/               # Comprehensive test suite
│   │   ├── 📄 error.rs            # Structured error handling
│   │   ├── 📄 logging.rs          # Logging infrastructure
│   │   └── 📄 dialog.rs           # UI dialog helpers
│   ├── 📄 Cargo.toml              # Rust dependencies and configuration
│   ├── 📄 tauri.conf.json         # Tauri application configuration
│   └── 📄 build.rs                # Build script for asset embedding
├── 📁 src/                          # Frontend source (TypeScript/React)
│   ├── 📄 main.tsx                 # React application entry
│   ├── 📄 App.tsx                  # Main application component
│   └── 📁 components/              # React components
├── 📁 public/                       # Static frontend assets
├── 📁 docs/                         # Documentation (this folder)
├── 📁 scripts/                      # Build and deployment automation
├── 📁 .github/workflows/            # CI/CD pipeline configuration
├── 📄 package.json                 # Node.js dependencies and scripts
├── 📄 vite.config.ts              # Frontend build configuration
├── 📄 tsconfig.json               # TypeScript configuration
└── 📄 README.md                   # Project overview and quick start
```

### Core Modules Deep Dive

#### Manager Pattern Architecture

Each manager is responsible for a specific domain and follows consistent patterns:

```rust
// Standard manager interface pattern
pub struct ManagerName {
    // Internal state
    state: Arc<RwLock<ManagerState>>,
    // Dependencies
    config_manager: Arc<ConfigManager>,
    // External clients
    http_client: Arc<HttpClient>,
}

impl ManagerName {
    // Constructor with dependency injection
    pub fn new(config_manager: Arc<ConfigManager>) -> Self { ... }
    
    // Public API methods
    pub async fn start(&mut self) -> Result<(), MindLinkError> { ... }
    pub async fn stop(&mut self) -> Result<(), MindLinkError> { ... }
    pub fn status(&self) -> ManagerStatus { ... }
    
    // Private implementation methods
    async fn internal_operation(&self) -> Result<(), MindLinkError> { ... }
}
```

#### AuthManager Implementation

**File**: `src-tauri/src/managers/auth_manager.rs`

```rust
pub struct AuthManager {
    /// OAuth2 client for ChatGPT authentication
    oauth_client: Arc<BasicClient>,
    /// Stored authentication tokens
    tokens: Arc<RwLock<Option<TokenSet>>>,
    /// PKCE verifier for OAuth flow
    pkce_verifier: Arc<RwLock<Option<PkceCodeVerifier>>>,
    /// Configuration manager for settings
    config_manager: Arc<ConfigManager>,
    /// HTTP client for API requests
    http_client: Arc<HttpClient>,
}

impl AuthManager {
    /// Initiate OAuth2 authentication flow
    pub async fn login(&self) -> Result<String, MindLinkError> {
        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        
        // Build authorization URL
        let (auth_url, _csrf_token) = self.oauth_client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge)
            .url();
            
        // Store verifier for later use
        *self.pkce_verifier.write().await = Some(pkce_verifier);
        
        Ok(auth_url.to_string())
    }
    
    /// Complete OAuth2 flow with authorization code
    pub async fn complete_login(&self, code: &str) -> Result<(), MindLinkError> {
        let pkce_verifier = self.pkce_verifier.read().await.clone()
            .ok_or(MindLinkError::AuthenticationError("No PKCE verifier found".into()))?;
            
        // Exchange code for tokens
        let token_response = self.oauth_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await?;
            
        // Store tokens securely
        let tokens = TokenSet {
            access_token: token_response.access_token().secret().clone(),
            refresh_token: token_response.refresh_token()
                .map(|t| t.secret().clone()),
            expires_at: Utc::now() + Duration::seconds(token_response.expires_in()
                .map(|d| d.as_secs() as i64).unwrap_or(3600)),
        };
        
        self.store_tokens(&tokens).await?;
        *self.tokens.write().await = Some(tokens);
        
        Ok(())
    }
    
    /// Refresh access token using refresh token
    pub async fn refresh_token(&self) -> Result<(), MindLinkError> {
        // Implementation for token refresh
        // Includes automatic retry logic and error handling
    }
}
```

#### ServerManager Implementation

**File**: `src-tauri/src/managers/server_manager.rs`

```rust
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Response, Json},
    routing::{get, post},
    Router,
};

pub struct ServerManager {
    /// Server handle for lifecycle management
    server_handle: Arc<RwLock<Option<ServerHandle>>>,
    /// Shared application state
    app_state: Arc<AppState>,
    /// Configuration manager
    config_manager: Arc<ConfigManager>,
}

impl ServerManager {
    /// Start HTTP server with OpenAI-compatible routes
    pub async fn start(&mut self, port: u16) -> Result<String, MindLinkError> {
        let app_state = self.app_state.clone();
        
        // Build router with all OpenAI-compatible endpoints
        let app = Router::new()
            // Chat completions endpoint
            .route("/v1/chat/completions", post(chat_completions))
            // Models endpoint  
            .route("/v1/models", get(list_models))
            .route("/v1/models/:model_id", get(get_model))
            // Health check
            .route("/health", get(health_check))
            // Dashboard static files
            .nest("/dashboard", dashboard_router())
            // CORS and security headers
            .layer(CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([AUTHORIZATION, CONTENT_TYPE]))
            .layer(DefaultHeadersLayer::new()
                .header("X-API-Source", "MindLink"))
            // Shared state
            .with_state(app_state);
            
        // Bind to address and start server
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(&addr).await?;
        let local_addr = listener.local_addr()?;
        
        // Spawn server task
        let server = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal());
            
        let server_handle = ServerHandle::new(server);
        *self.server_handle.write().await = Some(server_handle);
        
        Ok(format!("http://{}:{}", local_addr.ip(), local_addr.port()))
    }
}

/// Chat completions handler - OpenAI compatible
async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Response, MindLinkError> {
    // Validate authentication
    let auth_manager = &state.auth_manager;
    if !auth_manager.is_authenticated() {
        return Err(MindLinkError::AuthenticationError("Not authenticated".into()));
    }
    
    // Forward request to ChatGPT
    let response = forward_to_chatgpt(request, auth_manager).await?;
    
    // Handle streaming vs non-streaming responses
    if request.stream.unwrap_or(false) {
        Ok(create_streaming_response(response))
    } else {
        Ok(create_json_response(response))
    }
}
```

### Error Handling Architecture

**File**: `src-tauri/src/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MindLinkError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Tunnel error: {0}")]
    TunnelError(String),
    
    #[error("Server error: {0}")]
    ServerError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl MindLinkError {
    /// Convert error to user-friendly message with recovery suggestions
    pub fn user_message(&self) -> String {
        match self {
            MindLinkError::AuthenticationError(_) => {
                "Authentication failed. Please try logging in again through the system tray menu.".to_string()
            },
            MindLinkError::NetworkError(_) => {
                "Network connection failed. Please check your internet connection and try again.".to_string()
            },
            MindLinkError::TunnelError(_) => {
                "Failed to create tunnel. Please check your network connection and firewall settings.".to_string()
            },
            _ => "An unexpected error occurred. Please restart MindLink and try again.".to_string(),
        }
    }
    
    /// Get error severity for logging and alerting
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            MindLinkError::AuthenticationError(_) => ErrorSeverity::High,
            MindLinkError::NetworkError(_) => ErrorSeverity::Medium,
            MindLinkError::ConfigurationError(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
}
```

## Architecture Deep Dive

### Async Architecture Patterns

MindLink uses Tokio for async runtime and follows these patterns:

#### Manager Coordination Pattern

```rust
// Managers are coordinated through a central AppState
pub struct AppState {
    pub auth_manager: Arc<AuthManager>,
    pub server_manager: Arc<RwLock<ServerManager>>,
    pub tunnel_manager: Arc<RwLock<TunnelManager>>,
    pub config_manager: Arc<ConfigManager>,
}

impl AppState {
    /// Coordinated startup of all services
    pub async fn start_services(&self) -> Result<(), MindLinkError> {
        // Start in dependency order
        
        // 1. Ensure authentication
        if !self.auth_manager.is_authenticated() {
            return Err(MindLinkError::AuthenticationError(
                "Authentication required before starting services".into()
            ));
        }
        
        // 2. Start HTTP server
        let server_url = self.server_manager.write().await
            .start(self.config_manager.get_port()).await?;
        log::info!("Server started at {}", server_url);
        
        // 3. Start tunnel (depends on server)
        let tunnel_url = self.tunnel_manager.write().await
            .create_tunnel(&server_url).await?;
        log::info!("Tunnel created at {}", tunnel_url);
        
        Ok(())
    }
    
    /// Graceful shutdown of all services
    pub async fn shutdown(&self) -> Result<(), MindLinkError> {
        // Shutdown in reverse dependency order
        
        // 1. Stop tunnel first
        if let Err(e) = self.tunnel_manager.write().await.stop_tunnel().await {
            log::warn!("Error stopping tunnel: {}", e);
        }
        
        // 2. Stop server
        if let Err(e) = self.server_manager.write().await.stop().await {
            log::warn!("Error stopping server: {}", e);
        }
        
        // 3. Cleanup auth resources
        // (Auth manager doesn't need explicit shutdown)
        
        log::info!("All services shut down successfully");
        Ok(())
    }
}
```

#### Health Monitoring Pattern

```rust
/// Background health monitoring task
pub async fn start_health_monitor(app_state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        let health_status = HealthStatus {
            auth: check_auth_health(&app_state.auth_manager).await,
            server: check_server_health(&app_state.server_manager).await,
            tunnel: check_tunnel_health(&app_state.tunnel_manager).await,
            timestamp: Utc::now(),
        };
        
        // React to health issues
        match health_status.overall_status() {
            OverallHealth::Healthy => {},
            OverallHealth::Degraded => {
                log::warn!("System health degraded: {:?}", health_status);
                // Attempt automatic recovery
                if let Err(e) = attempt_recovery(&app_state, &health_status).await {
                    log::error!("Recovery failed: {}", e);
                }
            },
            OverallHealth::Critical => {
                log::error!("System health critical: {:?}", health_status);
                // Trigger alerts and emergency procedures
                trigger_critical_alert(&health_status).await;
            },
        }
        
        // Emit health status to UI
        app_state.emit_health_update(health_status).await;
    }
}
```

### Memory Management and Performance

#### Resource Pool Pattern

```rust
/// HTTP client pool for efficient connection reuse
pub struct HttpClientPool {
    clients: Arc<RwLock<Vec<Arc<HttpClient>>>>,
    max_size: usize,
    current_index: Arc<AtomicUsize>,
}

impl HttpClientPool {
    pub fn new(max_size: usize) -> Self {
        let mut clients = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            clients.push(Arc::new(
                HttpClient::builder()
                    .timeout(Duration::from_secs(30))
                    .pool_max_idle_per_host(10)
                    .build()
                    .expect("Failed to create HTTP client")
            ));
        }
        
        Self {
            clients: Arc::new(RwLock::new(clients)),
            max_size,
            current_index: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    /// Get next available client (round-robin)
    pub async fn get_client(&self) -> Arc<HttpClient> {
        let clients = self.clients.read().await;
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.max_size;
        clients[index].clone()
    }
}
```

#### Memory-Efficient Streaming

```rust
/// Streaming response handler that minimizes memory usage
pub async fn stream_chat_response(
    request: ChatCompletionRequest,
    auth_manager: &AuthManager,
) -> Result<impl Stream<Item = Result<Bytes, MindLinkError>>, MindLinkError> {
    let client = auth_manager.get_authenticated_client().await?;
    
    // Create streaming request
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .json(&request)
        .send()
        .await?;
    
    // Convert response to async stream
    let stream = response
        .bytes_stream()
        .map(|chunk| chunk.map_err(MindLinkError::from))
        .filter_map(|chunk| async move {
            match chunk {
                Ok(bytes) => {
                    // Process SSE format and convert to OpenAI format
                    if let Ok(processed) = process_sse_chunk(&bytes) {
                        Some(Ok(processed))
                    } else {
                        None
                    }
                },
                Err(e) => Some(Err(e)),
            }
        });
    
    Ok(stream)
}
```

## Development Workflow

### Daily Development Cycle

#### 1. Environment Setup
```bash
# Start development environment
export RUST_LOG=debug
export MINDLINK_ENV=development

# Start development server with hot reload
npm run tauri:dev
```

#### 2. Code Change Workflow

**Make Changes:**
```bash
# Edit Rust code
nvim src-tauri/src/managers/auth_manager.rs

# Edit frontend code  
nvim src/App.tsx
```

**Test Changes:**
```bash
# Run unit tests
cd src-tauri
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run specific test module
cargo test auth_manager_tests

# Test with different configurations
MINDLINK_CONFIG_FILE=test-config.json cargo test
```

**Quality Checks:**
```bash
# Format code
cargo fmt

# Run linting
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit

# Check for unused dependencies
cargo udeps
```

#### 3. Manual Testing

**Start Development Build:**
```bash
npm run tauri:dev
```

**Test Authentication Flow:**
1. Click tray icon → "Login & Serve"
2. Complete OAuth flow in browser
3. Verify services start correctly
4. Test API endpoints

**Test Configuration Changes:**
1. Open Dashboard → Configuration
2. Modify settings (port, tunnel type, etc.)
3. Apply changes and verify behavior
4. Test service restart with new settings

### Branch Management

#### Feature Development
```bash
# Create feature branch
git checkout -b feature/add-custom-model-routing
git push -u origin feature/add-custom-model-routing

# Make changes and commit
git add .
git commit -m "feat(routing): add custom model routing logic

Implements request-based model routing to optimize
performance and cost based on request characteristics.

Closes #123"

# Push changes
git push origin feature/add-custom-model-routing

# Create PR when ready
gh pr create --title "Add custom model routing" --body "..."
```

#### Bug Fix Workflow
```bash
# Create bug fix branch
git checkout -b fix/tunnel-reconnection-issue
git push -u origin fix/tunnel-reconnection-issue

# Reproduce bug
RUST_LOG=debug cargo run 2>&1 | tee bug-reproduction.log

# Implement fix
# ... make changes ...

# Test fix
cargo test tunnel_manager_tests::test_tunnel_reconnection

# Verify fix resolves original issue
# ... manual testing ...

# Commit and push
git commit -m "fix(tunnel): resolve reconnection race condition

Fixes issue where tunnel reconnection could fail when
multiple reconnection attempts happened simultaneously.

Fixes #456"
git push origin fix/tunnel-reconnection-issue
```

### Code Review Process

#### Preparing for Review

```bash
# Ensure all tests pass
cargo test --all

# Check code quality
cargo clippy -- -D warnings
cargo fmt -- --check

# Run security audit
cargo audit

# Generate test coverage report
cargo tarpaulin --out Html
# Review coverage/tarpaulin-report.html

# Document changes
# Update CHANGELOG.md if needed
# Update documentation if API changes
```

#### Self-Review Checklist

- [ ] All new code has comprehensive tests
- [ ] Error handling covers all edge cases  
- [ ] Documentation is updated for API changes
- [ ] Performance impact is considered and measured
- [ ] Security implications are reviewed
- [ ] Breaking changes are documented
- [ ] Memory safety is maintained (no unsafe code without justification)

## Testing Strategy

### Test Hierarchy

#### Unit Tests (src/tests/)

**Purpose**: Test individual components in isolation

```rust
// Example: AuthManager unit tests
#[cfg(test)]
mod auth_manager_tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_login_generates_valid_oauth_url() {
        // Arrange
        let config_manager = Arc::new(MockConfigManager::new());
        let auth_manager = AuthManager::new(config_manager);
        
        // Act
        let oauth_url = auth_manager.login().await.unwrap();
        
        // Assert
        assert!(oauth_url.starts_with("https://auth.openai.com"));
        assert!(oauth_url.contains("code_challenge"));
        assert!(oauth_url.contains("response_type=code"));
    }
    
    #[tokio::test]
    async fn test_token_refresh_handles_expired_tokens() {
        // Arrange
        let mut mock_client = MockHttpClient::new();
        mock_client.expect_post()
            .with(eq("https://auth.openai.com/oauth/token"))
            .returning(|_| Ok(TokenResponse {
                access_token: "new_token".to_string(),
                expires_in: 3600,
            }));
            
        let auth_manager = AuthManager::with_client(mock_client);
        
        // Set up expired token
        auth_manager.set_tokens(TokenSet {
            access_token: "expired_token".to_string(),
            expires_at: Utc::now() - Duration::hours(1),
        });
        
        // Act
        let result = auth_manager.refresh_token().await;
        
        // Assert
        assert!(result.is_ok());
        assert!(auth_manager.is_authenticated());
    }
}
```

#### Integration Tests (tests/)

**Purpose**: Test component interactions and workflows

```rust
// Example: Full authentication and server startup flow
#[tokio::test]
async fn test_complete_service_startup_flow() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    let config_manager = Arc::new(ConfigManager::new(config_path));
    let auth_manager = Arc::new(AuthManager::new(config_manager.clone()));
    let server_manager = Arc::new(RwLock::new(ServerManager::new(
        config_manager.clone(), 
        auth_manager.clone()
    )));
    
    let app_state = Arc::new(AppState {
        auth_manager,
        server_manager,
        config_manager,
        tunnel_manager: Arc::new(RwLock::new(TunnelManager::new())),
    });
    
    // Mock successful authentication
    app_state.auth_manager.set_test_tokens().await;
    
    // Act
    let result = app_state.start_services().await;
    
    // Assert
    assert!(result.is_ok());
    assert!(app_state.server_manager.read().await.is_running());
    
    // Test API endpoint
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3001/health")
        .send()
        .await
        .unwrap();
        
    assert_eq!(response.status(), 200);
    
    // Cleanup
    app_state.shutdown().await.unwrap();
}
```

#### End-to-End Tests

**Purpose**: Test complete user workflows

```rust
// Example: Complete user authentication and API usage flow
#[tokio::test]
async fn test_end_to_end_api_workflow() {
    // Start MindLink application
    let app = start_test_application().await;
    
    // Simulate OAuth authentication
    let auth_url = app.trigger_login().await.unwrap();
    let auth_code = simulate_oauth_completion(&auth_url).await.unwrap();
    app.complete_login(&auth_code).await.unwrap();
    
    // Wait for services to start
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Test API call
    let api_client = OpenAI::new()
        .with_base_url(app.get_api_url())
        .with_api_key("test-key");
        
    let response = api_client
        .chat()
        .completions()
        .create(ChatCompletionCreateParams {
            model: "gpt-4".to_string(),
            messages: vec![ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: "Hello, world!".to_string(),
                }
            )],
            ..Default::default()
        })
        .await
        .unwrap();
        
    assert!(!response.choices.is_empty());
    assert!(!response.choices[0].message.content.is_empty());
    
    // Cleanup
    app.shutdown().await.unwrap();
}
```

### Test Utilities

#### Mock Factories

```rust
/// Factory for creating configured mock HTTP clients
pub fn create_mock_http_client() -> MockHttpClient {
    let mut mock = MockHttpClient::new();
    
    // Default successful authentication response
    mock.expect_post()
        .with(eq("https://auth.openai.com/oauth/token"))
        .returning(|_| Ok(serde_json::json!({
            "access_token": "test_access_token",
            "refresh_token": "test_refresh_token", 
            "expires_in": 3600
        })));
        
    // Default successful ChatGPT API response
    mock.expect_post()
        .with(eq("https://api.openai.com/v1/chat/completions"))
        .returning(|_| Ok(serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Test response"
                },
                "finish_reason": "stop"
            }]
        })));
        
    mock
}

/// Factory for creating temporary test configurations
pub fn create_test_config() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.json");
    
    let config = serde_json::json!({
        "server": {
            "port": 0, // Use random available port
            "host": "127.0.0.1"
        },
        "tunnel": {
            "type": "quick",
            "health_check_interval": 5
        },
        "auth": {
            "token_refresh_margin": 300
        }
    });
    
    std::fs::write(&config_path, config.to_string()).unwrap();
    (temp_dir, config_path)
}
```

#### Test Environment Management

```rust
/// Test environment setup and cleanup
pub struct TestEnvironment {
    temp_dirs: Vec<TempDir>,
    running_processes: Vec<Child>,
    test_ports: Vec<u16>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self {
            temp_dirs: Vec::new(),
            running_processes: Vec::new(), 
            test_ports: Vec::new(),
        }
    }
    
    pub fn create_temp_dir(&mut self) -> &Path {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        self.temp_dirs.push(temp_dir);
        path
    }
    
    pub fn get_free_port(&mut self) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        self.test_ports.push(port);
        port
    }
    
    pub async fn spawn_test_server(&mut self, port: u16) -> String {
        // Implementation for spawning test HTTP server
        // Returns base URL for testing
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Clean up all resources
        for mut process in self.running_processes.drain(..) {
            let _ = process.kill();
        }
        // temp_dirs are automatically cleaned up by TempDir
    }
}
```

### Performance Testing

#### Benchmarking Infrastructure

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark authentication flow performance
fn benchmark_auth_flow(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let auth_manager = Arc::new(AuthManager::new_for_testing());
    
    c.bench_function("auth_token_validation", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(auth_manager.validate_token().await)
        })
    });
    
    c.bench_function("auth_token_refresh", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(auth_manager.refresh_token().await)
        })
    });
}

/// Benchmark API request handling performance
fn benchmark_api_performance(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server_manager = create_test_server_manager();
    
    c.bench_function("chat_completion_processing", |b| {
        let request = create_test_chat_request();
        b.to_async(&rt).iter(|| async {
            black_box(server_manager.handle_chat_completion(request.clone()).await)
        })
    });
}

criterion_group!(benches, benchmark_auth_flow, benchmark_api_performance);
criterion_main!(benches);
```

#### Load Testing

```bash
#!/bin/bash
# scripts/load-test.sh

# Start MindLink in test mode
RUST_LOG=info cargo run --release &
MINDLINK_PID=$!

# Wait for startup
sleep 10

# Run load test with various patterns
echo "Running concurrent user simulation..."
k6 run --vus 100 --duration 5m scripts/load-test.js

echo "Running burst traffic test..."
k6 run --stages '
  {duration: "5m", target: 10},
  {duration: "10m", target: 50}, 
  {duration: "5m", target: 100},
  {duration: "10m", target: 50},
  {duration: "5m", target: 0}
' scripts/load-test.js

echo "Running sustained load test..."
k6 run --vus 50 --duration 30m scripts/load-test.js

# Cleanup
kill $MINDLINK_PID
```

### Continuous Integration Testing

#### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
        
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
        
    - name: Install dependencies
      run: |
        npm install
        
    - name: Install system dependencies (Linux)
      if: runner.os == 'Linux'
      run: |
        sudo apt-get update
        sudo apt-get install -y libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev
        
    - name: Check formatting
      run: cargo fmt --all -- --check
      
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
      
    - name: Run unit tests
      run: cargo test --lib --all-features
      
    - name: Run integration tests  
      run: cargo test --test '*' --all-features
      
    - name: Run end-to-end tests
      run: npm run test:e2e
      
    - name: Generate coverage report
      if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml --all-features
        
    - name: Upload coverage
      if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
      uses: codecov/codecov-action@v3
      with:
        file: ./cobertura.xml
```

## Performance Optimization

### CPU Optimization

#### Async Task Optimization

```rust
/// Optimized task spawning with proper resource limits
pub struct TaskPool {
    semaphore: Arc<Semaphore>,
    active_tasks: Arc<AtomicUsize>,
    max_concurrent: usize,
}

impl TaskPool {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
            max_concurrent,
        }
    }
    
    /// Spawn task with backpressure control
    pub async fn spawn<F, T>(&self, task: F) -> Result<T, MindLinkError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Acquire permit (blocks if at capacity)
        let _permit = self.semaphore.acquire().await
            .map_err(|_| MindLinkError::InternalError("Task pool closed".into()))?;
            
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
        
        let active_tasks = self.active_tasks.clone();
        let result = tokio::spawn(async move {
            let result = task.await;
            active_tasks.fetch_sub(1, Ordering::Relaxed);
            result
        }).await.map_err(|e| MindLinkError::InternalError(format!("Task failed: {}", e)))?;
        
        Ok(result)
    }
    
    /// Get current task pool metrics
    pub fn metrics(&self) -> TaskPoolMetrics {
        TaskPoolMetrics {
            active_tasks: self.active_tasks.load(Ordering::Relaxed),
            max_concurrent: self.max_concurrent,
            available_permits: self.semaphore.available_permits(),
        }
    }
}
```

#### Hot Path Optimization

```rust
/// Zero-copy request processing for performance-critical paths
pub async fn process_chat_request_optimized(
    request: &[u8], // Raw bytes to avoid deserialization
    auth_token: &str,
) -> Result<Vec<u8>, MindLinkError> {
    // Fast path validation without full deserialization
    if !is_valid_chat_request_bytes(request) {
        return Err(MindLinkError::InvalidRequest("Malformed request".into()));
    }
    
    // Streaming transformation without buffering entire response
    let response_stream = forward_request_stream(request, auth_token).await?;
    
    // Process response chunks as they arrive
    let mut response_buffer = Vec::with_capacity(4096);
    pin_mut!(response_stream);
    
    while let Some(chunk) = response_stream.next().await {
        let chunk = chunk?;
        
        // Transform chunk without copying
        let transformed_chunk = transform_response_chunk_inplace(chunk);
        response_buffer.extend_from_slice(&transformed_chunk);
        
        // Yield periodically to prevent blocking
        if response_buffer.len() > 8192 {
            tokio::task::yield_now().await;
        }
    }
    
    Ok(response_buffer)
}

/// Fast request validation using SIMD when available
#[inline(always)]
fn is_valid_chat_request_bytes(data: &[u8]) -> bool {
    // Quick checks for required fields without full JSON parsing
    data.windows(b"\"messages\"".len()).any(|w| w == b"\"messages\"") &&
    data.windows(b"\"model\"".len()).any(|w| w == b"\"model\"")
}
```

### Memory Optimization

#### Smart Caching Strategy

```rust
/// LRU cache with size and time-based eviction
pub struct ResponseCache {
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    max_size: usize,
    max_age: Duration,
    metrics: CacheMetrics,
}

impl ResponseCache {
    pub fn new(max_size: usize, max_age: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(max_size))),
            max_size,
            max_age,
            metrics: CacheMetrics::new(),
        }
    }
    
    /// Get cached response with automatic cleanup
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache.write().await;
        
        // Remove expired entries during lookup
        if let Some(entry) = cache.peek(key) {
            if entry.is_expired(self.max_age) {
                cache.pop(key);
                self.metrics.record_expired();
                return None;
            }
        }
        
        match cache.get(key) {
            Some(entry) => {
                self.metrics.record_hit();
                Some(entry.data.clone())
            },
            None => {
                self.metrics.record_miss();
                None
            }
        }
    }
    
    /// Store response with compression for large responses
    pub async fn put(&self, key: String, data: Vec<u8>) {
        let compressed_data = if data.len() > 1024 {
            compress_response(&data)
        } else {
            data
        };
        
        let entry = CacheEntry {
            data: compressed_data,
            created_at: Instant::now(),
            access_count: 0,
        };
        
        let mut cache = self.cache.write().await;
        cache.put(key, entry);
        
        // Background cleanup if cache is getting full
        if cache.len() > self.max_size * 9 / 10 {
            self.cleanup_expired(&mut cache).await;
        }
    }
    
    async fn cleanup_expired(&self, cache: &mut LruCache<String, CacheEntry>) {
        let expired_keys: Vec<_> = cache
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.max_age))
            .map(|(key, _)| key.clone())
            .collect();
            
        for key in expired_keys {
            cache.pop(&key);
        }
    }
}
```

#### Memory Pool Pattern

```rust
/// Object pool for reusing expensive allocations
pub struct BufferPool {
    buffers: Arc<Mutex<Vec<Vec<u8>>>>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl BufferPool {
    pub fn new(buffer_size: usize, max_pool_size: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(Vec::with_capacity(max_pool_size))),
            buffer_size,
            max_pool_size,
        }
    }
    
    /// Get buffer from pool or create new one
    pub fn get_buffer(&self) -> Vec<u8> {
        let mut buffers = self.buffers.lock().unwrap();
        
        buffers.pop().unwrap_or_else(|| {
            Vec::with_capacity(self.buffer_size)
        })
    }
    
    /// Return buffer to pool after use
    pub fn return_buffer(&self, mut buffer: Vec<u8>) {
        // Clear but keep capacity
        buffer.clear();
        
        // Only return if pool isn't full and buffer is reasonable size
        if buffer.capacity() <= self.buffer_size * 2 {
            let mut buffers = self.buffers.lock().unwrap();
            if buffers.len() < self.max_pool_size {
                buffers.push(buffer);
            }
        }
    }
}

/// RAII wrapper for automatic buffer return
pub struct PooledBuffer {
    buffer: Option<Vec<u8>>,
    pool: Arc<BufferPool>,
}

impl PooledBuffer {
    pub fn new(pool: Arc<BufferPool>) -> Self {
        let buffer = pool.get_buffer();
        Self {
            buffer: Some(buffer),
            pool,
        }
    }
    
    pub fn as_mut(&mut self) -> &mut Vec<u8> {
        self.buffer.as_mut().unwrap()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.return_buffer(buffer);
        }
    }
}
```

### Network Optimization

#### Connection Pooling

```rust
/// HTTP connection pool with circuit breaker pattern
pub struct HttpConnectionPool {
    clients: Vec<Arc<HttpClient>>,
    current_index: AtomicUsize,
    circuit_breaker: Arc<CircuitBreaker>,
    metrics: ConnectionMetrics,
}

impl HttpConnectionPool {
    pub fn new(pool_size: usize) -> Self {
        let mut clients = Vec::with_capacity(pool_size);
        
        for _ in 0..pool_size {
            let client = reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Duration::from_secs(30))
                .timeout(Duration::from_secs(30))
                .tcp_keepalive(Duration::from_secs(60))
                .http2_adaptive_window(true)
                .http2_keep_alive_interval(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client");
                
            clients.push(Arc::new(client));
        }
        
        Self {
            clients,
            current_index: AtomicUsize::new(0),
            circuit_breaker: Arc::new(CircuitBreaker::new(
                10, // failure threshold
                Duration::from_secs(60), // recovery timeout
            )),
            metrics: ConnectionMetrics::new(),
        }
    }
    
    /// Get next client with circuit breaker protection
    pub async fn get_client(&self) -> Result<Arc<HttpClient>, MindLinkError> {
        if !self.circuit_breaker.is_available() {
            return Err(MindLinkError::NetworkError(
                "Circuit breaker is open".into()
            ));
        }
        
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        let client = self.clients[index].clone();
        
        self.metrics.record_client_request();
        Ok(client)
    }
    
    /// Record successful request for circuit breaker
    pub fn record_success(&self) {
        self.circuit_breaker.record_success();
        self.metrics.record_success();
    }
    
    /// Record failed request for circuit breaker  
    pub fn record_failure(&self) {
        self.circuit_breaker.record_failure();
        self.metrics.record_failure();
    }
}
```

### Profiling and Monitoring

#### Runtime Performance Monitoring

```rust
/// Performance monitoring with automatic alerts
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    alert_thresholds: AlertThresholds,
    last_alert: Arc<RwLock<Option<Instant>>>,
}

impl PerformanceMonitor {
    pub fn new(alert_thresholds: AlertThresholds) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            alert_thresholds,
            last_alert: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Record request timing and check for performance issues
    pub async fn record_request_timing(&self, duration: Duration, endpoint: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.record_request_duration(endpoint, duration);
        
        // Check for performance degradation
        if duration > self.alert_thresholds.max_response_time {
            self.check_and_send_alert(AlertType::HighLatency {
                endpoint: endpoint.to_string(),
                duration,
                threshold: self.alert_thresholds.max_response_time,
            }).await;
        }
        
        // Check for sustained high latency
        let avg_latency = metrics.get_average_latency(endpoint);
        if avg_latency > self.alert_thresholds.sustained_high_latency {
            self.check_and_send_alert(AlertType::SustainedHighLatency {
                endpoint: endpoint.to_string(),
                average_duration: avg_latency,
                threshold: self.alert_thresholds.sustained_high_latency,
            }).await;
        }
    }
    
    /// Get current performance snapshot
    pub async fn get_metrics_snapshot(&self) -> PerformanceSnapshot {
        let metrics = self.metrics.read().await;
        metrics.create_snapshot()
    }
    
    async fn check_and_send_alert(&self, alert: AlertType) {
        // Rate limit alerts to prevent spam
        const MIN_ALERT_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes
        
        let mut last_alert = self.last_alert.write().await;
        let now = Instant::now();
        
        if let Some(last) = *last_alert {
            if now - last < MIN_ALERT_INTERVAL {
                return; // Skip alert due to rate limiting
            }
        }
        
        // Send alert through configured channels
        self.send_alert(alert).await;
        *last_alert = Some(now);
    }
}
```

## Debugging and Profiling

### Logging Infrastructure

#### Structured Logging

```rust
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize comprehensive logging system
pub fn init_logging(config: &LoggingConfig) -> Result<(), MindLinkError> {
    let log_dir = dirs::data_local_dir()
        .ok_or(MindLinkError::ConfigurationError("Cannot determine log directory".into()))?
        .join("com.mindlink.mindlink")
        .join("logs");
        
    std::fs::create_dir_all(&log_dir)?;
    
    // File appender with rotation
    let file_appender = tracing_appender::rolling::daily(&log_dir, "mindlink.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
    
    // Console output for development
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true);
    
    // File output with JSON formatting  
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);
    
    // Performance tracing layer
    let performance_layer = if config.performance_tracing {
        Some(tracing_tracy::TracyLayer::new())
    } else {
        None
    };
    
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(performance_layer)
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| MindLinkError::ConfigurationError(format!("Failed to init logging: {}", e)))?;
        
    Ok(())
}

/// Instrument functions for automatic tracing
#[instrument(skip(self), fields(user_id = %user_id))]
pub async fn authenticate_user(&self, user_id: &str) -> Result<TokenSet, MindLinkError> {
    info!("Starting authentication for user");
    
    let result = self.perform_oauth_flow(user_id).await;
    
    match &result {
        Ok(tokens) => {
            info!("Authentication successful", tokens.expires_at = %tokens.expires_at);
        },
        Err(e) => {
            error!("Authentication failed: {}", e);
        }
    }
    
    result
}
```

#### Debug Utilities

```rust
/// Development-only debug utilities
#[cfg(debug_assertions)]
pub mod debug_utils {
    use super::*;
    
    /// Dump internal state for debugging
    pub async fn dump_auth_state(auth_manager: &AuthManager) -> serde_json::Value {
        serde_json::json!({
            "is_authenticated": auth_manager.is_authenticated(),
            "token_expires_at": auth_manager.get_token_expiry().await,
            "last_refresh": auth_manager.get_last_refresh_time().await,
            "oauth_client_config": auth_manager.get_oauth_config_debug(),
        })
    }
    
    /// Start debug HTTP server for inspecting state
    pub async fn start_debug_server(app_state: Arc<AppState>) -> Result<(), MindLinkError> {
        use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
        
        let debug_router = Router::new()
            .route("/debug/auth", get(debug_auth_state))
            .route("/debug/server", get(debug_server_state))
            .route("/debug/tunnel", get(debug_tunnel_state))
            .route("/debug/metrics", get(debug_metrics))
            .with_state(app_state);
            
        let listener = tokio::net::TcpListener::bind("127.0.0.1:9999").await?;
        info!("Debug server started at http://127.0.0.1:9999");
        
        axum::serve(listener, debug_router)
            .await
            .map_err(|e| MindLinkError::ServerError(format!("Debug server failed: {}", e)))
    }
    
    async fn debug_auth_state(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
        Json(dump_auth_state(&state.auth_manager).await)
    }
}
```

### Performance Profiling

#### CPU Profiling

```bash
#!/bin/bash
# scripts/profile-cpu.sh

echo "Starting CPU profiling session..."

# Build optimized binary with debug symbols
cargo build --release --bin mindlink

# Start profiling with perf (Linux)
if command -v perf &> /dev/null; then
    echo "Using perf for profiling..."
    perf record --call-graph dwarf -g ./target/release/mindlink &
    PROFILE_PID=$!
    
    # Run load test
    sleep 5  # Let app start
    ./scripts/generate-load.sh
    
    # Stop profiling
    kill $PROFILE_PID
    perf report --stdio > cpu-profile-report.txt
    echo "CPU profile saved to cpu-profile-report.txt"
    
# Use cargo flamegraph (cross-platform)
elif command -v cargo-flamegraph &> /dev/null; then
    echo "Using flamegraph for profiling..."
    cargo flamegraph --bin mindlink -- &
    PROFILE_PID=$!
    
    sleep 5
    ./scripts/generate-load.sh
    
    kill $PROFILE_PID
    echo "Flamegraph saved to flamegraph.svg"
    
else
    echo "No profiling tools available. Install perf or cargo-flamegraph"
    exit 1
fi

echo "Profiling complete!"
```

#### Memory Profiling

```bash
#!/bin/bash  
# scripts/profile-memory.sh

echo "Starting memory profiling..."

# Using Valgrind (Linux)
if command -v valgrind &> /dev/null; then
    echo "Profiling with Valgrind..."
    valgrind \
        --tool=massif \
        --massif-out-file=massif.out \
        --detailed-freq=1 \
        --threshold=0.1 \
        ./target/release/mindlink &
    PROFILE_PID=$!
    
    sleep 5
    ./scripts/generate-load.sh
    
    kill $PROFILE_PID
    
    # Generate report
    ms_print massif.out > memory-profile-report.txt
    echo "Memory profile saved to memory-profile-report.txt"
    
# Using heaptrack (alternative)
elif command -v heaptrack &> /dev/null; then
    echo "Profiling with heaptrack..."
    heaptrack ./target/release/mindlink &
    PROFILE_PID=$!
    
    sleep 5
    ./scripts/generate-load.sh
    
    kill $PROFILE_PID
    echo "Memory profile completed. Use heaptrack_gui to view results."
    
else
    echo "No memory profiling tools available"
    exit 1
fi
```

#### Custom Performance Metrics

```rust
/// Built-in performance metrics collection
pub struct MetricsCollector {
    request_durations: histogram::Histogram,
    memory_usage: gauge::Gauge,
    active_connections: gauge::Gauge,
    error_counts: counter::Counter,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            request_durations: histogram::Histogram::new(),
            memory_usage: gauge::Gauge::new(),
            active_connections: gauge::Gauge::new(),
            error_counts: counter::Counter::new(),
        }
    }
    
    /// Record request duration with automatic percentile calculation
    pub fn record_request_duration(&self, duration: Duration, endpoint: &str) {
        self.request_durations.record_value_with_tags(
            duration.as_millis() as u64,
            &[("endpoint", endpoint)]
        );
        
        // Alert on outliers
        if duration > Duration::from_millis(1000) {
            warn!("Slow request detected: {} took {}ms", endpoint, duration.as_millis());
        }
    }
    
    /// Update memory usage metrics
    pub fn update_memory_usage(&self) {
        if let Ok(usage) = get_current_memory_usage() {
            self.memory_usage.set(usage as f64);
            
            // Alert on high memory usage
            if usage > 100_000_000 { // 100MB
                warn!("High memory usage detected: {}MB", usage / 1_000_000);
            }
        }
    }
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();
        
        // Request duration histogram
        output.push_str("# HELP mindlink_request_duration_ms Request duration in milliseconds\n");
        output.push_str("# TYPE mindlink_request_duration_ms histogram\n");
        for (endpoint, histogram) in &self.request_durations.get_by_tags() {
            output.push_str(&format!(
                "mindlink_request_duration_ms_bucket{{endpoint=\"{}\",le=\"+Inf\"}} {}\n",
                endpoint, histogram.len()
            ));
        }
        
        // Memory usage gauge
        output.push_str("# HELP mindlink_memory_usage_bytes Current memory usage in bytes\n");
        output.push_str("# TYPE mindlink_memory_usage_bytes gauge\n");
        output.push_str(&format!("mindlink_memory_usage_bytes {}\n", self.memory_usage.get()));
        
        output
    }
}

/// Get current process memory usage
fn get_current_memory_usage() -> Result<usize, std::io::Error> {
    #[cfg(unix)]
    {
        let status = std::fs::read_to_string("/proc/self/status")?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let kb: usize = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                return Ok(kb * 1024); // Convert KB to bytes
            }
        }
    }
    
    #[cfg(windows)]
    {
        // Windows implementation using GetProcessMemoryInfo
        use windows::Win32::System::ProcessStatus::*;
        use windows::Win32::Foundation::*;
        
        unsafe {
            let process = GetCurrentProcess();
            let mut pmc = PROCESS_MEMORY_COUNTERS::default();
            
            if GetProcessMemoryInfo(process, &mut pmc, size_of::<PROCESS_MEMORY_COUNTERS>() as u32).as_bool() {
                return Ok(pmc.WorkingSetSize);
            }
        }
    }
    
    Ok(0)
}
```

## Build and Deployment

### Development Builds

#### Local Development Build

```bash
#!/bin/bash
# scripts/dev-build.sh

set -e

echo "🔧 Starting development build..."

# Check prerequisites
echo "📋 Checking prerequisites..."
command -v rustc >/dev/null 2>&1 || { echo "❌ Rust not installed"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "❌ Node.js not installed"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "❌ npm not installed"; exit 1; }

# Install/update dependencies
echo "📦 Installing dependencies..."
npm install
cd src-tauri && cargo check && cd ..

# Run code quality checks
echo "🔍 Running quality checks..."
cd src-tauri
cargo fmt --check || { echo "⚠️  Code formatting issues found. Run 'cargo fmt' to fix."; }
cargo clippy --all-targets --all-features -- -D warnings
cd ..

# Build frontend
echo "🏗️  Building frontend..."
npm run build

# Build Tauri application
echo "🦀 Building Rust backend..."
npm run tauri build -- --debug

echo "✅ Development build complete!"
echo "📍 Binary location: src-tauri/target/debug/bundle/"
```

#### Hot Reload Development

```bash
#!/bin/bash
# scripts/dev-server.sh

# Set development environment
export RUST_LOG=debug
export MINDLINK_ENV=development

# Start development server with hot reload
echo "🚀 Starting development server with hot reload..."
echo "🌐 Frontend: http://localhost:1420"
echo "🔧 Backend: Rust with auto-reload"
echo "📱 App: Will launch automatically"

npm run tauri dev
```

### Production Builds

#### Cross-Platform Build Script

```bash
#!/bin/bash
# scripts/build-release.sh

set -e

VERSION=$(grep '^version = ' src-tauri/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "🏗️  Building MindLink v$VERSION for production..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf src-tauri/target/release/bundle/
rm -rf dist/

# Install dependencies with clean state
echo "📦 Installing fresh dependencies..."
rm -rf node_modules package-lock.json
npm ci

# Run comprehensive tests
echo "🧪 Running test suite..."
cd src-tauri
cargo test --release --all-features
cd ..

# Security audit
echo "🔒 Running security audit..."
cd src-tauri
cargo audit --deny warnings
cd ..

# Build optimized frontend
echo "🌐 Building optimized frontend..."
NODE_ENV=production npm run build

# Build for all platforms
echo "🦀 Building cross-platform binaries..."

# Linux build
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "🐧 Building for Linux..."
    npm run tauri build -- --target x86_64-unknown-linux-gnu
    npm run tauri build -- --target aarch64-unknown-linux-gnu
fi

# macOS build
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 Building for macOS..."
    npm run tauri build -- --target x86_64-apple-darwin
    npm run tauri build -- --target aarch64-apple-darwin
fi

# Windows build (cross-compile or on Windows)
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    echo "🪟 Building for Windows..."
    npm run tauri build -- --target x86_64-pc-windows-msvc
fi

echo "✅ Production build complete!"
echo "📦 Packages available in: src-tauri/target/release/bundle/"
```

#### GitHub Actions Build Pipeline

```yaml
# .github/workflows/build.yml
name: Build and Release

on:
  push:
    tags: ['v*']
  workflow_dispatch:

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
          - platform: 'macos-latest'
            args: '--target x86_64-apple-darwin'
          - platform: 'ubuntu-20.04'
            args: '--target x86_64-unknown-linux-gnu'
          - platform: 'ubuntu-20.04'
            args: '--target aarch64-unknown-linux-gnu'
          - platform: 'windows-latest'
            args: '--target x86_64-pc-windows-msvc'

    runs-on: ${{ matrix.platform }}
    
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-20.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libayatana-appindicator3-dev librsvg2-dev

      - name: Rust setup
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || matrix.platform == 'ubuntu-20.04' && 'x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu' || 'x86_64-pc-windows-msvc' }}

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - name: Sync node version and setup cache
        uses: actions/setup-node@v4
        with:
          node-version: 'lts/*'
          cache: 'npm'

      - name: Install frontend dependencies
        run: npm ci

      - name: Build the app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'MindLink ${{ github.ref_name }}'
          releaseBody: 'See the assets to download and install this version.'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

### Code Signing and Distribution

#### macOS Code Signing

```bash
#!/bin/bash
# scripts/sign-macos.sh

APP_PATH="src-tauri/target/release/bundle/macos/MindLink.app"
DEVELOPER_ID="Developer ID Application: Your Name (TEAM_ID)"

echo "🔐 Signing macOS application..."

# Sign the binary
codesign --force --options runtime --deep --sign "$DEVELOPER_ID" "$APP_PATH"

# Verify signature
codesign --verify --deep --strict "$APP_PATH"
echo "✅ Code signing verification successful"

# Create DMG
echo "📦 Creating DMG package..."
create-dmg \
  --volname "MindLink Installer" \
  --volicon "assets/volume-icon.icns" \
  --window-pos 200 120 \
  --window-size 600 300 \
  --icon-size 100 \
  --icon "MindLink.app" 175 120 \
  --hide-extension "MindLink.app" \
  --app-drop-link 425 120 \
  "MindLink-${VERSION}.dmg" \
  "$APP_PATH"

# Sign DMG
codesign --force --sign "$DEVELOPER_ID" "MindLink-${VERSION}.dmg"

echo "✅ macOS package ready for distribution"
```

#### Windows Code Signing

```powershell
# scripts/sign-windows.ps1

$APP_PATH = "src-tauri\target\release\bundle\msi\MindLink_$VERSION_x64_en-US.msi"
$CERT_PATH = $env:WINDOWS_CERTIFICATE_PATH
$CERT_PASSWORD = $env:WINDOWS_CERTIFICATE_PASSWORD

Write-Host "🔐 Signing Windows installer..."

# Sign the MSI package
signtool sign /f $CERT_PATH /p $CERT_PASSWORD /t http://timestamp.digicert.com $APP_PATH

# Verify signature
signtool verify /pa $APP_PATH

Write-Host "✅ Windows package ready for distribution"
```

### Deployment Automation

#### Release Preparation Script

```bash
#!/bin/bash
# scripts/prepare-release.sh

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.2.3"
    exit 1
fi

VERSION=$1
CURRENT_VERSION=$(grep '^version = ' src-tauri/Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "🚀 Preparing release v$VERSION (current: v$CURRENT_VERSION)"

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "❌ Invalid version format. Use semantic versioning (e.g., 1.2.3)"
    exit 1
fi

# Update version in all files
echo "📝 Updating version numbers..."
sed -i.bak "s/version = \"$CURRENT_VERSION\"/version = \"$VERSION\"/" src-tauri/Cargo.toml
sed -i.bak "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$VERSION\"/" package.json
sed -i.bak "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

# Update Cargo.lock
cd src-tauri && cargo check && cd ..

# Generate changelog entry
echo "📄 Updating CHANGELOG.md..."
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    DATE=$(date +%Y-%m-%d)
    TEMP_FILE=$(mktemp)
    {
        head -n 6 CHANGELOG.md
        echo ""
        echo "## [$VERSION] - $DATE"
        echo ""
        echo "### Added"
        echo "- "
        echo ""
        echo "### Changed" 
        echo "- "
        echo ""
        echo "### Fixed"
        echo "- "
        echo ""
        tail -n +7 CHANGELOG.md
    } > "$TEMP_FILE"
    mv "$TEMP_FILE" CHANGELOG.md
    
    echo "⚠️  Please edit CHANGELOG.md to add release notes"
    ${EDITOR:-nano} CHANGELOG.md
fi

# Run tests
echo "🧪 Running test suite..."
npm test

# Build release packages
echo "🏗️  Building release packages..."
./scripts/build-release.sh

# Create git tag
echo "🏷️  Creating git tag..."
git add -A
git commit -m "chore: release v$VERSION" || echo "No changes to commit"
git tag -a "v$VERSION" -m "Release v$VERSION"

echo "✅ Release v$VERSION prepared successfully!"
echo ""
echo "Next steps:"
echo "1. Review the changes: git log --oneline -10"
echo "2. Push to GitHub: git push origin main && git push origin v$VERSION"
echo "3. GitHub Actions will automatically build and create a release"
echo "4. Edit the GitHub release to add release notes from CHANGELOG.md"
```

This comprehensive development guide provides everything needed to understand, develop, test, and deploy MindLink effectively. The guide covers the complete development lifecycle from environment setup through production deployment, ensuring consistent quality and maintainability.