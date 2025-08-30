import { BaseProviderPlugin } from '../base-plugin'
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
  readonly description = 'Connect to Claude models via Anthropic API'
  readonly homepage = 'https://www.anthropic.com'
  readonly authCommand = 'claude-code auth login'
  
  readonly oauthConfig: OAuthConfig = {
    authUrl: 'https://console.anthropic.com/oauth/authorize',
    scope: ['api'],
    clientId: (typeof process !== 'undefined' && process.env?.ANTHROPIC_CLIENT_ID) || 'your-client-id',
    redirectUri: 'http://localhost:3000/auth/anthropic/callback'
  }
  
  private apiKey: string | null = null
  private baseUrl = 'https://api.anthropic.com/v1'
  
  protected async onInitialize(): Promise<void> {
    // Check for API key in environment variables or token storage
    this.apiKey = await this.getToken()
    
    // Also check for ANTHROPIC_API_KEY environment variable
    if (!this.apiKey && typeof process !== 'undefined' && process.env?.ANTHROPIC_API_KEY) {
      this.apiKey = process.env.ANTHROPIC_API_KEY
      await this.setToken(this.apiKey) // Store for future use
    }
  }
  
  async getToken(): Promise<string | null> {
    // First check the stored token
    const storedToken = await super.getToken()
    if (storedToken) return storedToken
    
    // Check environment variable as fallback
    if (typeof process !== 'undefined' && process.env?.ANTHROPIC_API_KEY) {
      return process.env.ANTHROPIC_API_KEY
    }
    
    return null
  }
  
  protected async onTokenUpdated(token: string): Promise<void> {
    this.apiKey = token
  }
  
  protected async onTokenCleared(): Promise<void> {
    this.apiKey = null
  }
  
  async getConnectionStatus(): Promise<ProviderStatus> {
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
  
  async refreshConnectionInfo(): Promise<ProviderConnectionInfo | null> {
    if (!this.apiKey) return null
    
    try {
      // For Anthropic, we can test the connection by making a simple API call
      // Since there's no models endpoint, we'll simulate a lightweight request
      const testResponse = await this.testApiConnection()
      
      if (!testResponse) {
        return null
      }
      
      // Available Claude models (as of 2024)
      const supportedModels = [
        'claude-3-5-sonnet-20241022',
        'claude-3-opus-20240229',
        'claude-3-sonnet-20240229',
        'claude-3-haiku-20240307'
      ]
      
      // Default to the most capable model
      const preferredModel = 'claude-3-5-sonnet-20241022'
      
      // For demonstration, we'll use mock usage data
      // In a real implementation, you'd fetch this from Anthropic's usage API
      const connectionInfo: ProviderConnectionInfo = {
        email: 'user@example.com', // Would come from user info API if available
        model: preferredModel,
        plan: 'Pro', // Would come from account info
        lastUsed: new Date(Date.now() - Math.random() * 24 * 60 * 60 * 1000).toISOString(),
        tokensUsed: Math.floor(Math.random() * 15000) + 2000,
        requestsToday: Math.floor(Math.random() * 50) + 5
      }
      
      return connectionInfo
      
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
  
  protected async exchangeCodeForToken(code: string): Promise<string> {
    // In a real implementation, this would exchange the OAuth code for an API key
    // For now, we'll simulate this
    
    try {
      // This would be a call to Anthropic's token endpoint
      const response = await fetch('https://console.anthropic.com/oauth/token', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          grant_type: 'authorization_code',
          code,
          client_id: this.oauthConfig.clientId,
          redirect_uri: this.oauthConfig.redirectUri
        })
      })
      
      if (!response.ok) {
        throw new Error(`OAuth token exchange failed: ${response.status}`)
      }
      
      const data = await response.json()
      const apiKey = data.access_token
      
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
      
      // In a real implementation, you might make a small completion request
      // or check account info if such an endpoint exists
      
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
  
  async testConnection(): Promise<boolean> {
    try {
      const status = await this.getConnectionStatus()
      return status.status === 'connected'
    } catch (error) {
      return false
    }
  }
}