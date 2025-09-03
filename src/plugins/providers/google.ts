import { BaseProviderPlugin } from '../base-plugin'
import { ProviderStatus, ProviderConnectionInfo, OAuthConfig } from '../types'

/**
 * Google Provider Plugin
 * Handles authentication and connection management for Gemini API
 */
export class GooglePlugin extends BaseProviderPlugin {
  readonly id = 'google'
  readonly name = 'google'
  readonly displayName = 'Google'
  readonly version = '1.0.0'
  readonly description = 'Connect to Gemini models via Google AI Studio API'
  readonly homepage = 'https://ai.google.dev'
  readonly authCommand = 'gemini-cli auth login'
  
  readonly oauthConfig: OAuthConfig = {
    authUrl: 'https://accounts.google.com/oauth2/auth',
    scope: ['https://www.googleapis.com/auth/generative-language'],
    clientId: (typeof process !== 'undefined' && process.env?.GOOGLE_CLIENT_ID) || 'your-client-id',
    redirectUri: 'http://localhost:3000/auth/google/callback'
  }
  
  private apiKey: string | null = null
  private baseUrl = 'https://generativelanguage.googleapis.com/v1beta'
  
  protected async onInitialize(): Promise<void> {
    // Check for API key in environment variables or token storage
    this.apiKey = await this.getToken()
    
    // Also check for GOOGLE_API_KEY environment variable
    if (!this.apiKey && typeof process !== 'undefined' && process.env?.GOOGLE_API_KEY) {
      this.apiKey = process.env.GOOGLE_API_KEY
      await this.setToken(this.apiKey) // Store for future use
    }
  }
  
  async getToken(): Promise<string | null> {
    // First check the stored token
    const storedToken = await super.getToken()
    if (storedToken) return storedToken
    
    // Check environment variable as fallback
    if (typeof process !== 'undefined' && process.env?.GOOGLE_API_KEY) {
      return process.env.GOOGLE_API_KEY
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
      
      // Test API connection by fetching available models
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
      console.error('Google/Gemini connection test failed:', error)
      
      // Check if it's an authentication error
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      const isAuthError = errorMessage.includes('401') || 
                         errorMessage.includes('403') || 
                         errorMessage.includes('authentication') ||
                         errorMessage.includes('API key not valid')
      
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
      // Fetch available models to verify connection
      const modelsResponse = await this.makeApiRequest('/models')
      
      if (!modelsResponse.models) {
        throw new Error('No models returned from API')
      }
      
      // Get available Gemini models
      const models = modelsResponse.models
      const geminiModels = models
        .filter((model: any) => model.name.includes('gemini'))
        .map((model: any) => model.name.split('/').pop()) // Extract model name from full path
        .sort()
      
      // Get the most capable model as default
      const preferredModel = geminiModels.find(name => name.includes('pro')) ||
                           geminiModels.find(name => name.includes('gemini')) ||
                           geminiModels[0] || 'gemini-pro'
      
      // For demonstration, we'll use mock usage data
      // In a real implementation, you'd fetch this from Google Cloud usage APIs
      const connectionInfo: ProviderConnectionInfo = {
        email: 'user@gmail.com', // Would come from OAuth user info
        model: preferredModel,
        plan: 'Free Tier', // Would come from billing API
        lastUsed: new Date(Date.now() - Math.random() * 24 * 60 * 60 * 1000).toISOString(),
        tokensUsed: Math.floor(Math.random() * 8000) + 500,
        requestsToday: Math.floor(Math.random() * 30) + 3
      }
      
      return connectionInfo
      
    } catch (error) {
      console.error('Failed to refresh Google connection info:', error)
      return null
    }
  }
  
  async getSupportedModels(): Promise<string[]> {
    if (!this.apiKey) return []
    
    try {
      const response = await this.makeApiRequest('/models')
      
      if (!response.models) {
        return []
      }
      
      const models = response.models
      return models
        .filter((model: any) => model.name.includes('gemini'))
        .map((model: any) => model.name.split('/').pop()) // Extract model name
        .sort()
        
    } catch (error) {
      console.error('Failed to fetch Google models:', error)
      
      // Return known Gemini models as fallback
      return [
        'gemini-1.5-pro',
        'gemini-1.5-flash',
        'gemini-pro',
        'gemini-pro-vision'
      ]
    }
  }
  
  async getCurrentModel(): Promise<string | null> {
    const connectionInfo = await this.refreshConnectionInfo()
    return connectionInfo?.model || null
  }
  
  protected async exchangeCodeForToken(code: string): Promise<string> {
    // In a real implementation, this would exchange the OAuth code for an access token
    // and then use that to get an API key or service account credentials
    
    try {
      // This would be a call to Google's OAuth token endpoint
      const response = await fetch('https://oauth2.googleapis.com/token', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          grant_type: 'authorization_code',
          code,
          client_id: this.oauthConfig.clientId,
          redirect_uri: this.oauthConfig.redirectUri,
          client_secret: (typeof process !== 'undefined' && process.env?.GOOGLE_CLIENT_SECRET) || '' // Would need to be configured
        })
      })
      
      if (!response.ok) {
        throw new Error(`OAuth token exchange failed: ${response.status}`)
      }
      
      const data = await response.json()
      const accessToken = data.access_token
      
      // In practice, you might need to use this access token to create or retrieve
      // an API key for the Generative Language API
      // For now, we'll store the access token as the API key
      
      await this.setToken(accessToken)
      
      return accessToken
      
    } catch (error) {
      console.error('Google OAuth token exchange failed:', error)
      throw error
    }
  }
  
  private async makeApiRequest(endpoint: string): Promise<any> {
    if (!this.apiKey) {
      throw new Error('No API key available')
    }
    
    const url = `${this.baseUrl}${endpoint}?key=${this.apiKey}`
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json'
      }
    })
    
    if (!response.ok) {
      const errorText = await response.text()
      throw new Error(`Google API request failed: ${response.status} ${response.statusText} - ${errorText}`)
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