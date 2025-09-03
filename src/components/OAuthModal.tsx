import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './OAuthModal.css'

interface OAuthModalProps {
  isOpen: boolean
  onAuthSuccess: () => void
  onCancel?: () => void
  authType?: 'cloudflare' | 'chatgpt'
}

interface ServiceResponse {
  success: boolean
  message?: string
  auth_url?: string
  server_url?: string
  tunnel_url?: string
}

type AuthState = 'idle' | 'initiating' | 'waiting' | 'success' | 'error'

export default function OAuthModal({ isOpen, onAuthSuccess, onCancel, authType = 'chatgpt' }: OAuthModalProps) {
  const [authState, setAuthState] = useState<AuthState>('idle')
  const [error, setError] = useState<string | null>(null)
  const [pollTimeoutId, setPollTimeoutId] = useState<number | null>(null)

  // Cleanup effect to cancel polling when component unmounts or modal closes
  useEffect(() => {
    return () => {
      if (pollTimeoutId) {
        window.clearTimeout(pollTimeoutId)
        console.log('🧹 Cleanup: Authentication polling cleared on unmount')
      }
    }
  }, [pollTimeoutId])

  // Reset state when modal closes
  useEffect(() => {
    if (!isOpen) {
      if (pollTimeoutId) {
        window.clearTimeout(pollTimeoutId)
        setPollTimeoutId(null)
      }
      setAuthState('idle')
      setError(null)
    }
  }, [isOpen, pollTimeoutId])

  const handleStartOAuth = async () => {
    try {
      setAuthState('initiating')
      setError(null)

      if (authType === 'chatgpt') {
        // Start the complete ChatGPT OAuth authentication flow
        // This will open the browser automatically and handle the complete flow
        setAuthState('waiting')
        
        try {
          await invoke<string>('authenticate_chatgpt')
          // If we get here, authentication was successful
          setAuthState('success')
          setTimeout(() => {
            onAuthSuccess()
          }, 1500) // Brief delay to show success state
          return
        } catch (authError) {
          // If authentication fails, show the error
          throw authError
        }
      } else {
        // Start the Cloudflare tunnel authentication flow
        const response = await invoke<ServiceResponse>('oauth_login')
        
        if (response.success) {
          setAuthState('waiting')
          
          // The cloudflared login command opens the browser automatically
          // We just need to poll for authentication completion
          pollForAuthCompletion()
        } else {
          throw new Error(response.message || 'Failed to start Cloudflare authentication')
        }
      }
    } catch (err) {
      console.error(`${authType} authentication failed:`, err)
      setError(err instanceof Error ? err.message : `Failed to start ${authType} authentication`)
      setAuthState('error')
    }
  }

  const pollForAuthCompletion = async () => {
    const maxAttempts = 60 // 10 minutes (10 second intervals)
    let attempts = 0

    const checkAuth = async (): Promise<boolean> => {
      try {
        if (authType === 'chatgpt') {
          // Check ChatGPT authentication status
          const isAuthenticated = await invoke<boolean>('check_chatgpt_auth_status')
          return isAuthenticated
        } else {
          // Use dedicated auth check command for Cloudflare to avoid expensive get_status calls
          const isAuthenticated = await invoke<boolean>('check_auth_status')
          return isAuthenticated
        }
      } catch {
        return false
      }
    }

    const poll = async () => {
      attempts++
      console.log(`🔍 Checking authentication status (attempt ${attempts}/${maxAttempts})...`)
      
      try {
        if (await checkAuth()) {
          console.log('✅ Authentication successful!')
          setAuthState('success')
          // Clear polling timeout
          if (pollTimeoutId) {
            window.clearTimeout(pollTimeoutId)
            setPollTimeoutId(null)
          }
          setTimeout(() => {
            onAuthSuccess()
          }, 1500) // Brief delay to show success state
          return
        }
      } catch (error) {
        console.warn('Authentication check failed:', error)
      }

      if (attempts >= maxAttempts) {
        console.log('⏰ Authentication timed out')
        setError('Authentication timed out after 10 minutes. The browser window may have been closed. Please try again.')
        setAuthState('error')
        if (pollTimeoutId) {
          window.clearTimeout(pollTimeoutId)
          setPollTimeoutId(null)
        }
        return
      }

      // Continue polling with 10 second intervals (much less aggressive)
      const timeoutId = window.setTimeout(poll, 10000)
      setPollTimeoutId(timeoutId)
    }

    // Start with a 5 second delay to let cloudflared initialize
    const initialTimeoutId = window.setTimeout(poll, 5000)
    setPollTimeoutId(initialTimeoutId)
  }

  const handleCancel = () => {
    // Cancel any active polling
    if (pollTimeoutId) {
      window.clearTimeout(pollTimeoutId)
      setPollTimeoutId(null)
      console.log('🚫 Authentication polling canceled')
    }
    
    setAuthState('idle')
    setError(null)
    onCancel?.()
  }

  const handleRetry = () => {
    setAuthState('idle')
    setError(null)
  }

  if (!isOpen) return null

  return (
    <div className="oauth-modal-backdrop">
      <div className="oauth-modal">
        <div className="oauth-modal__content">
          {/* Header */}
          <div className="oauth-modal__header">
            <div className="oauth-modal__logo">
              {authType === 'chatgpt' ? (
                <div className="oauth-modal__chatgpt-icon">
                  <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
                    <circle cx="16" cy="16" r="14" fill="#10a37f"/>
                    <path d="M12 10h8c1.1 0 2 .9 2 2v8c0 1.1-.9 2-2 2h-8c-1.1 0-2-.9-2-2v-8c0-1.1.9-2 2-2zm4 3c-1.7 0-3 1.3-3 3s1.3 3 3 3 3-1.3 3-3-1.3-3-3-3z" fill="white"/>
                  </svg>
                </div>
              ) : (
                <div className="oauth-modal__cloudflare-icon">
                  <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
                    <path d="M24.6 18.2c-.1-.4-.4-.7-.8-.8l-11.8-1.9c-.2 0-.4-.1-.5-.3-.1-.2 0-.4.1-.5l3.2-2.8c.2-.2.3-.5.2-.8-.1-.3-.4-.5-.7-.5H9.1c-1.3 0-2.4.9-2.6 2.1L5.7 16c0 .1-.1.2-.1.3 0 .8.6 1.4 1.4 1.4h16.8c.4 0 .7-.2.9-.6.1-.3.1-.6-.1-.9z" fill="#F38020"/>
                    <path d="M29.1 20.1c-.3-1.8-1.9-3.1-3.8-3.1-.2 0-.4 0-.6.1-1.1-2.3-3.5-3.8-6.1-3.8-2.1 0-4 .9-5.3 2.4l-.8.9c-.1.1-.1.2 0 .3.1.1.2.1.3.1l12.5 2c1.4.2 2.4 1.4 2.4 2.8 0 .2 0 .4-.1.6 0 .1.1.2.2.2h.9c1.3 0 2.4-1 2.4-2.3v-.2z" fill="#F38020"/>
                    <path d="M7.2 18.9c-.8 0-1.4-.6-1.4-1.4 0-.1 0-.2.1-.3l.8-3.2c.2-1.2 1.3-2.1 2.6-2.1h5.2c.3 0 .6.2.7.5.1.3 0 .6-.2.8l-3.2 2.8c-.1.1-.2.3-.1.5.1.2.3.3.5.3l11.8 1.9c.4.1.7.4.8.8.2.3.2.6.1.9-.2.4-.5.6-.9.6H7.2z" fill="#F38020"/>
                  </svg>
                </div>
              )}
              <h2 className="oauth-modal__title">
                {authType === 'chatgpt' 
                  ? 'ChatGPT Authentication Required' 
                  : 'Cloudflare Tunnel Authentication Required'}
              </h2>
            </div>
          </div>

          {/* Body */}
          <div className="oauth-modal__body">
            {authState === 'idle' && (
              <>
                <p className="oauth-modal__description">
                  {authType === 'chatgpt' 
                    ? 'MindLink requires ChatGPT authentication to access GPT models directly through your ChatGPT Plus/Pro account.'
                    : 'MindLink requires Cloudflare tunnel authentication to create secure tunnels for public access to your local API server.'}
                </p>
                <div className="oauth-modal__features">
                  {authType === 'chatgpt' ? (
                    <>
                      <div className="oauth-modal__feature">
                        <div className="oauth-modal__feature-icon">
                          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                            <path d="M10 1L13 7L19 7.75L14.88 12.37L16 19L10 16L4 19L5.13 12.37L1 7.75L7 7L10 1Z" fill="currentColor"/>
                          </svg>
                        </div>
                        <span>Direct GPT model access</span>
                      </div>
                      <div className="oauth-modal__feature">
                        <div className="oauth-modal__feature-icon">
                          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                            <path d="M10 2L3 7V17H17V7L10 2ZM10 11C8.9 11 8 10.1 8 9S8.9 7 10 7 12 7.9 12 9 11.1 11 10 11Z" fill="currentColor"/>
                          </svg>
                        </div>
                        <span>Uses your ChatGPT subscription</span>
                      </div>
                      <div className="oauth-modal__feature">
                        <div className="oauth-modal__feature-icon">
                          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                            <path d="M10 0L15 5H12V9H8V5H5L10 0ZM18 11V15C18 16.66 16.66 18 15 18H5C3.34 18 2 16.66 2 15V11C2 9.34 3.34 8 5 8H6V10H5C4.45 10 4 10.45 4 11V15C4 15.55 4.45 16 5 16H15C15.55 16 16 15.55 16 15V11C16 10.45 15.55 10 15 10H14V8H15C16.66 8 18 9.34 18 11Z" fill="currentColor"/>
                          </svg>
                        </div>
                        <span>OpenAI-compatible API</span>
                      </div>
                    </>
                  ) : (
                    <>
                      <div className="oauth-modal__feature">
                        <div className="oauth-modal__feature-icon">
                          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                            <path d="M10 1L13 7L19 7.75L14.88 12.37L16 19L10 16L4 19L5.13 12.37L1 7.75L7 7L10 1Z" fill="currentColor"/>
                          </svg>
                        </div>
                        <span>Secure tunnel authentication</span>
                      </div>
                      <div className="oauth-modal__feature">
                        <div className="oauth-modal__feature-icon">
                          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                            <path d="M10 2L3 7V17H17V7L10 2ZM10 11C8.9 11 8 10.1 8 9S8.9 7 10 7 12 7.9 12 9 11.1 11 10 11Z" fill="currentColor"/>
                          </svg>
                        </div>
                        <span>Encrypted tunnel connections</span>
                      </div>
                      <div className="oauth-modal__feature">
                        <div className="oauth-modal__feature-icon">
                          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                            <path d="M10 0L15 5H12V9H8V5H5L10 0ZM18 11V15C18 16.66 16.66 18 15 18H5C3.34 18 2 16.66 2 15V11C2 9.34 3.34 8 5 8H6V10H5C4.45 10 4 10.45 4 11V15C4 15.55 4.45 16 5 16H15C15.55 16 16 15.55 16 15V11C16 10.45 15.55 10 15 10H14V8H15C16.66 8 18 9.34 18 11Z" fill="currentColor"/>
                          </svg>
                        </div>
                        <span>No port forwarding needed</span>
                      </div>
                    </>
                  )}
                </div>
              </>
            )}

            {authState === 'initiating' && (
              <div className="oauth-modal__status oauth-modal__status--loading">
                <div className="oauth-modal__spinner"></div>
                <p>Initializing authentication...</p>
              </div>
            )}

            {authState === 'waiting' && (
              <div className="oauth-modal__status oauth-modal__status--waiting">
                <div className="oauth-modal__pulse-icon">
                  <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                    <path d="M12 2L15.5 8.5L22 9L17 14L18.5 21L12 18L5.5 21L7 14L2 9L8.5 8.5L12 2Z" fill="currentColor"/>
                  </svg>
                </div>
                <h3>Complete Authentication in Browser</h3>
                <p>
                  {authType === 'chatgpt'
                    ? "We've opened your browser to the ChatGPT authentication page. Please sign in with your ChatGPT account to authorize API access."
                    : "We've opened your browser to the Cloudflare authentication page. Please sign in with your Cloudflare account to authorize tunnel access."
                  }
                </p>
                <div className="oauth-modal__waiting-steps">
                  {authType === 'chatgpt' ? (
                    <>
                      <div className="oauth-modal__step">
                        <span className="oauth-modal__step-number">1</span>
                        <span>Sign in to your ChatGPT account in the browser</span>
                      </div>
                      <div className="oauth-modal__step">
                        <span className="oauth-modal__step-number">2</span>
                        <span>Authorize MindLink API access</span>
                      </div>
                      <div className="oauth-modal__step">
                        <span className="oauth-modal__step-number">3</span>
                        <span>Return here - this window will detect completion</span>
                      </div>
                    </>
                  ) : (
                    <>
                      <div className="oauth-modal__step">
                        <span className="oauth-modal__step-number">1</span>
                        <span>Sign in to your Cloudflare account in the browser</span>
                      </div>
                      <div className="oauth-modal__step">
                        <span className="oauth-modal__step-number">2</span>
                        <span>Authorize MindLink to create tunnels</span>
                      </div>
                      <div className="oauth-modal__step">
                        <span className="oauth-modal__step-number">3</span>
                        <span>Return here - this window will detect completion</span>
                      </div>
                    </>
                  )}
                </div>
                <div className="oauth-modal__waiting-hint">
                  <small>🔍 Checking authentication status every 10 seconds...</small>
                </div>
              </div>
            )}

            {authState === 'success' && (
              <div className="oauth-modal__status oauth-modal__status--success">
                <div className="oauth-modal__success-icon">
                  <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
                    <circle cx="24" cy="24" r="22" fill="#059669" stroke="#047857" strokeWidth="2"/>
                    <path d="M16 24L20 28L32 16" stroke="white" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                </div>
                <h3>Authentication Successful!</h3>
                <p>
                  {authType === 'chatgpt' 
                    ? 'Welcome to MindLink. Your ChatGPT API access is ready to use.'
                    : 'Welcome to MindLink. Your Cloudflare tunnels are ready to use.'}
                </p>
              </div>
            )}

            {authState === 'error' && (
              <div className="oauth-modal__status oauth-modal__status--error">
                <div className="oauth-modal__error-icon">
                  <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
                    <circle cx="24" cy="24" r="22" fill="#dc2626" stroke="#b91c1c" strokeWidth="2"/>
                    <path d="M16 16L32 32M32 16L16 32" stroke="white" strokeWidth="3" strokeLinecap="round"/>
                  </svg>
                </div>
                <h3>Authentication Failed</h3>
                <p>{error}</p>
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="oauth-modal__footer">
            {authState === 'idle' && (
              <>
                <button
                  className="btn btn--primary oauth-modal__auth-button"
                  onClick={handleStartOAuth}
                >
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
                    <path d="M10 1L13 7L19 7.75L14.88 12.37L16 19L10 16L4 19L5.13 12.37L1 7.75L7 7L10 1Z" fill="currentColor"/>
                  </svg>
                  {authType === 'chatgpt' 
                    ? 'Authenticate with ChatGPT' 
                    : 'Authenticate with Cloudflare'}
                </button>
                {onCancel && (
                  <button
                    className="btn btn--ghost"
                    onClick={handleCancel}
                  >
                    Cancel
                  </button>
                )}
              </>
            )}

            {(authState === 'initiating' || authState === 'waiting') && onCancel && (
              <button
                className="btn btn--ghost"
                onClick={handleCancel}
              >
                Cancel
              </button>
            )}

            {authState === 'error' && (
              <>
                <button
                  className="btn btn--primary"
                  onClick={handleRetry}
                >
                  Try Again
                </button>
                {onCancel && (
                  <button
                    className="btn btn--ghost"
                    onClick={handleCancel}
                  >
                    Cancel
                  </button>
                )}
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}