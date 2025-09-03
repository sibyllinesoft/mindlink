/**
 * Plugin Architecture for OAuth Providers
 * 
 * This defines the contract that all OAuth provider plugins must implement
 * to integrate with the MindLink application.
 */

export interface ProviderConnectionInfo {
  email?: string
  username?: string
  model?: string
  plan?: string
  lastUsed?: string
  tokensUsed?: number
  requestsToday?: number
  // Ollama-specific fields
  modelCount?: number
  modelSize?: string
  modelParams?: string
  endpoint?: string
}

export interface ProviderStatus {
  status: 'connected' | 'disconnected' | 'error' | 'expired'
  connectionInfo?: ProviderConnectionInfo
  error?: string
  lastChecked: string
}

export interface OAuthConfig {
  authUrl?: string
  scope?: string[]
  clientId?: string
  redirectUri?: string
}

export interface ProviderPlugin {
  // Plugin metadata
  readonly id: string
  readonly name: string
  readonly displayName: string
  readonly iconUrl?: string
  readonly description?: string
  readonly version: string
  readonly homepage?: string
  
  // OAuth configuration
  readonly oauthConfig?: OAuthConfig
  readonly authCommand: string // CLI command to trigger OAuth
  
  // Configuration support
  readonly supportsConfiguration?: boolean
  
  // Core plugin methods
  initialize(): Promise<void>
  
  // Token management
  hasToken(): Promise<boolean>
  getToken(): Promise<string | null>
  setToken(token: string): Promise<void>
  clearToken(): Promise<void>
  
  // Connection status
  getConnectionStatus(): Promise<ProviderStatus>
  refreshConnectionInfo(): Promise<ProviderConnectionInfo | null>
  
  // OAuth flow
  initiateOAuth(): Promise<string> // Returns OAuth URL
  handleOAuthCallback?(code: string, state?: string): Promise<string> // Returns token
  
  // Health checks
  testConnection(): Promise<boolean>
  
  // Optional: Model/API specific methods
  getSupportedModels?(): Promise<string[]>
  getCurrentModel?(): Promise<string | null>
  
  // Plugin lifecycle
  destroy?(): Promise<void>
}

export interface PluginRegistry {
  registerPlugin(plugin: ProviderPlugin): void
  getPlugin(id: string): ProviderPlugin | null
  getAllPlugins(): ProviderPlugin[]
  getEnabledPlugins(): ProviderPlugin[]
  unregisterPlugin(id: string): void
}

export interface PluginConfig {
  enabled: boolean
  settings?: Record<string, any>
}

export interface PluginManifest {
  id: string
  name: string
  version: string
  description?: string
  author?: string
  main: string // Entry point file
  dependencies?: string[]
  mindlinkVersion?: string // Compatible MindLink version
}