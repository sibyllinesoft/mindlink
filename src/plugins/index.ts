/**
 * Plugin System Export
 * Main entry point for the MindLink plugin architecture
 */

// Core plugin system
export { BaseProviderPlugin } from './base-plugin'
export { ProviderPluginRegistry, pluginRegistry } from './registry'

// Types
export type {
  ProviderPlugin,
  ProviderStatus,
  ProviderConnectionInfo,
  OAuthConfig,
  PluginRegistry,
  PluginConfig,
  PluginManifest
} from './types'

// Provider plugins
export {
  OpenAIPlugin,
  AnthropicPlugin,
  GooglePlugin
} from './providers'