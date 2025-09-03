# MindLink Repository Guide for Claude Code

## Project Overview

**MindLink** is a production-ready desktop application built with **Rust/Tauri** that creates an OpenAI-compatible API server powered by ChatGPT Plus/Pro accounts. It provides secure Cloudflare tunneling for public API access.

### Key Architecture
- **Frontend**: React + TypeScript + Vite (port 1420)  
- **Backend**: Rust + Tauri + Axum web server
- **Build System**: Tauri v2 with cross-platform bundling
- **Development**: Modern TypeScript/React with Storybook

## Critical Development Information

### 🚨 File System Limitations
**IMPORTANT**: This project is stored on an external drive (`/media/nathan/Seagate Hub/Projects/mindlink/`) that **does not support symlinks**. This affects npm installation:

```bash
# ❌ Will fail with EPERM errors
npm install

# ✅ Required for this project
npm install --no-bin-links

# ✅ Run development server directly  
node node_modules/vite/bin/vite.js        # Frontend only
npm run tauri:dev                         # Full Tauri app
```

### Development Commands

```bash
# Frontend Development (React/Vite)
npm run dev                    # Vite dev server (port 1420)
node node_modules/vite/bin/vite.js --host  # Direct vite execution

# Tauri Development (Full App)
npm run tauri:dev             # Full desktop app with Rust backend
npm run start                 # Alias for tauri:dev

# Building
npm run build                 # Frontend build
npm run tauri:build          # Full Tauri build
npm run pack                  # Debug build
npm run dist                  # Distribution build

# Quality & Testing  
npm run lint                  # ESLint
npm run typecheck             # TypeScript checking
npm run storybook             # Component development (port 6006)

# Bifrost Management Scripts
npm run bifrost:build         # Build Bifrost bridge
npm run bifrost:test          # Test Bifrost integration
```

### Project Structure

```
mindlink/
├── src-tauri/                 # 🦀 Rust Backend (CORE LOGIC)
│   ├── src/
│   │   ├── main.rs           # App entry + system tray
│   │   ├── managers/         # Business logic modules
│   │   │   ├── auth_manager.rs    # ChatGPT OAuth2 flow
│   │   │   ├── server_manager.rs  # Axum API server  
│   │   │   ├── tunnel_manager.rs  # Cloudflare tunnels
│   │   │   ├── bifrost_manager.rs # Bridge management
│   │   │   └── config_manager.rs  # Settings management
│   │   ├── commands/         # Tauri IPC handlers
│   │   └── tests/           # Comprehensive test suite
│   ├── Cargo.toml           # Rust deps + STRICT linting
│   └── tauri.conf.json      # App configuration
├── src/                      # ⚛️ React Frontend  
│   ├── components/          # React components
│   ├── services/           # API client services
│   ├── plugins/            # Plugin system
│   └── design-system/      # UI components & tokens
├── .storybook/             # Component development
├── scripts/                # Build automation
└── docs/                   # Comprehensive documentation
```

## Tech Stack Details

### Frontend Stack
- **React 18.3** + **TypeScript 5.6**
- **Vite 6.0** (build tool)  
- **Storybook** (component development)
- **Design System** (custom tokens + components)
- **Tauri API** (@tauri-apps/api) for native integration

### Backend Stack  
- **Rust 2021** with **strict linting** (see Cargo.toml)
- **Tauri v2** (native desktop framework)
- **Axum 0.7** (async web server)
- **Tokio** (async runtime)
- **reqwest** (HTTP client)
- **serde** (serialization)

### Key Features
- **OpenAI-Compatible API** (`/v1/chat/completions`, `/v1/models`)
- **OAuth2 Authentication** with ChatGPT
- **Cloudflare Tunneling** for public access  
- **System Tray Integration** with native notifications
- **Bifrost Dashboard** (web management interface)
- **Enterprise Security** (encrypted credential storage)

## Development Workflow

### 1. Initial Setup
```bash
# Clone + navigate
cd "/media/nathan/Seagate Hub/Projects/mindlink"

# Install dependencies (symlink workaround required)
npm install --no-bin-links

# Install Rust dependencies  
cd src-tauri && cargo build
```

### 2. Development Modes

**Frontend Only** (React/Vite):
```bash
# Quick frontend iteration
node node_modules/vite/bin/vite.js
# Access: http://localhost:1420
```

**Full App Development** (Rust + React):
```bash  
# Complete desktop application
npm run tauri:dev
# Launches native app with hot-reload
```

### 3. Code Quality Standards

**Extremely Strict Rust Linting** (see `src-tauri/Cargo.toml`):
- `warnings = "deny"` - Zero tolerance for warnings
- `unsafe_code = "deny"` - Memory safety enforced
- `unwrap_used = "deny"` - Explicit error handling required  
- Clippy pedantic + nursery lints enabled
- **≥80% test coverage** required (`cargo tarpaulin`)

