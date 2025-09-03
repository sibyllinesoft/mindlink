# MindLink Plugin System

The MindLink plugin system provides a modular architecture for OAuth provider integrations. Each AI provider (OpenAI, Anthropic, Google) is implemented as an independent plugin that handles authentication, connection management, and API interactions.

## Architecture

```
plugins/
├── types.ts              # Plugin interface definitions
├── base-plugin.ts        # Base class with common functionality
├── registry.ts           # Plugin registration and lifecycle management
├── index.ts              # Main export file
└── providers/
    ├── openai.ts         # OpenAI provider plugin
    ├── anthropic.ts      # Anthropic (Claude) provider plugin
    ├── google.ts         # Google (Gemini) provider plugin
    └── index.ts          # Provider exports
```

## Core Concepts

### ProviderPlugin Interface

Each provider plugin must implement the `ProviderPlugin` interface which defines:

- **Metadata**: Plugin identification and description
- **Token Management**: Secure storage and retrieval of API keys/tokens
- **Connection Status**: Real-time authentication and connection health
- **OAuth Flow**: Complete OAuth 2.0/PKCE authentication handling
- **API Integration**: Provider-specific API calls and model information

### Plugin Registry

The `ProviderPluginRegistry` manages the lifecycle of all plugins:

- **Registration**: Dynamic plugin loading and initialization
- **Configuration**: Per-plugin settings and enable/disable state
- **Health Monitoring**: Automatic connection status checking
- **Coordination**: Batch operations across multiple providers

## Plugin Features

### Token Storage
- Secure localStorage-based token storage (with override capability)
- Environment variable fallback for development
- Automatic token validation and refresh

### Connection Management
- Real-time connection status monitoring
- Automatic error detection and recovery
- Health check endpoints for API validation

### OAuth Flow
- Standards-compliant OAuth 2.0 PKCE implementation
- State parameter validation for security
- Popup-based authentication flow
- Automatic token exchange and storage

### Provider-Specific Features

#### OpenAI Plugin
- API key and OAuth support
- Model discovery and selection
- Usage statistics tracking
- GPT model compatibility

#### Anthropic Plugin
- Claude API integration
- Model version management
- Connection testing via API calls
- Plan and usage information

#### Google Plugin
- Gemini API connectivity
- Google Cloud integration patterns
- Model enumeration and selection
- OAuth via Google accounts

## Usage

### Basic Setup
```typescript
import { pluginRegistry, OpenAIPlugin, AnthropicPlugin, GooglePlugin } from '../plugins'

// Register plugins
const plugins = [
  new OpenAIPlugin(),
  new AnthropicPlugin(),
  new GooglePlugin()
]

plugins.forEach(plugin => pluginRegistry.registerPlugin(plugin))

// Initialize system
await pluginRegistry.initialize()

// Get connection statuses
const statuses = await pluginRegistry.getConnectionStatuses()
```

### OAuth Integration
```typescript
const plugin = pluginRegistry.getPlugin('openai')
if (plugin) {
  // Initiate OAuth flow
  const oauthUrl = await plugin.initiateOAuth()
  
  // Open popup for user authentication
  const popup = window.open(oauthUrl, 'OAuth', 'width=600,height=700')
  
  // Handle callback (implementation depends on your OAuth callback setup)
  // const token = await plugin.handleOAuthCallback(code, state)
}
```

### Connection Monitoring
```typescript
// Check individual plugin
const status = await plugin.getConnectionStatus()
console.log(status.status) // 'connected' | 'disconnected' | 'error' | 'expired'

// Refresh all connections
await pluginRegistry.refreshAllConnections()
```

## Security Considerations

- Tokens are stored in localStorage by default (consider secure storage for production)
- OAuth state parameters prevent CSRF attacks
- All API calls include proper error handling and validation
- Plugin isolation prevents cross-contamination of credentials

## Extension Points

The plugin system is designed for easy extension:

### Adding New Providers
1. Extend `BaseProviderPlugin`
2. Implement provider-specific API calls
3. Configure OAuth endpoints and scopes
4. Register with the plugin registry

### Custom Token Storage
Override `getToken()`, `setToken()`, and `clearToken()` methods for custom storage backends (Tauri secure storage, encrypted files, etc.)

### Additional Features
- Add provider-specific methods for advanced features
- Implement custom health checks
- Extend connection info with provider-specific data

## Development

### Plugin Development
```bash
# Plugin files are TypeScript modules
# No build step required - imported directly
# Hot reload supported during development
```

### Testing
```bash
# Plugins can be tested independently
# Mock storage and network calls as needed
# Registry provides isolated testing environment
```

This plugin architecture ensures that OAuth providers are completely modular, testable, and maintainable while providing a consistent interface for the MindLink application.