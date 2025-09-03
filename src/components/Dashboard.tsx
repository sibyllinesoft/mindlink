import React, { useState } from 'react'
import ProvidersCard from './ProvidersCard'
import QRCodeCard from './QRCodeCard'
import AppsCard from './AppsCard'
import AppDetailsModal from './AppDetailsModal'
import './Dashboard.css'

interface App {
  id: string
  name: string
  virtualKey: string
  createdAt: string
  lastUsed: string
  requestCount: number
  status: 'active' | 'revoked' | 'expired'
}

interface DashboardProps {
  serverStatus: 'running' | 'error'
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
  publicUrl: string | null
  isAuthenticated: boolean
  onToggleTunnel: () => void
}

const Dashboard: React.FC<DashboardProps> = ({
  tunnelStatus,
  publicUrl,
}) => {
  const [selectedApp, setSelectedApp] = useState<App | null>(null)
  const [isModalOpen, setIsModalOpen] = useState(false)

  const handleAppClick = (app: App) => {
    setSelectedApp(app)
    setIsModalOpen(true)
  }

  const handleCloseModal = () => {
    setIsModalOpen(false)
    setSelectedApp(null)
  }

  const handleRevokeApp = (appId: string) => {
    // TODO: Implement actual Bifrost API call to revoke app access
    console.log('Revoking access for app:', appId)
  }

  return (
    <div className="dashboard">
      <div className="dashboard-grid">
        {/* QR Code Card */}
        <div className="dashboard-section">
          <QRCodeCard
            publicUrl={publicUrl}
            tunnelStatus={tunnelStatus}
          />
        </div>
        
        {/* Apps Card */}
        <div className="dashboard-section">
          <AppsCard
            tunnelStatus={tunnelStatus}
            onAppClick={handleAppClick}
          />
        </div>
        
        {/* AI Providers */}
        <div className="dashboard-section">
          <ProvidersCard />
        </div>
      </div>
      
      {/* App Details Modal */}
      <AppDetailsModal
        app={selectedApp}
        isOpen={isModalOpen}
        onClose={handleCloseModal}
        onRevoke={handleRevokeApp}
      />
    </div>
  )
}

export default Dashboard