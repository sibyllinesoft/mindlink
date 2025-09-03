import React from 'react'
import { X, Activity, Calendar, Key, BarChart3, Clock, Trash2, FileText } from 'lucide-react'
// import type { AppUsageStats, BifrostLogEntry } from '../services/bifrostService' // Not currently used
import './AppDetailsModal.css'

interface App {
  id: string
  name: string
  virtualKey: string
  createdAt: string
  lastUsed: string
  requestCount: number
  status: 'active' | 'revoked' | 'expired'
}


interface AppDetailsModalProps {
  app: App | null
  isOpen: boolean
  onClose: () => void
  onRevoke: (appId: string) => void
}

const AppDetailsModal: React.FC<AppDetailsModalProps> = ({
  app,
  isOpen,
  onClose,
  onRevoke
}) => {
  if (!isOpen || !app) return null

  
  const mockStats = {
    totalRequests: app.requestCount || 123,
    requestsToday: 12,
    requestsThisWeek: 45,
    requestsThisMonth: 123,
    totalTokens: 18450,
    averageLatency: 245,
    successRate: 98.5,
    lastUsed: app.lastUsed,
    providers: { 'openai': 123 },
    models: { 'gpt-4': 86, 'gpt-3.5-turbo': 37 }
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  const handleRevoke = () => {
    onRevoke(app.id)
    onClose()
  }

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose()
    }
  }

  return (
    <div className="modal-backdrop" onClick={handleBackdropClick}>
      <div className="modal modal--lg">
        <div className="modal__header">
          <div className="modal__title-section">
            <h3 className="modal__title">{app.name}</h3>
            <div className={`app-status app-status--${app.status}`}>
              {app.status}
            </div>
          </div>
          <button
            className="btn btn--icon btn--ghost"
            onClick={onClose}
            title="Close"
          >
            <X className="btn__icon" />
          </button>
        </div>

        <div className="modal__content">
          {/* App Information */}
          <section className="app-details-section">
            <h4 className="section-title">Application Details</h4>
            <div className="app-details-grid">
              <div className="detail-item">
                <div className="detail-icon">
                  <Key className="icon" />
                </div>
                <div className="detail-content">
                  <label className="detail-label">Virtual Key</label>
                  <code className="detail-value virtual-key-display">{app.virtualKey}</code>
                </div>
              </div>

              <div className="detail-item">
                <div className="detail-icon">
                  <Calendar className="icon" />
                </div>
                <div className="detail-content">
                  <label className="detail-label">Authorized</label>
                  <span className="detail-value">{formatDate(app.createdAt)}</span>
                </div>
              </div>

              <div className="detail-item">
                <div className="detail-icon">
                  <Clock className="icon" />
                </div>
                <div className="detail-content">
                  <label className="detail-label">Last Used</label>
                  <span className="detail-value">{formatDate(app.lastUsed)}</span>
                </div>
              </div>
            </div>
          </section>

          {/* Usage Statistics */}
          <section className="app-details-section">
            <h4 className="section-title">Usage Statistics</h4>
            <div className="usage-stats-grid">
              <div className="stat-card">
                <div className="stat-header">
                  <Activity className="stat-icon" />
                  <span className="stat-label">Today</span>
                </div>
                <div className="stat-value">{mockStats.requestsToday}</div>
                <div className="stat-unit">requests</div>
              </div>

              <div className="stat-card">
                <div className="stat-header">
                  <BarChart3 className="stat-icon" />
                  <span className="stat-label">This Week</span>
                </div>
                <div className="stat-value">{mockStats.requestsThisWeek}</div>
                <div className="stat-unit">requests</div>
              </div>

              <div className="stat-card">
                <div className="stat-header">
                  <BarChart3 className="stat-icon" />
                  <span className="stat-label">This Month</span>
                </div>
                <div className="stat-value">{mockStats.requestsThisMonth}</div>
                <div className="stat-unit">requests</div>
              </div>

              <div className="stat-card">
                <div className="stat-header">
                  <Clock className="stat-icon" />
                  <span className="stat-label">Avg Latency</span>
                </div>
                <div className="stat-value">{mockStats.averageLatency}</div>
                <div className="stat-unit">ms</div>
              </div>
            </div>
          </section>

          {/* Performance Metrics */}
          <section className="app-details-section">
            <h4 className="section-title">Performance</h4>
            <div className="performance-grid">
              <div className="performance-item">
                <label className="performance-label">Total Tokens</label>
                <span className="performance-value">{(mockStats.totalTokens / 1000).toFixed(1)}K</span>
              </div>
              <div className="performance-item">
                <label className="performance-label">Average Latency</label>
                <span className="performance-value">{mockStats.averageLatency}ms</span>
              </div>
              <div className="performance-item">
                <label className="performance-label">Success Rate</label>
                <span className="performance-value">{mockStats.successRate}%</span>
              </div>
            </div>
          </section>

          {/* Recent Activity - Simplified */}
          <section className="app-details-section">
            <h4 className="section-title">Recent Activity (24h)</h4>
            <div className="no-logs">
              <FileText className="no-logs-icon" />
              <p>No recent activity logged</p>
              <p className="text-muted">Connect Bifrost service to view request logs</p>
            </div>
          </section>
        </div>

        <div className="modal__footer">
          <div className="modal__actions">
            <button className="btn btn--secondary" onClick={onClose}>
              Close
            </button>
            {app.status === 'active' && (
              <button 
                className="btn btn--danger"
                onClick={handleRevoke}
              >
                <Trash2 className="btn__icon" />
                Revoke Access
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default AppDetailsModal