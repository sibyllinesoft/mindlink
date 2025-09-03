import { BaseProviderPlugin } from '../base-plugin'
import type { ProviderConnectionInfo, ProviderStatus } from '../types'

/**
 * Ollama Local Provider Plugin
 * Detects and connects to local Ollama instances
 */
export class OllamaPlugin extends BaseProviderPlugin {
  id = 'ollama'
  displayName = 'Ollama'
  description = 'Connect to local Ollama models'
  version = '1.0.0'
  
  // Ollama doesn't use OAuth - it's a local service
  authCommand = 'detect'
  oauthConfig = undefined
  supportsConfiguration = true // Ollama supports model selection and endpoint configuration
  
  private baseUrl = 'http://127.0.0.1:11434'
  private detectionTimeout = 5000 // 5 second timeout for detection
  private selectedModel: string | null = null
  
  // Configuration options
  private config = {
    baseUrl: 'http://127.0.0.1:11434',
    selectedModel: null as string | null,
    timeout: 5000
  }
  
  protected async onInitialize(): Promise<void> {
    // Load saved configuration
    await this.loadConfig()
    console.log('🦙 Ollama plugin initialized with config:', this.config)
  }
  
  // Configuration management
  async loadConfig(): Promise<void> {
    try {
      const configKey = `mindlink_${this.id}_config`
      const savedConfig = localStorage.getItem(configKey)
      if (savedConfig) {
        this.config = { ...this.config, ...JSON.parse(savedConfig) }
        this.baseUrl = this.config.baseUrl
        this.selectedModel = this.config.selectedModel
        this.detectionTimeout = this.config.timeout
      }
    } catch (error) {
      console.error('🦙 Failed to load Ollama config:', error)
    }
  }
  
  async saveConfig(): Promise<void> {
    try {
      const configKey = `mindlink_${this.id}_config`
      localStorage.setItem(configKey, JSON.stringify(this.config))
    } catch (error) {
      console.error('🦙 Failed to save Ollama config:', error)
    }
  }
  
  async updateConfig(newConfig: Partial<typeof this.config>): Promise<void> {
    this.config = { ...this.config, ...newConfig }
    this.baseUrl = this.config.baseUrl
    this.selectedModel = this.config.selectedModel
    this.detectionTimeout = this.config.timeout
    await this.saveConfig()
    console.log('🦙 Ollama config updated:', this.config)
  }
  
  // Get current configuration for UI
  getConfig(): typeof this.config {
    return { ...this.config }
  }
  
  async getConnectionStatus(): Promise<ProviderStatus> {
    const lastChecked = new Date().toISOString()
    
    try {
      console.log('🦙 Checking Ollama connection at', this.baseUrl)
      
      // Test connection to Ollama API
      const connectionInfo = await this.refreshConnectionInfo()
      
      if (connectionInfo) {
        console.log('✅ Ollama connected:', connectionInfo)
        return {
          status: 'connected',
          connectionInfo,
          lastChecked
        }
      } else {
        console.log('❌ Ollama not responding')
        return {
          status: 'disconnected',
          lastChecked,
          error: 'Ollama service not running on localhost:11434'
        }
      }
      
    } catch (error) {
      console.error('🦙 Ollama connection test failed:', error)
      
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      
      // Check if it's a network/connection error (service not running)
      const isServiceDown = errorMessage.includes('fetch') || 
                          errorMessage.includes('ECONNREFUSED') ||
                          errorMessage.includes('network') ||
                          errorMessage.includes('Failed to fetch')
      
      return {
        status: 'disconnected',
        error: isServiceDown ? 'Ollama service not running' : errorMessage,
        lastChecked
      }
    }
  }
  
  async refreshConnectionInfo(): Promise<ProviderConnectionInfo | null> {
    try {
      // Test Ollama API with timeout
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), this.detectionTimeout)
      
