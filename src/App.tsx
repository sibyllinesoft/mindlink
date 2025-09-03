import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Dashboard from './components/Dashboard'
import UnifiedNavigation from './components/UnifiedNavigation'
import type { StatusResponse, ServiceResponse } from './types/api'
import './design-system/index.css'
import './App.css'

interface AppState {
  serverStatus: 'running' | 'error' // Server is always running
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  publicUrl: string | null
  isAuthenticated: boolean
  autoStartAttempted: boolean
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
    authCheckComplete: false,
    errorMessage: null
  })

  useEffect(() => {
    // Initialize the app with auth check and auto-start behavior
    const initApp = async () => {
      try {
        // Check actual ChatGPT authentication status
        const isAuthenticated = await invoke<boolean>('check_chatgpt_auth_status')
        
        setState(prev => ({ 
          ...prev, 
          isAuthenticated,
          authCheckComplete: true,
          serverStatus: 'running' // Server is always running
        }))
        
        // Proceed with tunnel setup
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
      } catch (error) {
        console.error('Failed to initialize app:', error)
        // If we can't check auth status, assume not authenticated
        setState(prev => ({ 
          ...prev, 
          isAuthenticated: false,
          authCheckComplete: true,
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
      const isAuthenticated = await invoke<boolean>('check_chatgpt_auth_status')
      
      setState(prev => ({ 
        ...prev, 
        isAuthenticated,
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


  const handleBifrostError = (message: string) => {
    setState(prev => ({ ...prev, errorMessage: message }))
    // Auto-clear error after 5 seconds
    setTimeout(() => {
      setState(prev => ({ ...prev, errorMessage: null }))
    }, 5000)
  }

  return (
    <div className="app">

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
            onAuthSuccess={handleAuthSuccess}
          />

          {/* Main Content */}
          <main className="app-main">
            {state.isAuthenticated ? (
              <Dashboard
                serverStatus={state.serverStatus}
                tunnelStatus={state.tunnelStatus}
                publicUrl={state.publicUrl}
                isAuthenticated={state.isAuthenticated}
                onToggleTunnel={handleToggleTunnel}
              />
            ) : (
              <div className="auth-welcome">
                <div className="auth-welcome__content">
                  <div className="auth-welcome__icon">
                    <svg width="64" height="64" viewBox="0 0 64 64" fill="none" className="auth-welcome__chatgpt-logo">
                      <circle cx="32" cy="32" r="28" fill="#10a37f"/>
                      <path d="M20 24h24c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H20c-1.1 0-2-.9-2-2V26c0-1.1.9-2 2-2z" fill="white"/>
                    </svg>
                  </div>
                  <h2 className="auth-welcome__title">Welcome to MindLink</h2>
                  <p className="auth-welcome__description">
                    Connect your ChatGPT Plus/Pro account to start using MindLink as an OpenAI-compatible API gateway.
                  </p>
                  <div className="auth-welcome__features">
                    <div className="auth-welcome__feature">
                      <span className="auth-welcome__feature-icon">🔑</span>
                      <span>Uses your existing ChatGPT subscription</span>
                    </div>
                    <div className="auth-welcome__feature">
                      <span className="auth-welcome__feature-icon">🚀</span>
                      <span>OpenAI-compatible API endpoints</span>
                    </div>
                    <div className="auth-welcome__feature">
                      <span className="auth-welcome__feature-icon">🔒</span>
                      <span>Secure local processing</span>
                    </div>
                    <div className="auth-welcome__feature">
                      <span className="auth-welcome__feature-icon">🌐</span>
                      <span>Optional public tunnel access</span>
                    </div>
                  </div>
                  <p className="auth-welcome__cta">
                    Click <strong>"Login with ChatGPT"</strong> in the navigation bar above to get started.
                  </p>
                </div>
              </div>
            )}
          </main>
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