const fs = require('fs-extra');
const path = require('path');
const os = require('os');

/**
 * ConfigManager handles application configuration
 */
class ConfigManager {
    constructor() {
        this.configDir = path.join(os.homedir(), '.mindlink');
        this.configPath = path.join(this.configDir, 'config.json');
        this.config = null;
        this.defaultConfig = {
            server: {
                port: 3001,
                host: '127.0.0.1',
                maxRequestSize: '50mb'
            },
            tunnel: {
                enabled: true,
                type: 'quick', // 'quick' or 'named'
                customDomain: null,
                maxRetries: 5
            },
            authentication: {
                autoRefreshTokens: true,
                tokenRefreshThreshold: 300 // 5 minutes in seconds
            },
            monitoring: {
                healthCheckInterval: 30000, // 30 seconds
                maxErrorsBeforeAlert: 5,
                enableNotifications: true
            },
            features: {
                reasoningEffort: 'medium', // 'low', 'medium', 'high'
                reasoningSummary: 'auto', // 'auto', 'concise', 'detailed', 'none'
                reasoningCompat: 'think-tags' // 'think-tags', 'o3', 'legacy'
            },
            ui: {
                startMinimized: true,
                minimizeToTray: true,
                showBalloonNotifications: true,
                theme: 'system' // 'light', 'dark', 'system'
            }
        };
    }

    async initialize() {
        await fs.ensureDir(this.configDir);
        await this.loadConfig();
    }

    async loadConfig() {
        try {
            if (await fs.pathExists(this.configPath)) {
                const configData = await fs.readJSON(this.configPath);
                
                // Merge with default config to ensure all keys exist
                this.config = this.deepMerge(this.defaultConfig, configData);
                
                // Validate and clean up config
                this.validateConfig();
                
                // Save merged config back to ensure new defaults are persisted
                await this.saveConfig();
            } else {
                // Use default config
                this.config = { ...this.defaultConfig };
                await this.saveConfig();
            }
        } catch (error) {
            console.error('Error loading config:', error);
            this.config = { ...this.defaultConfig };
        }
    }

    async saveConfig(newConfig = null) {
        if (newConfig) {
            this.config = this.deepMerge(this.config, newConfig);
            this.validateConfig();
        }

        try {
            await fs.writeJSON(this.configPath, this.config, { spaces: 2 });
            return true;
        } catch (error) {
            console.error('Error saving config:', error);
            return false;
        }
    }

    getConfig(path = null) {
        if (path) {
            return this.getNestedValue(this.config, path);
        }
        return { ...this.config };
    }

    async updateConfig(updates) {
        this.config = this.deepMerge(this.config, updates);
        this.validateConfig();
        return this.saveConfig();
    }

    // Get specific config values with dot notation
    get(path, defaultValue = undefined) {
        const value = this.getNestedValue(this.config, path);
        return value !== undefined ? value : defaultValue;
    }

    // Set specific config values with dot notation
    async set(path, value) {
        this.setNestedValue(this.config, path, value);
        this.validateConfig();
        return this.saveConfig();
    }

    // Server configuration helpers
    getServerConfig() {
        return this.config.server;
    }

    async updateServerConfig(updates) {
        return this.updateConfig({ server: updates });
    }

    // Tunnel configuration helpers
    getTunnelConfig() {
        return this.config.tunnel;
    }

    async updateTunnelConfig(updates) {
        return this.updateConfig({ tunnel: updates });
    }

    // Features configuration helpers
    getFeaturesConfig() {
        return this.config.features;
    }

    async updateFeaturesConfig(updates) {
        return this.updateConfig({ features: updates });
    }

    // Monitoring configuration helpers
    getMonitoringConfig() {
        return this.config.monitoring;
    }

    async updateMonitoringConfig(updates) {
        return this.updateConfig({ monitoring: updates });
    }

    validateConfig() {
        // Validate server config
        if (this.config.server.port < 1024 || this.config.server.port > 65535) {
            console.warn('Invalid server port, resetting to default');
            this.config.server.port = this.defaultConfig.server.port;
        }

        // Validate tunnel config
        const validTunnelTypes = ['quick', 'named'];
        if (!validTunnelTypes.includes(this.config.tunnel.type)) {
            console.warn('Invalid tunnel type, resetting to default');
            this.config.tunnel.type = this.defaultConfig.tunnel.type;
        }

        // Validate features config
        const validEfforts = ['low', 'medium', 'high'];
        if (!validEfforts.includes(this.config.features.reasoningEffort)) {
            console.warn('Invalid reasoning effort, resetting to default');
            this.config.features.reasoningEffort = this.defaultConfig.features.reasoningEffort;
        }

        const validSummaries = ['auto', 'concise', 'detailed', 'none'];
        if (!validSummaries.includes(this.config.features.reasoningSummary)) {
            console.warn('Invalid reasoning summary, resetting to default');
            this.config.features.reasoningSummary = this.defaultConfig.features.reasoningSummary;
        }

        const validCompat = ['think-tags', 'o3', 'legacy'];
        if (!validCompat.includes(this.config.features.reasoningCompat)) {
            console.warn('Invalid reasoning compatibility, resetting to default');
            this.config.features.reasoningCompat = this.defaultConfig.features.reasoningCompat;
        }

        // Validate monitoring config
        if (this.config.monitoring.healthCheckInterval < 5000) {
            console.warn('Health check interval too low, setting minimum');
            this.config.monitoring.healthCheckInterval = 5000;
        }

        if (this.config.monitoring.maxErrorsBeforeAlert < 1) {
            console.warn('Invalid max errors threshold, resetting to default');
            this.config.monitoring.maxErrorsBeforeAlert = this.defaultConfig.monitoring.maxErrorsBeforeAlert;
        }

        // Validate UI config
        const validThemes = ['light', 'dark', 'system'];
        if (!validThemes.includes(this.config.ui.theme)) {
            console.warn('Invalid theme, resetting to default');
            this.config.ui.theme = this.defaultConfig.ui.theme;
        }
    }

