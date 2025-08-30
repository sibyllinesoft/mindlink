import React from 'react';
import type { Preview } from '@storybook/react-vite';
import '../src/design-system/index.css';
import { setupTauriMocks } from './tauri-mocks';

// Initialize Tauri mocks for all stories
setupTauriMocks();

const preview: Preview = {
  parameters: {
    layout: 'fullscreen',
    actions: { argTypesRegex: '^on[A-Z].*' },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    backgrounds: {
      default: 'dark',
      values: [
        {
          name: 'dark',
          value: '#18181b', // var(--color-charcoal-900)
        },
        {
          name: 'darker',
          value: '#0f172a', // var(--color-graphite-900)
        },
        {
          name: 'light',
          value: '#f8fafc', // var(--color-graphite-50)
        },
      ],
    },
    viewport: {
      viewports: {
        mobile: {
          name: 'Mobile',
          styles: {
            width: '375px',
            height: '667px',
          },
        },
        tablet: {
          name: 'Tablet',
          styles: {
            width: '768px',
            height: '1024px',
          },
        },
        desktop: {
          name: 'Desktop',
          styles: {
            width: '1024px',
            height: '768px',
          },
        },
        large: {
          name: 'Large Desktop',
          styles: {
            width: '1440px',
            height: '900px',
          },
        },
        tauri: {
          name: 'Tauri Window',
          styles: {
            width: '1200px',
            height: '800px',
          },
        },
      },
      defaultViewport: 'tauri',
    },
    docs: {
      toc: {
        contentsSelector: '.sbdocs-content',
        headingSelector: 'h1, h2, h3',
        title: 'Table of Contents',
        disable: false,
        unsafeTocbotOptions: {
          orderedList: false,
        },
      },
    },
    options: {
      storySort: {
        order: [
          'Design System',
          ['Introduction', 'Colors', 'Typography', 'Spacing', 'Components'],
          'Components',
          ['App', 'Dashboard', 'Navigation', 'Modals', 'Cards', 'Controls'],
          'Examples',
        ],
      },
    },
  },
  decorators: [
    (Story) => (
      <div style={{ 
        fontFamily: 'var(--font-family-primary)', 
        minHeight: '100vh',
        background: 'var(--color-surface-primary)',
        color: 'var(--color-text-primary)' 
      }}>
        <Story />
      </div>
    ),
  ],
  globalTypes: {
    theme: {
      description: 'Theme for components',
      defaultValue: 'dark',
      toolbar: {
        title: 'Theme',
        icon: 'paintbrush',
        items: [
          { value: 'dark', title: 'Dark (Default)' },
          { value: 'light', title: 'Light (Testing)' },
        ],
        dynamicTitle: true,
      },
    },
    tunnelStatus: {
      description: 'Global tunnel status for testing',
      defaultValue: 'connected',
      toolbar: {
        title: 'Tunnel Status',
        icon: 'globe',
        items: [
          { value: 'disconnected', title: 'Disconnected' },
          { value: 'connecting', title: 'Connecting' },
          { value: 'connected', title: 'Connected' },
          { value: 'error', title: 'Error' },
        ],
        dynamicTitle: true,
      },
    },
    authenticated: {
      description: 'Authentication state for testing',
      defaultValue: true,
      toolbar: {
        title: 'Auth State',
        icon: 'lock',
        items: [
          { value: true, title: 'Authenticated' },
          { value: false, title: 'Not Authenticated' },
        ],
        dynamicTitle: true,
      },
    },
  },
};

export default preview;