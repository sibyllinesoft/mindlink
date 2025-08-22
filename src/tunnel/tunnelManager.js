const { spawn } = require('child_process');
const axios = require('axios');
const fs = require('fs-extra');
const path = require('path');
const os = require('os');
const crypto = require('crypto');

/**
 * TunnelManager handles Cloudflare tunnel creation and management
 */
class TunnelManager {
    constructor() {
        this.cloudflaredPath = null;
        this.tunnelProcess = null;
        this.tunnelUrl = null;
        this.tunnelId = null;
        this.configDir = path.join(os.homedir(), '.mindlink');
        this.tunnelConfigPath = path.join(this.configDir, 'tunnel-config.json');
        this.isActive = false;
        this.localPort = 3001;
        this.connectionAttempts = 0;
        this.maxConnectionAttempts = 5;
    }

    async initialize() {
        await fs.ensureDir(this.configDir);
        await this.ensureCloudflaredInstalled();
        await this.loadTunnelConfig();
    }

    async ensureCloudflaredInstalled() {
        // Check if cloudflared is available in PATH
        try {
            const { spawn } = require('child_process');
            const testProcess = spawn('cloudflared', ['--version'], { stdio: 'pipe' });
            
            await new Promise((resolve, reject) => {
                testProcess.on('close', (code) => {
                    if (code === 0) {
                        this.cloudflaredPath = 'cloudflared';
                        resolve();
                    } else {
                        reject(new Error('cloudflared not found in PATH'));
                    }
                });
                
                testProcess.on('error', (error) => {
                    reject(error);
                });
            });

            console.log('Using cloudflared from PATH');
            return;

        } catch (error) {
            console.log('cloudflared not found in PATH, will download');
        }

        // Download cloudflared if not found
        await this.downloadCloudflared();
    }

    async downloadCloudflared() {
        const platform = os.platform();
        const arch = os.arch();
        let downloadUrl;
        let filename;

        // Determine download URL based on platform
        if (platform === 'win32') {
            filename = 'cloudflared.exe';
            downloadUrl = arch === 'x64' 
                ? 'https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe'
                : 'https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-386.exe';
        } else if (platform === 'darwin') {
            filename = 'cloudflared';
            downloadUrl = arch === 'arm64'
                ? 'https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz'
                : 'https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz';
        } else if (platform === 'linux') {
            filename = 'cloudflared';
            downloadUrl = arch === 'arm64'
                ? 'https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64'
                : 'https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64';
        } else {
            throw new Error(`Unsupported platform: ${platform}`);
        }

        const cloudflaredPath = path.join(this.configDir, filename);

        // Check if already downloaded
        if (await fs.pathExists(cloudflaredPath)) {
            this.cloudflaredPath = cloudflaredPath;
            await fs.chmod(cloudflaredPath, 0o755); // Make executable
            return;
        }

        console.log('Downloading cloudflared...');
        
        try {
            const response = await axios({
                method: 'GET',
                url: downloadUrl,
                responseType: 'stream'
            });

            const writer = fs.createWriteStream(cloudflaredPath);
            response.data.pipe(writer);

            await new Promise((resolve, reject) => {
                writer.on('finish', resolve);
                writer.on('error', reject);
            });

            // Make executable
            await fs.chmod(cloudflaredPath, 0o755);
            this.cloudflaredPath = cloudflaredPath;
            
            console.log('cloudflared downloaded successfully');

        } catch (error) {
            console.error('Failed to download cloudflared:', error);
            throw new Error('Failed to download cloudflared. Please install manually.');
        }
    }

    async loadTunnelConfig() {
        try {
            if (await fs.pathExists(this.tunnelConfigPath)) {
                const config = await fs.readJSON(this.tunnelConfigPath);
                this.tunnelId = config.tunnelId;
                this.tunnelUrl = config.tunnelUrl;
            }
        } catch (error) {
            console.error('Error loading tunnel config:', error);
        }
    }

    async saveTunnelConfig() {
        try {
            const config = {
                tunnelId: this.tunnelId,
                tunnelUrl: this.tunnelUrl,
                lastUsed: new Date().toISOString()
            };
            await fs.writeJSON(this.tunnelConfigPath, config, { spaces: 2 });
        } catch (error) {
            console.error('Error saving tunnel config:', error);
        }
    }

