import { BaseProviderPlugin, ProviderUtils } from '../base-plugin'
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
  override readonly description = 'Connect to Gemini models via Google AI Studio API'
  override readonly homepage = 'https://ai.google.dev'
  readonly authCommand = 'gemini-cli auth login'
  
  override readonly oauthConfig: OAuthConfig = {
    authUrl: 'https://accounts.google.com/oauth2/auth',
    scope: ['https://www.googleapis.com/auth/generative-language'],
    clientId: (typeof process !== 'undefined' && process.env?.['GOOGLE_CLIENT_ID']) || 'your-client-id',
    redirectUri: 'http://localhost:3000/auth/google/callback'
  }
  
  private apiKey: string | null = null
  private baseUrl = 'https://generativelanguage.googleapis.com/v1beta'
  
  protected override async onInitialize(): Promise<void> {
    // Check for API key in environment variables or token storage
    this.apiKey = await this.getToken()
    
    // Also check for GOOGLE_API_KEY environment variable
    if (!this.apiKey && typeof process !== 'undefined' && process.env?.['GOOGLE_API_KEY']) {
      this.apiKey = process.env['GOOGLE_API_KEY']
      await this.setToken(this.apiKey) // Store for future use
    }
  }
  
  override async getToken(): Promise<string | null> {
    // Use shared utility for environment fallback
    const storedToken = await super.getToken()
    return ProviderUtils.getTokenWithEnvironmentFallback(storedToken, 'GOOGLE_API_KEY')
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
  
  override async refreshConnectionInfo(): Promise<ProviderConnectionInfo | null> {
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
        .map((model: any) => (model.name as string).split('/').pop() as string) // Extract model name from full path
        .sort()
      
      // Get the most capable model as default
      const preferredModel = geminiModels.find((name: string) => name.includes('pro')) ||
                           geminiModels.find((name: string) => name.includes('gemini')) ||
                           geminiModels[0] || 'gemini-pro'
      
      // Use shared utility for mock connection info
      return ProviderUtils.createMockConnectionInfo(
        'google',
        preferredModel,
        'Free Tier'
      )
      
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
        .map((model: any) => (model.name as string).split('/').pop() as string) // Extract model name
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
  
  protected override async exchangeCodeForToken(code: string): Promise<string> {
    // Use shared utility for standard OAuth token exchange
    try {
      const accessToken = await ProviderUtils.makeStandardOAuthTokenExchange(
        'https://oauth2.googleapis.com/token',
        code,
        this.oauthConfig.clientId || '',
        this.oauthConfig.redirectUri || ''
      )
      
      // Store the token
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
  
  override async testConnection(): Promise<boolean> {
    try {
      const status = await this.getConnectionStatus()
      return status.status === 'connected'
    } catch (error) {
      return false
    }
  }
}