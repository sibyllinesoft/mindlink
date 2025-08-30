import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Dashboard from './components/Dashboard'
import UnifiedNavigation from './components/UnifiedNavigation'
import OAuthModal from './components/OAuthModal'
import type { StatusResponse, ServiceResponse } from './types/api'
import './design-system/index.css'
import './App.css'

interface AppState {
  serverStatus: 'running' | 'error' // Server is always running
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  publicUrl: string | null
  isAuthenticated: boolean
  autoStartAttempted: boolean
  showOAuthModal: boolean
  authCheckComplete: boolean
  errorMessage: string | null
}

function App() {
  const [state, setState] = useState<AppState>({
    serverStatus: 'running', // Server starts automatically
    tunnelStatus: 'disconnected',
    publicUrl: null,
    isAuthenticated: false,
    autoStartAttempted: false,
    showOAuthModal: false,
    authCheckComplete: false,
    errorMessage: null
  })

  useEffect(() => {
    // Initialize the app with auth check and auto-start behavior
    const initApp = async () => {
      try {
        // Get initial status including authentication
        const status = await invoke<StatusResponse>('get_status')
        const isAuthenticated = status?.is_authenticated === true
        
        setState(prev => ({ 
          ...prev, 
          isAuthenticated,
          authCheckComplete: true,
          showOAuthModal: !isAuthenticated,
          serverStatus: 'running' // Server is always running
        }))
        
        // Only proceed with tunnel setup if authenticated
        if (isAuthenticated) {
          // Get initial tunnel status
          try {
            const tunnelResponse = await invoke<ServiceResponse>('get_tunnel_status')
            const tunnelStatus = tunnelResponse?.success ? 
              (tunnelResponse.tunnel_url ? 'connected' : 'disconnected') : 'disconnected'
            
            // Set both tunnel status and public URL from initial response
            setState(prev => ({ 
              ...prev, 
              tunnelStatus: tunnelStatus as any,
              publicUrl: tunnelResponse?.tunnel_url || null
            }))
            
            // Auto-start tunnel if not already connected
            if (tunnelStatus !== 'connected') {
              setState(prev => ({ ...prev, autoStartAttempted: true, tunnelStatus: 'connecting' }))
              try {
                const tunnelResponse = await invoke<ServiceResponse>('create_tunnel')
                if (tunnelResponse?.success) {
                  setState(prev => ({ 
                    ...prev, 
                    tunnelStatus: 'connected',
                    publicUrl: tunnelResponse.tunnel_url || null
                  }))
                } else {
                  setState(prev => ({ ...prev, tunnelStatus: 'error' }))
                }
              } catch (error) {
                console.error('Auto-start tunnel failed:', error)
                setState(prev => ({ ...prev, tunnelStatus: 'error' }))
              }
            }
          } catch (error) {
            console.error('Failed to check tunnel status:', error)
            setState(prev => ({ ...prev, tunnelStatus: 'error' }))
          }
        }
      } catch (error) {
        console.error('Failed to initialize app:', error)
        // If we can't check auth status, assume not authenticated
        setState(prev => ({ 
          ...prev, 
          isAuthenticated: false,
          authCheckComplete: true,
          showOAuthModal: true
        }))
      }
    }

    initApp()

    // Set up event listeners for real-time updates
    const unsubscribeListeners: (() => void)[] = []

    // Listen for server status changes
    listen('server-status-changed', (event) => {
      setState(prev => ({ ...prev, serverStatus: event.payload as any }))
    }).then(unsub => unsubscribeListeners.push(unsub))

    // Listen for tunnel status changes
    listen('tunnel-status-changed', (event) => {
      setState(prev => ({ ...prev, tunnelStatus: event.payload as any }))
    }).then(unsub => unsubscribeListeners.push(unsub))

    // Listen for public URL updates
    listen('public-url-changed', (event) => {
      setState(prev => ({ ...prev, publicUrl: event.payload as string }))
    }).then(unsub => unsubscribeListeners.push(unsub))

    // Listen for auth status changes
    listen('auth-status-changed', (event) => {
      const isAuthenticated = event.payload as boolean
      setState(prev => ({ 
        ...prev, 
        isAuthenticated,
        showOAuthModal: !isAuthenticated
      }))
    }).then(unsub => unsubscribeListeners.push(unsub))

    // Cleanup listeners on unmount
    return () => {
      unsubscribeListeners.forEach(unsub => unsub())
    }
  }, [])

  // Server management removed - server is always running

  const handleToggleTunnel = async () => {
    try {
      if (state.tunnelStatus === 'connected') {
        const response = await invoke<ServiceResponse>('close_tunnel')
        if (response?.success) {
          setState(prev => ({ ...prev, tunnelStatus: 'disconnected', publicUrl: null }))
        } else {
          setState(prev => ({ ...prev, tunnelStatus: 'error' }))
        }
      } else if (state.tunnelStatus === 'disconnected') {
        setState(prev => ({ ...prev, tunnelStatus: 'connecting' }))
        const response = await invoke<ServiceResponse>('create_tunnel')
        if (response?.success) {
          setState(prev => ({ 
            ...prev, 
            tunnelStatus: 'connected',
            publicUrl: response.tunnel_url || null
          }))
        } else {
          setState(prev => ({ ...prev, tunnelStatus: 'error' }))
        }
      }
    } catch (error) {
      console.error('Failed to toggle tunnel:', error)
      setState(prev => ({ ...prev, tunnelStatus: 'error' }))
    }
  }

  // OAuth modal handlers
  const handleAuthSuccess = async () => {
    // Refresh status after successful authentication
    try {
      const status = await invoke<StatusResponse>('get_status')
      const isAuthenticated = status?.is_authenticated === true
      
      setState(prev => ({ 
        ...prev, 
        isAuthenticated,
        showOAuthModal: false
      }))

      // Auto-start tunnel after successful authentication
      if (isAuthenticated) {
        setState(prev => ({ ...prev, autoStartAttempted: true, tunnelStatus: 'connecting' }))
        try {
          const response = await invoke<ServiceResponse>('create_tunnel')
          if (response?.success) {
            setState(prev => ({ 
              ...prev, 
              tunnelStatus: 'connected',
              publicUrl: response.tunnel_url || null
            }))
          } else {
            setState(prev => ({ ...prev, tunnelStatus: 'error' }))
          }
        } catch (error) {
          console.error('Failed to start tunnel after auth:', error)
          setState(prev => ({ ...prev, tunnelStatus: 'error' }))
        }
      }
    } catch (error) {
      console.error('Failed to refresh status after auth:', error)
    }
  }

  const handleAuthCancel = () => {
    setState(prev => ({ ...prev, showOAuthModal: false }))
  }

  const handleBifrostError = (message: string) => {
    setState(prev => ({ ...prev, errorMessage: message }))
    // Auto-clear error after 5 seconds
    setTimeout(() => {
      setState(prev => ({ ...prev, errorMessage: null }))
    }, 5000)
  }

  return (
    <div className="app">
      {/* OAuth Modal - shown when authentication is required */}
      <OAuthModal
        isOpen={state.showOAuthModal && state.authCheckComplete}
        onAuthSuccess={handleAuthSuccess}
        onCancel={handleAuthCancel}
      />

      {/* Show main app when auth check is complete */}
      {state.authCheckComplete && (
        <>
          {/* Error Message Display */}
          {state.errorMessage && (
            <div className="error-banner">
              <div className="error-banner__content">
                <span className="error-banner__text">{state.errorMessage}</span>
                <button 
                  className="error-banner__close"
                  onClick={() => setState(prev => ({ ...prev, errorMessage: null }))}
                  title="Dismiss"
                >
                  ×
                </button>
              </div>
            </div>
          )}

          {/* Unified Navigation - Clean single top bar */}
          <UnifiedNavigation
            tunnelStatus={state.tunnelStatus}
            isAuthenticated={state.isAuthenticated}
            onToggleTunnel={handleToggleTunnel}
            onBifrostError={handleBifrostError}
          />

          {/* Main Content - Only show Dashboard when authenticated */}
          {state.isAuthenticated && (
            <main className="app-main">
              <Dashboard
                serverStatus={state.serverStatus}
                tunnelStatus={state.tunnelStatus}
                publicUrl={state.publicUrl}
                isAuthenticated={state.isAuthenticated}
                onToggleTunnel={handleToggleTunnel}
              />
            </main>
          )}
        </>
      )}

      {/* Loading state while checking authentication */}
      {!state.authCheckComplete && (
        <div className="app-loading">
          <div className="app-loading__content">
            <div className="app-loading__spinner"></div>
            <p>Initializing MindLink...</p>
          </div>
        </div>
      )}
    </div>
  )
}

export default App