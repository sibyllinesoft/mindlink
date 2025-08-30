import React, { useState, useEffect } from 'react'
import { Settings, Trash2, Activity, Calendar, Key } from 'lucide-react'
import { bifrostService, AppUsageStats } from '../services/bifrostService'
import './AppsCard.css'

interface App {
  id: string
  name: string
  virtualKey: string
  createdAt: string
  lastUsed: string
  requestCount: number
  status: 'active' | 'revoked' | 'expired'
}

interface AppsCardProps {
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  onAppClick: (app: App) => void
}

const AppsCard: React.FC<AppsCardProps> = ({ tunnelStatus, onAppClick }) => {
  const [apps, setApps] = useState<App[]>([])
  const [loading, setLoading] = useState(false)

  // Mock data for now - will be replaced with actual Bifrost API call
  const mockApps: App[] = [
    {
      id: '1',
      name: 'ChatGPT Extension',
      virtualKey: 'vk_ch4t6p7_x9y2z',
      createdAt: '2024-08-25T10:30:00Z',
      lastUsed: '2024-08-29T18:45:00Z',
      requestCount: 1247,
      status: 'active'
    },
    {
      id: '2', 
      name: 'Claude Desktop',
      virtualKey: 'vk_cl4ud3_a1b2c',
      createdAt: '2024-08-20T14:22:00Z',
      lastUsed: '2024-08-29T22:15:00Z',
      requestCount: 892,
      status: 'active'
    },
    {
      id: '3',
      name: 'Mobile App',
      virtualKey: 'vk_m0b1l3_d3f4g',
      createdAt: '2024-08-15T09:00:00Z',
      lastUsed: '2024-08-27T16:30:00Z',
      requestCount: 234,
      status: 'active'
    }
  ]

  useEffect(() => {
    if (tunnelStatus === 'connected') {
      setLoading(true)
      // Use mock data with option to fetch real stats when Bifrost is available
      const fetchAppsWithStats = async () => {
        try {
          // For now, just use mock data to avoid blocking the UI
          // When Bifrost is confirmed working, uncomment the API calls below
          setApps(mockApps)
          
          /* Future implementation when Bifrost API is stable:
          const appsWithStats = await Promise.all(
            mockApps.map(async (app) => {
              try {
                const stats = await bifrostService.getAppUsageStats(app.virtualKey)
                return {
                  ...app,
                  requestCount: stats.totalRequests,
                  lastUsed: stats.lastUsed || app.lastUsed
                }
              } catch (error) {
                return app
              }
            })
          )
          setApps(appsWithStats)
          */
        } catch (error) {
          console.error('Failed to fetch app stats:', error)
          setApps(mockApps)
        } finally {
          setLoading(false)
        }
      }
      
      fetchAppsWithStats()
    } else {
      setApps([])
    }
  }, [tunnelStatus])

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  const handleRevokeApp = (e: React.MouseEvent, appId: string) => {
    e.stopPropagation()
    // TODO: Implement revocation API call
    setApps(prev => prev.map(app => 
      app.id === appId ? { ...app, status: 'revoked' as const } : app
    ))
  }

  return (
    <div className="card card--elevated">
      <div className="card__header">
        <div className="card__header-flex">
          <h2 className="card__title">Apps</h2>
          <div className="apps-summary">
            {tunnelStatus === 'connected' && (
              <span className="text-secondary text-sm">
                {apps.filter(app => app.status === 'active').length} active
              </span>
            )}
          </div>
        </div>
      </div>
      
      <div className="card__content">
        {tunnelStatus !== 'connected' ? (
          <div className="apps-empty">
            <Key className="apps-empty-icon" />
            <p className="text-secondary">
              Start the tunnel to view authorized applications
            </p>
          </div>
        ) : loading ? (
          <div className="apps-loading">
            <div className="loading-spinner"></div>
            <p className="text-secondary">Loading applications...</p>
          </div>
        ) : apps.length === 0 ? (
          <div className="apps-empty">
            <Key className="apps-empty-icon" />
            <p className="text-secondary">
              No applications authorized yet
            </p>
            <p className="text-muted">
              scan to authorize
            </p>
          </div>
        ) : (
          <div className="apps-list">
            {apps.map((app) => (
              <div
                key={app.id}
                className={`app-item-slim ${app.status !== 'active' ? 'app-item-slim--disabled' : ''}`}
              >
                <div className="app-name-section">
                  <div className={`app-status-dot app-status-dot--${app.status}`} title={app.status}></div>
                  <span className="app-name">{app.name}</span>
                </div>
                <div className="app-actions">
                  <button
                    className="btn btn--icon btn--ghost btn--xs"
                    onClick={(e) => {
                      e.stopPropagation()
                      onAppClick(app)
                    }}
                    title="View details"
                  >
                    <Settings className="btn__icon" />
                  </button>
                  {app.status === 'active' && (
                    <button
                      className="btn btn--icon btn--ghost btn--xs"
                      onClick={(e) => handleRevokeApp(e, app.id)}
                      title="Revoke access"
                    >
                      <Trash2 className="btn__icon" />
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default AppsCard