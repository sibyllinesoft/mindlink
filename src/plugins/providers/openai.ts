import { BaseProviderPlugin } from '../base-plugin'
import { ProviderStatus, ProviderConnectionInfo, OAuthConfig } from '../types'

/**
 * OpenAI Provider Plugin
 * Handles authentication and connection management for OpenAI API
 */
export class OpenAIPlugin extends BaseProviderPlugin {
  readonly id = 'openai'
  readonly name = 'openai'
  readonly displayName = 'OpenAI'
  readonly version = '1.0.0'
  readonly description = 'Connect to OpenAI GPT models via API'
  readonly homepage = 'https://openai.com'
  readonly authCommand = 'openai-codex auth login'
  
  readonly oauthConfig: OAuthConfig = {
    authUrl: 'https://platform.openai.com/oauth/authorize',
    scope: ['api'],
    clientId: (typeof process !== 'undefined' && process.env?.OPENAI_CLIENT_ID) || 'your-client-id',
    redirectUri: 'http://localhost:3000/auth/openai/callback'
  }
  
  private apiKey: string | null = null
  private baseUrl = 'https://api.openai.com/v1'
  
  protected async onInitialize(): Promise<void> {
    // Check for API key in environment variables or token storage
    this.apiKey = await this.getToken()
    
    // Also check for OPENAI_API_KEY environment variable
    if (!this.apiKey && typeof process !== 'undefined' && process.env?.OPENAI_API_KEY) {
      this.apiKey = process.env.OPENAI_API_KEY
      await this.setToken(this.apiKey) // Store for future use
    }
  }
  
  async getToken(): Promise<string | null> {
    // First check the stored token
    const storedToken = await super.getToken()
    if (storedToken) return storedToken
    
    // Check environment variable as fallback
    if (typeof process !== 'undefined' && process.env?.OPENAI_API_KEY) {
      return process.env.OPENAI_API_KEY
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
      
      // Test API connection by fetching user info or models
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
      console.error('OpenAI connection test failed:', error)
      
      // Check if it's an authentication error
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      const isAuthError = errorMessage.includes('401') || errorMessage.includes('authentication')
      
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
      // For OpenAI, we can fetch models to verify connection
      const modelsResponse = await this.makeApiRequest('/models')
      
      if (modelsResponse.error) {
        throw new Error(modelsResponse.error.message)
      }
      
      // Get available models
      const models = modelsResponse.data || []
      const gptModels = models
        .filter((model: any) => model.id.includes('gpt'))
        .map((model: any) => model.id)
        .sort()
      
      // Get the most capable model as default
      const preferredModel = gptModels.find(id => id.includes('gpt-4')) || 
                           gptModels.find(id => id.includes('gpt-3.5')) ||
                           gptModels[0] || 'gpt-3.5-turbo'
      
      // For demonstration, we'll use mock usage data
      // In a real implementation, you'd fetch this from OpenAI's usage API
      const connectionInfo: ProviderConnectionInfo = {
        email: 'user@example.com', // Would come from user info API
        model: preferredModel,
        lastUsed: new Date(Date.now() - Math.random() * 24 * 60 * 60 * 1000).toISOString(),
        tokensUsed: Math.floor(Math.random() * 10000) + 1000,
        requestsToday: Math.floor(Math.random() * 100) + 10
      }
      
      return connectionInfo
      
    } catch (error) {
      console.error('Failed to refresh OpenAI connection info:', error)
      return null
    }
  }
  
  async getSupportedModels(): Promise<string[]> {
    if (!this.apiKey) return []
    
    try {
      const response = await this.makeApiRequest('/models')
      
      if (response.error) {
        throw new Error(response.error.message)
      }
      
      const models = response.data || []
      return models
        .filter((model: any) => model.id.includes('gpt'))
        .map((model: any) => model.id)
        .sort()
        
    } catch (error) {
      console.error('Failed to fetch OpenAI models:', error)
      return []
    }
  }
  
  async getCurrentModel(): Promise<string | null> {
    const connectionInfo = await this.refreshConnectionInfo()
    return connectionInfo?.model || null
  }
  
  protected async exchangeCodeForToken(code: string): Promise<string> {
    // In a real implementation, this would exchange the OAuth code for an API key
    // For now, we'll simulate this
    
    try {
      // This would be a call to OpenAI's token endpoint
      const response = await fetch('https://platform.openai.com/oauth/token', {
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
      console.error('OpenAI OAuth token exchange failed:', error)
      throw error
    }
  }
  
  private async makeApiRequest(endpoint: string): Promise<any> {
    if (!this.apiKey) {
      throw new Error('No API key available')
    }
    
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json'
      }
    })
    
    if (!response.ok) {
      throw new Error(`OpenAI API request failed: ${response.status} ${response.statusText}`)
    }
    
    return response.json()
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