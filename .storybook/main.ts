import type { StorybookConfig } from '@storybook/react-vite';
import { resolve } from 'path';

const config: StorybookConfig = {
  stories: [
    "../src/**/*.mdx",
    "../src/**/*.stories.@(js|jsx|mjs|ts|tsx)"
  ],
  addons: [
    "@chromatic-com/storybook",
    "@storybook/addon-docs",
    "@storybook/addon-controls",
    "@storybook/addon-actions",
    "@storybook/addon-viewport",
    "@storybook/addon-backgrounds",
    "@storybook/addon-toolbars",
    "@storybook/addon-measure",
    "@storybook/addon-outline",
    "@storybook/addon-a11y",
    "@storybook/addon-onboarding"
  ],
  framework: {
    name: "@storybook/react-vite",
    options: {}
  },
  viteFinal: async (config) => {
    // Mock Tauri APIs for Storybook
    config.define = {
      ...config.define,
      __TAURI__: true,
    };
    
    // Add alias for easy imports
    config.resolve = {
      ...config.resolve,
      alias: {
        ...config.resolve?.alias,
        '@': resolve(__dirname, '../src'),
        '@components': resolve(__dirname, '../src/components'),
        '@design-system': resolve(__dirname, '../src/design-system'),
      }
    };
    
    return config;
  },
  typescript: {
    check: false,
    reactDocgen: 'react-docgen-typescript',
    reactDocgenTypescriptOptions: {
      shouldExtractLiteralValuesFromEnum: true,
      propFilter: (prop) => (prop.parent ? !/node_modules/.test(prop.parent.fileName) : true),
    },
  },
  docs: {
    autodocs: 'tag',
    defaultName: 'Documentation',
  },
};

export default config;