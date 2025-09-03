import { BaseProviderPlugin, ProviderUtils } from '../base-plugin'
import { ProviderStatus, ProviderConnectionInfo, OAuthConfig } from '../types'

/**
 * Anthropic Provider Plugin
 * Handles authentication and connection management for Claude API
 */
export class AnthropicPlugin extends BaseProviderPlugin {
  readonly id = 'anthropic'
  readonly name = 'anthropic'
  readonly displayName = 'Anthropic'
  readonly version = '1.0.0'
  override readonly description = 'Connect to Claude models via Anthropic API'
  override readonly homepage = 'https://www.anthropic.com'
  readonly authCommand = 'claude-code auth login'
  
  override readonly oauthConfig: OAuthConfig = {
    authUrl: 'https://console.anthropic.com/oauth/authorize',
    scope: ['api'],
    clientId: (typeof process !== 'undefined' && process.env?.['ANTHROPIC_CLIENT_ID']) || 'your-client-id',
    redirectUri: 'http://localhost:3000/auth/anthropic/callback'
  }
  
  private apiKey: string | null = null
  private baseUrl = 'https://api.anthropic.com/v1'
  
  protected override async onInitialize(): Promise<void> {
    // Check for API key in environment variables or token storage
    this.apiKey = await this.getToken()
    
    // Also check for ANTHROPIC_API_KEY environment variable
    if (!this.apiKey && typeof process !== 'undefined' && process.env?.['ANTHROPIC_API_KEY']) {
      this.apiKey = process.env['ANTHROPIC_API_KEY']
      await this.setToken(this.apiKey) // Store for future use
    }
  }
  
  override async getToken(): Promise<string | null> {
    // Use shared utility for environment fallback
    const storedToken = await super.getToken()
    return ProviderUtils.getTokenWithEnvironmentFallback(storedToken, 'ANTHROPIC_API_KEY')
  }
  
  protected override async onTokenUpdated(token: string): Promise<void> {
    this.apiKey = token
  }
  
  protected override async onTokenCleared(): Promise<void> {
    this.apiKey = null
  }
  
  override async getConnectionStatus(): Promise<ProviderStatus> {
    const lastChecked = new Date().toISOString()
    
    try {
      if (!this.apiKey) {
        return {
          status: 'disconnected',
          lastChecked
        }
      }
      
      // Test API connection
      const connectionInfo = await this.refreshConnectionInfo()
      
      if (connectionInfo) {
        return {
          status: 'connected',
          connectionInfo,
          lastChecked
        }
      } else {
        return {
          status: 'error',
          error: 'Failed to fetch connection info',
          lastChecked
        }
      }
      
    } catch (error) {
      console.error('Anthropic connection test failed:', error)
      
      // Check if it's an authentication error
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      const isAuthError = errorMessage.includes('401') || errorMessage.includes('authentication') || errorMessage.includes('unauthorized')
      
      return {
        status: isAuthError ? 'disconnected' : 'error',
        error: errorMessage,
        lastChecked
      }
    }
  }
  
  override async refreshConnectionInfo(): Promise<ProviderConnectionInfo | null> {
    if (!this.apiKey) return null
    
    try {
      // For Anthropic, we can test the connection by making a simple API call
      const testResponse = await this.testApiConnection()
      
      if (!testResponse) {
        return null
      }
      
      // Default to the most capable model
      const preferredModel = 'claude-3-5-sonnet-20241022'
      
      // Use shared utility for mock connection info
      return ProviderUtils.createMockConnectionInfo(
        'anthropic',
        preferredModel,
        'Pro'
      )
      
    } catch (error) {
      console.error('Failed to refresh Anthropic connection info:', error)
      return null
    }
  }
  
  async getSupportedModels(): Promise<string[]> {
    // Anthropic models as of 2024
    return [
      'claude-3-5-sonnet-20241022',
      'claude-3-opus-20240229',
      'claude-3-sonnet-20240229',
      'claude-3-haiku-20240307'
    ]
  }
  
  async getCurrentModel(): Promise<string | null> {
    const connectionInfo = await this.refreshConnectionInfo()
    return connectionInfo?.model || null
  }
  
  protected override async exchangeCodeForToken(code: string): Promise<string> {
    // Use shared utility for standard OAuth token exchange
    try {
      const apiKey = await ProviderUtils.makeStandardOAuthTokenExchange(
        'https://console.anthropic.com/oauth/token',
        code,
        this.oauthConfig.clientId || '',
        this.oauthConfig.redirectUri || ''
      )
      
      // Store the token
      await this.setToken(apiKey)
      return apiKey
      
    } catch (error) {
      console.error('Anthropic OAuth token exchange failed:', error)
      throw error
    }
  }
  
  private async testApiConnection(): Promise<boolean> {
    if (!this.apiKey) {
      return false
    }
    
    try {
      // Test with a minimal API call
      // Since Anthropic doesn't have a simple health check endpoint,
      // we'll simulate a connection test
      
      const response = await fetch(`${this.baseUrl}/messages`, {
        method: 'POST',
        headers: {
          'x-api-key': this.apiKey,
          'Content-Type': 'application/json',
          'anthropic-version': '2023-06-01'
        },
        body: JSON.stringify({
          model: 'claude-3-haiku-20240307', // Use the fastest model for testing
          max_tokens: 1,
          messages: [
            {
              role: 'user',
              content: 'Hi'
            }
          ]
        })
      })
      
      // If we get a 400 (bad request) or 200, the API key is valid
      // If we get 401/403, the API key is invalid
      return response.status !== 401 && response.status !== 403
      
    } catch (error) {
      console.error('Anthropic API connection test failed:', error)
      return false
    }
  }
  
  override async testConnection(): Promise<boolean> {
    try {
      const status = await this.getConnectionStatus()
      return status.status === 'connected'
    } catch (error) {
      return false
    }
  }
}