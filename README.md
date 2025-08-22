# MindLink 🔗

**Local LLM API bridge with Cloudflare tunneling**

MindLink is a desktop application that creates an OpenAI-compatible API server powered by your ChatGPT Plus/Pro account. It automatically creates public tunnels via Cloudflare, allowing third-party applications to access your local LLMs through a standardized API interface.

## Features

- 🔐 **Secure Authentication** - Uses ChatGPT OAuth for secure access
- 🌐 **Public API Access** - Automatic Cloudflare tunnel creation  
- ⚡ **OpenAI Compatible** - Drop-in replacement for OpenAI API
- 🎯 **System Tray App** - Runs minimized with status monitoring
- 📊 **Health Monitoring** - Automatic reconnection and error handling
- ⚙️ **Configurable** - Adjustable reasoning effort, summaries, and more
- 🔔 **Smart Notifications** - Desktop alerts for connection issues

## Quick Start

### Prerequisites

- **Paid ChatGPT account** (Plus or Pro)
- **Node.js 18+** and npm
- **Windows, macOS, or Linux**

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/mindlink.git
   cd mindlink
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Start the application:**
   ```bash
   npm start
   ```

4. **Login and serve:**
   - Click "Login & Serve" in the system tray
   - Complete authentication in your browser
   - Your API will be automatically available!

### Usage

Once running, MindLink provides:

- **Local API**: `http://localhost:3001/v1`
- **Public API**: Automatically created Cloudflare URL
- **Dashboard**: `http://localhost:3001/dashboard`

#### Example API Usage

```python
from openai import OpenAI

# Use your MindLink tunnel URL
client = OpenAI(
    base_url="https://your-tunnel-url.trycloudflare.com/v1",
    api_key="any-key"  # API key is ignored
)

response = client.chat.completions.create(
    model="gpt-5",
    messages=[{"role": "user", "content": "Hello, world!"}]
)

print(response.choices[0].message.content)
```

```bash
# cURL example
curl https://your-tunnel-url.trycloudflare.com/v1/chat/completions \
  -H "Authorization: Bearer any-key" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-5",
    "messages": [{"role": "user", "content": "Hello, world!"}],
    "stream": true
  }'
```

## Supported Models

- **`gpt-5`** - Latest GPT-5 model with reasoning
- **`codex-mini`** - Code-focused model

## Features Deep Dive

### Authentication System

MindLink uses the same OAuth system as the original ChatMock project:

- Secure OAuth flow with OpenAI's auth system
- Automatic token refresh
- Stored credentials in `~/.mindlink/auth.json`

### Cloudflare Tunneling

Automatic tunnel creation with multiple strategies:

- **Quick Tunnels**: Zero-config public URLs via `trycloudflare.com`
- **Named Tunnels**: Custom domains (requires Cloudflare account)
- **Automatic retry**: Handles connection failures gracefully
- **Health monitoring**: Continuous tunnel health checks

### API Compatibility

Full OpenAI API compatibility including:

- Chat completions (`/v1/chat/completions`)
- Text completions (`/v1/completions`) 
- Model listing (`/v1/models`)
- Streaming responses
- Tool/function calling
- Vision/image understanding
- Reasoning summaries with `<think>` tags

### Configuration Options

Accessible via system tray → Settings:

**AI Features:**
- Reasoning effort: `low`, `medium`, `high`
- Reasoning summaries: `none`, `concise`, `auto`, `detailed`
- Reasoning compatibility: `think-tags`, `o3`, `legacy`

**Server Settings:**
- Custom port and host
- Request size limits
- CORS configuration

**Tunnel Settings:**
- Enable/disable tunneling
- Quick vs named tunnels
- Custom domains

**Monitoring:**
- Health check intervals
- Error thresholds
- Notification preferences

## System Tray Interface

Right-click the system tray icon for:

- **Login & Serve** - Start authentication and API service
- **Stop Serving** - Stop the API service
- **Connection Status** - View detailed connection info
- **Settings** - Open configuration window
- **Copy API URL** - Copy public API URL to clipboard
- **Open Dashboard** - Open web dashboard

### Status Indicators

- 🟢 **Connected** - Service active and healthy
- 🔵 **Connecting** - Starting up or reconnecting
- 🔴 **Error** - Connection issues detected
- ⚪ **Disconnected** - Service stopped

## Error Handling & Monitoring

MindLink includes robust error handling:

- **Automatic reconnection** on connection failures
- **Token refresh** for expired authentication
- **Health checks** every 30 seconds
- **Desktop notifications** for critical issues
- **Graceful degradation** when tunnel fails

### Connection Issues

If you see connection errors:

1. **Check Authentication**: Click "Connection Status" to verify login
2. **Restart Service**: Use "Stop Serving" then "Login & Serve"
3. **Check Firewall**: Ensure local port (default 3001) isn't blocked
4. **Manual Reconnect**: Click notification to trigger reconnection

## Development

### Project Structure

```
mindlink/
├── src/
│   ├── main.js              # Main Electron app
│   ├── auth/
│   │   └── authManager.js   # ChatGPT authentication
│   ├── server/
│   │   └── serverManager.js # Fastify API server
│   ├── tunnel/
│   │   └── tunnelManager.js # Cloudflare tunneling
│   └── config/
│       └── configManager.js # Configuration management
├── ui/
│   └── settings.html        # Settings interface
├── assets/                  # Icons and resources
└── package.json
```

### Building

```bash
# Development mode
npm run dev

# Build for current platform
npm run build

# Build for all platforms
npm run dist
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## Security & Privacy

- **Local Processing**: Authentication tokens stored locally only
- **No Data Collection**: MindLink doesn't collect or transmit user data
- **Secure Transport**: All API requests use HTTPS
- **Minimal Permissions**: Only requests necessary permissions

## Troubleshooting

### Common Issues

**"Authentication required" errors:**
- Run the login flow again: Tray → "Login & Serve"
- Check if your ChatGPT subscription is active

**"Failed to create tunnel" errors:**
- Check internet connection
- Verify local server is running on correct port
- Try restarting the application

**"Connection refused" errors:**
- Ensure port 3001 isn't used by another application
- Check firewall settings
- Try changing the port in Settings

**Performance issues:**
- Lower reasoning effort in Settings
- Disable reasoning summaries for faster responses
- Check system resources

### Debug Information

Enable debug logging by setting environment variable:
```bash
DEBUG=mindlink:* npm start
```

## Acknowledgments

This project is based on and inspired by:
- [ChatMock](https://github.com/user/chatmock) - Original OAuth and API implementation
- [Cloudflare Tunnels](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) - Tunnel technology

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/mindlink/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/mindlink/discussions)
- **Email**: support@mindlink.dev

---

**⚠️ Important**: This tool requires a paid ChatGPT subscription and is not affiliated with OpenAI. Use responsibly and in accordance with OpenAI's terms of service.