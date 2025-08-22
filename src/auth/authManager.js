const fs = require('fs-extra');
const path = require('path');
const os = require('os');
const crypto = require('crypto');
const http = require('http');
const { URL } = require('url');
const axios = require('axios');
const { shell } = require('electron');

/**
 * AuthManager handles ChatGPT OAuth authentication
 * Based on the ChatMock authentication system
 */
class AuthManager {
    constructor() {
        this.authDir = path.join(os.homedir(), '.mindlink');
        this.authFilePath = path.join(this.authDir, 'auth.json');
        this.clientId = process.env.CHATGPT_LOCAL_CLIENT_ID || 'app_EMoamEEZ73f0CkXaXp7hrann';
        this.issuer = 'https://auth.openai.com';
        this.tokenEndpoint = `${this.issuer}/oauth/token`;
        this.requiredPort = 1455;
        this.redirectUri = `http://localhost:${this.requiredPort}/auth/callback`;
        this.authData = null;
        this.oauthServer = null;
    }

    async initialize() {
        await fs.ensureDir(this.authDir);
        await this.loadAuthData();
    }

    async loadAuthData() {
        try {
            if (await fs.pathExists(this.authFilePath)) {
                const data = await fs.readJSON(this.authFilePath);
                this.authData = data;
                return true;
            }
        } catch (error) {
            console.error('Error loading auth data:', error);
        }
        return false;
    }

    async saveAuthData(data) {
        try {
            await fs.writeJSON(this.authFilePath, data, { spaces: 2 });
            // Set file permissions to be more secure (600)
            await fs.chmod(this.authFilePath, 0o600);
            this.authData = data;
            return true;
        } catch (error) {
            console.error('Error saving auth data:', error);
            return false;
        }
    }

    isAuthenticated() {
        if (!this.authData || !this.authData.tokens) return false;
        
        const { access_token, id_token } = this.authData.tokens;
        return !!(access_token && id_token);
    }

    getAuthTokens() {
        if (!this.authData || !this.authData.tokens) return null;
        
        return {
            accessToken: this.authData.tokens.access_token,
            accountId: this.authData.tokens.account_id,
            idToken: this.authData.tokens.id_token,
            refreshToken: this.authData.tokens.refresh_token
        };
    }

    async login() {
        return new Promise((resolve, reject) => {
            this.startOAuthServer()
                .then((server) => {
                    this.oauthServer = server;
                    const authUrl = this.buildAuthUrl();
                    
                    // Open browser for authentication
                    shell.openExternal(authUrl);
                    
                    // Wait for OAuth callback
                    server.on('auth-success', (authData) => {
                        this.oauthServer.close();
                        this.oauthServer = null;
                        resolve(authData);
                    });
                    
                    server.on('auth-error', (error) => {
                        this.oauthServer.close();
                        this.oauthServer = null;
                        reject(new Error(error));
                    });
                    
                    // Timeout after 5 minutes
                    setTimeout(() => {
                        if (this.oauthServer) {
                            this.oauthServer.close();
                            this.oauthServer = null;
                            reject(new Error('Authentication timeout'));
                        }
                    }, 300000);
                })
                .catch(reject);
        });
    }

    async startOAuthServer() {
        return new Promise((resolve, reject) => {
            const server = http.createServer();
            
            // Generate PKCE codes
            const pkce = this.generatePKCE();
            const state = crypto.randomBytes(32).toString('hex');
            
            server.pkce = pkce;
            server.state = state;
            
            server.on('request', async (req, res) => {
                const url = new URL(req.url, `http://localhost:${this.requiredPort}`);
                
                if (url.pathname === '/auth/callback') {
                    try {
                        const code = url.searchParams.get('code');
                        const returnedState = url.searchParams.get('state');
                        
                        if (!code) {
                            throw new Error('No authorization code received');
                        }
                        
                        if (returnedState !== state) {
                            throw new Error('Invalid state parameter');
                        }
                        
                        // Exchange code for tokens
                        const authData = await this.exchangeCodeForTokens(code, pkce.codeVerifier);
                        
                        // Save auth data
                        await this.saveAuthData(authData);
                        
                        // Send success response
                        res.writeHead(200, { 'Content-Type': 'text/html' });
                        res.end(this.getSuccessHtml());
                        
                        server.emit('auth-success', authData);
                        
                    } catch (error) {
                        console.error('OAuth callback error:', error);
                        res.writeHead(400, { 'Content-Type': 'text/html' });
                        res.end(this.getErrorHtml(error.message));
                        server.emit('auth-error', error.message);
                    }
                } else {
                    res.writeHead(404);
                    res.end('Not Found');
                }
            });
            
            server.listen(this.requiredPort, 'localhost', (err) => {
                if (err) {
                    reject(err);
                } else {
                    resolve(server);
                }
            });
        });
    }

    buildAuthUrl() {
        const params = new URLSearchParams({
            response_type: 'code',
            client_id: this.clientId,
            redirect_uri: this.redirectUri,
            scope: 'openid profile email offline_access',
            code_challenge: this.oauthServer.pkce.codeChallenge,
            code_challenge_method: 'S256',
            id_token_add_organizations: 'true',
            codex_cli_simplified_flow: 'true',
            state: this.oauthServer.state
        });
        
        return `${this.issuer}/oauth/authorize?${params.toString()}`;
    }

