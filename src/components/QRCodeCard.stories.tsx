import type { Meta, StoryObj } from '@storybook/react';
import QRCodeCard from './QRCodeCard';

/**
 * # QR Code Card Component
 * 
 * The QR Code Card provides an easy way for users to access their tunnel URL
 * on mobile devices. It displays a QR code containing connection information
 * and includes helpful status indicators.
 * 
 * ## Features
 * - **QR Code Generation**: Automatically creates QR code from tunnel URL
 * - **Status-aware Display**: Shows different states based on tunnel status
 * - **Mobile Optimized**: Designed for easy mobile scanning
 * - **Copy Functionality**: Quick URL copying for sharing
 * - **Professional Styling**: Consistent with MindLink design system
 */
const meta: Meta<typeof QRCodeCard> = {
  title: 'Components/QRCodeCard',
  component: QRCodeCard,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
The QRCodeCard displays a QR code for easy mobile access to tunnel URLs. It adapts its
display based on the current tunnel status and provides helpful messaging for each state.
        `,
      },
    },
  },
  argTypes: {
    publicUrl: {
      control: 'text',
      description: 'Public tunnel URL to encode in QR code',
    },
    tunnelStatus: {
      control: 'select',
      options: ['disconnected', 'connecting', 'connected', 'error'],
      description: 'Current tunnel connection status',
    },
  },
};

export default meta;
type Story = StoryObj<typeof QRCodeCard>;

/**
 * Connected state with active QR code
 */
export const Connected: Story = {
  args: {
    publicUrl: 'https://example-tunnel.trycloudflare.com',
    tunnelStatus: 'connected',
  },
  parameters: {
    docs: {
      description: {
        story: 'QR code card when tunnel is connected and URL is available. Shows active QR code for scanning.',
      },
    },
  },
};

/**
 * Disconnected state
 */
export const Disconnected: Story = {
  args: {
    publicUrl: null,
    tunnelStatus: 'disconnected',
  },
  parameters: {
    docs: {
      description: {
        story: 'QR code card when tunnel is disconnected. Shows message to connect tunnel first.',
      },
    },
  },
};

/**
 * Connecting state
 */
export const Connecting: Story = {
  args: {
    publicUrl: null,
    tunnelStatus: 'connecting',
  },
  parameters: {
    docs: {
      description: {
        story: 'QR code card while tunnel is connecting. Shows loading state with helpful message.',
      },
    },
  },
};

/**
 * Error state
 */
export const ErrorState: Story = {
  args: {
    publicUrl: null,
    tunnelStatus: 'error',
  },
  parameters: {
    docs: {
      description: {
        story: 'QR code card when tunnel connection failed. Shows error message and recovery guidance.',
      },
    },
  },
};

/**
 * Long URL test
 */
export const LongURL: Story = {
  args: {
    publicUrl: 'https://very-long-tunnel-name-for-testing-responsive-behavior-and-qr-code-generation.trycloudflare.com',
    tunnelStatus: 'connected',
  },
  parameters: {
    docs: {
      description: {
        story: 'QR code card with a very long URL to test text wrapping and QR code generation.',
      },
    },
  },
};

/**
 * Custom domain URL
 */
export const CustomDomain: Story = {
  args: {
    publicUrl: 'https://mindlink-demo.example.com',
    tunnelStatus: 'connected',
  },
  parameters: {
    docs: {
      description: {
        story: 'QR code card with a custom domain URL (non-Cloudflare tunnel).',
      },
    },
  },
};

/**
 * Localhost URL (development)
 */
export const LocalhostURL: Story = {
  args: {
    publicUrl: 'http://localhost:8080',
    tunnelStatus: 'connected',
  },
  parameters: {
    docs: {
      description: {
        story: 'QR code card with localhost URL for development/testing scenarios.',
      },
    },
  },
};

/**
 * Mobile responsive view
 */
export const MobileView: Story = {
  args: {
    publicUrl: 'https://example-tunnel.trycloudflare.com',
    tunnelStatus: 'connected',
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    docs: {
      description: {
        story: 'QR code card on mobile devices. QR code size may be optimized for mobile screens.',
      },
    },
  },
};

/**
 * Card in dark theme context
 */
export const DarkTheme: Story = {
  args: {
    publicUrl: 'https://example-tunnel.trycloudflare.com',
    tunnelStatus: 'connected',
  },
  decorators: [
    (Story) => (
      <div style={{
        padding: '40px',
        backgroundColor: 'var(--color-surface-primary)',
        borderRadius: '12px',
        minHeight: '400px',
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
        story: 'QR code card in the context of the dark application background.',
      },
    },
  },
};

/**
 * State transitions demonstration
 */
export const StateTransitions: Story = {
  args: {
    publicUrl: null,
    tunnelStatus: 'disconnected',
  },
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates different states the QR code card can display. Use controls to change the tunnel status.',
      },
    },
  },
  play: async () => {
    const states: Array<{
      status: 'disconnected' | 'connecting' | 'connected' | 'error';
      url: string | null;
      delay: number;
    }> = [
      { status: 'disconnected', url: null, delay: 1000 },
      { status: 'connecting', url: null, delay: 2000 },
      { status: 'connected', url: 'https://example-tunnel.trycloudflare.com', delay: 3000 },
      { status: 'error', url: null, delay: 1000 },
    ];

    let currentIndex = 0;
    
    const cycleStates = () => {
      const state = states[currentIndex];
      if (!state) return;
      // Note: In a real implementation, you'd update the component props
      // This is just for demonstration purposes
      console.log(`Transitioning to: ${state.status}`, state.url);
      
      currentIndex = (currentIndex + 1) % states.length;
      setTimeout(cycleStates, state.delay);
    };

    // Start the cycle after a short delay
    setTimeout(cycleStates, 500);
  },
};