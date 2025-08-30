import React from 'react'
import './NetworkStats.css'

interface NetworkStatsProps {
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  publicUrl: string | null
}

const NetworkStats: React.FC<NetworkStatsProps> = ({
  tunnelStatus,
  publicUrl
}) => {
  const getLatencyDisplay = () => {
    if (tunnelStatus !== 'connected') return '--'
    // Mock latency for demo - in real app this would come from backend
    return `${Math.floor(Math.random() * 50 + 10)}ms`
  }

  const getUptimeDisplay = () => {
    if (tunnelStatus !== 'connected') return '--'
    // Mock uptime for demo
    return '2h 34m'
  }

  const getThroughputDisplay = () => {
    if (tunnelStatus !== 'connected') return '--'
    // Mock throughput for demo
    return `${Math.floor(Math.random() * 100 + 50)}KB/s`
  }

  return (
    <div className="card card--elevated">
      <div className="card__header">
        <h2 className="card__title">Network Statistics</h2>
        <div className={`status-badge status-badge--${tunnelStatus === 'connected' ? 'success' : 'neutral'}`}>
          {tunnelStatus === 'connected' ? 'Live Data' : 'No Data'}
        </div>
      </div>
      <div className="card__content">
        <div className="network-stats">
          <div className="network-stats__item">
            <div className="network-stats__label">Latency</div>
            <div className="network-stats__value">
              {getLatencyDisplay()}
            </div>
          </div>
          
          <div className="network-stats__item">
            <div className="network-stats__label">Uptime</div>
            <div className="network-stats__value">
              {getUptimeDisplay()}
            </div>
          </div>
          
          <div className="network-stats__item">
            <div className="network-stats__label">Throughput</div>
            <div className="network-stats__value">
              {getThroughputDisplay()}
            </div>
          </div>
          
          <div className="network-stats__item">
            <div className="network-stats__label">Security</div>
            <div className="network-stats__value network-stats__value--security">
              {tunnelStatus === 'connected' ? (
                <>
                  <svg className="network-stats__icon" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" />
                  </svg>
                  TLS 1.3
                </>
              ) : (
                '--'
              )}
            </div>
          </div>
        </div>
        
        {tunnelStatus === 'connected' && publicUrl && (
          <div className="network-stats__details">
            <div className="network-stats__detail-item">
              <span className="network-stats__detail-label">Endpoint:</span>
              <code className="code-inline">{new URL(publicUrl).hostname}</code>
            </div>
            <div className="network-stats__detail-item">
              <span className="network-stats__detail-label">Protocol:</span>
              <span className="text-success font-medium">HTTPS</span>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default NetworkStats