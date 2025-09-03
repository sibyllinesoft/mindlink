// API Response Types for Tauri Commands

export interface StatusResponse {
  is_serving: boolean
  is_authenticated: boolean
  tunnel_url?: string
  server_url?: string
  bifrost_url?: string
  instance_token?: string
  last_error?: string
}

export interface ServiceResponse {
  success: boolean
  message?: string
  server_url?: string
  tunnel_url?: string
  auth_url?: string
}

export interface BinaryInstallationResponse {
  success: boolean
  message?: string
  binary_path?: string
  is_installed: boolean
}