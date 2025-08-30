# MindLink Architecture Documentation

## System Overview

MindLink is a production-grade desktop application built with Rust and Tauri that bridges local applications to ChatGPT Plus/Pro accounts through an OpenAI-compatible API. The system creates secure Cloudflare tunnels to enable global access while maintaining enterprise-level security and performance.

## Architecture Principles

### 1. **Separation of Concerns**
Each component has a single, well-defined responsibility:
- **AuthManager**: OAuth2 authentication and token lifecycle
- **ServerManager**: HTTP API server and OpenAI compatibility
- **TunnelManager**: Cloudflare tunnel creation and management
- **ConfigManager**: Application configuration and persistence
- **BifrostManager**: Web dashboard and monitoring interface

### 2. **Async-First Design**
All I/O operations use Tokio's async runtime for optimal performance:
- Non-blocking HTTP requests
- Concurrent connection handling
- Efficient resource utilization
- Responsive UI interactions

### 3. **Error-First Programming**
Comprehensive error handling with structured error types:
- All functions return `Result<T, MindLinkError>`
- Detailed error context and recovery suggestions
- Automatic retry mechanisms with exponential backoff
- Graceful degradation strategies

### 4. **Memory Safety**
Leverages Rust's ownership model for zero-cost safety:
- No memory leaks or buffer overflows
- Thread-safe concurrent operations
- Predictable resource cleanup
- Zero-copy optimizations where possible

## System Components

### Core Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Desktop UI    │    │   System Tray   │    │  Web Dashboard  │
│   (Tauri)       │    │   Integration   │    │   (Bifrost)     │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          ▼                      ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Tauri IPC Layer                           │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Rust Application Core                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────── │
│  │AuthManager  │ │ServerManager│ │TunnelManager│ │ConfigMgr    │ │
│  │- OAuth2     │ │- HTTP API   │ │- Cloudflare │ │- Settings   │ │
│  │- Tokens     │ │- OpenAI API │ │- SSL/TLS    │ │- Persistence│ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                   External Services                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐               │
│  │  ChatGPT    │ │ Cloudflare  │ │Local Storage│               │
│  │   OAuth     │ │  Tunnels    │ │  (Config)   │               │
│  └─────────────┘ └─────────────┘ └─────────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

### Manager Pattern Implementation

#### AuthManager
- **Responsibility**: OAuth2 authentication flow with ChatGPT
- **Key Features**:
  - PKCE-secured OAuth flow
  - Automatic token refresh (15 minutes before expiry)
  - Secure credential storage using OS keychain
  - Session validation and recovery

```rust
pub struct AuthManager {
    client: Arc<HttpClient>,
    config_manager: Arc<ConfigManager>,
    tokens: Arc<RwLock<Option<TokenSet>>>,
}

impl AuthManager {
    pub async fn login(&self) -> Result<(), MindLinkError>;
    pub async fn refresh_token(&self) -> Result<(), MindLinkError>;
    pub fn is_authenticated(&self) -> bool;
    pub async fn logout(&self) -> Result<(), MindLinkError>;
}
```

#### ServerManager
- **Responsibility**: OpenAI-compatible HTTP API server
- **Key Features**:
  - Full OpenAI API compatibility
  - Streaming response support
  - Request/response logging
  - CORS and security headers

```rust
pub struct ServerManager {
    server_handle: Option<ServerHandle>,
    config_manager: Arc<ConfigManager>,
    auth_manager: Arc<AuthManager>,
}

impl ServerManager {
    pub async fn start(&mut self) -> Result<(), MindLinkError>;
    pub async fn stop(&mut self) -> Result<(), MindLinkError>;
    pub fn is_running(&self) -> bool;
    pub fn get_local_url(&self) -> Option<String>;
}
```

#### TunnelManager
- **Responsibility**: Cloudflare tunnel lifecycle management
- **Key Features**:
  - Automatic tunnel creation
  - Health monitoring with auto-recovery
  - Multiple tunnel types (quick, named)
  - SSL certificate management

```rust
pub struct TunnelManager {
    tunnel_process: Option<Child>,
    tunnel_url: Arc<RwLock<Option<String>>>,
    config_manager: Arc<ConfigManager>,
}

impl TunnelManager {
    pub async fn create_tunnel(&mut self) -> Result<String, MindLinkError>;
    pub async fn stop_tunnel(&mut self) -> Result<(), MindLinkError>;
    pub fn get_tunnel_url(&self) -> Option<String>;
    pub async fn health_check(&self) -> Result<bool, MindLinkError>;
}
```

## Data Flow Architecture

### Request Processing Flow

```
Client Request → Cloudflare Tunnel → Local Server → ChatGPT API
     ↑                                                    ↓
Response ← Tunnel Response ← Server Response ← ChatGPT Response
```

### Authentication Flow

```
1. User clicks "Login & Serve"
2. AuthManager generates OAuth URL with PKCE
3. System browser opens to ChatGPT login
4. User authenticates with ChatGPT
5. ChatGPT redirects with auth code
6. AuthManager exchanges code for tokens
7. Tokens stored securely in OS keychain
8. Background token refresh scheduled
```

### Service Startup Flow