      // First, check if Ollama is running by hitting the API root
      const healthResponse = await fetch(`${this.baseUrl}/api/tags`, {
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json'
        }
      })
      
      clearTimeout(timeoutId)
      
      if (!healthResponse.ok) {
        console.log('🦙 Ollama API not responding:', healthResponse.status)
        return null
      }
      
      const modelsData = await healthResponse.json()
      console.log('🦙 Ollama models response:', modelsData)
      
      // Get available models
      const models = modelsData.models || []
      const modelNames = models.map((model: any) => model.name).sort()
      
      // Use selected model from config, or first available model as fallback
      const preferredModel = this.config.selectedModel || 
                           (modelNames.length > 0 ? modelNames[0] : 'llama2')
      
      // Get model info for the preferred model
      let modelSize = 'Unknown'
      let modelParams = 'Unknown'
      
      if (models.length > 0) {
        const firstModel = models[0]
        if (firstModel.size) {
          // Convert bytes to GB for display
          const sizeInGB = (firstModel.size / (1024 * 1024 * 1024)).toFixed(1)
          modelSize = `${sizeInGB} GB`
        }
        
        // Try to extract parameter count from model name or details
        if (firstModel.details?.parameter_size) {
          modelParams = firstModel.details.parameter_size
        } else if (firstModel.name) {
          // Try to extract from model name (e.g., "qwen3-coder:30b")
          const paramMatch = firstModel.name.match(/(\d+\.?\d*[bB])/i)
          if (paramMatch) {
            modelParams = paramMatch[1].toUpperCase()
          }
        }
      }
      
      const connectionInfo: ProviderConnectionInfo = {
        model: preferredModel,
        plan: 'Local Instance',
        lastUsed: new Date().toISOString(),
        // Additional Ollama-specific info
        modelCount: models.length,
        modelSize,
        modelParams,
        endpoint: this.baseUrl
      }
      
      console.log('🦙 Ollama connection info:', connectionInfo)
      return connectionInfo
      
    } catch (error) {
      console.error('🦙 Failed to refresh Ollama connection info:', error)
      
      // Check for specific error types
      if (error instanceof Error) {
        if (error.name === 'AbortError') {
          console.log('🦙 Ollama detection timed out after', this.detectionTimeout, 'ms')
        } else if (error.message.includes('fetch')) {
          console.log('🦙 Ollama service appears to be down')
        }
      }
      
      return null
    }
  }
  
  async getSupportedModels(): Promise<string[]> {
    try {
      const response = await fetch(`${this.baseUrl}/api/tags`, {
        headers: {
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        return []
      }
      
      const data = await response.json()
      const models = data.models || []
      
      return models.map((model: any) => model.name).sort()
      
    } catch (error) {
      console.error('🦙 Failed to fetch Ollama models:', error)
      return []
    }
  }
  
  async getCurrentModel(): Promise<string | null> {
    const connectionInfo = await this.refreshConnectionInfo()
    return connectionInfo?.model || null
  }
  
  // Ollama doesn't use OAuth, so we'll implement a simple detection flow
  async initiateOAuth(): Promise<string> {
    // For Ollama, we just return a local detection URL
    return 'ollama://detect'
  }
  
  protected async exchangeCodeForToken(code: string): Promise<string> {
    // Ollama doesn't use tokens, but we'll return a dummy token to indicate detection
    if (code === 'detect') {
      const connectionInfo = await this.refreshConnectionInfo()
      return connectionInfo ? 'ollama-detected' : 'ollama-not-found'
    }
    
    throw new Error('Invalid detection code for Ollama')
  }
  
  async testConnection(): Promise<boolean> {
    try {
      const status = await this.getConnectionStatus()
      return status.status === 'connected'
    } catch (error) {
      return false
    }
  }
  
  // Override the token methods since Ollama doesn't use authentication
  async getToken(): Promise<string | null> {
    // Check if Ollama is detected/running
    const isRunning = await this.testConnection()
    return isRunning ? 'ollama-running' : null
  }
  
  async setToken(token: string): Promise<void> {
    // No-op for Ollama since it doesn't use tokens
    console.log('🦙 Ollama token set (no-op):', token)
  }
  
  async clearToken(): Promise<void> {
    // No-op for Ollama
    console.log('🦙 Ollama token cleared (no-op)')
  }
}