# MindLink User Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [System Requirements](#system-requirements)
3. [Installation](#installation)
4. [First Time Setup](#first-time-setup)
5. [Using MindLink](#using-mindlink)
6. [Dashboard Overview](#dashboard-overview)
7. [Configuration](#configuration)
8. [Troubleshooting](#troubleshooting)
9. [Advanced Usage](#advanced-usage)
10. [Security & Privacy](#security--privacy)

## Getting Started

MindLink transforms your ChatGPT Plus/Pro subscription into a local API server that's compatible with OpenAI's API format. This means you can use your existing ChatGPT account with any application that supports OpenAI, while maintaining full control over your data and connections.

### What You'll Need

- **ChatGPT Plus or Pro subscription** (required)
- **Desktop computer** running Windows 10+, macOS 10.15+, or Linux
- **Internet connection** for initial setup and API requests
- **Web browser** for authentication (any modern browser)

### What MindLink Does

- Creates a local API server on your computer
- Authenticates with your ChatGPT account using secure OAuth2
- Creates secure public tunnels via Cloudflare (optional)
- Provides OpenAI-compatible endpoints for seamless integration
- Monitors connection health and automatically recovers from errors

## System Requirements

### Minimum Requirements

- **Operating System**: 
  - Windows 10 or later
  - macOS 10.15 (Catalina) or later
  - Linux (Ubuntu 18.04+ or equivalent)
- **Memory**: 512MB RAM
- **Storage**: 100MB available disk space
- **Network**: Broadband internet connection

### Recommended Requirements

- **Memory**: 1GB RAM or more
- **Storage**: 500MB available disk space
- **Network**: Stable broadband connection (10+ Mbps)

### Prerequisites

- **ChatGPT Plus/Pro Account**: You must have an active paid ChatGPT subscription
- **Modern Browser**: Chrome, Firefox, Safari, or Edge (for authentication)

## Installation

### Option 1: Pre-built Installers (Recommended)

1. **Download the installer** from [GitHub Releases](https://github.com/yourusername/mindlink/releases)

2. **Choose your platform**:
   - **Windows**: Download `MindLink_x.x.x_x64_en-US.msi`
   - **macOS**: Download `MindLink_x.x.x_x64.dmg`
   - **Linux**: Download `mindlink_x.x.x_amd64.deb` or `mindlink-x.x.x.AppImage`

3. **Install the application**:

   **Windows:**
   ```cmd
   # Run as administrator
   msiexec /i MindLink_1.0.0_x64_en-US.msi
   ```
   Or double-click the installer and follow the setup wizard.

   **macOS:**
   ```bash
   # Open the DMG file
   open MindLink_1.0.0_x64.dmg
   # Drag MindLink to Applications folder
   ```

   **Linux (DEB package):**
   ```bash
   sudo dpkg -i mindlink_1.0.0_amd64.deb
   sudo apt-get install -f  # Fix any dependency issues
   ```

   **Linux (AppImage):**
   ```bash
   chmod +x mindlink-1.0.0.AppImage
   ./mindlink-1.0.0.AppImage
   ```

### Option 2: Build from Source

If you prefer to build from source, see the [Development Guide](DEVELOPMENT.md) for complete instructions.

## First Time Setup

### 1. Launch MindLink

After installation, launch MindLink:

- **Windows**: Start Menu → MindLink
- **macOS**: Applications → MindLink
- **Linux**: Applications menu or run `mindlink` in terminal

### 2. System Tray Integration

MindLink will minimize to your system tray on startup. Look for the MindLink icon:

![System Tray Icon](../assets/tray-icon.png)

The icon color indicates status:
- 🟢 **Green**: Connected and serving
- 🔵 **Blue**: Connecting or authenticating
- 🟡 **Yellow**: Authenticated but not serving
- 🔴 **Red**: Error or disconnected
- ⚪ **Gray**: Not authenticated

### 3. Initial Authentication

1. **Right-click** the MindLink tray icon
2. **Click** "Login & Serve"
3. Your **default browser** will open to the ChatGPT login page
4. **Sign in** with your ChatGPT Plus/Pro account credentials
5. **Allow** MindLink to access your account
6. The browser will show a **success message** and redirect back
7. MindLink will display a **notification** confirming successful authentication

### 4. Service Activation

After successful authentication:

1. MindLink automatically starts the **local API server**
2. A **Cloudflare tunnel** is created for public access
3. You'll receive a **desktop notification** with your API endpoints
4. The **tray icon** turns green indicating active service

## Using MindLink

### Accessing Your API

Once MindLink is running, you have access to two endpoints:

#### Local Endpoint (Private)
```
http://localhost:3001/v1
```
- Only accessible from your computer
- Fastest performance (no network latency)
- Perfect for local development

#### Public Endpoint (Tunnel)
```
https://your-unique-id.trycloudflare.com/v1
```
- Accessible from anywhere on the internet
- Secured with HTTPS encryption
- Perfect for mobile apps or remote access
- URL is displayed in system tray menu

### Getting Your API URL

**Method 1: System Tray Menu**
1. Right-click the MindLink tray icon
2. Click "Copy API URL"
3. The public tunnel URL is copied to your clipboard

**Method 2: Desktop Notification**
- When service starts, a notification shows both URLs
- Click the notification to copy the public URL

**Method 3: Dashboard**
1. Right-click tray icon → "Open Dashboard"
2. URLs are displayed prominently on the main page

### Making Your First API Call

**Using cURL:**
```bash
# Replace with your actual tunnel URL
MINDLINK_URL="https://your-unique-id.trycloudflare.com"

curl "$MINDLINK_URL/v1/chat/completions" \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-5",
    "messages": [
      {"role": "user", "content": "Hello! Can you help me get started with the MindLink API?"}
    ]
  }'
```

**Using Python:**
```python
from openai import OpenAI

# Use your tunnel URL here
client = OpenAI(
    base_url="https://your-unique-id.trycloudflare.com/v1",
    api_key="any-string"  # Any string works - authentication is via ChatGPT
)

response = client.chat.completions.create(
    model="gpt-5",
    messages=[
        {"role": "user", "content": "Hello! This is my first MindLink API call."}
    ]
)

print(response.choices[0].message.content)
```

### Managing the Service

**Start Service:**
- Right-click tray icon → "Login & Serve"
- If already authenticated, just starts the service

**Stop Service:**
- Right-click tray icon → "Stop Serving"
- Stops API server and closes tunnel, but keeps app running

**Restart Service:**
- Right-click tray icon → "Restart Service"
- Quick restart without re-authentication

**Check Status:**
- Right-click tray icon → "Connection Status"
- Shows detailed health information

**Quit Application:**
- Right-click tray icon → "Quit MindLink"
- Fully exits the application

## Dashboard Overview

MindLink includes a comprehensive web dashboard accessible at `http://localhost:3001/dashboard`.

### Accessing the Dashboard

1. **Right-click** the MindLink tray icon
2. **Click** "Open Dashboard"
3. Your browser will open to the **Bifrost dashboard**

### Dashboard Features

#### 1. **Status Overview** (Home Page)
- **Service Health**: Current status of all components
- **API Endpoints**: Local and public URLs with copy buttons
- **Connection Info**: Authentication status and tunnel health
- **System Resources**: Memory and CPU usage monitoring

#### 2. **API Logs** Tab
- **Real-time Requests**: Live stream of API calls
- **Request Details**: Full request/response inspection
- **Filtering**: Search and filter by model, timestamp, or content
- **Export**: Download logs for analysis

#### 3. **Performance** Tab
- **Response Times**: Latency graphs and metrics
- **Throughput**: Requests per minute/hour
- **Error Rates**: Success vs failure analytics
- **Token Usage**: Consumption tracking

#### 4. **Configuration** Tab
- **Server Settings**: Port, host, CORS configuration
- **Tunnel Options**: Tunnel type and custom domains
- **AI Controls**: Reasoning effort and summary settings
- **Notifications**: Alert preferences and thresholds

#### 5. **Security** Tab
- **Authentication Status**: OAuth token information
- **Connection Logs**: Security events and access patterns
- **Rate Limiting**: Current limits and usage
- **Access Control**: API key requirements (optional)

### Dashboard Navigation Tips

- **Auto-refresh**: Most data updates automatically every 30 seconds
- **Keyboard Shortcuts**: Use `Ctrl+R` to manually refresh
- **Mobile Friendly**: Dashboard works on phones and tablets
- **Export Data**: Most views support CSV/JSON export
- **Dark Mode**: Automatically follows system theme

## Configuration

### Accessing Settings

**Method 1: System Tray**
1. Right-click MindLink tray icon
2. Click "Settings"

**Method 2: Dashboard**
1. Open dashboard
2. Click "Configuration" tab

### Server Configuration

#### Port and Host Settings
```json
{
  "server": {
    "port": 3001,           // Change if port is in use
    "host": "127.0.0.1",    // Usually leave as localhost
    "cors_enabled": true    // Enable for web browser access
  }
}
```

**Common Port Issues:**
- If port 3001 is busy, try 3002, 3003, etc.
- Ports below 1024 require administrator privileges
- Check firewall settings if external access fails

#### CORS Configuration
```json
{
  "server": {
    "cors_origins": ["*"],           // Allow all origins
    "cors_methods": ["GET", "POST"], // Allowed HTTP methods
    "cors_headers": ["*"]            // Allowed headers
  }
}
```

### Tunnel Configuration

#### Tunnel Types
- **Quick Tunnel**: Automatic, uses `trycloudflare.com` domain
- **Named Tunnel**: Requires Cloudflare account, custom domain possible

```json
{
  "tunnel": {
    "type": "quick",                    // or "named"
    "custom_domain": null,              // For named tunnels
    "health_check_interval": 30,        // Seconds between health checks
    "auto_restart": true                // Restart tunnel on failure
  }
}
```

#### Advanced Tunnel Settings
```json
{
  "tunnel": {
    "edge_locations": ["auto"],         // Geographic preference
    "protocol": "https",                // Always HTTPS for security
    "compression": true,                // Enable compression
    "no_tls_verify": false             // Don't disable unless necessary
  }
}
```

### Authentication Settings

```json
{
  "auth": {
    "token_refresh_margin": 900,        // Refresh 15 minutes early
    "auto_login": false,                // Prompt vs automatic login
    "remember_session": true,           // Persist across restarts
    "logout_on_exit": false             // Stay logged in when quitting
  }
}
```

### AI and Model Settings

```json
{
  "ai": {
    "default_model": "gpt-5",           // Default model for requests
    "reasoning_effort": "medium",       // low, medium, high
    "reasoning_summary": "auto",        // none, concise, auto, detailed
    "temperature": 0.7,                 // Default creativity level
    "max_tokens": 4000,                 // Default response length
    "stream_responses": true            // Enable streaming by default
  }
}
```

### UI and Notification Settings

```json
{
  "ui": {
    "minimize_to_tray": true,           // Hide window instead of closing
    "startup_behavior": "minimized",    // minimized, window, or tray
    "show_notifications": true,         // Desktop notifications
    "theme": "auto"                     // auto, light, or dark
  }
}
```

### Notification Configuration

```json
{
  "notifications": {
    "service_events": true,             // Service start/stop
    "connection_changes": true,         // Tunnel status changes
    "errors": true,                     // Error notifications
    "performance_alerts": false,        // High latency warnings
    "security_alerts": true            // Authentication issues
  }
}
```

## Troubleshooting

### Common Issues and Solutions

#### Authentication Problems

**Problem**: "Authentication required" error
```
Solutions:
1. Right-click tray → "Login & Serve"
2. Check ChatGPT subscription is active
3. Clear browser cache and retry
4. Restart MindLink application
```

**Problem**: OAuth flow gets stuck
```
Solutions:
1. Close all browser windows
2. Try incognito/private browsing mode
3. Disable browser extensions temporarily
4. Use a different browser
```

**Problem**: Token keeps expiring
```
Solutions:
1. Check system clock is accurate
2. Verify ChatGPT account hasn't been suspended
3. Check for conflicting ChatGPT sessions
4. Restart authentication process
```

#### Connection Issues

**Problem**: Cannot create tunnel
```
Solutions:
1. Check internet connectivity
2. Verify firewall allows outbound connections
3. Try different tunnel type in settings
4. Restart MindLink service
```

**Problem**: Local server won't start
```
Solutions:
1. Check if port is already in use:
   Windows: netstat -an | findstr :3001
   Mac/Linux: lsof -i :3001
2. Change port in settings
3. Run MindLink as administrator (Windows)
4. Check antivirus isn't blocking
```

**Problem**: Tunnel URL not accessible
```
Solutions:
1. Wait 30-60 seconds for propagation
2. Check Connection Status for errors
3. Try restarting tunnel service
4. Verify URL copied correctly (no extra characters)
```

#### Performance Issues

**Problem**: Slow response times
```
Solutions:
1. Check internet connection speed
2. Lower reasoning_effort to "low" in settings
3. Reduce max_tokens for shorter responses
4. Use local endpoint instead of tunnel
5. Check dashboard for error patterns
```

**Problem**: High memory usage
```
Solutions:
1. Restart MindLink service
2. Clear log files from dashboard
3. Disable detailed logging if enabled
4. Check for memory leaks in dashboard
```

**Problem**: Requests timing out
```
Solutions:
1. Increase timeout in client application
2. Check ChatGPT service status
3. Try smaller request size
4. Verify tunnel connectivity
```

#### API Errors

**Problem**: "Model not available" error
```
Solutions:
1. Check available models in /v1/models endpoint
2. Verify ChatGPT subscription includes model
3. Use "gpt-4" instead of "gpt-5" if needed
4. Check model spelling in request
```

**Problem**: Rate limiting errors
```
Solutions:
1. Reduce request frequency
2. Check ChatGPT usage limits
3. Implement exponential backoff
4. Monitor usage in dashboard
```

### Getting Detailed Logs

#### Enable Debug Logging

**Windows (Command Prompt):**
```cmd
set RUST_LOG=debug
mindlink.exe
```

**Windows (PowerShell):**
```powershell
$env:RUST_LOG="debug"
./mindlink.exe
```

**macOS/Linux:**
```bash
RUST_LOG=debug ./mindlink
```

#### Component-Specific Logging
```bash
# Auth issues
RUST_LOG=mindlink::managers::auth_manager=debug ./mindlink

# Server issues  
RUST_LOG=mindlink::managers::server_manager=debug ./mindlink

# Tunnel issues
RUST_LOG=mindlink::managers::tunnel_manager=debug ./mindlink

# All networking
RUST_LOG=mindlink::managers=debug ./mindlink
```

#### Log File Locations

**Windows:**
```
%APPDATA%/com.mindlink.mindlink/logs/
```

**macOS:**
```
~/Library/Application Support/com.mindlink.mindlink/logs/
```

**Linux:**
```
~/.local/share/com.mindlink.mindlink/logs/
```

### System Diagnostics

#### Connection Health Check
1. Right-click tray → "Connection Status"
2. Look for red indicators
3. Click "Run Diagnostics" for detailed report

#### API Test
```bash
# Test local endpoint
curl http://localhost:3001/health

# Test public endpoint  
curl https://your-tunnel.trycloudflare.com/health
```

Expected response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "services": {
    "auth": "authenticated", 
    "server": "running",
    "tunnel": "connected"
  }
}
```

### When to Contact Support

Contact support if you experience:
- Persistent authentication failures after trying all solutions
- Data corruption or loss
- Security concerns or suspicious activity
- Crashes or stability issues
- Feature requests or enhancement suggestions

**Support Channels:**
- GitHub Issues: [Bug reports and feature requests](https://github.com/yourusername/mindlink/issues)
- GitHub Discussions: [Questions and community help](https://github.com/yourusername/mindlink/discussions)
- Email: [support@mindlink.dev](mailto:support@mindlink.dev) for enterprise inquiries

## Advanced Usage

### Custom Model Routing

Configure MindLink to route different request types to specific models:

```json
{
  "routing": {
    "rules": [
      {
        "condition": "short_response",
        "model": "gpt-3.5-turbo",
        "criteria": {"max_tokens": {"<": 100}}
      },
      {
        "condition": "reasoning_task", 
        "model": "gpt-5",
        "criteria": {"reasoning_effort": {">=": "medium"}}
      }
    ]
  }
}
```

### Multiple MindLink Instances

Run multiple MindLink instances for different accounts or purposes:

```bash
# Instance 1 (default port 3001)
./mindlink

# Instance 2 (port 3002)  
MINDLINK_PORT=3002 MINDLINK_CONFIG_DIR=~/.mindlink2 ./mindlink
```

### Custom Domain Setup

For named tunnels with custom domains:

1. **Create Cloudflare account** and add your domain
2. **Generate tunnel credentials** in Cloudflare dashboard  
3. **Configure MindLink** with tunnel name and credentials:

```json
{
  "tunnel": {
    "type": "named",
    "name": "my-mindlink-tunnel",
    "custom_domain": "api.mydomain.com",
    "credentials_file": "/path/to/tunnel-credentials.json"
  }
}
```

### Load Balancing

Distribute requests across multiple tunnels:

```json
{
  "load_balancer": {
    "enabled": true,
    "strategy": "round_robin",  // or "least_connections"
    "tunnels": [
      "https://tunnel1.trycloudflare.com",
      "https://tunnel2.trycloudflare.com" 
    ],
    "health_check": true
  }
}
```

### Custom Headers and Middleware

Add custom headers to all requests:

```json
{
  "middleware": {
    "custom_headers": {
      "X-API-Source": "MindLink",
      "X-User-Agent": "MindLink/1.0.0"
    },
    "rate_limiting": {
      "enabled": true,
      "requests_per_minute": 100
    }
  }
}
```

### Webhook Integration

Configure webhooks for events:

```json
{
  "webhooks": {
    "enabled": true,
    "events": ["service_start", "service_stop", "auth_refresh", "error"],
    "url": "https://your-server.com/mindlink-webhook",
    "secret": "your-webhook-secret"
  }
}
```

### Prometheus Metrics

Enable metrics collection:

```json
{
  "metrics": {
    "enabled": true,
    "endpoint": "/metrics",
    "collect_detailed": true
  }
}
```

Access metrics at: `http://localhost:3001/metrics`

## Security & Privacy

### Data Privacy

**What MindLink DOES:**
- Securely store authentication tokens locally
- Route API requests between your applications and ChatGPT
- Log request metadata for monitoring (no content by default)
- Create encrypted tunnels for public access

**What MindLink DOES NOT:**
- Store or transmit your conversation data
- Share data with third parties
- Collect telemetry or analytics without consent
- Access your personal files or data

### Local Data Storage

**Configuration Files:**
- Location: OS-specific application data directories
- Content: Settings, preferences, tunnel information
- Security: Standard file system permissions

**Authentication Tokens:**
- Location: OS-specific secure storage (Keychain/Credential Manager)
- Content: OAuth tokens for ChatGPT access
- Security: Encrypted using OS-native security features

**Logs:**
- Location: Application data directory
- Content: Service events, error messages, request metadata
- Retention: Configurable, default 30 days

### Security Features

**Authentication:**
- OAuth2 with PKCE for secure ChatGPT authentication
- No password storage - uses secure token exchange
- Automatic token refresh with secure rotation

**Network Security:**
- All external communications use HTTPS/TLS 1.3
- Certificate validation and pinning
- No unencrypted data transmission

**Application Security:**
- Memory-safe Rust implementation
- Sandboxed execution environment
- Minimal system permissions required

### Privacy Controls

**Disable Logging:**
```json
{
  "logging": {
    "enabled": false,
    "level": "error_only",
    "log_requests": false,
    "log_responses": false
  }
}
```

**Clear All Data:**
1. Right-click tray → "Settings" → "Privacy"
2. Click "Clear All Data"
3. Confirm data deletion
4. Restart MindLink

**Export Data:**
1. Dashboard → "Security" tab
2. Click "Export Data" 
3. Download JSON file with your settings and logs
4. Use for backup or migration

### Compliance

**GDPR Compliance:**
- Right to data export and deletion
- Minimal data collection
- Local data processing
- No third-party data sharing

**Enterprise Security:**
- Audit logging capabilities
- Role-based access control (planned)
- SSO integration (roadmap)
- Compliance reporting tools

### Security Best Practices

**For Personal Use:**
- Keep MindLink updated to latest version
- Use strong ChatGPT account security (2FA)
- Regularly rotate authentication tokens (logout/login)
- Monitor dashboard for unusual activity

**For Team/Enterprise Use:**
- Deploy on dedicated machines or VMs
- Use named tunnels with custom domains
- Enable comprehensive audit logging
- Implement network monitoring
- Regular security assessments

This completes the comprehensive user guide for MindLink. Users now have all the information needed to successfully install, configure, and use MindLink for their OpenAI API integration needs.