const fastify = require('fastify');
const path = require('path');
const fs = require('fs-extra');
const { v4: uuidv4 } = require('uuid');

/**
 * BifrostManager handles the web UI server for MindLink management
 * Provides a comprehensive web interface for monitoring and controlling the system
 */
class BifrostManager {
    constructor() {
        this.app = null;
        this.port = 3002;
        this.host = '127.0.0.1';
        this.isRunning = false;
        this.authManager = null;
        this.serverManager = null;
        this.tunnelManager = null;
        this.configManager = null;
        this.sessions = new Map(); // Simple session management
        this.uiPath = path.join(__dirname, '..', '..', 'bifrost-ui');
    }

    setManagers(authManager, serverManager, tunnelManager, configManager) {
        this.authManager = authManager;
        this.serverManager = serverManager;
        this.tunnelManager = tunnelManager;
        this.configManager = configManager;
    }

    async start() {
        if (this.isRunning) return;

        this.createApp();
        await this.setupRoutes();
        
        try {
            await this.app.listen({ 
                port: this.port, 
                host: this.host 
            });
            
            this.isRunning = true;
            console.log(`Bifrost UI server running on ${this.getLocalUrl()}`);
            
        } catch (err) {
            console.error('Failed to start Bifrost server:', err);
            throw err;
        }
    }

    async stop() {
        if (!this.isRunning || !this.app) return;

        try {
            await this.app.close();
            this.isRunning = false;
            this.app = null;
            console.log('Bifrost server stopped');
        } catch (err) {
            console.error('Error stopping Bifrost server:', err);
        }
    }

    createApp() {
        this.app = fastify({
            logger: {
                level: 'info'
            },
            bodyLimit: 52428800 // 50MB
        });

        // Register CORS plugin
        this.app.register(require('@fastify/cors'), {
            origin: true,
            credentials: true,
            methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS']
        });

        // Global error handler
        this.app.setErrorHandler(async (error, request, reply) => {
            console.error('Bifrost server error:', error);
            return reply.status(500).send({
                error: {
                    message: 'Internal server error',
                    type: 'server_error'
                }
            });
        });
    }

