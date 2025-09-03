import { ProviderPlugin, PluginManifest } from './types'
import { pluginRegistry } from './registry'

/**
 * Dynamic Plugin Loader
 * Handles loading plugins from external directories or bundled modules
 */

export interface PluginLoaderConfig {
  pluginsDirectory?: string
  developmentMode?: boolean
  bundledPlugins?: (() => ProviderPlugin)[]
}

export class DynamicPluginLoader {
  private config: PluginLoaderConfig
  private loadedPlugins = new Map<string, ProviderPlugin>()
  
  constructor(config: PluginLoaderConfig = {}) {
    this.config = {
      developmentMode: import.meta.env.DEV || false,
      bundledPlugins: [],
      ...config
    }
  }
  
  /**
   * Load all available plugins
   * In development: Use bundled plugins
   * In production: Load from external directory via Tauri
   */
  async loadAllPlugins(): Promise<ProviderPlugin[]> {
    const plugins: ProviderPlugin[] = []
    
    try {
      console.log('🔌 DynamicPluginLoader config:', this.config)
      console.log('🔌 Development mode detected:', this.config.developmentMode)
      console.log('🔌 Environment DEV flag:', import.meta.env.DEV)
      
      if (this.config.developmentMode) {
        console.log('🔌 Development mode: Loading bundled plugins')
        plugins.push(...await this.loadBundledPlugins())
      } else {
        console.log('🔌 Production mode: Loading external plugins')
        plugins.push(...await this.loadExternalPlugins())
      }
      
      // Register all loaded plugins
      plugins.forEach(plugin => {
        this.loadedPlugins.set(plugin.id, plugin)
        pluginRegistry.registerPlugin(plugin)
      })
      
      console.log(`✅ Loaded ${plugins.length} plugins:`, plugins.map(p => p.displayName))
      return plugins
      
    } catch (error) {
      console.error('❌ Failed to load plugins:', error)
      return []
    }
  }
  
  /**
   * Load plugins bundled with the application (development mode)
   */
  private async loadBundledPlugins(): Promise<ProviderPlugin[]> {
    const plugins: ProviderPlugin[] = []
    
    try {
      console.log('🔌 Loading bundled plugins from ./providers')
      // Import bundled plugins dynamically
      const { OpenAIPlugin, AnthropicPlugin, GooglePlugin, OllamaPlugin } = await import('./providers')
      
      console.log('🔌 Plugin classes imported:', { OpenAIPlugin, AnthropicPlugin, GooglePlugin, OllamaPlugin })
      
      const openaiPlugin = new OpenAIPlugin()
      const anthropicPlugin = new AnthropicPlugin()
      const googlePlugin = new GooglePlugin()
      const ollamaPlugin = new OllamaPlugin()
      
      console.log('🔌 Plugin instances created:', [
        { id: openaiPlugin.id, name: openaiPlugin.displayName },
        { id: anthropicPlugin.id, name: anthropicPlugin.displayName },
        { id: googlePlugin.id, name: googlePlugin.displayName },
        { id: ollamaPlugin.id, name: ollamaPlugin.displayName }
      ])
      
      plugins.push(openaiPlugin, anthropicPlugin, googlePlugin, ollamaPlugin as ProviderPlugin)
      
      console.log('🔌 Bundled plugins loaded successfully:', plugins.length)
      return plugins
    } catch (error) {
      console.error('❌ Failed to load bundled plugins:', error)
      return []
    }
  }
  
  /**
   * Load plugins from external directory (production mode)
   */
  private async loadExternalPlugins(): Promise<ProviderPlugin[]> {
    const plugins: ProviderPlugin[] = []
    
    try {
      // Get available plugin manifests from Tauri
      const manifests = await this.getAvailablePluginManifests()
      
      for (const manifest of manifests) {
        try {
          const plugin = await this.loadPluginFromManifest(manifest)
          if (plugin) {
            plugins.push(plugin)
          }
        } catch (error) {
          console.error(`Failed to load plugin ${manifest.id}:`, error)
        }
      }
      
      return plugins
    } catch (error) {
      console.error('Failed to load external plugins:', error)
      return []
    }
  }
  
