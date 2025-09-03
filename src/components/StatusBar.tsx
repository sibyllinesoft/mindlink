import React from 'react'
import './StatusBar.css'

interface StatusBarProps {
  serverStatus: 'stopped' | 'starting' | 'running' | 'error'
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  publicUrl: string | null
}

const StatusBar: React.FC<StatusBarProps> = ({
  serverStatus,
  tunnelStatus,
  publicUrl,
}) => {
  return (
    <div className="status-bar">
      <div className="status-section">
        <span className="status-label">Server:</span>
        <div className={`status-indicator ${serverStatus}`}>
          {serverStatus === 'stopped' && 'Stopped'}
          {serverStatus === 'starting' && 'Starting...'}
          {serverStatus === 'running' && 'Running'}
          {serverStatus === 'error' && 'Error'}
        </div>
      </div>

      <div className="status-section">
        <span className="status-label">Tunnel:</span>
        <div className={`status-indicator ${tunnelStatus}`}>
          {tunnelStatus === 'disconnected' && 'Disconnected'}
          {tunnelStatus === 'connecting' && 'Connecting...'}
          {tunnelStatus === 'connected' && 'Connected'}
          {tunnelStatus === 'error' && 'Error'}
        </div>
      </div>

      {publicUrl && (
        <div className="status-section status-url">
          <span className="status-label">Public URL:</span>
          <code className="status-url-text">{publicUrl}</code>
        </div>
      )}

      <div className="status-section status-version">
        <span className="status-label">v1.0.0</span>
      </div>
    </div>
  )
}

export default StatusBar