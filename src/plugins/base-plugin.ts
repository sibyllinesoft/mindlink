import { ProviderPlugin, ProviderStatus, ProviderConnectionInfo, OAuthConfig } from './types'

/**
 * Base class for OAuth provider plugins
 * Provides common functionality and default implementations
 */
export abstract class BaseProviderPlugin implements ProviderPlugin {
  abstract readonly id: string
  abstract readonly name: string
  abstract readonly displayName: string
  abstract readonly version: string
  abstract readonly authCommand: string
  
  readonly iconUrl?: string
  readonly description?: string
  readonly homepage?: string
  readonly oauthConfig?: OAuthConfig
  
  private _initialized = false
  
  constructor() {
    // Plugin construction logic
  }
  
  async initialize(): Promise<void> {
    if (this._initialized) return
    
    await this.onInitialize()
    this._initialized = true
  }
  
  protected async onInitialize(): Promise<void> {
    // Override in subclasses for custom initialization
  }
  
  // Token storage methods - can be overridden for custom storage
  async hasToken(): Promise<boolean> {
    const token = await this.getToken()
    return token !== null && token.length > 0
  }
  
  async getToken(): Promise<string | null> {
    // Default implementation using localStorage
    // In a real app, this might use secure storage or environment variables
    try {
      const key = `mindlink_${this.id}_token`
      const token = localStorage.getItem(key)
      return token
    } catch (error) {
      console.error(`Error getting token for ${this.id}:`, error)
      return null
    }
  }
  
  async setToken(token: string): Promise<void> {
    try {
      const key = `mindlink_${this.id}_token`
      localStorage.setItem(key, token)
      
      // Notify that connection status may have changed
      await this.onTokenUpdated(token)
    } catch (error) {
      console.error(`Error setting token for ${this.id}:`, error)
      throw error
    }
  }
  
  async clearToken(): Promise<void> {
    try {
      const key = `mindlink_${this.id}_token`
      localStorage.removeItem(key)
      
      await this.onTokenCleared()
    } catch (error) {
      console.error(`Error clearing token for ${this.id}:`, error)
    }
  }
  
  protected async onTokenUpdated(_token: string): Promise<void> {
    // Override in subclasses to handle token updates
  }
  
  protected async onTokenCleared(): Promise<void> {
    // Override in subclasses to handle token clearing
  }
  
  // Connection status - must be implemented by subclasses
  abstract getConnectionStatus(): Promise<ProviderStatus>
  abstract refreshConnectionInfo(): Promise<ProviderConnectionInfo | null>
  
  // OAuth flow
  async initiateOAuth(): Promise<string> {
    if (!this.oauthConfig?.authUrl) {
      throw new Error(`OAuth not configured for ${this.displayName}`)
    }
    
    // Generate state parameter for security
    const state = this.generateState()
    
    const params = new URLSearchParams({
      response_type: 'code',
      client_id: this.oauthConfig.clientId || '',
      redirect_uri: this.oauthConfig.redirectUri || '',
      scope: this.oauthConfig.scope?.join(' ') || '',
      state
    })
    
    const authUrl = `${this.oauthConfig.authUrl}?${params.toString()}`
    
    // Store state for validation
    localStorage.setItem(`mindlink_${this.id}_oauth_state`, state)
    
    return authUrl
  }
  
  async handleOAuthCallback(code: string, state?: string): Promise<string> {
    // Verify state parameter
    const storedState = localStorage.getItem(`mindlink_${this.id}_oauth_state`)
    if (state !== storedState) {
      throw new Error('Invalid OAuth state parameter')
    }
    
    // Clean up stored state
    localStorage.removeItem(`mindlink_${this.id}_oauth_state`)
    
    // Exchange code for token - must be implemented by subclass
    return this.exchangeCodeForToken(code)
  }
  
  protected abstract exchangeCodeForToken(code: string): Promise<string>
  
  // Health checks
  async testConnection(): Promise<boolean> {
    try {
      const status = await this.getConnectionStatus()
      return status.status === 'connected'
    } catch (error) {
      console.error(`Connection test failed for ${this.id}:`, error)
      return false
    }
  }
  
  // Helper methods
  private generateState(): string {
    return Math.random().toString(36).substring(2, 15) + 
           Math.random().toString(36).substring(2, 15)
  }
  
  protected formatLastUsed(dateString: string): string {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    
    if (diffHours < 1) return 'Just now'
    if (diffHours < 24) return `${diffHours}h ago`
    const diffDays = Math.floor(diffHours / 24)
    return `${diffDays}d ago`
  }
  
  // Cleanup
  async destroy(): Promise<void> {
    this._initialized = false
    await this.onDestroy()
  }
  
  protected async onDestroy(): Promise<void> {
    // Override in subclasses for custom cleanup
  }
}