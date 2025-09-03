import React, { useState, useEffect } from 'react'
import { LogIn, Settings } from 'lucide-react'
import { invoke } from '@tauri-apps/api/core'
import { pluginLoader } from '../plugins/dynamic-loader'
import { pluginRegistry } from '../plugins/registry'
import type { ProviderPlugin, ProviderStatus } from '../plugins/types'
import { OllamaPlugin } from '../plugins/providers/ollama'
import OllamaConfigModal from './OllamaConfigModal'
import './ProvidersCard.css'

interface ProviderWithStatus {
  plugin: ProviderPlugin
  status: ProviderStatus
}

interface ProvidersCardProps {}

const ProvidersCard: React.FC<ProvidersCardProps> = () => {
  const [providers, setProviders] = useState<ProviderWithStatus[]>([])
  const [loading, setLoading] = useState(false)
  const [initialized, setInitialized] = useState(false)
  const [configModalOpen, setConfigModalOpen] = useState<string | null>(null)
  const [isAuthenticating, setIsAuthenticating] = useState(false)

  // Initialize the plugin system
  const initializePlugins = async () => {
    if (initialized) return
    
    try {
      console.log('🔌 Initializing plugin system with dynamic loader...')
      console.log('🔌 Plugin loader instance:', pluginLoader)
      
      // Load all plugins using the dynamic loader
      const plugins = await pluginLoader.loadAllPlugins()
      console.log('🔌 Loaded plugins result:', plugins)
      
      if (plugins.length > 0) {
        setInitialized(true)
        console.log('✅ Plugin system initialized successfully with', plugins.length, 'plugins')
        
        // Load initial provider statuses
        await refreshProviderStatuses()
      } else {
        console.warn('⚠️ No plugins were loaded - checking plugin loader state')
        console.log('🔌 Plugin loader loaded plugins map:', pluginLoader.getLoadedPluginIds())
      }
      
    } catch (error) {
      console.error('❌ Failed to initialize plugins:', error)
      console.error('❌ Error stack:', error.stack)
      console.error('❌ Error details:', {
        name: error.name,
        message: error.message,
        cause: error.cause
      })
    }
  }
  
  // Refresh all provider connection statuses
  const refreshProviderStatuses = async () => {
    setLoading(true)
    
    try {
      console.log('🔄 Refreshing provider statuses...')
      const statuses = await pluginRegistry.getConnectionStatuses()
      console.log('📊 Provider statuses:', statuses.map(s => ({ name: s.plugin.displayName, status: s.status.status })))
      setProviders(statuses)
    } catch (error) {
      console.error('❌ Failed to refresh provider statuses:', error)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    initializePlugins()
    
    // Set up periodic refresh of provider statuses
    const interval = setInterval(refreshProviderStatuses, 30000) // Refresh every 30 seconds
    
    return () => {
      clearInterval(interval)
    }
  }, [])

  const formatLastUsed = (dateString: string) => {
    const date = new Date(dateString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    
    if (diffHours < 1) return 'Just now'
    if (diffHours < 24) return `${diffHours}h ago`
    const diffDays = Math.floor(diffHours / 24)
    return `${diffDays}d ago`
  }

  const handleLogin = async (providerWithStatus: ProviderWithStatus) => {
    // Prevent multiple simultaneous authentication attempts
    if (isAuthenticating) {
      console.log('🚫 Authentication already in progress, ignoring request')
      return
    }

    try {
      setLoading(true)
      setIsAuthenticating(true)
      const { plugin } = providerWithStatus
      
      console.log(`Initiating login for ${plugin.displayName}`)
      
      // Special handling for ChatGPT - use backend authentication
      if (plugin.id === 'openai' || plugin.displayName === 'ChatGPT' || plugin.displayName === 'OpenAI') {
        try {
          console.log('🔑 Starting backend ChatGPT authentication...')
          // Use the backend ChatGPT authentication
          await invoke('authenticate_chatgpt')
          console.log('✅ Backend ChatGPT authentication completed')
          
          // Wait a moment for backend to update, then refresh provider statuses
          console.log('🔄 Waiting 2 seconds before refreshing provider statuses...')
          setTimeout(async () => {
            console.log('🔄 Refreshing provider statuses after authentication...')
            await refreshProviderStatuses()
            setIsAuthenticating(false)
          }, 2000)
          
        } catch (error) {
          console.error('❌ Backend ChatGPT authentication failed:', error)
          console.error('❌ Error details:', error)
          setIsAuthenticating(false)
        }
      } else {
        // Original plugin-based OAuth for other providers
        console.log(`Initiating OAuth for ${plugin.displayName} with command: ${plugin.authCommand}`)
        
        // Get OAuth URL from plugin
        const oauthUrl = await plugin.initiateOAuth()
        
        // Open OAuth URL in popup
        const popup = window.open(
          oauthUrl,
          `${plugin.displayName} OAuth`,
          'width=600,height=700,scrollbars=yes,resizable=yes'
        )
        
        // TODO: In a real implementation, listen for OAuth callback
        // For now, simulate success after a delay for demonstration
        setTimeout(async () => {
          try {
            // Simulate successful OAuth callback
            // In reality, this would be handled by the OAuth callback endpoint
            
            // Refresh the specific provider's status
            const updatedStatus = await plugin.getConnectionStatus()
            
            setProviders(prev => prev.map(p => 
              p.plugin.id === plugin.id 
                ? { ...p, status: updatedStatus }
                : p
            ))
            
            popup?.close()
          } catch (error) {
            console.error(`Failed to update ${plugin.displayName} status:`, error)
          }
        }, 3000)
      }
      
    } catch (error) {
      console.error('Login failed:', error)
      setIsAuthenticating(false)
    } finally {
      setLoading(false)
    }
  }


  const handleConfigure = (providerWithStatus: ProviderWithStatus) => {
    console.log(`Opening configuration for ${providerWithStatus.plugin.displayName}`)
    setConfigModalOpen(providerWithStatus.plugin.id)
  }

  const getStatusDot = (status: ProviderStatus['status']) => {
    switch (status) {
      case 'connected':
        return <div className="provider-status-dot provider-status-dot--connected" />
      case 'error':
      case 'expired':
        return <div className="provider-status-dot provider-status-dot--error" />
      default:
        return <div className="provider-status-dot provider-status-dot--disconnected" />
    }
  }

  return (
    <div className="card card--elevated">
      <div className="card__header">
        <div className="card__header-flex">
          <h2 className="card__title">AI Providers</h2>
          <div className="providers-summary">
            <span className="text-secondary text-sm">
              {providers.filter(p => p.status.status === 'connected').length}/{providers.length} connected
            </span>
          </div>
        </div>
      </div>
      
      <div className="card__content">
        <div className="providers-list">
          {providers.map((providerWithStatus) => {
            const { plugin, status } = providerWithStatus
            return (
              <div key={plugin.id} className="provider-item">
                <div className="provider-info">
                  <div className="provider-header">
                    <div className="provider-name-section">
                      {getStatusDot(status.status)}
                      <span className="provider-name">{plugin.displayName}</span>
                    </div>
                  </div>
                  
                  {/* For Ollama, show model on separate row */}
                  {plugin.id === 'ollama' && status.status === 'connected' && status.connectionInfo?.model && (
                    <div className="provider-model-row">
                      <span className="provider-model-name">{status.connectionInfo.model}</span>
                    </div>
                  )}
                  
                  {/* Status-based content */}
                  {status.status === 'connected' ? (
                    // Connected providers - show provider-specific details
                    (plugin.id === 'openai' || plugin.displayName === 'ChatGPT' || plugin.displayName === 'OpenAI') ? (
                      // ChatGPT - show minimal details (plan only)
                      <div className="provider-details">
                        {status.connectionInfo?.plan && (
                          <div className="provider-detail">
                            <span className="provider-detail-label">Plan:</span>
                            <span className="provider-detail-value">{status.connectionInfo.plan}</span>
                          </div>
                        )}
                      </div>
                    ) : plugin.id !== 'ollama' && status.connectionInfo ? (
                      // Other providers (non-Ollama, non-ChatGPT) - show full details
                      <div className="provider-details">
                        {status.connectionInfo.model && (
                          <div className="provider-detail">
                            <span className="provider-detail-label">Model:</span>
                            <span className="provider-detail-value">{status.connectionInfo.model}</span>
                          </div>
                        )}
                        {status.connectionInfo.lastUsed && (
                          <div className="provider-detail">
                            <span className="provider-detail-label">Last used:</span>
                            <span className="provider-detail-value">
                              {formatLastUsed(status.connectionInfo.lastUsed)}
                            </span>
                          </div>
                        )}
                        {status.connectionInfo.plan && (
                          <div className="provider-detail">
                            <span className="provider-detail-label">Plan:</span>
                            <span className="provider-detail-value">{status.connectionInfo.plan}</span>
                          </div>
                        )}
                      </div>
                    ) : null
                  ) : status.status === 'error' ? (
                    <div className="provider-disconnected">
                      <span className="text-secondary">Error: {status.error || 'Connection failed'}</span>
                    </div>
                  ) : plugin.id !== 'ollama' ? (
                    <div className="provider-disconnected">
                      <span className="text-secondary">Not authenticated</span>
                    </div>
                  ) : null}
                </div>
                
                <div className="provider-actions">
                  {status.status === 'connected' ? (
                    <div className="provider-connected-section">
                      {plugin.supportsConfiguration && (
                        <button
                          className="btn btn--ghost btn--sm"
                          onClick={() => handleConfigure(providerWithStatus)}
                          title={`Configure ${plugin.displayName}`}
                        >
                          <Settings className="btn__icon" />
                        </button>
                      )}
                    </div>
                  ) : (
                    <div className="provider-disconnected-section">
                      <button
                        className="btn btn--primary btn--sm"
                        onClick={() => handleLogin(providerWithStatus)}
                        disabled={loading || isAuthenticating}
                        title={`Login to ${plugin.displayName}`}
                      >
                        <LogIn className="btn__icon" />
                        {isAuthenticating && plugin.displayName === 'ChatGPT' ? 'Authenticating...' : 'Login'}
                      </button>
                      {plugin.supportsConfiguration && (
                        <button
                          className="btn btn--ghost btn--sm"
                          onClick={() => handleConfigure(providerWithStatus)}
                          disabled={loading}
                          title={`Configure ${plugin.displayName}`}
                        >
                          <Settings className="btn__icon" />
                        </button>
                      )}
                    </div>
                  )}
                </div>
              </div>
            )
          })}
        </div>
        
        <div className="providers-footer">
          <p className="text-muted providers-footer-text">
            Connect your AI providers to enable seamless model access through MindLink
          </p>
        </div>
      </div>

      {/* Configuration Modals */}
      {configModalOpen === 'ollama' && (
        <OllamaConfigModal
          isOpen={true}
          onClose={() => setConfigModalOpen(null)}
          plugin={providers.find(p => p.plugin.id === 'ollama')?.plugin as OllamaPlugin}
          onConfigSaved={refreshProviderStatuses}
        />
      )}
    </div>
  )
}

export default ProvidersCard