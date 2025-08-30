import type { Meta, StoryObj } from '@storybook/react';

/**
 * Typography System Documentation
 * 
 * MindLink uses Inter as the primary font family, providing excellent readability
 * and a professional appearance suitable for technical interfaces.
 */
const meta: Meta = {
  title: 'Design System/Typography',
  parameters: {
    docs: {
      description: {
        component: `
The typography system provides a consistent hierarchy with carefully chosen font sizes, 
weights, and line heights optimized for both UI elements and content readability.
        `,
      },
    },
  },
};

export default meta;
type Story = StoryObj;

const TypographyExample = ({ 
  label, 
  style, 
  text = "The quick brown fox jumps over the lazy dog" 
}: { 
  label: string; 
  style: React.CSSProperties; 
  text?: string;
}) => (
  <div style={{ 
    marginBottom: '32px',
    padding: '20px',
    backgroundColor: 'var(--color-surface-secondary)',
    borderRadius: '8px',
    border: '1px solid var(--color-border-primary)'
  }}>
    <div style={{ 
      fontSize: '12px',
      fontWeight: 600,
      marginBottom: '8px',
      color: 'var(--color-text-secondary)',
      fontFamily: 'var(--font-family-mono)',
      textTransform: 'uppercase',
      letterSpacing: '0.025em'
    }}>
      {label}
    </div>
    <div style={style}>{text}</div>
    <div style={{ 
      marginTop: '12px',
      fontSize: '11px',
      color: 'var(--color-text-tertiary)',
      fontFamily: 'var(--font-family-mono)'
    }}>
      font-size: {style.fontSize} | font-weight: {style.fontWeight} | line-height: {style.lineHeight}
    </div>
  </div>
);

export const FontSizes: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '800px' }}>
      <h2 style={{ 
        marginBottom: '32px',
        fontSize: '24px',
        fontWeight: 600,
        color: 'var(--color-text-primary)'
      }}>
        Font Size Scale
      </h2>
      
      <TypographyExample
        label="4XL (36px)"
        style={{
          fontSize: 'var(--font-size-4xl)',
          fontWeight: 'var(--font-weight-bold)',
          lineHeight: 'var(--line-height-tight)',
          color: 'var(--color-text-primary)'
        }}
        text="Main Headlines"
      />
      
      <TypographyExample
        label="3XL (30px)"
        style={{
          fontSize: 'var(--font-size-3xl)',
          fontWeight: 'var(--font-weight-semibold)',
          lineHeight: 'var(--line-height-tight)',
          color: 'var(--color-text-primary)'
        }}
        text="Page Headers"
      />
      
      <TypographyExample
        label="2XL (24px)"
        style={{
          fontSize: 'var(--font-size-2xl)',
          fontWeight: 'var(--font-weight-semibold)',
          lineHeight: 'var(--line-height-tight)',
          color: 'var(--color-text-primary)'
        }}
        text="Section Headings"
      />
      
      <TypographyExample
        label="XL (20px)"
        style={{
          fontSize: 'var(--font-size-xl)',
          fontWeight: 'var(--font-weight-medium)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Subsection Headings"
      />
      
      <TypographyExample
        label="LG (18px)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-medium)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Large Body Text"
      />
      
      <TypographyExample
        label="BASE (16px)"
        style={{
          fontSize: 'var(--font-size-base)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Standard Body Text"
      />
      
      <TypographyExample
        label="SM (14px)"
        style={{
          fontSize: 'var(--font-size-sm)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-secondary)'
        }}
        text="Small Body Text"
      />
      
      <TypographyExample
        label="XS (12px)"
        style={{
          fontSize: 'var(--font-size-xs)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-tertiary)'
        }}
        text="Captions and Helper Text"
      />
    </div>
  ),
};

export const FontWeights: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '600px' }}>
      <h2 style={{ 
        marginBottom: '32px',
        fontSize: '24px',
        fontWeight: 600,
        color: 'var(--color-text-primary)'
      }}>
        Font Weights
      </h2>
      
      <TypographyExample
        label="Light (300)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-light)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Light weight - use sparingly for special emphasis"
      />
      
      <TypographyExample
        label="Regular (400)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Regular weight - default for body text"
      />
      
      <TypographyExample
        label="Medium (500)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-medium)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Medium weight - for emphasis and labels"
      />
      
      <TypographyExample
        label="Semibold (600)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-semibold)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Semibold weight - for headings and important content"
      />
      
      <TypographyExample
        label="Bold (700)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-bold)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Bold weight - for strong emphasis and calls-to-action"
      />
    </div>
  ),
};

export const LineHeights: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '600px' }}>
      <h2 style={{ 
        marginBottom: '32px',
        fontSize: '24px',
        fontWeight: 600,
        color: 'var(--color-text-primary)'
      }}>
        Line Heights
      </h2>
      
      <TypographyExample
        label="Tight (1.25)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-tight)',
          color: 'var(--color-text-primary)'
        }}
        text="Tight line height is perfect for headings and titles where you want compact spacing. It works well for larger text sizes where readability isn't compromised by the reduced spacing."
      />
      
      <TypographyExample
        label="Normal (1.5)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)'
        }}
        text="Normal line height provides excellent readability for body text. It offers the right balance between compactness and breathing room, making it ideal for paragraphs and most UI text content."
      />
      
      <TypographyExample
        label="Relaxed (1.75)"
        style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-relaxed)',
          color: 'var(--color-text-primary)'
        }}
        text="Relaxed line height is great for long-form content where maximum readability is important. The extra spacing helps reduce fatigue when reading longer passages and can improve comprehension."
      />
    </div>
  ),
};