    async setupRoutes() {
        // Serve static UI files
        await this.app.register(require('@fastify/static'), {
            root: this.uiPath,
            prefix: '/static/'
        });

        // Main dashboard route
        this.app.get('/', async (request, reply) => {
            reply.type('text/html');
            return this.getDashboardHTML();
        });

        this.app.get('/dashboard', async (request, reply) => {
            reply.type('text/html');
            return this.getDashboardHTML();
        });

        // API routes
        this.app.get('/api/status', async (request, reply) => {
            return this.getSystemStatus();
        });

        this.app.get('/api/config', async (request, reply) => {
            return this.configManager?.getConfig() || {};
        });

        this.app.put('/api/config', async (request, reply) => {
            try {
                await this.configManager?.updateConfig(request.body);
                return { success: true };
            } catch (error) {
                return reply.status(500).send({ error: error.message });
            }
        });

        // Authentication management
        this.app.post('/api/auth/login', async (request, reply) => {
            try {
                await this.authManager?.login();
                return { success: true };
            } catch (error) {
                return reply.status(500).send({ error: error.message });
            }
        });

        this.app.get('/api/auth/status', async (request, reply) => {
            return {
                authenticated: this.authManager?.isAuthenticated() || false,
                tokens: this.authManager?.isAuthenticated() ? 
                    { hasTokens: true } : { hasTokens: false }
            };
        });

        // Service management
        this.app.post('/api/services/start', async (request, reply) => {
            try {
                if (!this.authManager?.isAuthenticated()) {
                    await this.authManager?.login();
                }
                
                await this.serverManager?.start();
                const tunnelUrl = await this.tunnelManager?.createTunnel();
                
                return { 
                    success: true, 
                    serverUrl: this.serverManager?.getLocalUrl(),
                    tunnelUrl 
                };
            } catch (error) {
                return reply.status(500).send({ error: error.message });
            }
        });

        this.app.post('/api/services/stop', async (request, reply) => {
            try {
                await this.tunnelManager?.closeTunnel();
                await this.serverManager?.stop();
                return { success: true };
            } catch (error) {
                return reply.status(500).send({ error: error.message });
            }
        });

        // Real-time logs endpoint
        this.app.get('/api/logs', async (request, reply) => {
            // In a real implementation, you'd stream logs here
            return { logs: this.getRecentLogs() };
        });

        // Health checks
        this.app.get('/health', async (request, reply) => {
            return {
                status: 'healthy',
                uptime: process.uptime(),
                timestamp: new Date().toISOString()
            };
        });

        // Tunnel management
        this.app.post('/api/tunnel/recreate', async (request, reply) => {
            try {
                await this.tunnelManager?.recreateTunnel();
                return { 
                    success: true, 
                    tunnelUrl: this.tunnelManager?.getCurrentUrl() 
                };
            } catch (error) {
                return reply.status(500).send({ error: error.message });
            }
        });

        // Model testing endpoint
        this.app.post('/api/test/completion', async (request, reply) => {
            try {
                const { message, model = 'gpt-5' } = request.body;
                
                if (!this.serverManager?.isRunning) {
                    return reply.status(503).send({ error: 'API server not running' });
                }

                // Make a test request to our own API
                const axios = require('axios');
                const response = await axios.post(`${this.serverManager.getLocalUrl()}/v1/chat/completions`, {
                    model,
                    messages: [{ role: 'user', content: message }],
                    stream: false
                });

                return { 
                    success: true, 
                    response: response.data.choices[0].message.content 
                };
            } catch (error) {
                return reply.status(500).send({ error: error.message });
            }
        });

        // System information
        this.app.get('/api/system/info', async (request, reply) => {
            const os = require('os');
            return {
                platform: os.platform(),
                arch: os.arch(),
                nodeVersion: process.version,
                memory: {
                    total: os.totalmem(),
                    free: os.freemem(),
                    used: process.memoryUsage()
                },
                uptime: {
                    system: os.uptime(),
                    process: process.uptime()
                }
            };
        });
    }

    getSystemStatus() {
        return {
            services: {
                api: {
                    running: this.serverManager?.isRunning || false,
                    url: this.serverManager?.getLocalUrl() || null
                },
                tunnel: {
                    active: this.tunnelManager?.isConnected() || false,
                    url: this.tunnelManager?.getCurrentUrl() || null
                },
                bifrost: {
                    running: this.isRunning,
                    url: this.getLocalUrl()
                }
            },
            authentication: {
                loggedIn: this.authManager?.isAuthenticated() || false
            },
            config: this.configManager?.getConfig() || {},
            timestamp: new Date().toISOString()
        };
    }

    getRecentLogs() {
        // Placeholder for log management
        // In a real implementation, you'd maintain a log buffer
        return [
            { timestamp: new Date().toISOString(), level: 'info', message: 'Bifrost server started' },
            { timestamp: new Date(Date.now() - 30000).toISOString(), level: 'info', message: 'System status check completed' }
        ];
    }

    async checkHealth() {
        if (!this.isRunning) return false;

        try {
            const axios = require('axios');
            const response = await axios.get(`${this.getLocalUrl()}/health`, {
                timeout: 5000
            });
            return response.status === 200;
        } catch (error) {
            console.error('Bifrost health check failed:', error.message);
            return false;
        }
    }

    getLocalUrl() {
        return `http://${this.host}:${this.port}`;
    }

