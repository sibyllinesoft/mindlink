import React, { useState } from 'react'
import './APIInformation.css'

interface APIInformationProps {
  serverRunning: boolean
  tunnelConnected: boolean
  publicUrl: string | null
}

const APIInformation: React.FC<APIInformationProps> = ({
  serverRunning,
  tunnelConnected,
  publicUrl,
}) => {
  const [copiedEndpoint, setCopiedEndpoint] = useState<string | null>(null)

  const localEndpoint = 'http://localhost:8080'

  const copyToClipboard = async (text: string, endpoint: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopiedEndpoint(endpoint)
      setTimeout(() => setCopiedEndpoint(null), 2000)
    } catch (error) {
      console.error('Failed to copy to clipboard:', error)
    }
  }

  const curlExample = tunnelConnected && publicUrl
    ? `curl -X POST "${publicUrl}/v1/chat/completions" \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer your-api-key" \\
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'`
    : `curl -X POST "${localEndpoint}/v1/chat/completions" \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer your-api-key" \\
  -d '{
    "model": "gpt-4",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'`

  return (
    <div className="card api-information">
      <h2>API Information</h2>
      
      <div className="api-endpoints">
        <div className="endpoint-section">
          <h3>Local Endpoint</h3>
          <div className="endpoint-row">
            <code className={`endpoint ${serverRunning ? 'active' : 'inactive'}`}>
              {localEndpoint}
            </code>
            <button
              className="btn btn-secondary copy-btn"
              onClick={() => copyToClipboard(localEndpoint, 'local')}
              disabled={!serverRunning}
            >
              {copiedEndpoint === 'local' ? 'Copied!' : 'Copy'}
            </button>
          </div>
          <p className="endpoint-description">
            {serverRunning 
              ? 'Available for local applications and testing'
              : 'Start the server to activate this endpoint'
            }
          </p>
        </div>

        <div className="endpoint-section">
          <h3>Public Endpoint</h3>
          <div className="endpoint-row">
            <code className={`endpoint ${tunnelConnected ? 'active' : 'inactive'}`}>
              {tunnelConnected ? publicUrl : 'No tunnel active'}
            </code>
            <button
              className="btn btn-secondary copy-btn"
              onClick={() => copyToClipboard(publicUrl!, 'public')}
              disabled={!tunnelConnected || !publicUrl}
            >
              {copiedEndpoint === 'public' ? 'Copied!' : 'Copy'}
            </button>
          </div>
          <p className="endpoint-description">
            {tunnelConnected 
              ? 'Available publicly via Cloudflare tunnel'
              : 'Start the tunnel to get a public endpoint'
            }
          </p>
        </div>
      </div>

      {(serverRunning || tunnelConnected) && (
        <div className="api-usage">
          <h3>Usage Example</h3>
          <div className="code-block">
            <pre><code>{curlExample}</code></pre>
            <button
              className="btn btn-secondary copy-btn copy-example"
              onClick={() => copyToClipboard(curlExample, 'example')}
            >
              {copiedEndpoint === 'example' ? 'Copied!' : 'Copy'}
            </button>
          </div>
        </div>
      )}

      <div className="api-features">
        <h3>API Features</h3>
        <ul className="feature-list">
          <li>✅ OpenAI-compatible API format</li>
          <li>✅ Supports chat completions</li>
          <li>✅ Streaming responses available</li>
          <li>✅ Model selection (gpt-4, gpt-3.5-turbo)</li>
          <li>✅ Custom system messages</li>
          <li>✅ Temperature and max_tokens controls</li>
        </ul>
      </div>
    </div>
  )
}

export default APIInformation