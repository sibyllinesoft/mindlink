import type { Meta, StoryObj } from '@storybook/react';
import Dashboard from './Dashboard';

/**
 * # Dashboard Component
 * 
 * The main dashboard displays the core functionality of MindLink in a clean,
 * organized grid layout. It shows tunnel status, QR code for mobile access,
 * and network statistics.
 * 
 * ## Features
 * - **QR Code Card**: Easy mobile access to tunnel URL
 * - **Network Stats**: Real-time tunnel and connection information
 * - **Responsive Grid**: Adapts to different screen sizes
 * - **Status Integration**: Reflects current tunnel and server states
 */
const meta: Meta<typeof Dashboard> = {
  title: 'Components/Dashboard',
  component: Dashboard,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: `
The Dashboard is the main content area of MindLink, displayed when users are authenticated.
It provides a clean overview of the current tunnel status and easy access to key features
through well-organized cards and components.
        `,
      },
    },
  },
  args: {
    onToggleTunnel: () => {},
  },
  argTypes: {
    serverStatus: {
      control: 'select',
      options: ['running', 'error'],
      description: 'Current status of the local server',
    },
    tunnelStatus: {
      control: 'select',
      options: ['disconnected', 'connecting', 'connected', 'error'],
      description: 'Current status of the Cloudflare tunnel',
    },
    publicUrl: {
      control: 'text',
      description: 'Public URL of the active tunnel',
    },
    isAuthenticated: {
      control: 'boolean',
      description: 'Whether the user is authenticated',
    },
    onToggleTunnel: {
      action: 'tunnel-toggled',
      description: 'Callback when tunnel toggle is triggered',
    },
  },
};

export default meta;
type Story = StoryObj<typeof Dashboard>;

/**
 * Default state with connected tunnel
 */
export const Default: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'The default dashboard state with an active tunnel connection and all services running.',
      },
    },
  },
};

/**
 * Disconnected state
 */
export const Disconnected: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'disconnected',
    publicUrl: null,
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard when the tunnel is disconnected but the server is running.',
      },
    },
  },
};

/**
 * Connecting state
 */
export const Connecting: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'connecting',
    publicUrl: null,
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard while the tunnel is in the process of connecting.',
      },
    },
  },
};

/**
 * Error state
 */
export const ErrorState: Story = {
  args: {
    serverStatus: 'error',
    tunnelStatus: 'error',
    publicUrl: null,
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard when both server and tunnel are in error states.',
      },
    },
  },
};

/**
 * Server error only
 */
export const ServerError: Story = {
  args: {
    serverStatus: 'error',
    tunnelStatus: 'disconnected',
    publicUrl: null,
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard when the server has an error but tunnel is simply disconnected.',
      },
    },
  },
};

/**
 * Tunnel error only
 */
export const TunnelError: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'error',
    publicUrl: null,
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard when the tunnel has an error but server is running normally.',
      },
    },
  },
};

/**
 * With custom tunnel URL
 */
export const CustomTunnelURL: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'connected',
    publicUrl: 'https://mindlink-demo-abc123.trycloudflare.com',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard with a custom-branded tunnel URL.',
      },
    },
  },
};

/**
 * Not authenticated (shouldn\'t normally be visible)
 */
export const NotAuthenticated: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
    isAuthenticated: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard when user is not authenticated (this state normally triggers the OAuth modal instead).',
      },
    },
  },
};

/**
 * Mobile responsive view
 */
export const MobileView: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
    isAuthenticated: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    docs: {
      description: {
        story: 'Dashboard layout on mobile devices with stacked card layout.',
      },
    },
  },
};

/**
 * Tablet responsive view
 */
export const TabletView: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
    isAuthenticated: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'tablet',
    },
    docs: {
      description: {
        story: 'Dashboard layout on tablet devices.',
      },
    },
  },
};

/**
 * Long URL test
 */
export const LongURL: Story = {
  args: {
    serverStatus: 'running',
    tunnelStatus: 'connected',
    publicUrl: 'https://very-long-tunnel-name-for-testing-responsive-behavior-and-text-wrapping.trycloudflare.com',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Dashboard with a very long tunnel URL to test text wrapping and responsive behavior.',
      },
    },
  },
};