    async exchangeCodeForTokens(code, codeVerifier) {
        const data = new URLSearchParams({
            grant_type: 'authorization_code',
            code: code,
            redirect_uri: this.redirectUri,
            client_id: this.clientId,
            code_verifier: codeVerifier
        });

        try {
            const response = await axios.post(this.tokenEndpoint, data, {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                }
            });

            const tokens = response.data;
            const idTokenClaims = this.parseJWTClaims(tokens.id_token);
            const accessTokenClaims = this.parseJWTClaims(tokens.access_token);
            
            // Extract account ID from token claims
            const authClaims = (idTokenClaims || {})['https://api.openai.com/auth'] || {};
            const accountId = authClaims.chatgpt_account_id || '';

            // Try to obtain API key (optional)
            let apiKey = null;
            try {
                apiKey = await this.obtainApiKey(tokens.id_token, idTokenClaims, accessTokenClaims);
            } catch (error) {
                console.warn('Could not obtain API key:', error.message);
            }

            const authData = {
                OPENAI_API_KEY: apiKey,
                tokens: {
                    id_token: tokens.id_token,
                    access_token: tokens.access_token,
                    refresh_token: tokens.refresh_token,
                    account_id: accountId
                },
                last_refresh: new Date().toISOString()
            };

            return authData;

        } catch (error) {
            console.error('Token exchange failed:', error.response?.data || error.message);
            throw new Error(`Token exchange failed: ${error.message}`);
        }
    }

    async obtainApiKey(idToken, idTokenClaims, accessTokenClaims) {
        const orgId = idTokenClaims?.organization_id;
        const projectId = idTokenClaims?.project_id;
        
        if (!orgId || !projectId) {
            throw new Error('No organization or project ID in token');
        }

        const today = new Date().toISOString().split('T')[0];
        const exchangeData = new URLSearchParams({
            grant_type: 'urn:ietf:params:oauth:grant-type:token-exchange',
            client_id: this.clientId,
            requested_token: 'openai-api-key',
            subject_token: idToken,
            subject_token_type: 'urn:ietf:params:oauth:token-type:id_token',
            name: `MindLink [auto-generated] (${today})`
        });

        const response = await axios.post(this.tokenEndpoint, exchangeData, {
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded'
            }
        });

        return response.data.access_token;
    }

    generatePKCE() {
        const codeVerifier = crypto.randomBytes(64).toString('hex');
        const codeChallenge = crypto
            .createHash('sha256')
            .update(codeVerifier)
            .digest('base64url');
        
        return {
            codeVerifier,
            codeChallenge
        };
    }

    parseJWTClaims(token) {
        if (!token || token.split('.').length !== 3) {
            return null;
        }

        try {
            const [, payload] = token.split('.');
            const paddedPayload = payload + '='.repeat((4 - payload.length % 4) % 4);
            const decoded = Buffer.from(paddedPayload, 'base64url').toString('utf8');
            return JSON.parse(decoded);
        } catch (error) {
            console.error('Error parsing JWT:', error);
            return null;
        }
    }

    getSuccessHtml() {
        return `
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>MindLink - Login Successful</title>
                <style>
                    body { 
                        font-family: system-ui, -apple-system, sans-serif; 
                        max-width: 640px; 
                        margin: 80px auto; 
                        padding: 20px;
                        text-align: center;
                    }
                    .success { color: #28a745; }
                    .code { 
                        background: #f8f9fa; 
                        padding: 4px 8px; 
                        border-radius: 4px; 
                        font-family: monospace; 
                    }
                </style>
            </head>
            <body>
                <h1 class="success">✅ Login Successful!</h1>
                <p>You can now close this window and return to MindLink.</p>
                <p>Your API service will be available shortly.</p>
            </body>
            </html>
        `;
    }

    getErrorHtml(error) {
        return `
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>MindLink - Login Error</title>
                <style>
                    body { 
                        font-family: system-ui, -apple-system, sans-serif; 
                        max-width: 640px; 
                        margin: 80px auto; 
                        padding: 20px;
                        text-align: center;
                    }
                    .error { color: #dc3545; }
                </style>
            </head>
            <body>
                <h1 class="error">❌ Login Failed</h1>
                <p>Error: ${error}</p>
                <p>Please try again or check your connection.</p>
            </body>
            </html>
        `;
    }

    async refreshTokens() {
        if (!this.authData?.tokens?.refresh_token) {
            throw new Error('No refresh token available');
        }

        try {
            const data = new URLSearchParams({
                grant_type: 'refresh_token',
                refresh_token: this.authData.tokens.refresh_token,
                client_id: this.clientId
            });

            const response = await axios.post(this.tokenEndpoint, data, {
                headers: {
                    'Content-Type': 'application/x-www-form-urlencoded'
                }
            });

            const tokens = response.data;
            
            // Update stored tokens
            this.authData.tokens.access_token = tokens.access_token;
            if (tokens.id_token) {
                this.authData.tokens.id_token = tokens.id_token;
            }
            if (tokens.refresh_token) {
                this.authData.tokens.refresh_token = tokens.refresh_token;
            }
            this.authData.last_refresh = new Date().toISOString();
            
            await this.saveAuthData(this.authData);
            return true;
            
        } catch (error) {
            console.error('Token refresh failed:', error);
            throw error;
        }
    }

    async checkConnection() {
        const tokens = this.getAuthTokens();
        if (!tokens) return false;

        try {
            // Try making a test request to ChatGPT API
            const response = await axios.get('https://chatgpt.com/backend-api/accounts/check', {
                headers: {
                    'Authorization': `Bearer ${tokens.accessToken}`,
                    'chatgpt-account-id': tokens.accountId
                },
                timeout: 10000
            });
            
            return response.status === 200;
        } catch (error) {
            console.error('Connection check failed:', error.message);
            
            // Try refreshing tokens if we get auth error
            if (error.response?.status === 401) {
                try {
                    await this.refreshTokens();
                    return true;
                } catch (refreshError) {
                    console.error('Token refresh failed:', refreshError);
                    return false;
                }
            }
            
            return false;
        }
    }
}

module.exports = AuthManager;