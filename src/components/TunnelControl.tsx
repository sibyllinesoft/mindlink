import React from 'react'
import './TunnelControl.css'

interface TunnelControlProps {
  status: 'disconnected' | 'connecting' | 'connected' | 'error'
  serverRunning: boolean
  onToggle: () => void
  isAuthenticated: boolean
}

const TunnelControl: React.FC<TunnelControlProps> = ({
  status,
  serverRunning,
  onToggle,
  isAuthenticated,
}) => {
  const getStatusDisplay = () => {
    switch (status) {
      case 'disconnected':
        return { text: 'Tunnel Offline', description: 'Ready to connect' }
      case 'connecting':
        return { text: 'Establishing Connection', description: 'Setting up secure tunnel...' }
      case 'connected':
        return { text: 'Tunnel Active', description: 'Secure connection established' }
      case 'error':
        return { text: 'Connection Failed', description: 'Please check authentication and try again' }
      default:
        return { text: 'Unknown Status', description: '' }
    }
  }

  const statusDisplay = getStatusDisplay()
  const canToggle = isAuthenticated && serverRunning && status !== 'connecting'

  return (
    <div className="card card--elevated">
      <div className="card__header">
        <div className="tunnel-control__header">
          <div className="tunnel-control__title-with-status">
            <div className={`status-indicator status-indicator--${status}`}>
              <div className="status-indicator__dot"></div>
            </div>
            <h2 className="card__title">Secure Tunnel</h2>
          </div>
          <div className="tunnel-control__status-text">
            {status === 'connected' ? 'Active' : status === 'connecting' ? 'Connecting' : 'Inactive'}
          </div>
        </div>
      </div>
      
      <div className="card__content">
        <div className="tunnel-status">
          <div className="tunnel-status__main">
            <h3 className="tunnel-status__title">{statusDisplay.text}</h3>
            <p className="tunnel-status__description text-secondary">
              {statusDisplay.description}
            </p>
          </div>
        </div>

        {!isAuthenticated && (
          <div className="tunnel-control__warning">
            <div className="tunnel-control__warning-icon">⚠️</div>
            <div>
              <p className="font-medium text-warning">Authentication Required</p>
              <p className="text-secondary text-sm">Please authenticate with Cloudflare to enable tunnel functionality.</p>
            </div>
          </div>
        )}

        {!serverRunning && isAuthenticated && (
          <div className="tunnel-control__warning">
            <div className="tunnel-control__warning-icon">🔧</div>
            <div>
              <p className="font-medium text-warning">Server Error</p>
              <p className="text-secondary text-sm">Local server must be running to establish tunnel connection.</p>
            </div>
          </div>
        )}

        <div className="tunnel-control__actions">
          <div className="tunnel-toggle-section">
            <div className="tunnel-toggle-header">
              <span className="tunnel-toggle-title">TUNNEL</span>
              <div className={`tunnel-status-indicator tunnel-status-indicator--${status}`}>
                <div className="tunnel-status-indicator__dot"></div>
                <span className="tunnel-status-indicator__text">
                  {status === 'connected' ? 'Active' : status === 'connecting' ? 'Connecting' : status === 'error' ? 'Error' : 'Inactive'}
                </span>
              </div>
            </div>
            <label className="toggle-btn">
              <input
                type="checkbox"
                className="toggle-btn__input"
                checked={status === 'connected'}
                onChange={onToggle}
                disabled={!canToggle}
              />
              <div className="toggle-btn__track">
                <div className="toggle-btn__thumb"></div>
              </div>
              <span className="toggle-btn__label">
                {status === 'connected' ? 'Tunnel Enabled' : 'Tunnel Disabled'}
              </span>
            </label>
          </div>
          
          {status === 'connecting' && (
            <div className="tunnel-control__connecting">
              <div className="btn btn--primary btn--loading">Establishing Connection</div>
            </div>
          )}
        </div>
      </div>

      <div className="card__footer">
        <p className="text-secondary text-sm">
          Secure tunnel routes external traffic through Cloudflare's global network to your local server.
        </p>
      </div>
    </div>
  )
}

export default TunnelControl