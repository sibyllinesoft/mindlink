# MindLink Frontend

Modern React + TypeScript + Vite frontend for the MindLink Tauri application.

## Architecture

- **Framework**: React 18 with TypeScript
- **Build Tool**: Vite 6 (fast HMR and optimized builds)
- **Styling**: Modern CSS with CSS Grid and Flexbox
- **Integration**: Tauri API for native functionality
- **Code Quality**: ESLint + TypeScript strict mode

## Components Structure

```
src/
├── components/
│   ├── Dashboard.tsx          # Main dashboard view
│   ├── Settings.tsx           # Settings configuration
│   ├── ServerControl.tsx      # Local server controls
│   ├── TunnelControl.tsx      # Cloudflare tunnel controls  
│   ├── APIInformation.tsx     # API endpoints and examples
│   └── StatusBar.tsx          # Bottom status indicator
├── App.tsx                    # Main application component
├── main.tsx                   # React entry point
└── vite-env.d.ts             # TypeScript declarations
```

## Features

- **Real-time Status Updates**: WebSocket-like event system via Tauri
- **Responsive Design**: Mobile-friendly responsive layout
- **Modern UX**: Smooth animations and visual feedback
- **Accessible**: WCAG compliant interface
- **Type Safe**: 100% TypeScript with strict configuration

## Development

```bash
# Start development server
npm run dev

# Build for production
npm run build

# Type checking
npm run typecheck

# Linting
npm run lint

# Start Tauri dev (includes frontend)
npm start
```

## Integration with Rust Backend

The frontend communicates with the Rust backend through:

- **Tauri Commands**: Direct function calls to Rust
- **Event System**: Real-time updates from backend
- **State Management**: React state synchronized with backend

### Backend Commands Used

- `start_server()` - Start the local API server
- `stop_server()` - Stop the local API server
- `start_tunnel()` - Create Cloudflare tunnel
- `stop_tunnel()` - Destroy Cloudflare tunnel
- `get_config()` - Retrieve current configuration
- `update_config(config)` - Update configuration
- `authenticate_chatgpt()` - Browser-based authentication

### Event Listeners

- `server-status-changed` - Server state updates
- `tunnel-status-changed` - Tunnel state updates  
- `public-url-changed` - Public URL updates
- `auth-status-changed` - Authentication status

## Build Process

1. **Development**: Vite dev server runs on localhost:1420
2. **Build**: TypeScript compilation + Vite bundling
3. **Output**: Static files in `dist/` directory
4. **Tauri**: Rust app serves files from `dist/`

## Performance Features

- **Code Splitting**: Vendor chunks for better caching
- **Modern Targets**: ES2022+ for smaller bundles
- **Optimized CSS**: Modern CSS features and custom properties
- **Fast Refresh**: Vite HMR for instant updates