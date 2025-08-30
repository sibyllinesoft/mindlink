import type { Meta, StoryObj } from '@storybook/react';
import NetworkStats from './NetworkStats';

/**
 * # Network Stats Component
 * 
 * The Network Stats component displays important connection information and
 * tunnel details in an organized, easy-to-read format. It provides users
 * with key technical details about their tunnel connection.
 * 
 * ## Features
 * - **Status Display**: Clear tunnel connection status
 * - **URL Information**: Public tunnel URL with copy functionality
 * - **Technical Details**: Protocol, port, and connection info
 * - **Status-Aware**: Different information based on tunnel state
 * - **Professional Layout**: Clean, technical aesthetic
 */
const meta: Meta<typeof NetworkStats> = {
  title: 'Components/NetworkStats',
  component: NetworkStats,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
NetworkStats provides detailed information about the current tunnel connection,
including status, URL, and technical details that help users understand their
current network configuration.
        `,
      },
    },
  },
  argTypes: {
    tunnelStatus: {
      control: 'select',
      options: ['disconnected', 'connecting', 'connected', 'error'],
      description: 'Current tunnel connection status',
    },
    publicUrl: {
      control: 'text',
      description: 'Public tunnel URL',
    },
  },
};

export default meta;
type Story = StoryObj<typeof NetworkStats>;

/**
 * Connected state with full network information
 */
export const Connected: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats when tunnel is connected, showing full connection details and URL.',
      },
    },
  },
};

/**
 * Disconnected state
 */
export const Disconnected: Story = {
  args: {
    tunnelStatus: 'disconnected',
    publicUrl: null,
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats when tunnel is disconnected, showing local server information only.',
      },
    },
  },
};

/**
 * Connecting state
 */
export const Connecting: Story = {
  args: {
    tunnelStatus: 'connecting',
    publicUrl: null,
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats while tunnel is connecting, showing in-progress status.',
      },
    },
  },
};

/**
 * Error state
 */
export const ErrorState: Story = {
  args: {
    tunnelStatus: 'error',
    publicUrl: null,
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats when tunnel connection failed, showing error information.',
      },
    },
  },
};

/**
 * Long URL test
 */
export const LongURL: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://very-long-tunnel-name-for-testing-responsive-behavior-and-text-wrapping-in-network-stats.trycloudflare.com',
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats with a very long URL to test text wrapping and responsive behavior.',
      },
    },
  },
};

/**
 * Custom domain
 */
export const CustomDomain: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://api.mindlink.dev',
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats with a custom domain (non-Cloudflare tunnel).',
      },
    },
  },
};

/**
 * HTTPS tunnel
 */
export const HTTPSTunnel: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://secure-tunnel-12345.trycloudflare.com',
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats showing an HTTPS tunnel connection with security indicators.',
      },
    },
  },
};

/**
 * HTTP tunnel (development)
 */
export const HTTPTunnel: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'http://dev-tunnel-67890.trycloudflare.com',
  },
  parameters: {
    docs: {
      description: {
        story: 'Network stats showing an HTTP tunnel (development/testing scenario).',
      },
    },
  },
};

/**
 * Mobile responsive view
 */
export const MobileView: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    docs: {
      description: {
        story: 'Network stats on mobile devices. Layout may stack vertically.',
      },
    },
  },
};

/**
 * Card in dark theme context
 */
export const DarkTheme: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
  },
  decorators: [
    (Story) => (
      <div style={{
        padding: '40px',
        backgroundColor: 'var(--color-surface-primary)',
        borderRadius: '12px',
        minHeight: '300px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center'
      }}>
        <Story />
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Network stats card in the context of the dark application background.',
      },
    },
  },
};

/**
 * Technical details focus
 */
export const TechnicalDetails: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
  },
  decorators: [
    (Story) => (
      <div>
        <Story />
        <div style={{
          margin: '20px',
          padding: '16px',
          backgroundColor: 'var(--color-surface-secondary)',
          borderRadius: '8px',
          fontSize: '14px',
          color: 'var(--color-text-secondary)'
        }}>
          <strong>Technical Information Displayed:</strong>
          <ul style={{ margin: '8px 0', paddingLeft: '20px' }}>
            <li>Connection Status (Connected, Connecting, Error, etc.)</li>
            <li>Public URL with protocol information</li>
            <li>Local server details (localhost:8080)</li>
            <li>Tunnel provider information (Cloudflare)</li>
            <li>Connection timestamp and uptime</li>
          </ul>
        </div>
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Highlights the technical information provided by the network stats component.',
      },
    },
  },
};

/**
 * Compact layout test
 */
export const CompactLayout: Story = {
  args: {
    tunnelStatus: 'connected',
    publicUrl: 'https://example-tunnel.trycloudflare.com',
  },
  decorators: [
    (Story) => (
      <div style={{ maxWidth: '300px' }}>
        <Story />
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Network stats in a compact layout to test responsive behavior in narrow containers.',
      },
    },
  },
};