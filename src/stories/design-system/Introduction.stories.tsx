import type { Meta, StoryObj } from '@storybook/react';

const meta: Meta = {
  title: 'Design System/Introduction',
  parameters: {
    docs: {
      page: () => (
        <div style={{ padding: '40px', maxWidth: '800px' }}>
          <h1 style={{ 
            fontSize: '2.5rem', 
            fontWeight: 700, 
            marginBottom: '24px',
            color: 'var(--color-text-primary)'
          }}>
            MindLink Design System
          </h1>
          <div style={{ fontSize: '16px', lineHeight: '1.6', color: 'var(--color-text-secondary)' }}>
            <p style={{ marginBottom: '24px' }}>
              Welcome to the <strong>MindLink Design System</strong> - a professional, dark-themed design system 
              inspired by modern network and DevOps tools. Built with a focus on clarity, accessibility, and technical precision.
            </p>
            
            <h2 style={{ fontSize: '1.5rem', fontWeight: 600, marginBottom: '16px', color: 'var(--color-text-primary)' }}>
              Overview
            </h2>
            <p style={{ marginBottom: '24px' }}>
              MindLink's design system uses a <strong>graphite-inspired color palette</strong> with deep charcoal backgrounds 
              and precise typography, creating an interface that feels both professional and approachable for technical users.
            </p>

            <h3 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '12px', color: 'var(--color-text-primary)' }}>
              Key Characteristics
            </h3>
            <ul style={{ marginBottom: '24px', paddingLeft: '20px' }}>
              <li><strong>Dark-first design</strong> optimized for extended use</li>
              <li><strong>High contrast ratios</strong> meeting WCAG 2.1 AA standards</li>
              <li><strong>Graphite color palette</strong> with subtle gradients and professional tones</li>
              <li><strong>Technical aesthetic</strong> suitable for network monitoring and system administration</li>
              <li><strong>Component-based architecture</strong> with reusable design tokens</li>
            </ul>

            <h2 style={{ fontSize: '1.5rem', fontWeight: 600, marginBottom: '16px', color: 'var(--color-text-primary)' }}>
              Color Philosophy
            </h2>
            <p style={{ marginBottom: '16px' }}>
              The color system is built around three primary scales:
            </p>
            <ul style={{ marginBottom: '24px', paddingLeft: '20px' }}>
              <li><strong>Graphite Scale</strong> - Primary brand colors with blue-gray undertones</li>
              <li><strong>Charcoal Scale</strong> - Deep backgrounds for maximum readability</li>
              <li><strong>Slate Scale</strong> - UI elements and secondary content</li>
            </ul>
            <p style={{ marginBottom: '24px' }}>
              Each scale provides 10 carefully calibrated steps from light to dark, ensuring consistent visual hierarchy and accessibility.
            </p>

            <h2 style={{ fontSize: '1.5rem', fontWeight: 600, marginBottom: '16px', color: 'var(--color-text-primary)' }}>
              Getting Started
            </h2>
            <p style={{ marginBottom: '16px' }}>
              Explore the individual sections to learn about:
            </p>
            <ul style={{ paddingLeft: '20px' }}>
              <li><strong>Colors</strong> - Complete color palette and usage guidelines</li>
              <li><strong>Typography</strong> - Font sizes, weights, and hierarchy</li>
              <li><strong>Spacing</strong> - Layout system and component spacing</li>
              <li><strong>Components</strong> - Individual component documentation</li>
            </ul>
          </div>
        </div>
      ),
    },
  },
};

export default meta;
type Story = StoryObj;

export const Introduction: Story = {
  render: () => null,
  parameters: {
    docs: {
      source: { code: null },
    },
  },
};