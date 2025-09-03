import type { Meta, StoryObj } from '@storybook/react';
import OAuthModal from './OAuthModal';

/**
 * # OAuth Modal Component
 * 
 * The OAuth Modal handles user authentication through Cloudflare's OAuth system.
 * It provides a clean, secure interface for users to authenticate and authorize
 * MindLink to create tunnels on their behalf.
 * 
 * ## Features
 * - **Secure Authentication**: Uses Cloudflare OAuth for secure auth
 * - **Step-by-step Flow**: Clear progression through auth steps
 * - **Error Handling**: Graceful error states and recovery
 * - **Professional Design**: Consistent with MindLink branding
 * - **Responsive Layout**: Works on all device sizes
 */
const meta: Meta<typeof OAuthModal> = {
  title: 'Components/Modals',
  component: OAuthModal,
  parameters: {
    layout: 'fullscreen',
    docs: {
      description: {
        component: `
The OAuthModal manages the complete authentication flow for MindLink users. It guides
users through the OAuth process with clear instructions and provides feedback for
each step of the authentication process.
        `,
      },
    },
  },
  args: {
    onAuthSuccess: () => {},
    onCancel: () => {},
  },
  argTypes: {
    isOpen: {
      control: 'boolean',
      description: 'Whether the modal is visible',
    },
    onAuthSuccess: {
      action: 'auth-success',
      description: 'Callback when authentication succeeds',
    },
    onCancel: {
      action: 'auth-cancel',
      description: 'Callback when user cancels authentication',
    },
  },
};

export default meta;
type Story = StoryObj<typeof OAuthModal>;

/**
 * Modal open and ready for authentication
 */
export const Default: Story = {
  args: {
    isOpen: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'The default OAuth modal state, ready for user to start authentication.',
      },
    },
  },
};

/**
 * Modal closed/hidden
 */
export const Closed: Story = {
  args: {
    isOpen: false,
  },
  parameters: {
    docs: {
      description: {
        story: 'The modal when closed/hidden. Users see the application behind it.',
      },
    },
  },
};

/**
 * Authentication in progress
 */
export const AuthInProgress: Story = {
  args: {
    isOpen: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Modal state while authentication is in progress. Shows loading/waiting state.',
      },
    },
  },
  play: async ({ canvasElement }) => {
    // Simulate clicking the auth button to show loading state
    const authButton = canvasElement.querySelector('.auth-button') as HTMLButtonElement;
    if (authButton && !authButton.disabled) {
      authButton.click();
    }
  },
};

/**
 * Authentication error state
 */
export const AuthError: Story = {
  args: {
    isOpen: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Modal showing an authentication error with recovery options.',
      },
    },
  },
  play: async ({ canvasElement }) => {
    // Simulate an authentication error
    // This would typically be triggered by a failed auth attempt
    setTimeout(() => {
      const errorMessage = canvasElement.querySelector('.error-message');
      if (errorMessage) {
        errorMessage.textContent = 'Authentication failed. Please try again.';
      }
    }, 100);
  },
};

/**
 * Mobile responsive view
 */
export const MobileView: Story = {
  args: {
    isOpen: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    docs: {
      description: {
        story: 'OAuth modal on mobile devices. May use full-screen layout.',
      },
    },
  },
};

/**
 * Tablet responsive view
 */
export const TabletView: Story = {
  args: {
    isOpen: true,
  },
  parameters: {
    viewport: {
      defaultViewport: 'tablet',
    },
    docs: {
      description: {
        story: 'OAuth modal on tablet devices.',
      },
    },
  },
};

/**
 * Without cancel option
 */
export const NoCancelOption: Story = {
  args: {
    isOpen: true,
    // onCancel omitted to test optional behavior
  },
  parameters: {
    docs: {
      description: {
        story: 'OAuth modal without a cancel option (authentication required scenario).',
      },
    },
  },
};

/**
 * Dark background demonstration
 */
export const DarkBackground: Story = {
  args: {
    isOpen: true,
  },
  decorators: [
    (Story) => (
      <div style={{
        minHeight: '100vh',
        backgroundColor: 'var(--color-surface-primary)',
        position: 'relative'
      }}>
        {/* Simulate app content behind modal */}
        <div style={{
          padding: '40px',
          color: 'var(--color-text-primary)'
        }}>
          <h1>MindLink Dashboard</h1>
          <p>This content is behind the modal overlay...</p>
        </div>
        <Story />
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'OAuth modal with application content visible behind it to show overlay effect.',
      },
    },
  },
};

/**
 * Accessibility demonstration
 */
export const AccessibilityFeatures: Story = {
  args: {
    isOpen: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates accessibility features like focus management and keyboard navigation.',
      },
    },
  },
  play: async ({ canvasElement }) => {
    // Focus on the first interactive element
    const firstButton = canvasElement.querySelector('button') as HTMLButtonElement;
    if (firstButton) {
      firstButton.focus();
    }
    
    // Test keyboard navigation
    setTimeout(() => {
      const event = new KeyboardEvent('keydown', { key: 'Tab' });
      canvasElement.dispatchEvent(event);
    }, 500);
  },
};

/**
 * Success state simulation
 */
export const SuccessFlow: Story = {
  args: {
    isOpen: true,
  },
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates the complete success flow from authentication to completion.',
      },
    },
  },
  play: async ({ args }) => {
    // Simulate successful authentication after delay
    setTimeout(() => {
      console.log('Simulating auth success...');
      if (args.onAuthSuccess) {
        args.onAuthSuccess();
      }
    }, 3000);
  },
};

/**
 * Loading states focus
 */
export const LoadingStates: Story = {
  args: {
    isOpen: true,
  },
  decorators: [
    (Story) => (
      <div>
        <Story />
        <div style={{
          position: 'fixed',
          bottom: '20px',
          left: '20px',
          right: '20px',
          padding: '16px',
          backgroundColor: 'var(--color-surface-secondary)',
          borderRadius: '8px',
          fontSize: '14px',
          color: 'var(--color-text-secondary)',
          border: '1px solid var(--color-border-primary)',
          zIndex: 2000
        }}>
          <strong>Authentication Flow States:</strong>
          <ul style={{ margin: '8px 0', paddingLeft: '20px' }}>
            <li><strong>Idle:</strong> Ready to start authentication</li>
            <li><strong>Initiating:</strong> Creating auth session</li>
            <li><strong>Waiting:</strong> User completing auth in browser</li>
            <li><strong>Success:</strong> Authentication completed</li>
            <li><strong>Error:</strong> Authentication failed</li>
          </ul>
        </div>
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Shows the different loading states the modal can display during authentication.',
      },
    },
  },
};