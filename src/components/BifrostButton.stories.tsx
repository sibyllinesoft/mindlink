import type { Meta, StoryObj } from '@storybook/react';
import BifrostButton from './BifrostButton';

/**
 * # Bifrost Button Component
 * 
 * The Bifrost Button provides access to advanced configuration options through
 * the Bifrost interface. It's designed as a subtle but accessible control that
 * fits seamlessly into the navigation bar.
 * 
 * ## Features
 * - **Clean Design**: Minimal, professional appearance
 * - **Error Handling**: Graceful error reporting via callback
 * - **Tauri Integration**: Opens external Bifrost configuration
 * - **Accessible**: Proper ARIA labels and keyboard support
 * - **Status Indicators**: Visual feedback for interactions
 */
const meta: Meta<typeof BifrostButton> = {
  title: 'Components/Controls',
  component: BifrostButton,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
The BifrostButton provides access to advanced configuration through the external
Bifrost application. It handles the communication with the Tauri backend and
provides error feedback when operations fail.
        `,
      },
    },
  },
  args: {
    onError: () => {},
  },
  argTypes: {
    onError: {
      action: 'error-occurred',
      description: 'Callback when Bifrost configuration encounters an error',
    },
  },
};

export default meta;
type Story = StoryObj<typeof BifrostButton>;

/**
 * Default button state
 */
export const Default: Story = {
  args: {},
  parameters: {
    docs: {
      description: {
        story: 'The default Bifrost button state, ready for user interaction.',
      },
    },
  },
};

/**
 * Button with hover state
 */
export const HoverState: Story = {
  args: {},
  parameters: {
    pseudo: { hover: true },
    docs: {
      description: {
        story: 'Bifrost button in hover state showing interaction feedback.',
      },
    },
  },
};

/**
 * Button with focus state
 */
export const FocusState: Story = {
  args: {},
  parameters: {
    pseudo: { focus: true },
    docs: {
      description: {
        story: 'Bifrost button with keyboard focus for accessibility.',
      },
    },
  },
};

/**
 * Button with active/pressed state
 */
export const ActiveState: Story = {
  args: {},
  parameters: {
    pseudo: { active: true },
    docs: {
      description: {
        story: 'Bifrost button in pressed/active state.',
      },
    },
  },
};

/**
 * Error handling demonstration
 */
export const ErrorHandling: Story = {
  args: {},
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates error handling when Bifrost configuration fails to open.',
      },
    },
  },
  play: async ({ canvasElement }) => {
    // Override the Tauri invoke to simulate an error
    const originalInvoke = window.__TAURI_INVOKE__;
    window.__TAURI_INVOKE__ = (command: string): Promise<any> => {
      if (command === 'open_bifrost_config') {
        return Promise.reject(new Error('Failed to open Bifrost configuration: Service unavailable'));
      }
      return originalInvoke?.(command) ?? Promise.resolve({});
    };

    // Click the button to trigger the error
    const button = canvasElement.querySelector('button') as HTMLButtonElement;
    if (button) {
      button.click();
    }

    // Restore original invoke after test
    setTimeout(() => {
      if (originalInvoke) {
        window.__TAURI_INVOKE__ = originalInvoke;
      }
    }, 1000);
  },
};

/**
 * Success flow demonstration
 */
export const SuccessFlow: Story = {
  args: {},
  parameters: {
    docs: {
      description: {
        story: 'Demonstrates successful Bifrost configuration opening.',
      },
    },
  },
  play: async ({ canvasElement }) => {
    // Click the button to show success behavior
    const button = canvasElement.querySelector('button') as HTMLButtonElement;
    if (button) {
      button.click();
    }
  },
};

/**
 * In navigation context
 */
export const InNavigationContext: Story = {
  args: {},
  decorators: [
    (Story) => (
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '16px',
        padding: '16px 24px',
        backgroundColor: 'var(--color-surface-secondary)',
        borderRadius: '8px',
        border: '1px solid var(--color-border-primary)'
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '12px',
          color: 'var(--color-text-primary)'
        }}>
          <img 
            src="/logo.webp" 
            alt="MindLink" 
            style={{ width: '32px', height: '32px' }}
          />
          <span style={{ fontWeight: 600 }}>MindLink</span>
        </div>
        <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: '12px' }}>
          <Story />
          <div style={{
            padding: '8px 16px',
            backgroundColor: 'var(--color-surface-tertiary)',
            borderRadius: '20px',
            fontSize: '12px',
            color: 'var(--color-text-secondary)'
          }}>
            Other Controls
          </div>
        </div>
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Bifrost button as it appears in the actual navigation context.',
      },
    },
  },
};

/**
 * Mobile responsive view
 */
export const MobileView: Story = {
  args: {},
  parameters: {
    viewport: {
      defaultViewport: 'mobile',
    },
    docs: {
      description: {
        story: 'Bifrost button on mobile devices. May have adjusted touch target size.',
      },
    },
  },
};

/**
 * Disabled state simulation
 */
export const DisabledState: Story = {
  args: {},
  decorators: [
    (Story) => (
      <div style={{ opacity: 0.5, pointerEvents: 'none' }}>
        <Story />
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Bifrost button in a disabled state (when feature is unavailable).',
      },
    },
  },
};

/**
 * Loading state simulation
 */
export const LoadingState: Story = {
  args: {},
  decorators: [
    (Story) => (
      <div style={{ position: 'relative' }}>
        <Story />
        <div style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: 'var(--color-surface-overlay)',
          borderRadius: '6px'
        }}>
          <div style={{
            width: '16px',
            height: '16px',
            border: '2px solid var(--color-text-secondary)',
            borderTop: '2px solid transparent',
            borderRadius: '50%',
            animation: 'spin 1s linear infinite'
          }} />
        </div>
        <style>
          {`
            @keyframes spin {
              0% { transform: rotate(0deg); }
              100% { transform: rotate(360deg); }
            }
          `}
        </style>
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Bifrost button showing a loading state while opening configuration.',
      },
    },
  },
};

/**
 * Button variations
 */
export const ButtonVariations: Story = {
  args: {},
  decorators: [
    (Story) => (
      <div style={{
        display: 'flex',
        flexDirection: 'column',
        gap: '16px',
        alignItems: 'flex-start'
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--color-text-secondary)',
          fontSize: '14px'
        }}>
          <strong>Standard:</strong>
          <Story />
        </div>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--color-text-secondary)',
          fontSize: '14px'
        }}>
          <strong>With Badge:</strong>
          <div style={{ position: 'relative' }}>
            <BifrostButton />
            <div style={{
              position: 'absolute',
              top: '-4px',
              right: '-4px',
              width: '8px',
              height: '8px',
              backgroundColor: 'var(--color-status-error)',
              borderRadius: '50%',
              border: '2px solid var(--color-surface-secondary)'
            }} />
          </div>
        </div>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          color: 'var(--color-text-secondary)',
          fontSize: '14px'
        }}>
          <strong>Large Size:</strong>
          <div style={{ transform: 'scale(1.25)' }}>
            <BifrostButton />
          </div>
        </div>
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story: 'Different visual variations of the Bifrost button.',
      },
    },
  },
};