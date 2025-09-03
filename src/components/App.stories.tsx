import type { Meta, StoryObj } from '@storybook/react';
import App from '../App';
import { mockScenarios, updateMockState } from '../../.storybook/tauri-mocks';

/**
 * # App Component
 * 
 * The main application component that handles authentication, tunnel management,
 * and overall application state. This is the root component of the MindLink application.
 * 
 * ## Features
 * - **Authentication Flow**: OAuth modal for Cloudflare authentication
 * - **Tunnel Management**: Create, monitor, and close Cloudflare tunnels
 * - **Real-time Updates**: Listen for status changes via Tauri events
 * - **Error Handling**: Display error messages with auto-dismiss
 * - **Responsive Design**: Adapts to different screen sizes
 * 
 * ## States
 * - **Loading**: Initial authentication check
 * - **Unauthenticated**: Shows OAuth modal
 * - **Authenticated**: Shows main dashboard with tunnel controls
 * - **Error States**: Handles various error conditions
 */
const meta: Meta<typeof App> = {
  title: 'Components/App',
  component: App,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: `
The App component is the root of the MindLink application. It manages authentication state,
tunnel connections, and provides the overall layout structure. The component automatically
handles initial authentication checks and can display different UI states based on the
current application status.
        `,
      },
    },
  },
  decorators: [
    (Story) => (
      <div style={{ height: '100vh', overflow: 'hidden' }}>
        <Story />
      </div>
    ),
  ],
};

export default meta;
type Story = StoryObj<typeof App>;

/**
 * Default authenticated state showing the main dashboard
 */
export const Default: Story = {
  parameters: {
    docs: {
      description: {
        story: 'The default state shows an authenticated user with a connected tunnel and full dashboard access.',
      },
    },
  },
  play: async () => {
    // Set up the default connected state
    mockScenarios.allConnected();
  },
};

/**
 * Loading state during initial authentication check
 */
export const Loading: Story = {
  parameters: {
    docs: {
      description: {
        story: 'The loading state is shown while the app checks authentication status on startup.',
      },
    },
  },
  play: async () => {
    // Simulate a slower auth check to show loading state
    updateMockState({
      isAuthenticated: false,
    });
    
    // Override the invoke mock to add delay
    const originalInvoke = window.__TAURI_INVOKE__;
    window.__TAURI_INVOKE__ = (command: string): Promise<any> => {
      if (command === 'get_status') {
        return new Promise((resolve) => {
          setTimeout(() => {
            resolve({
              server_running: true,
              tunnel_connected: false,
              tunnel_url: null,
              is_authenticated: false
            });
          }, 5000); // 5 second delay to show loading
        });
      }
      return originalInvoke?.(command) ?? Promise.resolve({});
    };
  },
};

/**
 * Unauthenticated state showing OAuth modal
 */
export const Unauthenticated: Story = {
  parameters: {
    docs: {
      description: {
        story: 'When the user is not authenticated, the OAuth modal is displayed to initiate the authentication flow.',
      },
    },
  },
  play: async () => {
    mockScenarios.notAuthenticated();
  },
};

/**
 * Tunnel connecting state
 */
export const TunnelConnecting: Story = {
  parameters: {
    docs: {
      description: {
        story: 'Shows the app while a tunnel is in the process of being established.',
      },
    },
  },
  play: async () => {
    mockScenarios.tunnelConnecting();
  },
};

/**
 * Tunnel error state
 */
export const TunnelError: Story = {
  parameters: {
    docs: {
      description: {
        story: 'Displays error state when tunnel creation or connection fails.',
      },
    },
  },
  play: async () => {
    mockScenarios.tunnelError();
  },
};

/**
 * Server error state
 */
export const ServerError: Story = {
  parameters: {
    docs: {
      description: {
        story: 'Shows the app when the local server encounters an error.',
      },
    },
  },
  play: async () => {
    mockScenarios.serverError();
  },
};

/**
 * With error message banner
 */
export const WithErrorMessage: Story = {
  parameters: {
    docs: {
      description: {
        story: 'Displays an error message banner that appears when operations fail (e.g., Bifrost configuration errors).',
      },
    },
  },
  play: async () => {
    mockScenarios.allConnected();
    
    // Simulate an error message after component loads
    setTimeout(() => {
      // Trigger an error by trying to open Bifrost config with a failure
      window.__TAURI_INVOKE__ = (command: string): Promise<any> => {
        if (command === 'open_bifrost_config') {
          return Promise.reject(new Error('Failed to open Bifrost configuration: Service unavailable'));
        }
        return Promise.resolve({ success: true });
      };
    }, 100);
  },
};

/**
 * Mobile responsive view
 */
export const MobileView: Story = {
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    docs: {
      description: {
        story: 'Shows how the app adapts to mobile screen sizes.',
      },
    },
  },
  play: async () => {
    mockScenarios.allConnected();
  },
};

/**
 * Tablet responsive view
 */
export const TabletView: Story = {
  parameters: {
    viewport: {
      defaultViewport: 'tablet',
    },
    docs: {
      description: {
        story: 'Shows how the app adapts to tablet screen sizes.',
      },
    },
  },
  play: async () => {
    mockScenarios.allConnected();
  },
};

/**
 * Full workflow demonstration
 */
export const AuthenticationFlow: Story = {
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates the complete authentication flow from unauthenticated to authenticated state.',
      },
    },
  },
  play: async ({ canvasElement }) => {
    // Start unauthenticated
    mockScenarios.notAuthenticated();
    
    // Simulate authentication after a delay
    setTimeout(() => {
      mockScenarios.allConnected();
      
      // Trigger auth success event
      const authEvent = new CustomEvent('auth-success');
      canvasElement.dispatchEvent(authEvent);
    }, 3000);
  },
};