import { ProviderPlugin, PluginRegistry, PluginConfig } from './types'

/**
 * Plugin Registry
 * Manages registration, loading, and lifecycle of provider plugins
 */
export class ProviderPluginRegistry implements PluginRegistry {
  private plugins = new Map<string, ProviderPlugin>()
  private configs = new Map<string, PluginConfig>()
  private initialized = false
  
  constructor() {
    this.loadPluginConfigs()
  }
  
  async initialize(): Promise<void> {
    if (this.initialized) return
    
    // Initialize all registered plugins
    const initPromises = Array.from(this.plugins.values()).map(async (plugin) => {
      try {
        await plugin.initialize()
        console.log(`Plugin ${plugin.id} initialized successfully`)
      } catch (error) {
        console.error(`Failed to initialize plugin ${plugin.id}:`, error)
      }
    })
    
    await Promise.all(initPromises)
    this.initialized = true
  }
  
  registerPlugin(plugin: ProviderPlugin): void {
    if (this.plugins.has(plugin.id)) {
      console.warn(`Plugin with id '${plugin.id}' is already registered. Replacing existing plugin.`)
    }
    
    this.plugins.set(plugin.id, plugin)
    
    // Initialize default config if not exists
    if (!this.configs.has(plugin.id)) {
      this.configs.set(plugin.id, { enabled: true })
      this.savePluginConfigs()
    }
    
    console.log(`Plugin '${plugin.displayName}' (${plugin.id}) registered successfully`)
  }
  
  unregisterPlugin(id: string): void {
    const plugin = this.plugins.get(id)
    if (plugin) {
      // Cleanup plugin if it has a destroy method
      if (plugin.destroy) {
        plugin.destroy().catch(error => {
          console.error(`Error during plugin ${id} cleanup:`, error)
        })
      }
      
      this.plugins.delete(id)
      console.log(`Plugin '${id}' unregistered`)
    }
  }
  
  getPlugin(id: string): ProviderPlugin | null {
    return this.plugins.get(id) || null
  }
  
  getAllPlugins(): ProviderPlugin[] {
    return Array.from(this.plugins.values())
  }
  
  getEnabledPlugins(): ProviderPlugin[] {
    return Array.from(this.plugins.values()).filter(plugin => {
      const config = this.configs.get(plugin.id)
      return config?.enabled !== false
    })
  }
  
  getPluginConfig(id: string): PluginConfig | null {
    return this.configs.get(id) || null
  }
  
  setPluginConfig(id: string, config: PluginConfig): void {
    this.configs.set(id, config)
    this.savePluginConfigs()
  }
  
  async getConnectionStatuses(): Promise<Array<{
    plugin: ProviderPlugin
    status: any
  }>> {
    const enabledPlugins = this.getEnabledPlugins()
    
    const statusPromises = enabledPlugins.map(async (plugin) => {
      try {
        const status = await plugin.getConnectionStatus()
        return { plugin, status }
      } catch (error) {
        console.error(`Error getting status for plugin ${plugin.id}:`, error)
        return {
          plugin,
          status: {
            status: 'error' as const,
            error: error instanceof Error ? error.message : 'Unknown error',
            lastChecked: new Date().toISOString()
          }
        }
      }
    })
    
    return Promise.all(statusPromises)
  }
  
  async refreshAllConnections(): Promise<void> {
    const enabledPlugins = this.getEnabledPlugins()
    
    const refreshPromises = enabledPlugins.map(async (plugin) => {
      try {
        await plugin.refreshConnectionInfo()
      } catch (error) {
        console.error(`Error refreshing connection for plugin ${plugin.id}:`, error)
      }
    })
    
    await Promise.all(refreshPromises)
  }
  
  private loadPluginConfigs(): void {
    try {
      const configsJson = localStorage.getItem('mindlink_plugin_configs')
      if (configsJson) {
        const configs = JSON.parse(configsJson)
        this.configs = new Map(Object.entries(configs))
      }
    } catch (error) {
      console.error('Error loading plugin configurations:', error)
    }
  }
  
  private savePluginConfigs(): void {
    try {
      const configsObject = Object.fromEntries(this.configs)
      localStorage.setItem('mindlink_plugin_configs', JSON.stringify(configsObject))
    } catch (error) {
      console.error('Error saving plugin configurations:', error)
    }
  }
  
  // Cleanup
  async destroy(): Promise<void> {
    const destroyPromises = Array.from(this.plugins.values()).map(async (plugin) => {
      if (plugin.destroy) {
        try {
          await plugin.destroy()
        } catch (error) {
          console.error(`Error destroying plugin ${plugin.id}:`, error)
        }
      }
    })
    
    await Promise.all(destroyPromises)
    
    this.plugins.clear()
    this.configs.clear()
    this.initialized = false
  }
}

// Global registry instance
export const pluginRegistry = new ProviderPluginRegistry()