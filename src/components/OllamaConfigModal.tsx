import React, { useState, useEffect } from 'react'
import { createPortal } from 'react-dom'
import { X, Settings, RefreshCw } from 'lucide-react'
import { OllamaPlugin } from '../plugins/providers/ollama'
import './OllamaConfigModal.css'

interface OllamaConfigModalProps {
  isOpen: boolean
  onClose: () => void
  plugin: OllamaPlugin
  onConfigSaved: () => void
}

interface OllamaConfig {
  baseUrl: string
  selectedModel: string | null
  timeout: number
}

const OllamaConfigModal: React.FC<OllamaConfigModalProps> = ({
  isOpen,
  onClose,
  plugin,
  onConfigSaved
}) => {
  const [config, setConfig] = useState<OllamaConfig>({
    baseUrl: 'http://127.0.0.1:11434',
    selectedModel: null,
    timeout: 5000
  })
  const [availableModels, setAvailableModels] = useState<string[]>([])
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load current configuration when modal opens
  useEffect(() => {
    if (isOpen) {
      const currentConfig = plugin.getConfig()
      setConfig(currentConfig)
      loadAvailableModels()
    }
  }, [isOpen, plugin])

  const loadAvailableModels = async () => {
    setLoading(true)
    setError(null)
    
    try {
      console.log('🦙 Loading available Ollama models...')
      const models = await plugin.getSupportedModels()
      console.log('🦙 Available models:', models)
      
      setAvailableModels(models)
      
      // If no model is selected and models are available, select the first one
      if (!config.selectedModel && models.length > 0) {
        setConfig(prev => ({ ...prev, selectedModel: models[0] }))
      }
    } catch (err) {
      console.error('🦙 Failed to load models:', err)
      setError('Failed to load models from Ollama. Make sure Ollama is running.')
      setAvailableModels([])
    } finally {
      setLoading(false)
    }
  }

  const handleSave = async () => {
    setSaving(true)
    setError(null)
    
    try {
      console.log('🦙 Saving Ollama configuration:', config)
      await plugin.updateConfig(config)
      console.log('✅ Ollama configuration saved successfully')
      
      onConfigSaved()
      onClose()
    } catch (err) {
      console.error('❌ Failed to save Ollama configuration:', err)
      setError('Failed to save configuration. Please try again.')
    } finally {
      setSaving(false)
    }
  }

  const handleRefreshModels = () => {
    loadAvailableModels()
  }

  const handleConfigChange = (field: keyof OllamaConfig, value: any) => {
    setConfig(prev => ({
      ...prev,
      [field]: value
    }))
  }

  const testConnection = async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Test connection with current config
      const tempPlugin = new OllamaPlugin()
      await tempPlugin.updateConfig({ baseUrl: config.baseUrl, timeout: config.timeout })
      const connectionStatus = await tempPlugin.getConnectionStatus()
      
      if (connectionStatus.status === 'connected') {
        setError(null)
        // Reload models with new endpoint
        await loadAvailableModels()
      } else {
        setError(connectionStatus.error || 'Failed to connect to Ollama')
      }
    } catch (err) {
      console.error('🦙 Connection test failed:', err)
      setError('Failed to connect to Ollama at the specified URL')
    } finally {
      setLoading(false)
    }
  }

  if (!isOpen) return null

  return createPortal(
    <div className="modal-overlay">
      <div className="modal-content ollama-config-modal">
        <div className="modal-header">
          <div className="modal-header-flex">
            <Settings className="modal-icon" />
            <h2 className="modal-title">Ollama Configuration</h2>
          </div>
          <button className="modal-close-btn" onClick={onClose} title="Close">
            <X />
          </button>
        </div>
        
        <div className="modal-body">
          {error && (
            <div className="error-message">
              <span>{error}</span>
            </div>
          )}
          
          <div className="config-section">
            <label className="config-label">
              Ollama Endpoint URL
            </label>
            <div className="config-input-group">
              <input
                type="text"
                className="config-input"
                value={config.baseUrl}
                onChange={(e) => handleConfigChange('baseUrl', e.target.value)}
                placeholder="http://127.0.0.1:11434"
              />
              <button
                className="btn btn--ghost btn--sm"
                onClick={testConnection}
                disabled={loading}
                title="Test Connection"
              >
                {loading ? <RefreshCw className="spinner" /> : 'Test'}
              </button>
            </div>
            <p className="config-help">
              The URL where your Ollama service is running (default: http://127.0.0.1:11434)
            </p>
          </div>

          <div className="config-section">
            <div className="config-section-header">
              <label className="config-label">
                Selected Model
              </label>
              <button
                className="btn btn--ghost btn--sm"
                onClick={handleRefreshModels}
                disabled={loading}
                title="Refresh Models"
              >
                <RefreshCw className={loading ? "spinner" : ""} />
                Refresh
              </button>
            </div>
            
            {loading ? (
              <div className="loading-state">
                <RefreshCw className="spinner" />
                <span>Loading models...</span>
              </div>
            ) : availableModels.length > 0 ? (
              <div className="model-selection">
                <select
                  className="config-select"
                  value={config.selectedModel || ''}
                  onChange={(e) => handleConfigChange('selectedModel', e.target.value || null)}
                >
                  <option value="">Select a model...</option>
                  {availableModels.map(model => (
                    <option key={model} value={model}>
                      {model}
                    </option>
                  ))}
                </select>
                <p className="config-help">
                  Choose which Ollama model to use for API requests
                </p>
              </div>
            ) : (
              <div className="empty-state">
                <p>No models found. Make sure Ollama is running and has models installed.</p>
                <p className="config-help">
                  Run <code>ollama pull [model-name]</code> to install models.
                </p>
              </div>
            )}
          </div>

          <div className="config-section">
            <label className="config-label">
              Connection Timeout (ms)
            </label>
            <input
              type="number"
              className="config-input"
              value={config.timeout}
              onChange={(e) => handleConfigChange('timeout', parseInt(e.target.value) || 5000)}
              min="1000"
              max="30000"
              step="1000"
            />
            <p className="config-help">
              How long to wait for Ollama to respond (1000-30000ms)
            </p>
          </div>
        </div>

        <div className="modal-footer">
          <button
            className="btn btn--ghost"
            onClick={onClose}
            disabled={saving}
          >
            Cancel
          </button>
          <button
            className="btn btn--primary"
            onClick={handleSave}
            disabled={saving || loading || !config.selectedModel}
          >
            {saving ? (
              <>
                <RefreshCw className="spinner" />
                Saving...
              </>
            ) : (
              'Save Configuration'
            )}
          </button>
        </div>
      </div>
    </div>,
    document.body
  )
}

export default OllamaConfigModal