    getDashboardHTML() {
        return `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MindLink Bifrost Dashboard</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            color: #333;
        }
        
        .header {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            padding: 1rem 2rem;
            box-shadow: 0 2px 20px rgba(0, 0, 0, 0.1);
            position: sticky;
            top: 0;
            z-index: 100;
        }
        
        .header-content {
            max-width: 1200px;
            margin: 0 auto;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        
        .logo {
            display: flex;
            align-items: center;
            gap: 1rem;
        }
        
        .logo h1 {
            background: linear-gradient(135deg, #667eea, #764ba2);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            font-size: 1.5rem;
            font-weight: 700;
        }
        
        .status-indicator {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem 1rem;
            border-radius: 25px;
            background: rgba(39, 174, 96, 0.1);
            color: #27ae60;
            font-size: 0.9rem;
            font-weight: 500;
        }
        
        .status-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: #27ae60;
            animation: pulse 2s infinite;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        
        .container {
            max-width: 1200px;
            margin: 2rem auto;
            padding: 0 2rem;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 2rem;
        }
        
        .card {
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 16px;
            padding: 2rem;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }
        
        .card:hover {
            transform: translateY(-4px);
            box-shadow: 0 12px 40px rgba(0, 0, 0, 0.15);
        }
        
        .card h2 {
            color: #2c3e50;
            margin-bottom: 1rem;
            font-size: 1.25rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
        
        .service-status {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1rem;
            margin: 0.5rem 0;
            background: #f8f9fa;
            border-radius: 8px;
            border-left: 4px solid #3498db;
        }
        
        .service-status.active {
            border-left-color: #27ae60;
            background: #d4edda;
        }
        
        .service-status.inactive {
            border-left-color: #e74c3c;
            background: #f8d7da;
        }
        
        .btn {
            padding: 0.75rem 1.5rem;
            border: none;
            border-radius: 8px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s ease;
            text-decoration: none;
            display: inline-block;
            text-align: center;
        }
        
        .btn-primary {
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
        }
        
        .btn-primary:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
        }
        
        .btn-success {
            background: #27ae60;
            color: white;
        }
        
        .btn-danger {
            background: #e74c3c;
            color: white;
        }
        
        .btn-secondary {
            background: #6c757d;
            color: white;
        }
        
        .metric {
            display: flex;
            justify-content: space-between;
            padding: 0.5rem 0;
            border-bottom: 1px solid #eee;
        }
        
        .metric:last-child {
            border-bottom: none;
        }
        
        .metric-value {
            font-weight: 600;
            color: #2c3e50;
        }
        
        .url-display {
            background: #f8f9fa;
            padding: 0.75rem;
            border-radius: 4px;
            font-family: monospace;
            font-size: 0.9rem;
            word-break: break-all;
            margin: 0.5rem 0;
        }
        
        .actions {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
            margin-top: 1rem;
        }
        
        .test-section {
            margin-top: 1.5rem;
        }
        
        .test-input {
            width: 100%;
            padding: 0.75rem;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 0.9rem;
            margin-bottom: 0.5rem;
        }
        
        .test-output {
            background: #f8f9fa;
            padding: 1rem;
            border-radius: 4px;
            min-height: 100px;
            font-family: monospace;
            font-size: 0.85rem;
            white-space: pre-wrap;
            margin-top: 0.5rem;
        }
        
        .loading {
            display: inline-block;
            width: 16px;
            height: 16px;
            border: 2px solid #f3f3f3;
            border-top: 2px solid #3498db;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
    </style>
</head>
<body>
    <div class="header">
        <div class="header-content">
            <div class="logo">
                <h1>🌈 MindLink Bifrost</h1>
            </div>
            <div class="status-indicator" id="globalStatus">
                <div class="status-dot"></div>
                <span>System Online</span>
            </div>
        </div>
    </div>

    <div class="container">
        <!-- System Overview -->
        <div class="card">
            <h2>🎛️ System Overview</h2>
            <div id="systemStatus">
                <div class="service-status" id="apiStatus">
                    <span>API Server</span>
                    <span>Loading...</span>
                </div>
                <div class="service-status" id="tunnelStatus">
                    <span>Tunnel</span>
                    <span>Loading...</span>
                </div>
                <div class="service-status" id="authStatus">
                    <span>Authentication</span>
                    <span>Loading...</span>
                </div>
            </div>
            <div class="actions">
                <button class="btn btn-success" onclick="startServices()">Start All Services</button>
                <button class="btn btn-danger" onclick="stopServices()">Stop All Services</button>
                <button class="btn btn-secondary" onclick="refreshStatus()">Refresh</button>
            </div>
        </div>

        <!-- API Endpoints -->
        <div class="card">
            <h2>🔗 API Endpoints</h2>
            <div id="apiEndpoints">
                <div class="metric">
                    <span>Local API:</span>
                    <span id="localApiUrl">-</span>
                </div>
                <div class="metric">
                    <span>Public Tunnel:</span>
                    <span id="publicApiUrl">-</span>
                </div>
                <div class="metric">
                    <span>OpenAI Compatible:</span>
                    <span id="openaiApiUrl">-</span>
                </div>
            </div>
            <div class="actions">
                <button class="btn btn-primary" onclick="copyApiUrl()">Copy API URL</button>
                <button class="btn btn-secondary" onclick="openApiDocs()">View API Docs</button>
            </div>
        </div>

        <!-- Quick Test -->
        <div class="card">
            <h2>⚡ Quick Test</h2>
            <div class="test-section">
                <input type="text" class="test-input" id="testMessage" placeholder="Enter a message to test the API..." value="Hello, how are you?">
                <div class="actions">
                    <button class="btn btn-primary" onclick="testCompletion()">Test Completion</button>
                    <select id="modelSelect" style="padding: 0.5rem; border-radius: 4px;">
                        <option value="gpt-5">GPT-5</option>
                        <option value="codex-mini">Codex Mini</option>
                    </select>
                </div>
                <div class="test-output" id="testOutput">Test output will appear here...</div>
            </div>
        </div>

        <!-- System Metrics -->
        <div class="card">
            <h2>📊 System Metrics</h2>
            <div id="systemMetrics">
                <div class="metric">
                    <span>Uptime:</span>
                    <span class="metric-value" id="uptime">-</span>
                </div>
                <div class="metric">
                    <span>Memory Usage:</span>
                    <span class="metric-value" id="memoryUsage">-</span>
                </div>
                <div class="metric">
                    <span>Platform:</span>
                    <span class="metric-value" id="platform">-</span>
                </div>
                <div class="metric">
                    <span>Node Version:</span>
                    <span class="metric-value" id="nodeVersion">-</span>
                </div>
            </div>
        </div>

        <!-- Configuration -->
        <div class="card">
            <h2>⚙️ Quick Settings</h2>
            <div id="quickSettings">
                <div class="metric">
                    <span>Reasoning Effort:</span>
                    <select id="reasoningEffort" onchange="updateSetting('features.reasoningEffort', this.value)">
                        <option value="low">Low</option>
                        <option value="medium">Medium</option>
                        <option value="high">High</option>
                    </select>
                </div>
                <div class="metric">
                    <span>Tunnel Enabled:</span>
                    <input type="checkbox" id="tunnelEnabled" onchange="updateSetting('tunnel.enabled', this.checked)">
                </div>
            </div>
        </div>
    </div>

    <script>
        let statusInterval;

        async function loadStatus() {
            try {
                const response = await fetch('/api/status');
                const status = await response.json();
                updateUI(status);
            } catch (error) {
                console.error('Failed to load status:', error);
            }
        }

        async function loadSystemInfo() {
            try {
                const response = await fetch('/api/system/info');
                const info = await response.json();
                updateSystemMetrics(info);
            } catch (error) {
                console.error('Failed to load system info:', error);
            }
        }

        function updateUI(status) {
            // Update service statuses
            const apiStatus = document.getElementById('apiStatus');
            const tunnelStatus = document.getElementById('tunnelStatus');
            const authStatus = document.getElementById('authStatus');

            updateServiceStatus(apiStatus, status.services.api.running, 'API Server');
            updateServiceStatus(tunnelStatus, status.services.tunnel.active, 'Tunnel');
            updateServiceStatus(authStatus, status.authentication.loggedIn, 'Authentication');

            // Update API URLs
            document.getElementById('localApiUrl').textContent = status.services.api.url || '-';
            document.getElementById('publicApiUrl').textContent = status.services.tunnel.url || '-';
            
            const openaiUrl = status.services.tunnel.url || status.services.api.url;
            document.getElementById('openaiApiUrl').textContent = openaiUrl ? openaiUrl + '/v1' : '-';

            // Update quick settings
            if (status.config.features) {
                document.getElementById('reasoningEffort').value = status.config.features.reasoningEffort || 'medium';
            }
            if (status.config.tunnel) {
                document.getElementById('tunnelEnabled').checked = status.config.tunnel.enabled !== false;
            }
        }

        function updateServiceStatus(element, isActive, serviceName) {
            element.className = 'service-status ' + (isActive ? 'active' : 'inactive');
            element.querySelector('span:last-child').textContent = isActive ? 'Active' : 'Inactive';
        }

        function updateSystemMetrics(info) {
            document.getElementById('uptime').textContent = formatUptime(info.uptime.process);
            document.getElementById('memoryUsage').textContent = formatMemory(info.memory.used.heapUsed);
            document.getElementById('platform').textContent = info.platform + ' (' + info.arch + ')';
            document.getElementById('nodeVersion').textContent = info.nodeVersion;
        }

        async function startServices() {
            try {
                const response = await fetch('/api/services/start', { method: 'POST' });
                const result = await response.json();
                if (result.success) {
                    setTimeout(loadStatus, 2000); // Refresh status after start
                } else {
                    alert('Failed to start services: ' + (result.error || 'Unknown error'));
                }
            } catch (error) {
                alert('Failed to start services: ' + error.message);
            }
        }

        async function stopServices() {
            try {
                const response = await fetch('/api/services/stop', { method: 'POST' });
                const result = await response.json();
                if (result.success) {
                    setTimeout(loadStatus, 1000); // Refresh status after stop
                }
            } catch (error) {
                alert('Failed to stop services: ' + error.message);
            }
        }

        async function testCompletion() {
            const message = document.getElementById('testMessage').value;
            const model = document.getElementById('modelSelect').value;
            const output = document.getElementById('testOutput');
            
            if (!message.trim()) {
                alert('Please enter a message to test');
                return;
            }

            output.textContent = 'Testing... ⏳';
            
            try {
                const response = await fetch('/api/test/completion', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ message, model })
                });
                
                const result = await response.json();
                
                if (result.success) {
                    output.textContent = 'Response: ' + result.response;
                } else {
                    output.textContent = 'Error: ' + (result.error || 'Unknown error');
                }
            } catch (error) {
                output.textContent = 'Error: ' + error.message;
            }
        }

        async function updateSetting(path, value) {
            try {
                const keys = path.split('.');
                const config = {};
                let current = config;
                
                for (let i = 0; i < keys.length - 1; i++) {
                    current[keys[i]] = {};
                    current = current[keys[i]];
                }
                current[keys[keys.length - 1]] = value;

                const response = await fetch('/api/config', {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(config)
                });
                
                if (!response.ok) {
                    throw new Error('Failed to update setting');
                }
            } catch (error) {
                alert('Failed to update setting: ' + error.message);
            }
        }

        function copyApiUrl() {
            const url = document.getElementById('openaiApiUrl').textContent;
            if (url && url !== '-') {
                navigator.clipboard.writeText(url);
                alert('API URL copied to clipboard!');
            } else {
                alert('No API URL available to copy');
            }
        }

        function openApiDocs() {
            const url = document.getElementById('localApiUrl').textContent;
            if (url && url !== '-') {
                window.open(url + '/dashboard', '_blank');
            }
        }

        function refreshStatus() {
            loadStatus();
            loadSystemInfo();
        }

        function formatUptime(seconds) {
            const hours = Math.floor(seconds / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            return hours + 'h ' + minutes + 'm';
        }

        function formatMemory(bytes) {
            return Math.round(bytes / 1024 / 1024) + ' MB';
        }

        // Initialize dashboard
        document.addEventListener('DOMContentLoaded', function() {
            loadStatus();
            loadSystemInfo();
            
            // Auto-refresh every 30 seconds
            statusInterval = setInterval(() => {
                loadStatus();
                loadSystemInfo();
            }, 30000);
        });

        // Cleanup on page unload
        window.addEventListener('beforeunload', function() {
            if (statusInterval) {
                clearInterval(statusInterval);
            }
        });
    </script>
</body>
</html>
        `;
    }
}

module.exports = BifrostManager;