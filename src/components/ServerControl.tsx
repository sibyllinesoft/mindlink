import React from 'react'
import './ServerControl.css'

interface ServerControlProps {
  status: 'stopped' | 'starting' | 'running' | 'error'
  isAuthenticated: boolean
  onStart: () => void
  onStop: () => void
}

const ServerControl: React.FC<ServerControlProps> = ({
  status,
  isAuthenticated,
  onStart,
  onStop,
}) => {
  const getStatusDisplay = () => {
    switch (status) {
      case 'stopped':
        return { text: 'Stopped', className: 'stopped' }
      case 'starting':
        return { text: 'Starting...', className: 'starting' }
      case 'running':
        return { text: 'Running', className: 'running' }
      case 'error':
        return { text: 'Error', className: 'error' }
      default:
        return { text: 'Unknown', className: 'error' }
    }
  }

  const statusDisplay = getStatusDisplay()

  return (
    <div className="card server-control">
      <h2>Local API Server</h2>
      
      <div className="server-status">
        <div className={`status-indicator ${statusDisplay.className}`}>
          {statusDisplay.text}
        </div>
        
        {status === 'running' && (
          <div className="server-info">
            <p className="text-muted">Running on localhost:8080</p>
          </div>
        )}
      </div>

      {!isAuthenticated && (
        <div className="auth-warning">
          <p>⚠️ ChatGPT authentication required</p>
          <p className="text-muted">Please authenticate with ChatGPT in Settings before starting the server.</p>
        </div>
      )}

      <div className="server-controls">
        {status === 'starting' ? (
          <button
            className="btn btn-primary"
            disabled={true}
          >
            <span className="spinner" />
            Starting...
          </button>
        ) : status === 'running' ? (
          <button
            className="btn btn-danger"
            onClick={onStop}
          >
            Stop Server
          </button>
        ) : (
          <button
            className="btn btn-primary"
            onClick={onStart}
            disabled={!isAuthenticated}
          >
            Start Server
          </button>
        )}
      </div>

      <div className="server-description">
        <p className="text-muted">
          The local API server provides an OpenAI-compatible interface powered by your ChatGPT account.
        </p>
      </div>
    </div>
  )
}

export default ServerControl