    async createTunnel() {
        if (this.isActive && this.tunnelUrl) {
            return this.tunnelUrl;
        }

        try {
            this.connectionAttempts++;
            console.log(`Creating Cloudflare tunnel (attempt ${this.connectionAttempts}/${this.maxConnectionAttempts})`);

            // Use quick tunnel for automatic setup
            const tunnelArgs = [
                'tunnel',
                '--url', `http://localhost:${this.localPort}`,
                '--no-autoupdate'
            ];

            this.tunnelProcess = spawn(this.cloudflaredPath, tunnelArgs, {
                stdio: ['ignore', 'pipe', 'pipe']
            });

            // Parse tunnel URL from output
            await this.parseTunnelOutput();

            this.isActive = true;
            this.connectionAttempts = 0; // Reset on success
            await this.saveTunnelConfig();

            console.log(`Tunnel created successfully: ${this.tunnelUrl}`);
            return this.tunnelUrl;

        } catch (error) {
            console.error('Failed to create tunnel:', error);
            
            if (this.connectionAttempts < this.maxConnectionAttempts) {
                console.log('Retrying tunnel creation...');
                await this.delay(2000);
                return this.createTunnel();
            } else {
                throw new Error(`Failed to create tunnel after ${this.maxConnectionAttempts} attempts: ${error.message}`);
            }
        }
    }

    async parseTunnelOutput() {
        return new Promise((resolve, reject) => {
            let output = '';
            const timeout = setTimeout(() => {
                reject(new Error('Timeout waiting for tunnel URL'));
            }, 30000); // 30 second timeout

            this.tunnelProcess.stdout.on('data', (data) => {
                output += data.toString();
                
                // Look for the tunnel URL in the output
                const urlMatch = output.match(/https:\/\/[a-zA-Z0-9-]+\.trycloudflare\.com/);
                if (urlMatch) {
                    this.tunnelUrl = urlMatch[0];
                    clearTimeout(timeout);
                    resolve();
                }

                // Also look for tunnel creation confirmation
                if (output.includes('Connection') && output.includes('registered')) {
                    // Extract tunnel ID if available
                    const idMatch = output.match(/Tunnel\s+([a-f0-9-]+)\s+/);
                    if (idMatch) {
                        this.tunnelId = idMatch[1];
                    }
                }
            });

            this.tunnelProcess.stderr.on('data', (data) => {
                const errorOutput = data.toString();
                console.error('Cloudflared stderr:', errorOutput);
                
                // Check for specific error conditions
                if (errorOutput.includes('connection refused') || errorOutput.includes('no such host')) {
                    clearTimeout(timeout);
                    reject(new Error('Local server not accessible'));
                }
                
                if (errorOutput.includes('authentication') || errorOutput.includes('login')) {
                    clearTimeout(timeout);
                    reject(new Error('Cloudflare authentication required'));
                }
            });

            this.tunnelProcess.on('error', (error) => {
                clearTimeout(timeout);
                reject(error);
            });

            this.tunnelProcess.on('exit', (code) => {
                if (code !== 0 && code !== null) {
                    clearTimeout(timeout);
                    reject(new Error(`Cloudflared exited with code ${code}`));
                }
            });
        });
    }

    async closeTunnel() {
        if (this.tunnelProcess) {
            console.log('Closing Cloudflare tunnel...');
            
            // Gracefully terminate the process
            this.tunnelProcess.kill('SIGTERM');
            
            // Wait for process to exit, then force kill if necessary
            setTimeout(() => {
                if (this.tunnelProcess && !this.tunnelProcess.killed) {
                    this.tunnelProcess.kill('SIGKILL');
                }
            }, 5000);

            this.tunnelProcess = null;
        }

        this.isActive = false;
        this.tunnelUrl = null;
        this.connectionAttempts = 0;
        console.log('Tunnel closed');
    }

