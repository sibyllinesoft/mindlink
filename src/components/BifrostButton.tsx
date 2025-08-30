import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Settings } from 'lucide-react'
import type { StatusResponse, ServiceResponse } from '../types/api'
import './BifrostButton.css'

interface BifrostButtonProps {
  onError?: (message: string) => void
}

const BifrostButton: React.FC<BifrostButtonProps> = ({ onError }) => {
  const [isLoading, setIsLoading] = useState(false)

  const handleOpenBifrost = async () => {
    if (isLoading) return

    setIsLoading(true)
    try {
      // Check if Bifrost is available first
      const status = await invoke<StatusResponse>('get_status')
      
      if (status?.bifrost_url) {
        // Bifrost is running, open the dashboard
        console.log('Opening existing Bifrost dashboard:', status.bifrost_url)
        await invoke('open_external_url', { url: status.bifrost_url })
      } else {
        // Bifrost is not running, try to start it
        console.log('Starting Bifrost service...')
        const result = await invoke<ServiceResponse>('start_bifrost')
        
        if (result?.success && result?.server_url) {
          console.log('Bifrost started successfully, opening dashboard:', result.server_url)
          // Wait a moment for service to fully start, then open dashboard
          setTimeout(async () => {
            try {
              await invoke('open_external_url', { url: result.server_url })
            } catch (error) {
              console.error('Failed to open Bifrost after starting:', error)
              onError?.('Bifrost started but failed to open dashboard')
            }
          }, 3000) // Give it 3 seconds to fully initialize
        } else {
          const errorMessage = result?.message || 'Failed to start Bifrost service'
          console.error('Bifrost startup failed:', errorMessage)
          throw new Error(errorMessage)
        }
      }
    } catch (error) {
      console.error('Bifrost button error:', error)
      let errorMessage = 'Failed to access Bifrost'
      
      if (error instanceof Error) {
        errorMessage = error.message
      } else if (typeof error === 'string') {
        errorMessage = error
      }
      
      // Provide more helpful error messages
      if (errorMessage.includes('binary')) {
        errorMessage = 'Bifrost binary not installed. Please install it first from Settings.'
      } else if (errorMessage.includes('port')) {
        errorMessage = 'Bifrost port is occupied. Please check if another instance is running.'
      }
      
      onError?.(errorMessage)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <button
      className={`btn bifrost-button ${isLoading ? 'bifrost-button--loading' : ''}`}
      onClick={handleOpenBifrost}
      disabled={isLoading}
      title="Open Bifrost LLM Router Dashboard"
    >
      <Settings className="bifrost-button__icon" size={18} />
      <span className="bifrost-button__label">Bifrost</span>
      {isLoading && (
        <div className="bifrost-button__loading">
          <div className="bifrost-button__spinner"></div>
        </div>
      )}
    </button>
  )
}

export default BifrostButton