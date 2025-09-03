import React, { useState } from 'react'
// import { Settings } from 'lucide-react' // Not currently used
import { invoke } from '@tauri-apps/api/core'
import BifrostButton from './BifrostButton'
import './UnifiedNavigation.css'

interface UnifiedNavigationProps {
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  isAuthenticated: boolean
  onToggleTunnel: () => void
  onBifrostError: (message: string) => void
  onAuthSuccess?: () => void
}

const UnifiedNavigation: React.FC<UnifiedNavigationProps> = ({
  tunnelStatus,
  isAuthenticated,
  onToggleTunnel,
  onBifrostError,
  onAuthSuccess,
}) => {
  const [isAuthenticating, setIsAuthenticating] = useState(false)
  const canToggle = isAuthenticated && tunnelStatus !== 'connecting'

  const handleChatGPTLogin = async () => {
    if (isAuthenticating) return
    
    setIsAuthenticating(true)
    try {
      console.log('Starting ChatGPT OAuth flow...')
      await invoke('authenticate_chatgpt')
      console.log('ChatGPT authentication successful!')
      onAuthSuccess?.()
    } catch (error) {
      console.error('ChatGPT authentication failed:', error)
      onBifrostError(`Authentication failed: ${error}`)
    } finally {
      setIsAuthenticating(false)
    }
  }

  return (
    <nav className="unified-nav">
      <div className="unified-nav__container">
        {/* Brand Section - Left Side */}
        <div className="unified-nav__brand">
          <img src="/logo.webp" alt="MindLink" className="unified-nav__logo" />
          <div className="unified-nav__brand-text">
            <h1 className="unified-nav__title">MindLink</h1>
            <span className="unified-nav__subtitle">Network Gateway</span>
          </div>
        </div>

        {/* Controls Section - Right Side */}
        <div className="unified-nav__controls">
          <BifrostButton onError={onBifrostError} />
          
          {/* Tunnel Toggle */}
          {isAuthenticated ? (
            <div className="unified-nav__tunnel-control">
              <label className="tunnel-toggle">
                <span className="tunnel-toggle__label">Tunnel</span>
                <input
                  type="checkbox"
                  className="tunnel-toggle__input"
                  checked={tunnelStatus === 'connected'}
                  onChange={onToggleTunnel}
                  disabled={!canToggle}
                />
                <div className={`tunnel-toggle__track tunnel-toggle__track--${tunnelStatus}`}>
                  <div className="tunnel-toggle__thumb"></div>
                </div>
              </label>
            </div>
          ) : (
            <div className="unified-nav__auth-section">
              <button
                className="btn btn--primary unified-nav__login-btn"
                onClick={handleChatGPTLogin}
                disabled={isAuthenticating}
              >
                {isAuthenticating ? (
                  <>
                    <div className="unified-nav__spinner"></div>
                    Authenticating...
                  </>
                ) : (
                  <>
                    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" className="unified-nav__chatgpt-icon">
                      <circle cx="8" cy="8" r="7" fill="#10a37f"/>
                      <path d="M5 6h6c.55 0 1 .45 1 1v2c0 .55-.45 1-1 1H5c-.55 0-1-.45-1-1V7c0-.55.45-1 1-1z" fill="white"/>
                    </svg>
                    Login with ChatGPT
                  </>
                )}
              </button>
            </div>
          )}
        </div>
      </div>
    </nav>
  )
}

export default UnifiedNavigation