    async checkHealth() {
        if (!this.isActive || !this.tunnelUrl) {
            return false;
        }

        try {
            // Check if local server is accessible through tunnel
            const response = await axios.get(`${this.tunnelUrl}/health`, {
                timeout: 10000
            });
            
            return response.status === 200;

        } catch (error) {
            console.error('Tunnel health check failed:', error.message);
            
            // Check if the tunnel process is still running
            if (this.tunnelProcess && this.tunnelProcess.exitCode !== null) {
                console.log('Tunnel process has exited, marking as unhealthy');
                this.isActive = false;
                return false;
            }
            
            return false;
        }
    }

    getCurrentUrl() {
        return this.isActive ? this.tunnelUrl : null;
    }

    isConnected() {
        return this.isActive && this.tunnelUrl && this.tunnelProcess && this.tunnelProcess.exitCode === null;
    }

    async getConnectionStatus() {
        return {
            isActive: this.isActive,
            tunnelUrl: this.tunnelUrl,
            tunnelId: this.tunnelId,
            processRunning: this.tunnelProcess && this.tunnelProcess.exitCode === null,
            connectionAttempts: this.connectionAttempts,
            lastHealthCheck: new Date().toISOString()
        };
    }

    async recreateTunnel() {
        console.log('Recreating tunnel...');
        await this.closeTunnel();
        await this.delay(1000);
        return this.createTunnel();
    }

    // Alternative method using named tunnels (requires Cloudflare account setup)
    async createNamedTunnel(tunnelName) {
        try {
            // This would require user to have authenticated with cloudflared login
            console.log('Creating named tunnel:', tunnelName);
            
            const createArgs = ['tunnel', 'create', tunnelName];
            const createProcess = spawn(this.cloudflaredPath, createArgs, { stdio: 'pipe' });
            
            let output = '';
            createProcess.stdout.on('data', (data) => {
                output += data.toString();
            });

            await new Promise((resolve, reject) => {
                createProcess.on('close', (code) => {
                    if (code === 0) {
                        // Extract tunnel ID from output
                        const idMatch = output.match(/([a-f0-9-]{36})/);
                        if (idMatch) {
                            this.tunnelId = idMatch[1];
                        }
                        resolve();
                    } else {
                        reject(new Error(`Failed to create named tunnel: exit code ${code}`));
                    }
                });
            });

            // Create config file for the tunnel
            const configPath = path.join(this.configDir, `${tunnelName}.yml`);
            const config = `
tunnel: ${this.tunnelId}
credentials-file: ${path.join(this.configDir, `${this.tunnelId}.json`)}

ingress:
  - hostname: ${tunnelName}.yourdomain.com
    service: http://localhost:${this.localPort}
  - service: http_status:404
`;
            
            await fs.writeFile(configPath, config);

            // Start the named tunnel
            const runArgs = ['tunnel', '--config', configPath, 'run', this.tunnelId];
            this.tunnelProcess = spawn(this.cloudflaredPath, runArgs, { stdio: 'pipe' });
            
            this.tunnelUrl = `https://${tunnelName}.yourdomain.com`;
            this.isActive = true;
            
            await this.saveTunnelConfig();
            return this.tunnelUrl;

        } catch (error) {
            console.error('Named tunnel creation failed:', error);
            throw error;
        }
    }

    async authenticateCloudflare() {
        // Helper method to authenticate with Cloudflare (opens browser)
        const authProcess = spawn(this.cloudflaredPath, ['tunnel', 'login'], { 
            stdio: 'inherit' 
        });

        return new Promise((resolve, reject) => {
            authProcess.on('close', (code) => {
                if (code === 0) {
                    resolve();
                } else {
                    reject(new Error(`Authentication failed with exit code ${code}`));
                }
            });
        });
    }

    delay(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    // Cleanup method
    async cleanup() {
        await this.closeTunnel();
        
        // Optionally clean up tunnel resources
        if (this.tunnelId) {
            try {
                const deleteProcess = spawn(this.cloudflaredPath, ['tunnel', 'delete', this.tunnelId], {
                    stdio: 'pipe'
                });
                
                await new Promise((resolve) => {
                    deleteProcess.on('close', resolve);
                });
                
                console.log('Tunnel resources cleaned up');
            } catch (error) {
                console.error('Error cleaning up tunnel:', error);
            }
        }
    }
}

module.exports = TunnelManager;