```
1. AuthManager validates existing tokens
2. ServerManager starts HTTP server on configured port
3. TunnelManager creates Cloudflare tunnel
4. System tray updates to "Connected" status
5. Dashboard becomes available at local URL
6. Public API accessible via tunnel URL
```

## Security Architecture

### Authentication Security
- **OAuth2 with PKCE**: Prevents authorization code interception
- **Secure Storage**: OS-native credential management (Keychain/Credential Manager)
- **Token Rotation**: Automatic refresh prevents token stagnation
- **Session Validation**: Regular token validation against ChatGPT

### Network Security
- **TLS Everywhere**: All connections use HTTPS/TLS 1.3
- **Certificate Pinning**: Prevents man-in-the-middle attacks
- **CORS Policy**: Controlled cross-origin access
- **Rate Limiting**: Built-in protection against abuse

### Application Security
- **Memory Safety**: Rust's ownership model prevents memory vulnerabilities
- **Input Validation**: All external inputs validated and sanitized
- **Principle of Least Privilege**: Minimal system permissions required
- **Sandboxed Execution**: Tauri security model restricts system access

## Performance Characteristics

### Latency Targets
- **Local API**: < 5ms overhead
- **Tunnel API**: < 50ms additional latency
- **Authentication**: < 2s for token refresh
- **Tunnel Creation**: < 30s for initial setup

### Throughput Capabilities
- **Concurrent Connections**: 100+ simultaneous clients
- **Request Rate**: 1000+ requests/minute
- **Streaming**: Full-duplex streaming support
- **Memory Usage**: < 50MB resident memory

### Scalability Considerations
- **Horizontal Scaling**: Multiple tunnel endpoints
- **Load Distribution**: Client-side load balancing
- **Resource Pooling**: Connection reuse and pooling
- **Caching Strategy**: Intelligent response caching

## Configuration Management

### Configuration Hierarchy
1. **Default Settings**: Hard-coded sensible defaults
2. **User Configuration**: Stored in OS-appropriate config directory
3. **Runtime Settings**: Modified through UI or API
4. **Environment Variables**: Override for advanced users

### Configuration Schema
```json
{
  "server": {
    "port": 3001,
    "host": "127.0.0.1",
    "cors_enabled": true
  },
  "tunnel": {
    "type": "quick",
    "custom_domain": null,
    "health_check_interval": 30
  },
  "auth": {
    "token_refresh_margin": 900,
    "auto_login": false
  },
  "ui": {
    "minimize_to_tray": true,
    "startup_behavior": "minimized"
  }
}
```

## Error Handling Strategy

### Error Categories
- **Authentication Errors**: Token expiry, OAuth failures
- **Network Errors**: Connection failures, timeout errors
- **Configuration Errors**: Invalid settings, file corruption
- **System Errors**: Permission denied, resource exhaustion

### Recovery Mechanisms
- **Automatic Retry**: Exponential backoff with jitter
- **Graceful Degradation**: Local-only mode when tunnel fails
- **User Guidance**: Actionable error messages with next steps
- **Health Monitoring**: Proactive issue detection and recovery

## Monitoring and Observability

### Logging Strategy
- **Structured Logging**: JSON format with correlation IDs
- **Log Levels**: Debug, Info, Warn, Error with appropriate filtering
- **Sensitive Data**: Automatic redaction of tokens and credentials
- **Log Rotation**: Automatic cleanup of old log files

### Health Metrics
- **Service Health**: Authentication, server, tunnel status
- **Performance Metrics**: Response time, error rate, throughput
- **Resource Usage**: Memory, CPU, network utilization
- **User Actions**: Feature usage and error patterns

## Deployment Architecture

### Build Process
- **Multi-Stage Build**: Optimized for size and performance
- **Cross-Compilation**: Windows, macOS, Linux from single codebase
- **Asset Bundling**: Static assets embedded in binary
- **Code Signing**: Platform-specific signing for security

### Distribution Strategy
- **GitHub Releases**: Automated release builds
- **Platform Packages**: MSI, DMG, DEB, AppImage formats
- **Auto-Updates**: Secure update mechanism with rollback
- **Version Management**: Semantic versioning with migration support

## Testing Architecture

### Test Strategy
- **Unit Tests**: 90%+ coverage of individual components
- **Integration Tests**: Manager interaction and workflow testing
- **End-to-End Tests**: Full user journey automation
- **Performance Tests**: Load testing and benchmarking

### Test Infrastructure
- **Mock Services**: HTTP servers for isolated testing
- **Temporary Environments**: Isolated file system for tests
- **Concurrent Testing**: Multi-threaded execution patterns
- **CI/CD Integration**: Automated testing on all platforms

## Future Architecture Considerations

### Planned Enhancements
- **Multi-Account Support**: Switch between ChatGPT accounts
- **Custom Model Integration**: Local LLM support (Ollama, LMStudio)
- **Plugin Architecture**: Extensible functionality system
- **Enterprise Features**: SSO, audit logging, compliance tools

### Scalability Roadmap
- **Microservice Architecture**: Component separation for scaling
- **Container Deployment**: Docker/Kubernetes support
- **Cloud Integration**: Managed tunnel services
- **API Analytics**: Advanced usage analytics and billing

This architecture enables MindLink to provide enterprise-grade reliability while maintaining the simplicity and performance expected by individual developers and teams.