**TypeScript/React Standards**:
- ESLint with React hooks rules
- Strict TypeScript configuration  
- Component testing with Storybook
- Design system compliance

## Key Integration Points

### Tauri Commands (Rust ↔ Frontend)
Located in `src-tauri/src/commands/` - these handle frontend → backend communication:
- Authentication flows  
- Service management (start/stop)
- Configuration updates
- System status queries

### Bifrost Bridge System  
**Already Implemented** - sophisticated web dashboard:
- Real-time API monitoring
- Service health checks  
- Configuration management
- Performance analytics

### Plugin Architecture
Extensible system in `src/plugins/` for:
- AI provider integrations (OpenAI, Anthropic, Google)
- Dynamic plugin loading
- Provider registry management

## Testing Strategy

### Rust Backend Tests
```bash
cd src-tauri

# Unit tests
cargo test

# Integration tests  
cargo test --test '*'

# Coverage reporting (≥80% required)
cargo tarpaulin --config ci --out html
```

### Frontend Tests
```bash
# Component testing via Storybook
npm run storybook

# Type checking
npm run typecheck

# Linting  
npm run lint
```

## Configuration & Settings

### Application Configuration
- **Tauri Config**: `src-tauri/tauri.conf.json`
- **Vite Config**: `vite.config.ts` (includes Tauri-specific optimizations)
- **Rust Config**: `src-tauri/Cargo.toml` (with enterprise linting)

### Runtime Configuration  
- **Local Storage**: `~/.mindlink/` directory
- **Secure Storage**: OS-native credential storage
- **Ports**: 1420 (dev), 3001 (API), 3002 (Bifrost)

## Build & Distribution

### Development Builds
```bash
npm run pack                  # Debug build for testing
```

### Production Builds  
```bash
npm run dist                  # Full distribution packages
```

**Output Locations**:
- **Windows**: `src-tauri/target/release/bundle/msi/`
- **macOS**: `src-tauri/target/release/bundle/dmg/`  
- **Linux**: `src-tauri/target/release/bundle/deb/` + `appimage/`

## Security Considerations

### Enterprise-Grade Security
- **Memory Safety**: Rust backend eliminates memory vulnerabilities
- **Credential Storage**: OS-native secure storage (Keychain/Credential Manager)  
- **TLS 1.3**: All communication encrypted
- **Zero Data Collection**: Local-first architecture
- **Principle of Least Privilege**: Minimal system permissions

### Authentication Flow
1. **OAuth2/PKCE** with ChatGPT  
2. **Token Refresh** (automatic, 15min before expiry)
3. **Encrypted Storage** of credentials
4. **Session Validation** and recovery

## Common Issues & Solutions

### 1. Symlink Errors (npm install)
**Problem**: `EPERM: operation not permitted, symlink`  
**Solution**: Always use `npm install --no-bin-links`

### 2. Vite Not Found  
**Problem**: `sh: 1: vite: not found`
**Solution**: Run directly: `node node_modules/vite/bin/vite.js`

### 3. Tauri Dev Port Conflicts
**Problem**: Port 1420 in use
**Solution**: Configure in `vite.config.ts` server.port

### 4. Rust Compilation Errors
**Problem**: Strict linting failures
**Solution**: Fix ALL warnings - zero tolerance policy

## Documentation Resources

- **📖 [docs/README.md](docs/README.md)** - Complete documentation index
- **👤 [docs/USER_GUIDE.md](docs/USER_GUIDE.md)** - User installation & setup  
- **🔌 [docs/API.md](docs/API.md)** - OpenAI API compatibility
- **🏗️ [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System architecture  
- **💻 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** - Development workflows
- **🔧 [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Problem solving

## Development Tips

### Fast Iteration
- Use **frontend-only mode** (`vite`) for UI work
- Use **Storybook** for component development  
- **Rust tests** are fast - run frequently

### Code Quality  
- **Fix warnings immediately** - Rust build will fail otherwise
- **Test coverage** is tracked - maintain ≥80%
- **Security-first** - all inputs validated, no unsafe code

### Performance
- **Async-first** Rust backend (Tokio)
- **Modern React patterns** (hooks, suspense)
- **Bundle optimization** configured in Vite

---

## Quick Reference

**Start Development**: `npm run tauri:dev`  
**Frontend Only**: `node node_modules/vite/bin/vite.js`  
**Run Tests**: `cd src-tauri && cargo test`
**Build Production**: `npm run dist`
**Fix Dependencies**: `rm -rf node_modules && npm install --no-bin-links`

**Key Ports**: 1420 (dev), 3001 (API), 3002 (Bifrost), 6006 (Storybook)