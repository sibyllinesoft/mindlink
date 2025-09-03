import type { Meta, StoryObj } from '@storybook/react';

/**
 * Color Palette Documentation
 * 
 * MindLink uses a carefully crafted color system with three primary scales:
 * - Graphite: Primary brand colors with blue-gray undertones
 * - Charcoal: Deep backgrounds optimized for dark UI
 * - Slate: UI elements and secondary content
 */
const meta: Meta = {
  title: 'Design System/Colors',
  parameters: {
    docs: {
      description: {
        component: `
The MindLink color system is built around professional, technical aesthetics with high contrast ratios 
and accessibility in mind. All colors meet WCAG 2.1 AA standards for contrast.
        `,
      },
    },
  },
};

export default meta;
type Story = StoryObj;

const ColorSwatch = ({ 
  name, 
  value, 
  description 
}: { 
  name: string; 
  value: string; 
  description?: string;
}) => (
  <div style={{ 
    display: 'flex', 
    alignItems: 'center', 
    marginBottom: '12px',
    border: '1px solid rgba(255, 255, 255, 0.1)',
    borderRadius: '8px',
    overflow: 'hidden'
  }}>
    <div style={{
      width: '80px',
      height: '60px',
      backgroundColor: value,
      flexShrink: 0,
    }} />
    <div style={{
      padding: '12px 16px',
      flex: 1,
      backgroundColor: 'var(--color-surface-secondary)',
    }}>
      <div style={{ 
        fontWeight: 600, 
        fontSize: '14px', 
        marginBottom: '4px',
        color: 'var(--color-text-primary)'
      }}>
        {name}
      </div>
      <div style={{ 
        fontSize: '12px', 
        fontFamily: 'var(--font-family-mono)',
        color: 'var(--color-text-secondary)'
      }}>
        {value}
      </div>
      {description && (
        <div style={{ 
          fontSize: '12px', 
          marginTop: '4px',
          color: 'var(--color-text-tertiary)'
        }}>
          {description}
        </div>
      )}
    </div>
  </div>
);

const ColorScale = ({ 
  title, 
  description, 
  colors 
}: { 
  title: string; 
  description: string;
  colors: Array<{ name: string; value: string; description?: string; textColor?: string }>;
}) => (
  <div style={{ marginBottom: '48px' }}>
    <h3 style={{ 
      margin: '0 0 12px 0',
      fontSize: '18px',
      fontWeight: 600,
      color: 'var(--color-text-primary)'
    }}>
      {title}
    </h3>
    <p style={{ 
      margin: '0 0 24px 0',
      fontSize: '14px',
      lineHeight: 1.6,
      color: 'var(--color-text-secondary)'
    }}>
      {description}
    </p>
    {colors.map((color) => (
      <ColorSwatch key={color.name} {...color} />
    ))}
  </div>
);

export const GraphiteScale: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '600px' }}>
      <ColorScale
        title="Graphite Scale"
        description="Primary brand colors with professional blue-gray undertones. Used for primary actions, links, and brand elements."
        colors={[
          { name: 'Graphite 50', value: '#f8fafc', description: 'Lightest shade - backgrounds and subtle highlights' },
          { name: 'Graphite 100', value: '#f1f5f9', description: 'Very light - card backgrounds in light themes' },
          { name: 'Graphite 200', value: '#e2e8f0', description: 'Light - borders and dividers' },
          { name: 'Graphite 300', value: '#cbd5e1', description: 'Light medium - disabled states' },
          { name: 'Graphite 400', value: '#94a3b8', description: 'Medium - placeholder text and icons' },
          { name: 'Graphite 500', value: '#64748b', description: 'Base - secondary text and balanced UI elements' },
          { name: 'Graphite 600', value: '#475569', description: 'Medium dark - primary text in light themes' },
          { name: 'Graphite 700', value: '#334155', description: 'Dark - emphasis and strong contrast' },
          { name: 'Graphite 800', value: '#1e293b', description: 'Very dark - high contrast backgrounds' },
          { name: 'Graphite 900', value: '#0f172a', description: 'Darkest - deepest backgrounds and maximum contrast' },
        ]}
      />
    </div>
  ),
};

export const CharcoalScale: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '600px' }}>
      <ColorScale
        title="Charcoal Scale"
        description="Deep, neutral backgrounds optimized for dark UI themes. Provides excellent readability and reduces eye strain."
        colors={[
          { name: 'Charcoal 50', value: '#fafafa', description: 'Lightest - light theme backgrounds' },
          { name: 'Charcoal 100', value: '#f4f4f5', description: 'Very light - subtle contrast in light themes' },
          { name: 'Charcoal 200', value: '#e4e4e7', description: 'Light - borders in light themes' },
          { name: 'Charcoal 300', value: '#d4d4d8', description: 'Light medium - muted content' },
          { name: 'Charcoal 400', value: '#a1a1aa', description: 'Medium - secondary content' },
          { name: 'Charcoal 500', value: '#71717a', description: 'Base - balanced neutral tone' },
          { name: 'Charcoal 600', value: '#52525b', description: 'Medium dark - secondary backgrounds' },
          { name: 'Charcoal 700', value: '#3f3f46', description: 'Dark - tertiary backgrounds' },
          { name: 'Charcoal 800', value: '#27272a', description: 'Very dark - secondary app backgrounds' },
          { name: 'Charcoal 900', value: '#18181b', description: 'Darkest - primary app background' },
        ]}
      />
    </div>
  ),
};

