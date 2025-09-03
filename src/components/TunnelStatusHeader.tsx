import React from 'react'
import './TunnelStatusHeader.css'

interface TunnelStatusHeaderProps {
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  publicUrl: string | null
  isAuthenticated: boolean
  onToggleTunnel: () => void
  autoStartAttempted: boolean
}

const TunnelStatusHeader: React.FC<TunnelStatusHeaderProps> = ({
  tunnelStatus,
  publicUrl,
  isAuthenticated,
  onToggleTunnel,
  autoStartAttempted,
}) => {
  const getStatusText = () => {
    switch (tunnelStatus) {
      case 'connected':
        return 'Tunnel Active'
      case 'connecting':
        return autoStartAttempted ? 'Auto-connecting...' : 'Connecting...'
      case 'disconnected':
        return 'Tunnel Offline'
      case 'error':
        return 'Connection Error'
      default:
        return 'Unknown Status'
    }
  }

  const getStatusDescription = () => {
    switch (tunnelStatus) {
      case 'connected':
        return `Secure tunnel established ${publicUrl ? `• ${publicUrl}` : ''}`
      case 'connecting':
        return 'Establishing secure connection...'
      case 'disconnected':
        return 'No active tunnel connection'
      case 'error':
        return 'Failed to establish tunnel connection'
      default:
        return ''
    }
  }

  const canToggle = isAuthenticated && tunnelStatus !== 'connecting'

  return (
    <header className="tunnel-status-header">
      <div className="tunnel-status-header__container">
        <div className="tunnel-status-header__status">
          <div className={`status-indicator status-indicator--${tunnelStatus}`}>
            <div className="status-indicator__dot"></div>
            <div className="status-indicator__content">
              <div className="status-indicator__title">
                {getStatusText()}
              </div>
              <div className="status-indicator__description">
                {getStatusDescription()}
              </div>
            </div>
          </div>
        </div>

        <div className="tunnel-status-header__controls">
          {!isAuthenticated ? (
            <div className="tunnel-status-header__auth-notice">
              <span className="text-secondary text-sm">
                Authentication required to enable tunnel
              </span>
            </div>
          ) : (
            <div className="tunnel-control-group">
              <div className="tunnel-control-label">
                <span className="tunnel-control-label__text">Tunnel</span>
                <div className={`tunnel-indicator tunnel-indicator--${tunnelStatus}`}>
                  <div className="tunnel-indicator__dot"></div>
                </div>
              </div>
              <label className="toggle-btn">
                <input
                  type="checkbox"
                  className="toggle-btn__input"
                  checked={tunnelStatus === 'connected'}
                  onChange={onToggleTunnel}
                  disabled={!canToggle}
                />
                <div className="toggle-btn__track">
                  <div className="toggle-btn__thumb"></div>
                </div>
                <span className="toggle-btn__label">
                  {tunnelStatus === 'connected' ? 'Enabled' : 'Disabled'}
                </span>
              </label>
            </div>
          )}
        </div>
      </div>

      {/* Connection Details */}
      {tunnelStatus === 'connected' && publicUrl && (
        <div className="tunnel-status-header__details">
          <div className="tunnel-status-header__container">
            <div className="connection-details">
              <div className="connection-details__item">
                <span className="connection-details__label">Public URL:</span>
                <code className="connection-details__value">{publicUrl}</code>
                <button
                  className="btn btn--ghost btn--sm"
                  onClick={() => navigator.clipboard.writeText(publicUrl)}
                  title="Copy to clipboard"
                >
                  Copy
                </button>
              </div>
              <div className="connection-details__item">
                <span className="connection-details__label">Security:</span>
                <span className="connection-details__value text-success">
                  End-to-end encrypted
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </header>
  )
}

export default TunnelStatusHeader