export const MonospaceFont: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '600px' }}>
      <h2 style={{ 
        marginBottom: '32px',
        fontSize: '24px',
        fontWeight: 600,
        color: 'var(--color-text-primary)'
      }}>
        Monospace Typography
      </h2>
      
      <p style={{
        marginBottom: '24px',
        fontSize: '14px',
        color: 'var(--color-text-secondary)',
        lineHeight: 1.6
      }}>
        Monospace fonts are used for code, URLs, configuration values, and other technical content that benefits from fixed-width characters.
      </p>
      
      <TypographyExample
        label="Inline Code"
        style={{
          fontSize: 'var(--font-size-sm)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-text-primary)',
          fontFamily: 'var(--font-family-mono)',
          backgroundColor: 'var(--color-surface-tertiary)',
          padding: '2px 6px',
          borderRadius: '4px'
        }}
        text="npm install @mindlink/ui"
      />
      
      <TypographyExample
        label="URL/Endpoint"
        style={{
          fontSize: 'var(--font-size-sm)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-normal)',
          color: 'var(--color-accent-primary)',
          fontFamily: 'var(--font-family-mono)'
        }}
        text="https://api.mindlink.dev/v1/status"
      />
      
      <TypographyExample
        label="Configuration"
        style={{
          fontSize: 'var(--font-size-sm)',
          fontWeight: 'var(--font-weight-regular)',
          lineHeight: 'var(--line-height-relaxed)',
          color: 'var(--color-text-primary)',
          fontFamily: 'var(--font-family-mono)',
          backgroundColor: 'var(--color-surface-tertiary)',
          padding: '16px',
          borderRadius: '8px',
          whiteSpace: 'pre' as const
        }}
        text={`{
  "tunnel": {
    "enabled": true,
    "domain": "*.trycloudflare.com",
    "port": 8080
  }
}`}
      />
    </div>
  ),
};

export const TypographyHierarchy: Story = {
  render: () => (
    <div style={{ padding: '24px', maxWidth: '800px' }}>
      <h1 style={{
        fontSize: 'var(--font-size-4xl)',
        fontWeight: 'var(--font-weight-bold)',
        lineHeight: 'var(--line-height-tight)',
        color: 'var(--color-text-primary)',
        marginBottom: '16px'
      }}>
        Typography Hierarchy Example
      </h1>
      
      <p style={{
        fontSize: 'var(--font-size-lg)',
        color: 'var(--color-text-secondary)',
        lineHeight: 'var(--line-height-normal)',
        marginBottom: '32px'
      }}>
        This demonstrates how different typography levels work together to create a clear visual hierarchy.
      </p>
      
      <h2 style={{
        fontSize: 'var(--font-size-2xl)',
        fontWeight: 'var(--font-weight-semibold)',
        lineHeight: 'var(--line-height-tight)',
        color: 'var(--color-text-primary)',
        marginBottom: '16px'
      }}>
        Section Heading
      </h2>
      
      <p style={{
        fontSize: 'var(--font-size-base)',
        color: 'var(--color-text-primary)',
        lineHeight: 'var(--line-height-normal)',
        marginBottom: '24px'
      }}>
        Regular body text provides the foundation of most content. It should be highly readable 
        and comfortable for extended reading sessions. The base font size and normal line height 
        work together to create optimal readability.
      </p>
      
      <h3 style={{
        fontSize: 'var(--font-size-xl)',
        fontWeight: 'var(--font-weight-medium)',
        lineHeight: 'var(--line-height-normal)',
        color: 'var(--color-text-primary)',
        marginBottom: '12px'
      }}>
        Subsection Heading
      </h3>
      
      <p style={{
        fontSize: 'var(--font-size-base)',
        color: 'var(--color-text-primary)',
        lineHeight: 'var(--line-height-normal)',
        marginBottom: '16px'
      }}>
        Code examples and technical content use monospace typography:
      </p>
      
      <div style={{
        backgroundColor: 'var(--color-surface-tertiary)',
        padding: '16px',
        borderRadius: '8px',
        marginBottom: '16px',
        fontSize: 'var(--font-size-sm)',
        fontFamily: 'var(--font-family-mono)',
        color: 'var(--color-text-primary)',
        lineHeight: 'var(--line-height-relaxed)'
      }}>
        const tunnel = await createTunnel(&#123;<br />
        &nbsp;&nbsp;domain: 'example.trycloudflare.com',<br />
        &nbsp;&nbsp;port: 8080<br />
        &#125;);
      </div>
      
      <p style={{
        fontSize: 'var(--font-size-sm)',
        color: 'var(--color-text-secondary)',
        lineHeight: 'var(--line-height-normal)',
        marginBottom: '24px'
      }}>
        Small text is used for captions, helper text, and secondary information that supports 
        the main content without competing for attention.
      </p>
      
      <div style={{
        padding: '16px',
        backgroundColor: 'var(--color-surface-card)',
        borderRadius: '8px',
        border: '1px solid var(--color-border-primary)'
      }}>
        <h4 style={{
          fontSize: 'var(--font-size-lg)',
          fontWeight: 'var(--font-weight-medium)',
          color: 'var(--color-text-primary)',
          marginBottom: '8px'
        }}>
          Typography Best Practices
        </h4>
        <ul style={{
          fontSize: 'var(--font-size-sm)',
          color: 'var(--color-text-secondary)',
          lineHeight: 'var(--line-height-normal)',
          paddingLeft: '20px',
          margin: 0
        }}>
          <li>Use appropriate font weights to create clear hierarchy</li>
          <li>Maintain consistent line heights within content blocks</li>
          <li>Reserve monospace fonts for technical content only</li>
          <li>Test readability across different screen sizes</li>
        </ul>
      </div>
    </div>
  ),
};