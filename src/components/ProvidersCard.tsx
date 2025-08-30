import React, { useState, useEffect } from 'react'
import { CheckCircle, AlertCircle, LogIn } from 'lucide-react'
import { pluginLoader } from '../plugins/dynamic-loader'
import { pluginRegistry } from '../plugins/registry'
import type { ProviderPlugin, ProviderStatus } from '../plugins/types'
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
    try {
      setLoading(true)
      const { plugin } = providerWithStatus
      
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
        } finally {
          setLoading(false)
        }
      }, 3000)
      
    } catch (error) {
      console.error('OAuth login failed:', error)
      setLoading(false)
    }
  }

  const getStatusIcon = (status: ProviderStatus['status']) => {
    switch (status) {
      case 'connected':
        return <CheckCircle className="provider-status-icon provider-status-icon--connected" />
      case 'error':
      case 'expired':
        return <AlertCircle className="provider-status-icon provider-status-icon--error" />
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
                      <span className="provider-name">{plugin.displayName}</span>
                      {getStatusIcon(status.status)}
                    </div>
                  </div>
                  
                  {status.status === 'connected' && status.connectionInfo ? (
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
                  ) : status.status === 'error' ? (
                    <div className="provider-disconnected">
                      <span className="text-secondary">Error: {status.error || 'Connection failed'}</span>
                    </div>
                  ) : (
                    <div className="provider-disconnected">
                      <span className="text-secondary">Not authenticated</span>
                    </div>
                  )}
                </div>
                
                <div className="provider-actions">
                  {status.status === 'connected' ? (
                    <div className="provider-connected-badge">
                      <CheckCircle className="provider-connected-icon" />
                      <span className="text-success">Connected</span>
                    </div>
                  ) : (
                    <button
                      className="btn btn--primary btn--sm"
                      onClick={() => handleLogin(providerWithStatus)}
                      disabled={loading}
                      title={`Login to ${plugin.displayName}`}
                    >
                      <LogIn className="btn__icon" />
                      Login
                    </button>
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
    </div>
  )
}

export default ProvidersCard