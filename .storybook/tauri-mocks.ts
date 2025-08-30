/**
 * Tauri API Mocks for Storybook
 * 
 * This file provides comprehensive mocking of Tauri APIs to allow components
 * to work properly in Storybook environment.
 */

export interface MockServiceResponse {
  success: boolean;
  message?: string;
  auth_url?: string;
  server_url?: string;
  tunnel_url?: string;
}

export interface MockStatusResponse {
  server_running: boolean;
  tunnel_connected: boolean;
  tunnel_url: string | null;
  is_authenticated: boolean;
}

// Global state for mocks (can be manipulated by stories)
export const mockState = {
  isAuthenticated: true,
  tunnelStatus: 'connected' as 'disconnected' | 'connecting' | 'connected' | 'error',
  serverRunning: true,
  tunnelUrl: 'https://example-tunnel.trycloudflare.com',
  authUrl: 'https://example-auth.cloudflareaccess.com',
};

// Update mock state for story testing
export const updateMockState = (updates: Partial<typeof mockState>) => {
  Object.assign(mockState, updates);
};

// Mock Tauri invoke function with realistic responses
export const mockTauriInvoke = (command: string, args?: any): Promise<any> => {
  console.log(`[Tauri Mock] Invoking: ${command}`, args);
  
  // Add realistic delay for some operations
  const delay = ['create_tunnel', 'close_tunnel', 'start_oauth'].includes(command) ? 
    Math.random() * 1000 + 500 : 100;
  
  return new Promise((resolve) => {
    setTimeout(() => {
      switch (command) {
        case 'get_status':
          resolve({
            server_running: mockState.serverRunning,
            tunnel_connected: mockState.tunnelStatus === 'connected',
            tunnel_url: mockState.tunnelStatus === 'connected' ? mockState.tunnelUrl : null,
            is_authenticated: mockState.isAuthenticated,
          } as MockStatusResponse);
          break;
        
        case 'get_tunnel_status':
          resolve({
            success: mockState.tunnelStatus !== 'error',
            tunnel_url: mockState.tunnelStatus === 'connected' ? mockState.tunnelUrl : null,
          } as MockServiceResponse);
          break;
        
        case 'create_tunnel':
          // Simulate tunnel creation
          mockState.tunnelStatus = 'connected';
          resolve({
            success: true,
            tunnel_url: mockState.tunnelUrl,
            message: 'Tunnel created successfully',
          } as MockServiceResponse);
          break;
        
        case 'close_tunnel':
          // Simulate tunnel closure
          mockState.tunnelStatus = 'disconnected';
          resolve({
            success: true,
            message: 'Tunnel closed successfully',
          } as MockServiceResponse);
          break;
        
        case 'start_oauth':
          resolve({
            success: true,
            auth_url: mockState.authUrl,
            message: 'OAuth flow initiated',
          } as MockServiceResponse);
          break;
        
        case 'check_oauth_status':
          resolve({
            success: true,
            authenticated: mockState.isAuthenticated,
          });
          break;
        
        case 'open_bifrost_config':
          resolve({
            success: true,
            message: 'Bifrost configuration opened',
          } as MockServiceResponse);
          break;
        
        case 'start_server':
          mockState.serverRunning = true;
          resolve({
            success: true,
            server_url: 'http://localhost:8080',
            message: 'Server started successfully',
          } as MockServiceResponse);
          break;
        
        case 'stop_server':
          mockState.serverRunning = false;
          resolve({
            success: true,
            message: 'Server stopped successfully',
          } as MockServiceResponse);
          break;
        
        case 'get_server_status':
          resolve({
            success: mockState.serverRunning,
            server_url: mockState.serverRunning ? 'http://localhost:8080' : null,
          });
          break;
        
        default:
          console.warn(`[Tauri Mock] Unknown command: ${command}`);
          resolve({ 
            success: true, 
            message: `Mock response for ${command}` 
          });
      }
    }, delay);
  });
};

// Mock Tauri listen function with event simulation
export const mockTauriListen = (event: string, handler: (event: any) => void) => {
  console.log(`[Tauri Mock] Listening to: ${event}`);
  
  // Store handler for potential event emission
  mockEventHandlers.set(event, handler);
  
  // Return a mock unsubscribe function
  return Promise.resolve(() => {
    console.log(`[Tauri Mock] Unsubscribed from: ${event}`);
    mockEventHandlers.delete(event);
  });
};

// Event handler storage for mock events
const mockEventHandlers = new Map<string, (event: any) => void>();

// Simulate Tauri events (can be called from stories for testing)
export const simulateTauriEvent = (eventName: string, payload: any) => {
  const handler = mockEventHandlers.get(eventName);
  if (handler) {
    console.log(`[Tauri Mock] Emitting event: ${eventName}`, payload);
    handler({ payload });
  } else {
    console.warn(`[Tauri Mock] No handler for event: ${eventName}`);
  }
};

// Preset scenarios for testing
export const mockScenarios = {
  // User not authenticated
  notAuthenticated: () => {
    updateMockState({
      isAuthenticated: false,
      tunnelStatus: 'disconnected',
      tunnelUrl: null,
    });
  },
  
  // Tunnel connecting state
  tunnelConnecting: () => {
    updateMockState({
      isAuthenticated: true,
      tunnelStatus: 'connecting',
      tunnelUrl: null,
    });
  },
  
  // Tunnel error state
  tunnelError: () => {
    updateMockState({
      isAuthenticated: true,
      tunnelStatus: 'error',
      tunnelUrl: null,
    });
  },
  
  // Happy path - everything working
  allConnected: () => {
    updateMockState({
      isAuthenticated: true,
      tunnelStatus: 'connected',
      serverRunning: true,
      tunnelUrl: 'https://example-tunnel.trycloudflare.com',
    });
  },
  
  // Server error
  serverError: () => {
    updateMockState({
      isAuthenticated: true,
      serverRunning: false,
      tunnelStatus: 'error',
    });
  },
};

// Inject mocks into global scope for Storybook
export const setupTauriMocks = () => {
  // @ts-ignore
  window.__TAURI__ = true;
  // @ts-ignore
  window.__TAURI_INVOKE__ = mockTauriInvoke;
  // @ts-ignore
  window.__TAURI_LISTEN__ = mockTauriListen;
};