  /**
   * Get available plugin manifests from the plugins directory
   */
  private async getAvailablePluginManifests(): Promise<PluginManifest[]> {
    try {
      // Use Tauri command to get plugin manifests
      // @ts-ignore - Tauri invoke will be available at runtime
      if (typeof window !== 'undefined' && window.__TAURI__ && window.__TAURI__.invoke) {
        console.log('🔌 Getting plugin manifests from Tauri...')
        // @ts-ignore
        const response = await window.__TAURI__.invoke('get_plugin_manifests')
        console.log('🔌 Plugin discovery response:', response)
        
        if (response.success && response.manifests) {
          return response.manifests
        } else {
          console.warn('Plugin discovery failed:', response.error)
          return this.getBuiltinManifests()
        }
      }
      
      // Fallback for development/testing (when Tauri is not available)
      console.log('🔌 Tauri not available, using built-in manifests')
      return this.getBuiltinManifests()
      
    } catch (error) {
      console.error('Failed to get plugin manifests:', error)
      return this.getBuiltinManifests()
    }
  }
  
  /**
   * Load a plugin from its manifest
   */
  private async loadPluginFromManifest(manifest: PluginManifest): Promise<ProviderPlugin | null> {
    try {
      // In production, this would:
      // 1. Read the plugin file from the external directory
      // 2. Safely evaluate/import the plugin code
      // 3. Instantiate the plugin class
      
      // For now, we'll map manifest IDs to built-in plugins
      return this.createPluginFromManifest(manifest)
      
    } catch (error) {
      console.error(`Failed to load plugin from manifest ${manifest.id}:`, error)
      return null
    }
  }
  
  /**
   * Create plugin instances from manifest (temporary implementation)
   */
  private async createPluginFromManifest(manifest: PluginManifest): Promise<ProviderPlugin | null> {
    try {
      const { OpenAIPlugin, AnthropicPlugin, GooglePlugin, OllamaPlugin } = await import('./providers')
      
      switch (manifest.id) {
        case 'openai':
          return new OpenAIPlugin()
        case 'anthropic':
          return new AnthropicPlugin()
        case 'google':
          return new GooglePlugin()
        case 'ollama':
          return new OllamaPlugin() as ProviderPlugin
        default:
          console.warn(`Unknown plugin ID: ${manifest.id}`)
          return null
      }
    } catch (error) {
      console.error(`Failed to create plugin for ${manifest.id}:`, error)
      return null
    }
  }
  
  /**
   * Get built-in plugin manifests (fallback)
   */
  private getBuiltinManifests(): PluginManifest[] {
    return [
      {
        id: 'openai',
        name: 'OpenAI',
        version: '1.0.0',
        description: 'Connect to OpenAI GPT models',
        author: 'MindLink Team',
        main: 'openai.js',
        mindlinkVersion: '1.0.0'
      },
      {
        id: 'anthropic',
        name: 'Anthropic',
        version: '1.0.0',
        description: 'Connect to Claude models',
        author: 'MindLink Team',
        main: 'anthropic.js',
        mindlinkVersion: '1.0.0'
      },
      {
        id: 'google',
        name: 'Google',
        version: '1.0.0',
        description: 'Connect to Gemini models',
        author: 'MindLink Team',
        main: 'google.js',
        mindlinkVersion: '1.0.0'
      },
      {
        id: 'ollama',
        name: 'Ollama',
        version: '1.0.0',
        description: 'Connect to local Ollama models',
        author: 'MindLink Team',
        main: 'ollama.js',
        mindlinkVersion: '1.0.0'
      }
    ]
  }
  
  /**
   * Reload a specific plugin
   */
  async reloadPlugin(pluginId: string): Promise<boolean> {
    try {
      // Unregister the existing plugin
      pluginRegistry.unregisterPlugin(pluginId)
      this.loadedPlugins.delete(pluginId)
      
      // Load the plugin again
      const plugins = await this.loadAllPlugins()
      const reloadedPlugin = plugins.find(p => p.id === pluginId)
      
      return reloadedPlugin !== undefined
    } catch (error) {
      console.error(`Failed to reload plugin ${pluginId}:`, error)
      return false
    }
  }
  
  /**
   * Get all loaded plugin IDs
   */
  getLoadedPluginIds(): string[] {
    return Array.from(this.loadedPlugins.keys())
  }
  
  /**
   * Check if a plugin is loaded
   */
  isPluginLoaded(pluginId: string): boolean {
    return this.loadedPlugins.has(pluginId)
  }
}

// Global plugin loader instance
export const pluginLoader = new DynamicPluginLoader()