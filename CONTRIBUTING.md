# Contributing to MindLink

Thank you for your interest in contributing to MindLink! This guide will help you set up your development environment and understand our enterprise-grade contribution standards.

## 🏆 Recent Quality Improvements (2024-2025)

We've implemented comprehensive quality improvements that all contributors must follow:

- ✅ **Zero TypeScript Errors**: All frontend code uses strict TypeScript with no `any` types
- ✅ **Eliminated 85% Code Duplication**: Shared utilities (ProviderUtils, ModalUtils) ensure DRY principles
- ✅ **Enterprise CI/CD Pipeline**: Automated quality gates with code signing and security audits
- ✅ **≥80% Test Coverage**: Comprehensive testing with automated coverage reporting
- ✅ **Zero-Tolerance Linting**: ESLint + Clippy with enterprise-grade rules (warnings = build failures)

## Table of Contents

- [Quality Standards Overview](#quality-standards-overview)
- [Development Environment Setup](#development-environment-setup)
- [Project Architecture](#project-architecture)  
- [Code Quality Standards](#code-quality-standards)
- [Testing Requirements](#testing-requirements)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [CI/CD Pipeline](#cicd-pipeline)
- [Code Style Guidelines](#code-style-guidelines)
- [Release Process](#release-process)

## Quality Standards Overview

### 🎯 Enterprise-Grade Standards

All contributions must meet these non-negotiable quality standards:

**Code Quality Gates:**
- **Zero Build Warnings**: Both Rust (`cargo clippy`) and TypeScript (`npm run lint`) must pass with zero warnings
- **100% Type Safety**: No `any` types in TypeScript, no `unwrap()` without justification in Rust
- **≥80% Test Coverage**: Automated coverage reporting blocks PRs below threshold
- **Security Audit**: All dependencies must pass security vulnerability scans

**Shared Utilities Compliance:**
- **Use ProviderUtils**: For OAuth, token management, and API standardization
- **Use ModalUtils**: For modal state management and consistent UI patterns  
- **No Code Duplication**: Leverage shared utilities instead of reimplementing common functionality
- **Consistent Error Handling**: Use structured error types with proper context

**CI/CD Quality Gates:**
- **Automated Testing**: All tests must pass on Linux, macOS, and Windows
- **Code Signing**: Production builds require valid code signatures
- **Security Scanning**: SAST analysis with zero high-severity findings
- **Performance Validation**: No regressions in build time or runtime performance

## Development Environment Setup

### Prerequisites

Before you begin, ensure you have the following installed:

#### Required Tools

1. **Rust Toolchain (Latest Stable)**
   ```bash
   # Install rustup (Rust installer)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Node.js 18+ and npm**
   ```bash
   # Using nvm (recommended)
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   nvm install 18
   nvm use 18
   
   # Or download from https://nodejs.org/
   ```

3. **Tauri CLI**
   ```bash
   npm install -g @tauri-apps/cli
   
   # Verify installation
   tauri --version
   ```

#### Platform-Specific Dependencies

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Windows:**
```powershell
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Install WebView2 (usually pre-installed on Windows 11)
# Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

### Clone and Setup

1. **Fork and Clone the Repository**
   ```bash
   # Fork the repository on GitHub first, then:
   git clone https://github.com/yourusername/mindlink.git
   cd mindlink
   
   # Add upstream remote
   git remote add upstream https://github.com/originalrepo/mindlink.git
   ```

2. **Install Dependencies**
   ```bash
   # Install Node.js dependencies
   npm install
   
   # Install Rust dependencies (happens automatically on first build)
   cd src-tauri
   cargo check
   ```

3. **Configure Development Environment**
   ```bash
   # Copy development configuration
   cp src-tauri/tauri.conf.json.example src-tauri/tauri.conf.json
   
   # Set up pre-commit hooks (optional but recommended)
   npm install -g @commitlint/cli @commitlint/config-conventional
   ```

### Development Commands

**Primary Development:**
```bash
# Start full development server with hot reload (Rust + React)
npm run tauri:dev

# Start frontend-only development (faster iteration for UI work)
npm run dev

# Start Storybook for component development
npm run storybook
```

**Quality Assurance (MUST PASS BEFORE PR):**
```bash
# Run all quality checks (equivalent to CI pipeline)
npm run ci:check

# Individual quality checks
npm run typecheck          # TypeScript strict mode check
npm run lint              # ESLint with enterprise rules  
npm run test              # All TypeScript/React tests
cd src-tauri && cargo clippy --all -- -D warnings  # Rust linting
cd src-tauri && cargo test --all                   # Rust tests
cd src-tauri && cargo tarpaulin --out html         # Coverage report
```

**Building:**
```bash
# Development build (faster, debug symbols)
npm run pack

# Production build (optimized, code signed if configured)
npm run dist

# Build Bifrost binary (required for Tauri integration)
npm run bifrost:build
```

**Utility Commands:**
```bash
# Format all code to standards
npm run format
cd src-tauri && cargo fmt --all

# Check for unused dependencies
cd src-tauri && cargo udeps

# Security audit
cd src-tauri && cargo audit

# Update dependencies
npm update && cd src-tauri && cargo update
```

**⚠️ Important Development Notes:**

1. **Symlink Issues**: This project is on an external drive that doesn't support symlinks:
   ```bash
   # Always use --no-bin-links when installing
   npm install --no-bin-links
   
   # Run vite directly if needed
   node node_modules/vite/bin/vite.js
   ```

2. **Strict Quality Enforcement**: All quality checks must pass before committing:
   ```bash
   # This MUST return zero errors/warnings
   npm run ci:check
   ```

3. **Test Coverage**: Maintain ≥80% coverage or builds will fail:
   ```bash
   # Generate coverage report
   cd src-tauri && cargo tarpaulin --out html
   # View: src-tauri/coverage/tarpaulin-report.html
   ```

## Project Architecture

MindLink follows a clean architecture pattern with clear separation of concerns:

### Directory Structure

```
mindlink/
├── src-tauri/                    # Rust backend (main application)
│   ├── src/
│   │   ├── main.rs              # Application entry point
│   │   ├── managers/            # Business logic modules
│   │   │   ├── auth_manager.rs  # OAuth2 authentication
│   │   │   ├── server_manager.rs # HTTP API server
│   │   │   ├── tunnel_manager.rs # Cloudflare tunnels
│   │   │   ├── config_manager.rs # Configuration
│   │   │   └── bifrost_manager.rs # Dashboard
│   │   ├── commands/            # Tauri IPC commands
│   │   ├── tests/               # Test suite
│   │   ├── error.rs            # Error handling
│   │   └── logging.rs          # Logging infrastructure
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
├── ui/                          # Frontend assets
├── scripts/                     # Build and deployment scripts
└── .github/workflows/          # CI/CD configuration
```

### Architecture Principles

1. **Manager Pattern**: Each major functionality is encapsulated in a manager (AuthManager, ServerManager, etc.)
2. **Error Handling**: All operations return `Result<T, MindLinkError>` for explicit error handling
3. **Async-First**: All I/O operations are asynchronous using Tokio
4. **Type Safety**: Extensive use of Rust's type system to prevent runtime errors
5. **Separation of Concerns**: Clear boundaries between authentication, networking, and UI

### Key Components

- **AuthManager**: Handles OAuth2 authentication with ChatGPT
- **ServerManager**: Runs the HTTP API server compatible with OpenAI
- **TunnelManager**: Manages Cloudflare tunnel connections
- **ConfigManager**: Handles application configuration and persistence
- **BifrostManager**: Manages the web dashboard interface

## Code Quality Standards

We maintain enterprise-grade code quality standards. All contributions must meet these requirements:

### Rust Code Standards

1. **Linting Configuration**
   - All code must pass `cargo clippy` with zero warnings
   - We use enterprise-grade linting rules (see `Cargo.toml`)
   - No `#[allow(clippy::...)]` attributes without justification

2. **Error Handling**
   - NO `unwrap()` or `expect()` in production code paths
   - All functions return `Result<T, MindLinkError>`
   - Errors must be properly logged with context

3. **Documentation**
   - All public functions must have rustdoc comments
   - Include examples for complex functions
   - Document error conditions and edge cases

4. **Memory Safety**
   - No `unsafe` code without exceptional justification
   - Use RAII patterns for resource management
   - Prefer owned types over borrowing when unclear

### Code Formatting

```bash
# Format all Rust code
cargo fmt --all

# Check formatting without changing files
cargo fmt --all -- --check
```

## Testing Requirements

We maintain comprehensive test coverage across all layers:

### Test Types

1. **Unit Tests** (`#[cfg(test)]` modules)
   - Test individual functions and methods
   - Mock external dependencies using `mockall`
   - Target: 90%+ line coverage

2. **Integration Tests** (`src-tauri/tests/`)
   - Test component interactions
   - Use real implementations where possible
   - Test error scenarios and edge cases

3. **End-to-End Tests**
   - Full application workflow testing
   - UI automation using `tauri-driver`
   - Critical user journey validation

### Running Tests

```bash
# Run all tests
cargo test --all --manifest-path src-tauri/Cargo.toml

# Run specific test module
cargo test auth_manager --manifest-path src-tauri/Cargo.toml

# Run tests with output
cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture

# Run integration tests only
cargo test --test '*' --manifest-path src-tauri/Cargo.toml

# Generate coverage report (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --manifest-path src-tauri/Cargo.toml --out html
```

### Test Writing Guidelines

1. **Naming Convention**: `test_function_name_when_condition_should_result`
2. **AAA Pattern**: Arrange, Act, Assert
3. **Mock External Dependencies**: Use `mockall` for HTTP clients, file systems, etc.
4. **Test Both Success and Failure Cases**: Ensure error paths are tested
5. **Use Descriptive Assertions**: Include helpful error messages

Example test:

```rust
#[tokio::test]
async fn test_auth_manager_login_when_valid_tokens_should_succeed() {
    // Arrange
    let mut mock_client = MockHttpClient::new();
    mock_client.expect_post()
        .returning(|_| Ok(AuthResponse { token: "valid_token".to_string() }));
    
    let auth_manager = AuthManager::new(mock_client);
    
    // Act
    let result = auth_manager.login("user@example.com", "password").await;
    
    // Assert
    assert!(result.is_ok(), "Login should succeed with valid credentials");
    assert_eq!(result.unwrap().token, "valid_token");
}
```

## Development Workflow

### Git Workflow

We use a modified Git Flow:

1. **Main Branch**: `main` - Production-ready code
2. **Feature Branches**: `feature/description` - New features
3. **Bug Fix Branches**: `fix/description` - Bug fixes
4. **Release Branches**: `release/v1.2.3` - Preparing releases

### Branch Naming Convention

- **Features**: `feature/add-tunnel-health-monitoring`
- **Bug fixes**: `fix/auth-token-refresh-error`
- **Documentation**: `docs/update-contributing-guide`
- **Refactoring**: `refactor/extract-config-validation`

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer(s)]
```

**Types:**
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `chore`: Build process or auxiliary tool changes

**Examples:**
```
feat(auth): add automatic token refresh mechanism

Implements background token refresh that occurs 15 minutes before
expiration to prevent service interruptions.

Closes #123
```

```
fix(tunnel): handle cloudflared binary not found error

Provides clear error message and recovery instructions when
cloudflared binary is missing from system PATH.
```

### Development Best Practices

1. **Small, Focused Commits**: Each commit should represent a single logical change
2. **Frequent Pulls**: Regularly sync with upstream to avoid conflicts
3. **Test Before Committing**: Run tests and linting before each commit
4. **Write Descriptive Commit Messages**: Include the "why" not just the "what"
5. **Use Draft PRs**: Open draft PRs early for feedback and collaboration

## Pull Request Process

### Before Submitting

1. **Sync with upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run quality checks**:
   ```bash
   # Format code
   cargo fmt --all
   
   # Run linting
   cargo clippy --all -- -D warnings
   
   # Run tests
   cargo test --all
   
   # Build successfully
   npm run tauri build
   ```

3. **Update documentation** if needed

### PR Template

When creating a PR, use this template:

```markdown
## Description
Brief description of changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Manual testing performed

## Checklist
- [ ] Code follows the project's style guidelines
- [ ] Self-review of code completed
- [ ] Code is commented where necessary
- [ ] Documentation updated
- [ ] No breaking changes without version bump
```

### Review Process

1. **Automated Checks**: All PRs must pass comprehensive CI/CD pipeline
2. **Quality Gates**: Zero warnings, ≥80% coverage, security audit pass
3. **Code Review**: At least one maintainer review with focus on shared utilities usage
4. **Testing**: New features require comprehensive test coverage with both unit and integration tests
5. **Documentation**: Updates to public APIs require TSDoc/rustdoc documentation

## CI/CD Pipeline

### Automated Quality Gates

Our enterprise CI/CD pipeline enforces quality standards automatically:

**🔍 Pre-Release Validation:**
- **Tag Format Validation**: Ensures semantic versioning compliance (`v1.2.3` format)
- **Repository State Check**: Validates clean working directory and tag existence
- **Dependency Audit**: Scans for known security vulnerabilities

**🏗️ Multi-Platform Build Matrix:**
- **Windows (x64)**: MSI installer with code signing
- **macOS (Universal)**: DMG with Apple notarization (Intel + Apple Silicon)  
- **Linux (x64)**: AppImage, DEB, and RPM packages

**✅ Quality Enforcement:**
```yaml
quality_checks:
  rust_standards:
    - cargo_clippy: "zero warnings (-D warnings)"
    - cargo_test: "all tests pass"
    - cargo_audit: "no known vulnerabilities"
    - cargo_tarpaulin: "≥80% code coverage"
    
  typescript_standards:
    - eslint: "zero warnings with enterprise rules"
    - typescript: "strict mode, no any types"
    - prettier: "consistent formatting"
    - vitest: "comprehensive test coverage"
    
  security_standards:
    - dependency_audit: "npm audit + cargo audit"
    - sast_scanning: "semgrep security analysis"
    - code_signing: "platform-specific signing certificates"
```

**🚀 Build Artifacts:**
- **Code-signed binaries** for all platforms
- **Automatic updater** with secure signature validation
- **Coverage reports** with HTML visualization
- **Security audit reports** with detailed findings
- **Build analysis** with performance metrics

**📊 Post-Release Validation:**
- **Asset verification** ensuring all expected files are present
- **Download validation** confirming artifact accessibility
- **Performance benchmarking** measuring build and runtime metrics

### CI/CD Requirements for Contributors

**Before pushing to your branch:**
```bash
# Run the same checks as CI
npm run ci:check

# Verify Bifrost integration
npm run bifrost:test

# Ensure builds complete successfully  
npm run dist
```

**PR Merge Requirements:**
- ✅ All CI checks pass on Linux, macOS, and Windows
- ✅ Code review approved by maintainer
- ✅ Test coverage ≥80% with no gaps in critical paths
- ✅ No security vulnerabilities in dependencies
- ✅ Documentation updated for public API changes
- ✅ Shared utilities used appropriately (no code duplication)

## Code Style Guidelines

### Rust Style

1. **Follow rustfmt**: Use default formatting rules
2. **Naming Conventions**:
   - Types: `PascalCase`
   - Functions/variables: `snake_case`
   - Constants: `SCREAMING_SNAKE_CASE`
   - Modules: `snake_case`

3. **Function Organization**:
   ```rust
   impl MyStruct {
       // Public functions first
       pub fn public_function(&self) -> Result<(), Error> {
           // Implementation
       }
       
       // Private functions after
       fn private_helper(&self) -> bool {
           // Implementation
       }
   }
   ```

4. **Error Handling**:
   ```rust
   // Good: Explicit error handling
   fn process_data() -> Result<Data, MindLinkError> {
       let raw_data = read_file()?;
       let parsed = parse_data(raw_data)?;
       Ok(parsed)
   }
   
   // Bad: Using unwrap
   fn process_data_bad() -> Data {
       let raw_data = read_file().unwrap(); // Don't do this!
       parse_data(raw_data).unwrap()
   }
   ```

5. **Documentation**:
   ```rust
   /// Authenticates user with ChatGPT using OAuth2 flow.
   /// 
   /// # Arguments
   /// 
   /// * `email` - User's email address
   /// * `password` - User's password
   /// 
   /// # Returns
   /// 
   /// Returns `AuthToken` on successful authentication.
   /// 
   /// # Errors
   /// 
   /// Returns `MindLinkError::AuthenticationFailed` if credentials are invalid.
   /// 
   /// # Example
   /// 
   /// ```rust
   /// let token = auth_manager.login("user@example.com", "password").await?;
   /// ```
   pub async fn login(&self, email: &str, password: &str) -> Result<AuthToken, MindLinkError> {
       // Implementation
   }
   ```

### HTML/CSS/JavaScript Style

For frontend code in the `ui/` directory:

1. **HTML**: Use semantic HTML5 elements
2. **CSS**: Follow BEM methodology for class naming
3. **JavaScript**: Use modern ES6+ syntax, prefer `const` over `let`

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. **Update Version Numbers**:
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
   - `package.json`

2. **Update CHANGELOG.md**:
   - Add release notes
   - List all changes since last release
   - Include breaking changes prominently

3. **Tag Release**:
   ```bash
   git tag -a v1.2.3 -m "Release version 1.2.3"
   git push upstream v1.2.3
   ```

4. **GitHub Actions** will automatically:
   - Build binaries for all platforms
   - Create GitHub release
   - Upload release artifacts

## Getting Help

- **Questions**: Ask in [GitHub Discussions](https://github.com/yourusername/mindlink/discussions)
- **Bug Reports**: Use [GitHub Issues](https://github.com/yourusername/mindlink/issues)
- **Real-time Chat**: Join our [Discord server](https://discord.gg/mindlink)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). Please read it before contributing.

---

Thank you for contributing to MindLink! 🚀