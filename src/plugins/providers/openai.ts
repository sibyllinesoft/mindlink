import { BaseProviderPlugin } from '../base-plugin'
import { ProviderStatus, ProviderConnectionInfo, OAuthConfig } from '../types'
import { invoke } from '@tauri-apps/api/core'

/**
 * OpenAI Provider Plugin
 * Handles authentication and connection management for OpenAI API
 */
export class OpenAIPlugin extends BaseProviderPlugin {
  readonly id = 'openai'
  readonly name = 'openai'
  readonly displayName = 'ChatGPT'
  readonly version = '1.0.0'
  readonly description = 'Connect to ChatGPT Plus/Pro via OAuth'
  readonly homepage = 'https://chatgpt.com'
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
      // Check backend ChatGPT authentication status
      console.log('🔍 ChatGPT plugin: checking backend auth status...')
      const isAuthenticated = await invoke<boolean>('check_chatgpt_auth_status')
      console.log('🔍 ChatGPT plugin: backend auth status =', isAuthenticated)
      
      if (!isAuthenticated) {
        console.log('❌ ChatGPT plugin: not authenticated')
        return {
          status: 'disconnected',
          lastChecked
        }
      }
      
      console.log('✅ ChatGPT plugin: authenticated!')
      
      // Get connection info from backend
      try {
        const authInfo = await invoke('get_chatgpt_auth_info') as any
        
        if (authInfo) {
          const connectionInfo: ProviderConnectionInfo = {
            email: authInfo.email || 'user@chatgpt.com',
            model: 'ChatGPT Plus',
            lastUsed: new Date().toISOString(),
            plan: 'ChatGPT Plus'
          }
          
          return {
            status: 'connected',
            connectionInfo,
            lastChecked
          }
        }
      } catch (error) {
        console.warn('Could not get ChatGPT auth info:', error)
        // Still show as connected if auth status is true
      }
      
      // Fallback - authenticated but no detailed info
      return {
        status: 'connected',
        connectionInfo: {
          email: 'user@chatgpt.com',
          model: 'ChatGPT Plus',
          lastUsed: new Date().toISOString(),
          plan: 'ChatGPT Plus'
        },
        lastChecked
      }
      
    } catch (error) {
      console.error('❌ ChatGPT backend connection test failed:', error)
      console.error('❌ Error details:', {
        message: error instanceof Error ? error.message : 'Unknown error',
        stack: error instanceof Error ? error.stack : 'No stack',
        type: typeof error,
        value: error
      })
      
      return {
        status: 'error',
        error: error instanceof Error ? error.message : 'Backend connection failed',
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