export const StatusColors: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '600px' }}>
      <ColorScale
        title="Status Colors"
        description="Semantic colors for different system states. Each status has both a main color and a background variant for different contexts."
        colors={[
          { name: 'Connected', value: '#059669', description: 'Success, connected, active states' },
          { name: 'Connected Background', value: '#d1fae5', description: 'Background for success states' },
          { name: 'Connecting', value: '#d97706', description: 'Warning, in-progress, loading states' },
          { name: 'Connecting Background', value: '#fef3c7', description: 'Background for warning states' },
          { name: 'Disconnected', value: '#6b7280', description: 'Neutral, inactive, disabled states' },
          { name: 'Disconnected Background', value: '#f3f4f6', description: 'Background for neutral states' },
          { name: 'Error', value: '#dc2626', description: 'Error, danger, critical states' },
          { name: 'Error Background', value: '#fecaca', description: 'Background for error states' },
        ]}
      />
    </div>
  ),
};

export const SemanticColors: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '600px' }}>
      <ColorScale
        title="Semantic Color Assignments"
        description="How colors are applied to different UI elements. These semantic assignments ensure consistency across components."
        colors={[
          { name: 'Primary Surface', value: 'var(--color-charcoal-900)', description: 'Main application background' },
          { name: 'Secondary Surface', value: 'var(--color-charcoal-800)', description: 'Card and container backgrounds' },
          { name: 'Tertiary Surface', value: 'var(--color-charcoal-700)', description: 'Elevated elements and overlays' },
          { name: 'Card Surface', value: 'rgba(255, 255, 255, 0.05)', description: 'Transparent card backgrounds with subtle visibility' },
          { name: 'Primary Text', value: 'rgba(255, 255, 255, 0.95)', description: 'Main text content - high contrast' },
          { name: 'Secondary Text', value: 'rgba(255, 255, 255, 0.7)', description: 'Secondary text - good contrast' },
          { name: 'Tertiary Text', value: 'rgba(255, 255, 255, 0.5)', description: 'Helper text and captions' },
          { name: 'Muted Text', value: 'rgba(255, 255, 255, 0.4)', description: 'Disabled text and placeholders' },
          { name: 'Primary Border', value: 'rgba(255, 255, 255, 0.1)', description: 'Default border color' },
          { name: 'Focus Border', value: 'var(--color-accent-primary)', description: 'Focus states and active borders' },
        ]}
      />
    </div>
  ),
};

export const UsageGuidelines: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '800px' }}>
      <h3 style={{ 
        margin: '0 0 24px 0',
        fontSize: '18px',
        fontWeight: 600,
        color: 'var(--color-text-primary)'
      }}>
        Color Usage Guidelines
      </h3>
      
      <div style={{ 
        display: 'grid', 
        gap: '24px', 
        gridTemplateColumns: 'repeat(auto-fit, minmax(350px, 1fr))',
        marginBottom: '32px' 
      }}>
        <div style={{
          padding: '20px',
          backgroundColor: 'var(--color-surface-secondary)',
          borderRadius: '8px',
          border: '1px solid var(--color-border-primary)'
        }}>
          <h4 style={{ 
            margin: '0 0 12px 0',
            fontSize: '16px',
            fontWeight: 600,
            color: 'var(--color-text-primary)'
          }}>
            ✅ Do's
          </h4>
          <ul style={{ 
            margin: 0,
            paddingLeft: '16px',
            color: 'var(--color-text-secondary)',
            fontSize: '14px',
            lineHeight: 1.6
          }}>
            <li>Use semantic color assignments for consistency</li>
            <li>Test contrast ratios with accessibility tools</li>
            <li>Use status colors consistently across components</li>
            <li>Leverage CSS custom properties for theming</li>
            <li>Consider color-blind users when designing</li>
          </ul>
        </div>
        
        <div style={{
          padding: '20px',
          backgroundColor: 'var(--color-surface-secondary)',
          borderRadius: '8px',
          border: '1px solid var(--color-border-primary)'
        }}>
          <h4 style={{ 
            margin: '0 0 12px 0',
            fontSize: '16px',
            fontWeight: 600,
            color: 'var(--color-text-primary)'
          }}>
            ❌ Don'ts
          </h4>
          <ul style={{ 
            margin: 0,
            paddingLeft: '16px',
            color: 'var(--color-text-secondary)',
            fontSize: '14px',
            lineHeight: 1.6
          }}>
            <li>Don't use hardcoded color values in components</li>
            <li>Don't use colors that fail contrast requirements</li>
            <li>Don't mix status color meanings across contexts</li>
            <li>Don't rely solely on color to convey information</li>
            <li>Don't create custom colors outside the system</li>
          </ul>
        </div>
      </div>

      <div style={{
        padding: '20px',
        backgroundColor: 'var(--color-surface-card)',
        borderRadius: '8px',
        border: '1px solid var(--color-border-primary)'
      }}>
        <h4 style={{ 
          margin: '0 0 12px 0',
          fontSize: '16px',
          fontWeight: 600,
          color: 'var(--color-text-primary)'
        }}>
          Accessibility Standards
        </h4>
        <p style={{ 
          margin: '0 0 12px 0',
          fontSize: '14px',
          color: 'var(--color-text-secondary)',
          lineHeight: 1.6
        }}>
          All color combinations in this design system meet or exceed WCAG 2.1 AA standards:
        </p>
        <ul style={{ 
          margin: 0,
          paddingLeft: '16px',
          color: 'var(--color-text-secondary)',
          fontSize: '14px',
          lineHeight: 1.6
        }}>
          <li><strong>Normal text:</strong> 4.5:1 contrast ratio minimum</li>
          <li><strong>Large text:</strong> 3:1 contrast ratio minimum</li>
          <li><strong>Non-text elements:</strong> 3:1 contrast ratio for UI components</li>
          <li><strong>Focus indicators:</strong> High contrast and clearly visible</li>
        </ul>
      </div>
    </div>
  ),
};