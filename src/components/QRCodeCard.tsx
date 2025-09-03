import React from 'react'
import QRCode from 'react-qr-code'
import { Copy } from 'lucide-react'
import './QRCodeCard.css'

interface QRCodeCardProps {
  publicUrl: string | null
  tunnelStatus: 'disconnected' | 'connecting' | 'connected' | 'error'
}

interface ConnectionData {
  url: string
  timestamp: string
  type: 'mindlink-tunnel'
}

const QRCodeCard: React.FC<QRCodeCardProps> = ({ publicUrl, tunnelStatus }) => {
  // Generate the data to encode in the QR code
  const generateQRData = (): string => {
    if (!publicUrl || tunnelStatus !== 'connected') {
      return JSON.stringify({
        type: 'mindlink-tunnel',
        status: 'disconnected',
        message: 'Tunnel not available'
      })
    }

    const connectionData: ConnectionData = {
      url: publicUrl,
      timestamp: new Date().toISOString(),
      type: 'mindlink-tunnel'
    }

    return JSON.stringify(connectionData)
  }

  const handleCopyUrl = async () => {
    if (publicUrl && tunnelStatus === 'connected') {
      try {
        await navigator.clipboard.writeText(publicUrl)
        // You could add a toast notification here
      } catch (error) {
        console.error('Failed to copy URL:', error)
      }
    }
  }


  const qrData = generateQRData()
  const isConnected = tunnelStatus === 'connected' && publicUrl

  return (
    <div className="card card--elevated">
      <div className="card__header">
        <h2 className="card__title">Enable Application</h2>
      </div>
      
      <div className="card__content">
        <div className="qr-code-container">
          {/* QR Code Display */}
          <div className="qr-code-wrapper">
            <QRCode
              value={qrData}
              size={140}
              level="M"
              bgColor="#ffffff"
              fgColor="#000000"
              className="qr-code"
            />
          </div>
          
          {/* Connection Status */}
          <div className="qr-code-status">
            {isConnected ? (
              <p className="text-secondary qr-scan-text mb-2">
                scan to authorize
              </p>
            ) : (
              <p className="text-secondary mb-4">
                {tunnelStatus === 'connecting' 
                  ? 'Establishing tunnel connection...' 
                  : 'Start the tunnel to enable application authorization'
                }
              </p>
            )}
          </div>
          
          {/* Action Buttons */}
          {isConnected && (
            <div className="qr-code-actions">
              <button 
                className="btn btn--secondary btn--sm"
                onClick={handleCopyUrl}
                title="Copy authorization data"
              >
                <Copy className="btn__icon" />
                Copy Authorization
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default QRCodeCard