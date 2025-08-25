# MindLink 🔗

**Production-Ready Local LLM API Bridge with Cloudflare Tunneling**

[![Build Status](https://github.com/yourusername/mindlink/actions/workflows/release.yml/badge.svg)](https://github.com/yourusername/mindlink/actions/workflows/release.yml)
[![Code Coverage](https://codecov.io/gh/yourusername/mindlink/branch/main/graph/badge.svg)](https://codecov.io/gh/yourusername/mindlink)
[![Rust Version](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MindLink is a high-performance desktop application built with **Rust** and **Tauri** that creates an OpenAI-compatible API server powered by your ChatGPT Plus/Pro account. It automatically creates secure public tunnels via Cloudflare, enabling seamless integration with third-party applications and development tools.

## Key Features

- 🦀 **Rust-Powered Backend** - High-performance, memory-safe core engine
- 🔐 **Enterprise Security** - OAuth2 authentication with automatic token refresh
- 🌐 **Global API Access** - Secure Cloudflare tunnel creation with custom domains
- ⚡ **OpenAI Compatible** - Drop-in replacement for OpenAI API endpoints
- 🎯 **Native System Integration** - Cross-platform system tray with real-time status
- 📊 **Advanced Monitoring** - Comprehensive health checks and error recovery
- 🎛️ **Professional Dashboard** - Bifrost web interface for management and analytics
- ⚙️ **Enterprise Configuration** - Fine-tuned control over AI reasoning and performance
- 🔔 **Intelligent Notifications** - Smart desktop alerts with actionable insights
- 🚀 **Auto-Updates** - Seamless application updates with rollback support

## Quick Start

### System Requirements

- **Paid ChatGPT account** (Plus or Pro subscription required)
- **Operating System**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 18.04+)
- **Memory**: 512MB RAM minimum, 1GB recommended
- **Network**: Internet connection for initial setup and API operations

### Installation Options

#### Option 1: Download Pre-built Binaries (Recommended)

1. **Download the latest release:**
   - Visit our [Releases page](https://github.com/yourusername/mindlink/releases)
   - Download the appropriate installer for your platform:
     - **Windows**: `MindLink_x.x.x_x64_en-US.msi`
     - **macOS**: `MindLink_x.x.x_x64.dmg`  
     - **Linux**: `mindlink_x.x.x_amd64.deb` or `mindlink-x.x.x.AppImage`

2. **Install the application:**
   - **Windows**: Run the `.msi` installer as administrator
   - **macOS**: Open the `.dmg` and drag MindLink to Applications
   - **Linux**: Install via package manager or run the AppImage directly

3. **First launch:**
   - Launch MindLink from your applications menu or desktop
   - Look for the MindLink icon in your system tray
   - Right-click the tray icon and select "Login & Serve"

#### Option 2: Build from Source

1. **Prerequisites for building:**
   ```bash
   # Install Rust (latest stable)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Install Node.js 18+ and npm
   # Visit https://nodejs.org/ for installation instructions
   
   # Install Tauri CLI
   npm install -g @tauri-apps/cli
   ```

2. **Clone and build:**
   ```bash
   git clone https://github.com/yourusername/mindlink.git
   cd mindlink
   npm install
   npm run tauri build
   ```

3. **Run the built application:**
   - Navigate to `src-tauri/target/release/bundle/`
   - Install the appropriate package for your platform

### Getting Started

1. **Initial Setup:**
   - After installation, MindLink will appear in your system tray
   - Right-click the tray icon to access the menu
   - Click "Login & Serve" to begin authentication

2. **Authentication:**
   - Your default browser will open to the ChatGPT login page
   - Sign in with your ChatGPT Plus/Pro account
   - Grant permissions to MindLink
   - The browser will redirect back to confirm successful authentication

3. **Service Activation:**
   - MindLink will automatically start the local API server
   - A Cloudflare tunnel will be created for public access
   - You'll receive a notification with your API endpoints

## API Usage

Once MindLink is running, you have access to two API endpoints:

- **Local API**: `http://localhost:3001/v1` (configurable port)
- **Public API**: Secure Cloudflare tunnel URL (displayed in notifications)
- **Bifrost Dashboard**: `http://localhost:3001/dashboard` (management interface)

### OpenAI SDK Integration

MindLink provides a fully compatible OpenAI API interface. Simply point your existing OpenAI client to your MindLink endpoint:

```python
from openai import OpenAI

# Use your MindLink tunnel URL (copy from system tray menu)
client = OpenAI(
    base_url="https://your-unique-tunnel.trycloudflare.com/v1",
    api_key="any-string"  # API key is ignored - authentication via ChatGPT OAuth
)

# Standard OpenAI API calls work seamlessly
response = client.chat.completions.create(
    model="gpt-5",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Explain quantum computing in simple terms."}
    ],
    stream=True,  # Streaming is fully supported
    max_tokens=1000
)

for chunk in response:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")
```

### Direct HTTP API

```bash
# Get your tunnel URL from the system tray menu "Copy API URL"
MINDLINK_URL="https://your-unique-tunnel.trycloudflare.com"

# Standard chat completions
curl "$MINDLINK_URL/v1/chat/completions" \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-5",
    "messages": [{"role": "user", "content": "Write a Python function to calculate fibonacci numbers"}],
    "stream": false,
    "temperature": 0.7
  }'

# Streaming responses
curl "$MINDLINK_URL/v1/chat/completions" \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-5", 
    "messages": [{"role": "user", "content": "Explain machine learning"}],
    "stream": true
  }'
```

### Advanced API Features

```python
# Reasoning control (GPT-5 specific)
response = client.chat.completions.create(
    model="gpt-5",
    messages=[{"role": "user", "content": "Solve this math problem step by step: 2x + 5 = 13"}],
    extra_body={
        "reasoning_effort": "high",  # low, medium, high
        "reasoning_summary": "detailed"  # none, concise, auto, detailed
    }
)

# Function calling support
functions = [{
    "name": "get_weather",
    "description": "Get weather information",
    "parameters": {
        "type": "object",
        "properties": {
            "location": {"type": "string", "description": "City name"}
        },
        "required": ["location"]
    }
}]

response = client.chat.completions.create(
    model="gpt-5",
    messages=[{"role": "user", "content": "What's the weather in San Francisco?"}],
    functions=functions,
    function_call="auto"
)
```

## Supported Models

MindLink provides access to the latest ChatGPT models through an OpenAI-compatible interface:

- **`gpt-5`** - Latest GPT-5 model with advanced reasoning capabilities
- **`gpt-4o`** - GPT-4 Omni with vision and multimodal support  
- **`gpt-4`** - Standard GPT-4 for complex reasoning tasks
- **`gpt-3.5-turbo`** - Fast, efficient model for everyday tasks

*Note: Available models depend on your ChatGPT Plus/Pro subscription tier*

## Bifrost Dashboard

MindLink includes a comprehensive web-based management interface accessible at `http://localhost:3001/dashboard`:

### Dashboard Features

- **🔍 Real-Time Monitoring**: Live API request/response logs with filtering and search
- **📊 Performance Analytics**: Request latency, token usage, and error rate metrics
- **🎛️ Configuration Management**: Visual interface for all MindLink settings
- **🔌 Connection Status**: Real-time health monitoring of ChatGPT and tunnel connections
- **📈 Usage Statistics**: Detailed analytics on API usage patterns and model performance
- **🚨 Alert Center**: Centralized notification management and error reporting
- **🔐 Security Dashboard**: OAuth token status, authentication logs, and security events
- **⚙️ Advanced Settings**: Fine-tune performance, caching, and rate limiting

### Dashboard Interface

The Bifrost dashboard provides:

1. **Status Overview**: Current service health, active connections, and system resources
2. **API Logs**: Real-time stream of API requests with detailed request/response inspection
3. **Configuration Panel**: Modify settings without restarting the application
4. **Performance Metrics**: Graphs showing response times, throughput, and error rates
5. **Tunnel Management**: Monitor Cloudflare tunnel status and manage public URLs
6. **Authentication Panel**: OAuth token management and re-authentication controls

## Features Deep Dive

### Enterprise-Grade Authentication

MindLink implements a robust OAuth2 authentication system:

- **Secure OAuth Flow**: Industry-standard OpenAI authentication with PKCE
- **Automatic Token Refresh**: Seamless token renewal with zero downtime
- **Credential Security**: Encrypted storage in platform-specific secure storage
- **Session Management**: Automatic session validation and recovery
- **Multi-Account Support**: Switch between different ChatGPT accounts (coming soon)

### Advanced Cloudflare Tunneling

Production-ready tunnel management with enterprise features:

- **Quick Tunnels**: Zero-configuration public URLs via `trycloudflare.com`
- **Named Tunnels**: Custom domains with Cloudflare account integration
- **Load Balancing**: Automatic failover between multiple tunnel endpoints
- **Health Monitoring**: Continuous tunnel health checks with automatic recovery
- **SSL Termination**: Automatic HTTPS with Let's Encrypt certificates
- **Custom Headers**: Support for custom authentication and routing headers

### OpenAI API Compatibility

100% compatible OpenAI API implementation:

- **Chat Completions** (`/v1/chat/completions`) - Full feature parity including streaming
- **Model Listing** (`/v1/models`) - Dynamic model discovery based on subscription
- **Function Calling** - Native support for tools and function execution
- **Streaming Responses** - Real-time response streaming with proper SSE formatting
- **Vision Support** - Image understanding and multimodal interactions
- **Advanced Reasoning** - GPT-5 reasoning controls with detailed thinking summaries

### Professional Configuration

Comprehensive settings accessible via system tray and dashboard:

**AI & Reasoning Controls:**
- **Reasoning Effort**: `low`, `medium`, `high` - Control depth of AI reasoning
- **Reasoning Summaries**: `none`, `concise`, `auto`, `detailed` - Format of thinking output
- **Model Selection**: Automatic or manual model routing based on request context
- **Response Optimization**: Fine-tune for speed vs. quality based on use case

**Network & Performance:**
- **Custom Port/Host**: Configure local server binding and port selection
- **Request Limits**: Set maximum request size and concurrent connection limits
- **CORS Configuration**: Fine-grained cross-origin request control
- **Caching Strategy**: Intelligent response caching for improved performance

**Security & Privacy:**
- **API Key Validation**: Optional API key enforcement for additional security
- **Rate Limiting**: Protect against abuse with configurable rate limits
- **Request Logging**: Control what gets logged for privacy and compliance
- **Tunnel Security**: Advanced tunnel authentication and access controls

**Monitoring & Alerts:**
- **Health Check Intervals**: Customize monitoring frequency and sensitivity
- **Error Thresholds**: Configure when to trigger alerts and notifications
- **Performance Targets**: Set SLA targets for response time and availability
- **Notification Channels**: Desktop, email, and webhook alert delivery

## System Tray Interface

MindLink runs as a native system tray application with intelligent context menus and real-time status indicators.

### Tray Menu Options

Right-click the MindLink system tray icon to access:

- **🚀 Login & Serve** - Initiate OAuth authentication and start API service
- **⏹️ Stop Serving** - Gracefully shutdown API service and tunnel
- **📊 Connection Status** - Detailed health information and diagnostic data
- **⚙️ Settings** - Open configuration window with advanced options
- **📋 Copy API URL** - Copy public tunnel URL to clipboard for easy sharing
- **🎛️ Open Dashboard** - Launch Bifrost web interface in default browser
- **🔄 Restart Service** - Quick restart without full application restart
- **📱 Show Notifications** - Toggle desktop notification preferences
- **❌ Quit MindLink** - Safely exit application

### Dynamic Status Indicators

The tray icon changes color and tooltip to reflect current system status:

- **🟢 Connected & Serving** - All systems operational, API accessible
- **🔵 Connecting** - Authentication in progress or tunnel establishing
- **🟡 Authenticated** - Logged in but service not started
- **🔴 Error State** - Connection issues, authentication failures, or service errors
- **⚪ Disconnected** - Service stopped, not authenticated
- **🔄 Updating** - Application update in progress

### Smart Notifications

MindLink provides contextual desktop notifications for:

- **Service Events**: Successful startup, authentication status, service shutdown
- **Connection Changes**: Tunnel URL updates, connection restored, network issues
- **Error Recovery**: Automatic reconnection attempts, token refresh, error resolution
- **Security Alerts**: Authentication failures, unusual activity, token expiration warnings
- **Performance Issues**: High latency, rate limiting, service degradation

## Error Handling & Monitoring

MindLink implements enterprise-grade error handling and monitoring:

### Automatic Recovery Systems

- **Intelligent Reconnection**: Exponential backoff with jitter for connection failures
- **Proactive Token Refresh**: Automatic OAuth token renewal 15 minutes before expiration
- **Health Monitoring**: Comprehensive system health checks every 30 seconds
- **Circuit Breaker**: Automatic service isolation during upstream failures
- **Graceful Degradation**: Local-only mode when tunnel services are unavailable

### Comprehensive Error Reporting

- **Structured Logging**: JSON-formatted logs with correlation IDs and context
- **Error Classification**: Automatic categorization of errors by severity and type
- **User-Friendly Messages**: Technical errors translated to actionable user guidance
- **Diagnostic Information**: Detailed system state capture for troubleshooting
- **Error Recovery Suggestions**: Contextual recommendations for issue resolution

### Troubleshooting Guide

**Authentication Issues:**
1. **"Authentication required" errors**: Click tray → "Login & Serve" to re-authenticate
2. **Token expired**: Service will auto-refresh; if persistent, check ChatGPT subscription status
3. **OAuth flow stuck**: Clear browser cache and retry authentication process

**Connection Problems:**
1. **"Failed to create tunnel"**: Check internet connectivity and firewall settings
2. **Port conflicts**: Modify port in Settings → Server → Custom Port
3. **Firewall blocking**: Ensure port 3001 (or custom) is allowed in firewall rules
4. **Service unreachable**: Verify local server status in Connection Status panel

**Performance Issues:**
1. **Slow responses**: Lower reasoning effort in Settings → AI Controls
2. **High latency**: Check tunnel health and consider switching tunnel regions
3. **Rate limiting**: Verify ChatGPT subscription limits and usage patterns
4. **Memory usage**: Monitor system resources and restart service if needed

### Debug Mode

Enable comprehensive debugging:

```bash
# Windows (PowerShell)
$env:RUST_LOG="debug"; ./mindlink.exe

# macOS/Linux
RUST_LOG=debug ./mindlink

# Debug specific components
RUST_LOG=mindlink::server_manager=debug,mindlink::tunnel_manager=debug ./mindlink
```

## Architecture Overview

MindLink is built with a modern, production-ready architecture:

### Technology Stack

- **🦀 Rust Backend**: High-performance, memory-safe core engine with Tauri framework
- **⚡ Axum Web Server**: Async HTTP server for OpenAI API compatibility
- **🌐 Tauri Frontend**: Native desktop UI with web technologies
- **☁️ Cloudflare Integration**: Enterprise tunnel management and SSL termination
- **🔐 OAuth2 Security**: Industry-standard authentication with PKCE flow
- **📊 Telemetry**: Comprehensive monitoring and observability

### Project Structure

```
mindlink/
├── src-tauri/                    # Rust backend (production code)
│   ├── src/
│   │   ├── main.rs              # Application entry point and tray setup
│   │   ├── managers/            # Core business logic modules
│   │   │   ├── auth_manager.rs  # OAuth2 authentication handling
│   │   │   ├── server_manager.rs # Axum web server and API endpoints
│   │   │   ├── tunnel_manager.rs # Cloudflare tunnel management
│   │   │   ├── config_manager.rs # Configuration and settings
│   │   │   └── bifrost_manager.rs # Dashboard and monitoring
│   │   ├── commands/            # Tauri command handlers
│   │   ├── tests/               # Comprehensive test suite
│   │   └── error.rs            # Structured error handling
│   ├── Cargo.toml              # Rust dependencies and configuration
│   └── tauri.conf.json         # Tauri application configuration
├── ui/                          # Frontend interface files
│   └── settings.html           # Settings configuration interface
├── bifrost-ui/                  # Dashboard static assets
├── scripts/                     # Build and deployment automation
├── .github/workflows/          # CI/CD pipeline configuration
└── package.json               # Node.js build dependencies
```

### Security Architecture

- **Zero-Trust Design**: All components validate inputs and authenticate requests
- **Credential Isolation**: OAuth tokens stored in OS-specific secure storage
- **Encrypted Communication**: All API traffic uses HTTPS with certificate pinning
- **Principle of Least Privilege**: Each component has minimal required permissions
- **Audit Logging**: Comprehensive security event logging and monitoring

## Security & Privacy

MindLink implements enterprise-grade security and privacy protections:

### Data Protection
- **Local-First Architecture**: All authentication tokens and user data stored locally only
- **Zero Data Collection**: MindLink never collects, stores, or transmits user conversation data
- **Encrypted Storage**: Credentials stored using OS-native secure storage (Keychain/Credential Manager)
- **Memory Safety**: Rust backend eliminates entire classes of memory vulnerabilities
- **Secure Communication**: All API requests use TLS 1.3 with certificate pinning

### Privacy Guarantees
- **No Telemetry**: Optional anonymous usage analytics (disabled by default)
- **Local Processing**: All API routing and request handling occurs locally
- **Conversation Privacy**: User conversations never pass through MindLink servers
- **Audit Trail**: Complete local logging with user-controlled retention policies
- **GDPR Compliant**: Right to data portability and deletion built-in

### Security Features
- **Minimal Attack Surface**: Only necessary network ports exposed
- **Sandboxed Execution**: Tauri security model with restricted system access
- **Regular Updates**: Automatic security updates with cryptographic verification
- **Vulnerability Disclosure**: Responsible security reporting process
- **Third-Party Audits**: Regular security assessments and penetration testing

## Support & Community

### Getting Help

- **📖 Documentation**: Comprehensive guides at [docs.mindlink.dev](https://docs.mindlink.dev)
- **🐛 Bug Reports**: [GitHub Issues](https://github.com/yourusername/mindlink/issues) for technical problems
- **💬 Community**: [GitHub Discussions](https://github.com/yourusername/mindlink/discussions) for questions and ideas
- **📧 Direct Support**: [support@mindlink.dev](mailto:support@mindlink.dev) for enterprise inquiries

### Development & Testing

MindLink maintains enterprise-grade code quality with comprehensive testing and coverage reporting:

#### Running Tests Locally

```bash
# Navigate to the Rust workspace
cd src-tauri

# Run all unit tests
cargo test

# Run integration tests
cargo test --test '*'

# Run specific test module
cargo test auth_manager_tests
```

#### Code Coverage

We maintain **≥80%** code coverage with automated reporting:

```bash
# Install coverage tool (first time only)
cargo install cargo-tarpaulin

# Generate comprehensive coverage report
./scripts/generate-coverage.sh

# CI-compatible coverage (XML + HTML)
cd src-tauri
cargo tarpaulin --config ci --out xml --out html
```

**Coverage Reports:**
- 📊 **Interactive HTML**: `coverage/tarpaulin-report.html`  
- 🤖 **CI Integration**: `coverage/cobertura.xml` (Codecov compatible)
- 📋 **JSON Data**: `coverage/tarpaulin-report.json` (programmatic access)

**Coverage Standards:**
- **Minimum**: 80% line coverage required for CI/CD
- **Target**: 85% line coverage for production releases  
- **Critical Code**: 100% coverage required for authentication, security, and API endpoints
- **Exclusions**: Test files, main.rs, and debug-only code are excluded from coverage requirements

#### Quality Gates

All code must pass these automated quality checks:

- ✅ **Zero Warnings**: `cargo clippy --all-targets --all-features -- -D warnings`
- ✅ **Formatting**: `cargo fmt --check` 
- ✅ **Test Coverage**: ≥80% line coverage with `cargo-tarpaulin`
- ✅ **Security Audit**: `cargo audit` (no known vulnerabilities)
- ✅ **Dependency Check**: `cargo udeps` (no unused dependencies)

### Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details on:
- Development environment setup
- Code style and quality standards
- Testing requirements
- Pull request process

### Roadmap

Upcoming features in development:
- **Multi-Account Support**: Switch between different ChatGPT accounts
- **Custom Model Integration**: Support for local LLM models (Ollama, LMStudio)
- **Enterprise SSO**: SAML/OIDC integration for enterprise deployments
- **API Analytics**: Advanced usage analytics and billing integration
- **Plugin System**: Extensible architecture for custom integrations

## Acknowledgments

MindLink builds on the shoulders of giants:

- **[Tauri](https://tauri.app/)** - Secure, fast, and lightweight desktop application framework
- **[Axum](https://github.com/tokio-rs/axum)** - Modern, performant web framework for Rust
- **[Cloudflare](https://www.cloudflare.com/)** - Global tunnel infrastructure and SSL termination
- **[Tokio](https://tokio.rs/)** - Asynchronous runtime for scalable network applications

Special thanks to the Rust community for creating an ecosystem that enables building secure, high-performance applications.

## License

MIT License - see [LICENSE](LICENSE) file for complete terms.

## Legal Notice

**⚠️ Important Disclaimers:**

- This application requires a paid ChatGPT Plus or Pro subscription
- MindLink is not affiliated with, endorsed by, or sponsored by OpenAI
- Users are responsible for compliance with OpenAI's Terms of Service
- Use of this software is subject to ChatGPT's usage policies and rate limits
- Ensure your use case complies with OpenAI's usage policies before deployment

---

*Built with ❤️ and 🦀 by the MindLink team*