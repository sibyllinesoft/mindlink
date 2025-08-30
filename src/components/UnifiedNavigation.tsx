import React from 'react'
import { Settings } from 'lucide-react'
import BifrostButton from './BifrostButton'
import './UnifiedNavigation.css'

interface UnifiedNavigationProps {
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  isAuthenticated: boolean
  onToggleTunnel: () => void
  onBifrostError: (message: string) => void
}

const UnifiedNavigation: React.FC<UnifiedNavigationProps> = ({
  tunnelStatus,
  isAuthenticated,
  onToggleTunnel,
  onBifrostError,
}) => {
  const canToggle = isAuthenticated && tunnelStatus !== 'connecting'

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
            <div className="unified-nav__auth-notice">
              <span className="unified-nav__auth-text">Authentication required</span>
            </div>
          )}
        </div>
      </div>
    </nav>
  )
}

export default UnifiedNavigation