    deepMerge(target, source) {
        const output = { ...target };
        
        if (this.isObject(target) && this.isObject(source)) {
            Object.keys(source).forEach(key => {
                if (this.isObject(source[key])) {
                    if (!(key in target)) {
                        output[key] = source[key];
                    } else {
                        output[key] = this.deepMerge(target[key], source[key]);
                    }
                } else {
                    output[key] = source[key];
                }
            });
        }
        
        return output;
    }

    isObject(item) {
        return item && typeof item === 'object' && !Array.isArray(item);
    }

    getNestedValue(obj, path) {
        return path.split('.').reduce((current, key) => {
            return current && current[key] !== undefined ? current[key] : undefined;
        }, obj);
    }

    setNestedValue(obj, path, value) {
        const keys = path.split('.');
        const lastKey = keys.pop();
        
        const target = keys.reduce((current, key) => {
            if (current[key] === undefined) {
                current[key] = {};
            }
            return current[key];
        }, obj);
        
        target[lastKey] = value;
    }

    // Export config for backup
    async exportConfig() {
        return {
            config: { ...this.config },
            timestamp: new Date().toISOString(),
            version: '1.0.0'
        };
    }

    // Import config from backup
    async importConfig(importData) {
        try {
            if (importData.config) {
                this.config = this.deepMerge(this.defaultConfig, importData.config);
                this.validateConfig();
                await this.saveConfig();
                return true;
            }
            return false;
        } catch (error) {
            console.error('Error importing config:', error);
            return false;
        }
    }

    // Reset to default config
    async resetToDefaults() {
        this.config = { ...this.defaultConfig };
        await this.saveConfig();
        return true;
    }

    // Get config schema for UI
    getConfigSchema() {
        return {
            server: {
                title: 'Server Settings',
                properties: {
                    port: {
                        type: 'number',
                        title: 'Port',
                        description: 'Local server port (1024-65535)',
                        minimum: 1024,
                        maximum: 65535,
                        default: this.defaultConfig.server.port
                    },
                    host: {
                        type: 'string',
                        title: 'Host',
                        description: 'Local server host address',
                        default: this.defaultConfig.server.host
                    }
                }
            },
            tunnel: {
                title: 'Tunnel Settings',
                properties: {
                    enabled: {
                        type: 'boolean',
                        title: 'Enable Tunneling',
                        description: 'Create public tunnel via Cloudflare',
                        default: this.defaultConfig.tunnel.enabled
                    },
                    type: {
                        type: 'string',
                        title: 'Tunnel Type',
                        enum: ['quick', 'named'],
                        enumNames: ['Quick Tunnel', 'Named Tunnel'],
                        default: this.defaultConfig.tunnel.type
                    },
                    customDomain: {
                        type: 'string',
                        title: 'Custom Domain',
                        description: 'Custom domain for named tunnels (optional)'
                    }
                }
            },
            features: {
                title: 'AI Features',
                properties: {
                    reasoningEffort: {
                        type: 'string',
                        title: 'Reasoning Effort',
                        enum: ['low', 'medium', 'high'],
                        enumNames: ['Low (Fastest)', 'Medium (Balanced)', 'High (Most Accurate)'],
                        default: this.defaultConfig.features.reasoningEffort
                    },
                    reasoningSummary: {
                        type: 'string',
                        title: 'Reasoning Summary',
                        enum: ['auto', 'concise', 'detailed', 'none'],
                        enumNames: ['Auto', 'Concise', 'Detailed', 'None'],
                        default: this.defaultConfig.features.reasoningSummary
                    }
                }
            },
            ui: {
                title: 'Interface',
                properties: {
                    startMinimized: {
                        type: 'boolean',
                        title: 'Start Minimized',
                        description: 'Start the app minimized to system tray',
                        default: this.defaultConfig.ui.startMinimized
                    },
                    showBalloonNotifications: {
                        type: 'boolean',
                        title: 'Show Notifications',
                        description: 'Show desktop notifications',
                        default: this.defaultConfig.ui.showBalloonNotifications
                    },
                    theme: {
                        type: 'string',
                        title: 'Theme',
                        enum: ['light', 'dark', 'system'],
                        enumNames: ['Light', 'Dark', 'System'],
                        default: this.defaultConfig.ui.theme
                    }
                }
            }
        };
    }
}

module.exports = ConfigManager;