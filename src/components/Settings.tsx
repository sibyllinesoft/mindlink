import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './Settings.css'

interface SettingsData {
  chatgpt_session_token?: string
  api_port?: number
  enable_cors?: boolean
  log_level?: string
  auto_start_server?: boolean
  auto_create_tunnel?: boolean
}

const Settings: React.FC = () => {
  const [settings, setSettings] = useState<SettingsData>({})
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [message, setMessage] = useState<{ type: 'success' | 'error', text: string } | null>(null)
  const [isAuthenticated, setIsAuthenticated] = useState(false)

  useEffect(() => {
    loadSettings()
    checkAuthStatus()
  }, [])

  const loadSettings = async () => {
    try {
      setLoading(true)
      const config = await invoke('get_config') as SettingsData
      setSettings(config)
    } catch (error) {
      console.error('Failed to load settings:', error)
      showMessage('error', 'Failed to load settings')
    } finally {
      setLoading(false)
    }
  }

  const checkAuthStatus = async () => {
    try {
      const status = await invoke('check_auth_status') as boolean
      setIsAuthenticated(status)
    } catch (error) {
      console.error('Failed to check auth status:', error)
    }
  }

  const saveSettings = async () => {
    try {
      setSaving(true)
      await invoke('update_config', { config: settings })
      await checkAuthStatus() // Re-check auth after saving
      showMessage('success', 'Settings saved successfully')
    } catch (error) {
      console.error('Failed to save settings:', error)
      showMessage('error', 'Failed to save settings')
    } finally {
      setSaving(false)
    }
  }

  const authenticateWithChatGPT = async () => {
    try {
      setSaving(true)
      await invoke('authenticate_chatgpt')
      await checkAuthStatus()
      showMessage('success', 'Successfully authenticated with ChatGPT')
    } catch (error) {
      console.error('Failed to authenticate:', error)
      showMessage('error', 'Failed to authenticate with ChatGPT')
    } finally {
      setSaving(false)
    }
  }

  const showMessage = (type: 'success' | 'error', text: string) => {
    setMessage({ type, text })
    setTimeout(() => setMessage(null), 5000)
  }

  const handleInputChange = (key: keyof SettingsData, value: any) => {
    setSettings(prev => ({
      ...prev,
      [key]: value
    }))
  }

  if (loading) {
    return (
      <div className="settings-loading">
        <div className="spinner" />
        <p>Loading settings...</p>
      </div>
    )
  }

  return (
    <div className="settings">
      <div className="settings-container">
        <h1>Settings</h1>

        {message && (
          <div className={`alert alert-${message.type}`}>
            {message.text}
          </div>
        )}

        <div className="settings-sections">
          {/* Authentication Section */}
          <section className="settings-section">
            <div className="card">
              <h2>ChatGPT Authentication</h2>
              
              <div className="auth-status">
                <div className={`status-indicator ${isAuthenticated ? 'connected' : 'stopped'}`}>
                  {isAuthenticated ? 'Authenticated' : 'Not Authenticated'}
                </div>
              </div>

              <div className="form-group">
                <label className="form-label">
                  Session Token (Optional)
                </label>
                <input
                  type="password"
                  className="form-input"
                  value={settings.chatgpt_session_token || ''}
                  onChange={(e) => handleInputChange('chatgpt_session_token', e.target.value)}
                  placeholder="Paste your ChatGPT session token here"
                />
                <small className="form-help">
                  You can manually paste your session token, or use the browser authentication below.
                </small>
              </div>

              <div className="auth-actions">
                <button
                  className="btn btn-primary"
                  onClick={authenticateWithChatGPT}
                  disabled={saving}
                >
                  {saving ? (
                    <>
                      <span className="spinner" />
                      Authenticating...
                    </>
                  ) : (
                    'Authenticate with Browser'
                  )}
                </button>
              </div>

              <div className="auth-help">
                <p className="text-muted">
                  MindLink needs to authenticate with your ChatGPT account to provide API access. 
                  Click "Authenticate with Browser" to log in through the ChatGPT website.
                </p>
              </div>
            </div>
          </section>

          {/* Server Configuration */}
          <section className="settings-section">
            <div className="card">
              <h2>Server Configuration</h2>

              <div className="form-group">
                <label className="form-label">
                  API Port
                </label>
                <input
                  type="number"
                  className="form-input"
                  value={settings.api_port || 8080}
                  onChange={(e) => handleInputChange('api_port', parseInt(e.target.value))}
                  min="1024"
                  max="65535"
                />
                <small className="form-help">
                  The port number for the local API server (default: 8080)
                </small>
              </div>

              <div className="form-group">
                <label className="form-label">
                  Log Level
                </label>
                <select
                  className="form-select"
                  value={settings.log_level || 'info'}
                  onChange={(e) => handleInputChange('log_level', e.target.value)}
                >
                  <option value="error">Error</option>
                  <option value="warn">Warning</option>
                  <option value="info">Info</option>
                  <option value="debug">Debug</option>
                  <option value="trace">Trace</option>
                </select>
                <small className="form-help">
                  Controls the verbosity of application logs
                </small>
              </div>

              <div className="form-group">
                <label className="form-checkbox-label">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={settings.enable_cors || false}
                    onChange={(e) => handleInputChange('enable_cors', e.target.checked)}
                  />
                  Enable CORS
                </label>
                <small className="form-help">
                  Allow cross-origin requests from web browsers
                </small>
              </div>
            </div>
          </section>

          {/* Automation Settings */}
          <section className="settings-section">
            <div className="card">
              <h2>Automation</h2>

              <div className="form-group">
                <label className="form-checkbox-label">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={settings.auto_start_server || false}
                    onChange={(e) => handleInputChange('auto_start_server', e.target.checked)}
                  />
                  Auto-start server on app launch
                </label>
                <small className="form-help">
                  Automatically start the API server when MindLink launches
                </small>
              </div>

              <div className="form-group">
                <label className="form-checkbox-label">
                  <input
                    type="checkbox"
                    className="form-checkbox"
                    checked={settings.auto_create_tunnel || false}
                    onChange={(e) => handleInputChange('auto_create_tunnel', e.target.checked)}
                  />
                  Auto-create tunnel when server starts
                </label>
                <small className="form-help">
                  Automatically create a Cloudflare tunnel when the server starts
                </small>
              </div>
            </div>
          </section>
        </div>

        <div className="settings-actions">
          <button
            className="btn btn-primary"
            onClick={saveSettings}
            disabled={saving}
          >
            {saving ? (
              <>
                <span className="spinner" />
                Saving...
              </>
            ) : (
              'Save Settings'
            )}
          </button>
        </div>
      </div>
    </div>
  )
}

export default Settings