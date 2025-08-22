const { app, BrowserWindow, Tray, Menu, nativeImage, dialog, shell, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs-extra');
const os = require('os');
const notifier = require('node-notifier');
const { spawn, exec } = require('child_process');

// Import our local modules
const AuthManager = require('./auth/authManager');
const TunnelManager = require('./tunnel/tunnelManager');
const ServerManager = require('./server/serverManager');
const ConfigManager = require('./config/configManager');
const BifrostManager = require('./bifrost/bifrostManager');

class MindLinkApp {
    constructor() {
        this.tray = null;
        this.mainWindow = null;
        this.authManager = new AuthManager();
        this.tunnelManager = new TunnelManager();
        this.serverManager = new ServerManager();
        this.configManager = new ConfigManager();
        this.bifrostManager = new BifrostManager();
        this.isServing = false;
        this.lastError = null;
        this.errorCount = 0;
        this.maxErrors = 5;
        this.checkInterval = null;
    }

    async initialize() {
        // Create app data directory
        const appDataDir = path.join(os.homedir(), '.mindlink');
        await fs.ensureDir(appDataDir);

        // Initialize managers
        await this.configManager.initialize();
        await this.authManager.initialize();
        await this.tunnelManager.initialize();
        
        // Set up inter-manager dependencies
        this.serverManager.setAuthManager(this.authManager);
        this.bifrostManager.setManagers(this.authManager, this.serverManager, this.tunnelManager, this.configManager);

        // Start Bifrost management dashboard
        await this.startBifrostDashboard();

        this.createTray();
        this.setupEventListeners();
        this.updateTrayMenu();
        
        // Start health monitoring
        this.startHealthCheck();
    }

    createTray() {
        // Create tray icon
        const iconPath = this.getIconPath('disconnected');
        const trayIcon = nativeImage.createFromPath(iconPath);
        this.tray = new Tray(trayIcon.resize({ width: 16, height: 16 }));
        this.tray.setToolTip('MindLink - Disconnected');
    }

    getIconPath(status) {
        const iconName = `icon-${status}.png`;
        return path.join(__dirname, '..', 'assets', iconName);
    }

    updateTrayIcon(status) {
        const iconPath = this.getIconPath(status);
        if (fs.existsSync(iconPath)) {
            const trayIcon = nativeImage.createFromPath(iconPath);
            this.tray.setImage(trayIcon.resize({ width: 16, height: 16 }));
        }
        
        const statusMap = {
            'connected': 'MindLink - Connected',
            'connecting': 'MindLink - Connecting...',
            'error': 'MindLink - Error',
            'disconnected': 'MindLink - Disconnected'
        };
        this.tray.setToolTip(statusMap[status] || 'MindLink');
    }

    updateTrayMenu() {
        const isLoggedIn = this.authManager.isAuthenticated();
        const config = this.configManager.getConfig();
        
        const contextMenu = Menu.buildFromTemplate([
            {
                label: isLoggedIn ? 'Logged In' : 'Not Logged In',
                enabled: false
            },
            {
                type: 'separator'
            },
            {
                label: isLoggedIn ? 'Start Serving' : 'Login & Serve',
                click: () => this.handleLoginAndServe(),
                enabled: !this.isServing
            },
            {
                label: 'Stop Serving',
                click: () => this.stopServing(),
                enabled: this.isServing
            },
            {
                type: 'separator'
            },
            {
                label: 'Bifrost Dashboard',
                click: () => this.openBifrostDashboard()
            },
            {
                label: 'Connection Status',
                click: () => this.showConnectionStatus()
            },
            {
                label: 'Settings',
                click: () => this.showSettings()
            },
            {
                type: 'separator'
            },
            {
                label: 'Open API Dashboard',
                click: () => this.openDashboard(),
                enabled: this.isServing
            },
            {
                label: 'Copy API URL',
                click: () => this.copyApiUrl(),
                enabled: this.isServing
            },
            {
                type: 'separator'
            },
            {
                label: 'Help',
                click: () => shell.openExternal('https://github.com/mindlink/docs')
            },
            {
                label: 'Quit',
                click: () => app.quit()
            }
        ]);

        this.tray.setContextMenu(contextMenu);
    }

    async handleLoginAndServe() {
        if (!this.authManager.isAuthenticated()) {
            await this.login();
        }
        
        if (this.authManager.isAuthenticated()) {
            await this.startServing();
        }
    }

    async login() {
        try {
            this.updateTrayIcon('connecting');
            this.showNotification('Starting authentication...', 'Please complete login in your browser');
            
            await this.authManager.login();
            
            this.showNotification('Login successful!', 'You can now start serving API requests');
            this.updateTrayIcon('connected');
            
        } catch (error) {
            console.error('Login failed:', error);
            this.showNotification('Login failed', error.message);
            this.updateTrayIcon('error');
        }
        
        this.updateTrayMenu();
    }

    async startServing() {
        try {
            this.isServing = true;
            this.updateTrayIcon('connecting');
            this.updateTrayMenu();
            
            // Start local server
            await this.serverManager.start();
            
            // Setup Cloudflare tunnel
            const tunnelUrl = await this.tunnelManager.createTunnel();
            
            this.updateTrayIcon('connected');
            this.showNotification('MindLink Active!', `API available at: ${tunnelUrl}`);
            
            // Reset error counter on successful start
            this.errorCount = 0;
            
        } catch (error) {
            console.error('Failed to start serving:', error);
            this.isServing = false;
            this.updateTrayIcon('error');
            this.showNotification('Failed to start', error.message);
        }
        
        this.updateTrayMenu();
    }

    async stopServing() {
        try {
            this.isServing = false;
            this.updateTrayIcon('disconnected');
            
            await this.tunnelManager.closeTunnel();
            await this.serverManager.stop();
            
            this.showNotification('MindLink stopped', 'API service has been stopped');
            
        } catch (error) {
            console.error('Error stopping service:', error);
            this.showNotification('Error stopping', error.message);
        }
        
        this.updateTrayMenu();
    }

    startHealthCheck() {
        // Check every 30 seconds
        this.checkInterval = setInterval(() => {
            this.performHealthCheck();
        }, 30000);
    }

    async performHealthCheck() {
        if (!this.isServing) return;

        try {
            const isHealthy = await this.serverManager.checkHealth();
            const isTunnelHealthy = await this.tunnelManager.checkHealth();
            const isBifrostHealthy = await this.bifrostManager.checkHealth();
            
            if (!isHealthy || !isTunnelHealthy || !isBifrostHealthy) {
                this.errorCount++;
                this.lastError = new Date();
                
                if (this.errorCount >= this.maxErrors) {
                    this.updateTrayIcon('error');
                    this.showErrorNotification();
                    this.errorCount = 0; // Reset to avoid spam
                }
                
                // Try to restart Bifrost if it's unhealthy
                if (!isBifrostHealthy && this.bifrostManager.isRunning) {
                    console.log('Bifrost dashboard unhealthy, attempting restart...');
                    try {
                        await this.bifrostManager.stop();
                        await this.bifrostManager.start();
                    } catch (restartError) {
                        console.error('Failed to restart Bifrost dashboard:', restartError);
                    }
                }
            } else {
                // Reset error count on successful health check
                if (this.errorCount > 0) {
                    this.errorCount = 0;
                    this.updateTrayIcon('connected');
                }
            }
        } catch (error) {
            console.error('Health check failed:', error);
            this.errorCount++;
            this.lastError = new Date();
        }
    }

    showErrorNotification() {
        notifier.notify({
            title: 'MindLink Connection Issue',
            message: 'Connection problems detected. Click to reconnect.',
            icon: this.getIconPath('error'),
            sound: true,
            wait: true
        }, (err, response) => {
            if (response === 'activate') {
                this.handleReconnect();
            }
        });
    }

    async handleReconnect() {
        this.showNotification('Reconnecting...', 'Attempting to restore connection');
        
        try {
            // Check authentication first
            if (!this.authManager.isAuthenticated()) {
                await this.login();
            }
            
            // Restart services
            await this.stopServing();
            await this.startServing();
            
        } catch (error) {
            console.error('Reconnection failed:', error);
            this.showNotification('Reconnection failed', 'Please try manually restarting the service');
        }
    }

    showNotification(title, message) {
        notifier.notify({
            title: title,
            message: message,
            icon: this.getIconPath('connected'),
            sound: false,
            timeout: 5
        });
    }

    showConnectionStatus() {
        const status = this.isServing ? 'Connected' : 'Disconnected';
        const tunnelUrl = this.tunnelManager.getCurrentUrl();
        const serverUrl = this.serverManager.getLocalUrl();
        const bifrostUrl = this.bifrostManager.getLocalUrl();
        const bifrostStatus = this.bifrostManager.isRunning ? 'Running' : 'Stopped';
        
        let message = `Status: ${status}\n`;
        if (this.isServing) {
            message += `Local: ${serverUrl}\n`;
            if (tunnelUrl) {
                message += `Public: ${tunnelUrl}\n`;
            }
        }
        
        message += `Dashboard: ${bifrostStatus}`;
        if (this.bifrostManager.isRunning) {
            message += ` (${bifrostUrl})`;
        }
        
        if (this.lastError) {
            message += `\nLast Error: ${this.lastError.toLocaleTimeString()}`;
        }
        
        dialog.showMessageBox({
            type: 'info',
            title: 'MindLink Status',
            message: message,
            buttons: ['OK']
        });
    }

    showSettings() {
        // Create settings window if it doesn't exist
        if (!this.mainWindow) {
            this.createSettingsWindow();
        } else {
            this.mainWindow.show();
        }
    }

    createSettingsWindow() {
        this.mainWindow = new BrowserWindow({
            width: 600,
            height: 500,
            webPreferences: {
                nodeIntegration: true,
                contextIsolation: false
            },
            icon: this.getIconPath('connected')
        });

        this.mainWindow.loadFile(path.join(__dirname, '..', 'ui', 'settings.html'));
        
        this.mainWindow.on('closed', () => {
            this.mainWindow = null;
        });

        // Hide window instead of closing when user clicks X
        this.mainWindow.on('close', (event) => {
            event.preventDefault();
            this.mainWindow.hide();
        });
    }

    openDashboard() {
        const url = this.tunnelManager.getCurrentUrl() || this.serverManager.getLocalUrl();
        if (url) {
            shell.openExternal(url);
        }
    }

    copyApiUrl() {
        const url = this.tunnelManager.getCurrentUrl() || this.serverManager.getLocalUrl();
        if (url) {
            require('electron').clipboard.writeText(`${url}/v1`);
            this.showNotification('Copied!', 'API URL copied to clipboard');
        }
    }

    async startBifrostDashboard() {
        try {
            await this.bifrostManager.start();
            console.log(`Bifrost dashboard started on ${this.bifrostManager.getLocalUrl()}`);
        } catch (error) {
            console.error('Failed to start Bifrost dashboard:', error);
            this.showNotification('Bifrost Warning', 'Management dashboard failed to start');
        }
    }

    openBifrostDashboard() {
        const bifrostUrl = this.bifrostManager.getLocalUrl();
        if (bifrostUrl && this.bifrostManager.isRunning) {
            shell.openExternal(bifrostUrl);
        } else {
            this.showNotification('Bifrost Unavailable', 'Management dashboard is not running');
        }
    }

    setupEventListeners() {
        app.on('window-all-closed', (e) => {
            e.preventDefault(); // Prevent app from quitting
        });

        app.on('before-quit', async () => {
            if (this.checkInterval) {
                clearInterval(this.checkInterval);
            }
            
            await this.stopServing();
            
            // Stop Bifrost dashboard
            try {
                await this.bifrostManager.stop();
            } catch (error) {
                console.error('Error stopping Bifrost dashboard:', error);
            }
        });

        // IPC handlers for settings window
        ipcMain.handle('get-config', () => {
            return this.configManager.getConfig();
        });

        ipcMain.handle('save-config', async (event, config) => {
            await this.configManager.saveConfig(config);
            this.updateTrayMenu();
            return { success: true };
        });

        ipcMain.handle('get-status', () => {
            return {
                isServing: this.isServing,
                isAuthenticated: this.authManager.isAuthenticated(),
                tunnelUrl: this.tunnelManager.getCurrentUrl(),
                serverUrl: this.serverManager.getLocalUrl(),
                lastError: this.lastError
            };
        });
    }
}

// App lifecycle
app.whenReady().then(async () => {
    const mindLink = new MindLinkApp();
    await mindLink.initialize();
});

app.on('activate', () => {
    // macOS specific - recreate window when dock icon is clicked
});

// Prevent multiple instances
const gotTheLock = app.requestSingleInstanceLock();
if (!gotTheLock) {
    app.quit();
} else {
    app.on('second-instance', () => {
        // Someone tried to run a second instance, focus our window instead
        if (global.mindLink && global.mindLink.mainWindow) {
            global.mindLink.mainWindow.show();
        }
    });
}