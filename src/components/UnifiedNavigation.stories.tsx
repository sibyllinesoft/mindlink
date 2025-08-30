import type { Meta, StoryObj } from '@storybook/react';
import UnifiedNavigation from './UnifiedNavigation';

/**
 * # Unified Navigation Component
 * 
 * The main navigation bar provides access to key controls and displays the application
 * branding. It features a clean, professional design with integrated tunnel controls
 * and Bifrost configuration access.
 * 
 * ## Features
 * - **Brand Identity**: Logo and application title
 * - **Tunnel Toggle**: Visual switch for tunnel connection
 * - **Bifrost Access**: Quick access to configuration
 * - **Status Indicators**: Visual feedback for different states
 * - **Responsive Design**: Adapts to different screen sizes
 */
const meta: Meta<typeof UnifiedNavigation> = {
  title: 'Components/Navigation',
  component: UnifiedNavigation,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: `
The UnifiedNavigation provides the main navigation and controls for the MindLink application.
It combines branding, tunnel management, and configuration access in a clean, accessible interface.
        `,
      },
    },
  },
  args: {
    onToggleTunnel: () => {},
    onBifrostError: () => {},
  },
  argTypes: {
    tunnelStatus: {
      control: 'select',
      options: ['disconnected', 'connecting', 'connected', 'error'],
      description: 'Current status of the Cloudflare tunnel',
    },
    isAuthenticated: {
      control: 'boolean',
      description: 'Whether the user is authenticated',
    },
    onToggleTunnel: {
      action: 'tunnel-toggled',
      description: 'Callback when tunnel toggle is clicked',
    },
    onBifrostError: {
      action: 'bifrost-error',
      description: 'Callback when Bifrost encounters an error',
    },
  },
};

export default meta;
type Story = StoryObj<typeof UnifiedNavigation>;

/**
 * Default authenticated state with connected tunnel
 */
export const Default: Story = {
  args: {
    tunnelStatus: 'connected',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'The default navigation state for an authenticated user with an active tunnel.',
      },
    },
  },
};

/**
 * Authenticated but tunnel disconnected
 */
export const Disconnected: Story = {
  args: {
    tunnelStatus: 'disconnected',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Navigation when the user is authenticated but the tunnel is disconnected.',
      },
    },
  },
};

/**
 * Tunnel connecting state
 */
export const Connecting: Story = {
  args: {
    tunnelStatus: 'connecting',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Navigation while the tunnel is in the process of connecting. The toggle is disabled during this state.',
      },
    },
  },
};

/**
 * Tunnel error state
 */
export const ErrorState: Story = {
  args: {
    tunnelStatus: 'error',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Navigation when the tunnel is in an error state. The toggle shows error styling.',
      },
    },
  },
};

/**
 * Unauthenticated state
 */
export const Unauthenticated: Story = {
  args: {
    tunnelStatus: 'disconnected',
    isAuthenticated: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Navigation when the user is not authenticated. Shows authentication required message instead of tunnel controls.',
      },
    },
  },
};

/**
 * Unauthenticated with tunnel error
 */
export const UnauthenticatedError: Story = {
  args: {
    tunnelStatus: 'error',
    isAuthenticated: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'Navigation when both authentication and tunnel have issues.',
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
    isAuthenticated: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    docs: {
      description: {
        story: 'Navigation on mobile devices. Text may be shortened and controls optimized for touch.',
      },
    },
  },
};

/**
 * Tablet responsive view
 */
export const TabletView: Story = {
  args: {
    tunnelStatus: 'connected',
    isAuthenticated: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'tablet',
    },
    docs: {
      description: {
        story: 'Navigation on tablet devices.',
      },
    },
  },
};

/**
 * Focus states demonstration
 */
export const FocusStates: Story = {
  args: {
    tunnelStatus: 'connected',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates focus states for keyboard navigation and accessibility.',
      },
    },
  },
  play: async ({ canvasElement }) => {
    // Focus on the tunnel toggle to show focus state
    const toggle = canvasElement.querySelector('.tunnel-toggle__input') as HTMLElement;
    if (toggle) {
      toggle.focus();
    }
  },
};

/**
 * Interactive toggle demonstration
 */
export const InteractiveToggle: Story = {
  args: {
    tunnelStatus: 'disconnected',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates the tunnel toggle interaction. Click the toggle to see state changes.',
      },
    },
  },
  play: async ({ canvasElement, args }) => {
    const toggle = canvasElement.querySelector('.tunnel-toggle__input') as HTMLInputElement;
    if (toggle) {
      // Simulate toggle interactions
      toggle.addEventListener('change', () => {
        console.log('Toggle clicked!', args);
      });
    }
  },
};

/**
 * Brand section only
 */
export const BrandOnly: Story = {
  args: {
    tunnelStatus: 'connected',
    isAuthenticated: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Focuses on the brand section of the navigation.',
      },
    },
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
          <strong>Brand Section Details:</strong>
          <ul style={{ margin: '8px 0', paddingLeft: '20px' }}>
            <li>Logo: 24x24px PNG with transparent background</li>
            <li>Title: "MindLink" in semibold weight</li>
            <li>Subtitle: "Network Gateway" in smaller, muted text</li>
            <li>Total height: 64px (--layout-header-height)</li>
          </ul>
        </div>
      </div>
    ),
  ],
};