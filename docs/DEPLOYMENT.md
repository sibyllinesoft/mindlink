# MindLink Deployment Guide

## Table of Contents

1. [Overview](#overview)
2. [Pre-Deployment Checklist](#pre-deployment-checklist)
3. [Build Process](#build-process)
4. [Platform-Specific Deployment](#platform-specific-deployment)
5. [Distribution Strategies](#distribution-strategies)
6. [Security and Code Signing](#security-and-code-signing)
7. [Release Management](#release-management)
8. [Auto-Updates](#auto-updates)
9. [Monitoring and Analytics](#monitoring-and-analytics)
10. [Rollback Procedures](#rollback-procedures)

## Overview

MindLink is deployed as a native desktop application across Windows, macOS, and Linux platforms. The deployment process involves building optimized binaries, code signing for security, and distributing through multiple channels including GitHub Releases and platform-specific package managers.

### Deployment Targets

| Platform | Architectures | Package Formats | Distribution |
|----------|---------------|-----------------|--------------|
| **Windows** | x86_64 | MSI, EXE | GitHub Releases, Microsoft Store (planned) |
| **macOS** | x86_64, arm64 (Apple Silicon) | DMG, APP | GitHub Releases, Mac App Store (planned) |
| **Linux** | x86_64, arm64 | DEB, RPM, AppImage, Snap | GitHub Releases, Package Repositories |

### Deployment Architecture

```
Developer Machine
       ↓
   Git Repository
       ↓
  GitHub Actions CI/CD
       ↓
┌─────────────────────────────────────┐
│     Multi-Platform Build Matrix    │
├─────────────────────────────────────┤
│ Windows x86_64 → MSI, EXE          │
│ macOS x86_64   → DMG, APP          │
│ macOS arm64    → DMG, APP          │  
│ Linux x86_64   → DEB, RPM, AppImage│
│ Linux arm64    → DEB, RPM, AppImage│
└─────────────────────────────────────┘
       ↓
   Code Signing & Notarization
       ↓
   Package Verification & Testing
       ↓
   Distribution Channels
       ↓
┌─────────────────────────────────────┐
│     End User Installation          │
├─────────────────────────────────────┤
│ • GitHub Releases (Primary)        │
│ • Package Managers (Linux)         │
│ • App Stores (Future)              │
│ • Auto-Updates (Existing Users)    │
└─────────────────────────────────────┘
```

## Pre-Deployment Checklist

### Code Quality Gates

Before any deployment, ensure all quality gates pass:

```bash
#!/bin/bash
# scripts/pre-deployment-check.sh

set -e

echo "🔍 Running pre-deployment checklist..."

# 1. Code formatting
echo "📐 Checking code formatting..."
cd src-tauri
cargo fmt --check
cd ..
npm run lint

# 2. Static analysis
echo "🔬 Running static analysis..."
cd src-tauri
cargo clippy --all-targets --all-features -- -D warnings
cd ..

# 3. Security audit
echo "🛡️ Running security audit..."
cd src-tauri
cargo audit --deny warnings
cd ..
npm audit --audit-level moderate

# 4. Test suite
echo "🧪 Running comprehensive test suite..."
cd src-tauri
cargo test --all-features --release
cd ..

# 5. Coverage check
echo "📊 Checking test coverage..."
cd src-tauri
cargo tarpaulin --skip-clean --out Xml --all-features
COVERAGE=$(grep -o 'line-rate="[^"]*"' cobertura.xml | head -1 | grep -o '[0-9.]*')
if (( $(echo "$COVERAGE < 0.80" | bc -l) )); then
    echo "❌ Coverage too low: $COVERAGE (minimum: 0.80)"
    exit 1
fi
echo "✅ Coverage: $COVERAGE"
cd ..

# 6. Performance benchmarks
echo "⚡ Running performance benchmarks..."
cd src-tauri
cargo bench --bench auth_benchmarks
cargo bench --bench server_benchmarks
cd ..

# 7. Build verification
echo "🏗️ Verifying builds..."
npm run build
npm run tauri build --debug

echo "✅ Pre-deployment checklist passed!"
```

### Version Management

```bash
#!/bin/bash
# scripts/version-check.sh

# Verify version consistency across all files
CARGO_VERSION=$(grep '^version = ' src-tauri/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
PACKAGE_VERSION=$(node -p "require('./package.json').version")
TAURI_VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version")

if [[ "$CARGO_VERSION" != "$PACKAGE_VERSION" ]] || [[ "$PACKAGE_VERSION" != "$TAURI_VERSION" ]]; then
    echo "❌ Version mismatch detected:"
    echo "   Cargo.toml:     $CARGO_VERSION"
    echo "   package.json:   $PACKAGE_VERSION"
    echo "   tauri.conf.json: $TAURI_VERSION"
    exit 1
fi

echo "✅ Version consistency verified: $CARGO_VERSION"
```

### Dependency Verification

```bash
#!/bin/bash
# scripts/dependency-check.sh

echo "📦 Verifying dependencies..."

# Check for security vulnerabilities
cd src-tauri
cargo audit --deny warnings
cd ..

# Check for unused dependencies
cd src-tauri
cargo udeps --all-targets --all-features
cd ..

# Verify lockfile integrity
cd src-tauri
cargo verify-lockfile
cd ..
npm ci --audit

echo "✅ Dependencies verified"
```

## Build Process

### Local Build Environment

#### Prerequisites Installation

**Automated Setup Script:**
```bash
#!/bin/bash
# scripts/setup-build-env.sh

set -e

echo "🔧 Setting up build environment..."

# Detect platform
PLATFORM=$(uname -s)
ARCH=$(uname -m)

case $PLATFORM in
    "Linux")
        echo "🐧 Setting up Linux build environment..."
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y \
                build-essential \
                libwebkit2gtk-4.0-dev \
                libssl-dev \
                libgtk-3-dev \
                libayatana-appindicator3-dev \
                librsvg2-dev \
                curl \
                wget \
                file
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y \
                gcc \
                gcc-c++ \
                webkit2gtk3-devel \
                openssl-devel \
                gtk3-devel \
                libappindicator-gtk3-devel \
                librsvg2-devel
        fi
        ;;
    "Darwin")
        echo "🍎 Setting up macOS build environment..."
        if ! command -v xcode-select &> /dev/null; then
            echo "Installing Xcode Command Line Tools..."
            xcode-select --install
        fi
        ;;
    "MINGW"*|"MSYS"*)
        echo "🪟 Setting up Windows build environment..."
        echo "Please ensure Visual Studio Build Tools are installed"
        echo "Download: https://visualstudio.microsoft.com/downloads/"
        ;;
esac

# Install Rust
if ! command -v rustc &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install Node.js
if ! command -v node &> /dev/null; then
    echo "📦 Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi

# Install Tauri CLI
if ! command -v cargo-tauri &> /dev/null; then
    echo "🚀 Installing Tauri CLI..."
    cargo install tauri-cli
fi

# Verify installation
echo "✅ Build environment setup complete!"
echo "Versions:"
echo "  Rust: $(rustc --version)"
echo "  Node: $(node --version)"
echo "  Tauri: $(cargo tauri --version)"
```

#### Build Configuration

**Development Build:**
```bash
#!/bin/bash
# scripts/build-dev.sh

export RUST_LOG=debug
export NODE_ENV=development

echo "🛠️ Building development version..."

# Install dependencies
npm ci
cd src-tauri && cargo check && cd ..

# Build with debug symbols and hot reload support
npm run tauri build -- --debug --verbose

echo "✅ Development build complete!"
echo "📍 Location: src-tauri/target/debug/bundle/"
```

**Production Build:**
```bash
#!/bin/bash
# scripts/build-production.sh

export NODE_ENV=production
export RUST_BACKTRACE=0

echo "🏗️ Building production version..."

# Clean build
rm -rf dist/ src-tauri/target/release/bundle/

# Install dependencies with exact versions
npm ci --production=false

# Build optimized frontend
npm run build

# Build optimized Tauri app with maximum optimization
RUSTFLAGS="-C target-cpu=native" npm run tauri build -- --verbose

echo "✅ Production build complete!"
echo "📍 Location: src-tauri/target/release/bundle/"
```

### Cross-Platform Build Matrix

#### GitHub Actions Build Pipeline

```yaml
# .github/workflows/build-release.yml
name: Build Release Packages

on:
  push:
    tags: ['v*']
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to build (e.g., v1.2.3)'
        required: true

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          # Windows builds
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            platform: win32
            arch: x64
            
          # macOS builds
          - os: macos-latest
            target: x86_64-apple-darwin
            platform: darwin
            arch: x64
            
          - os: macos-latest
            target: aarch64-apple-darwin
            platform: darwin
            arch: arm64
            
          # Linux builds
          - os: ubuntu-20.04
            target: x86_64-unknown-linux-gnu
            platform: linux
            arch: x64
            
          - os: ubuntu-20.04
            target: aarch64-unknown-linux-gnu
            platform: linux
            arch: arm64

    runs-on: ${{ matrix.os }}
    
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
          components: rustfmt, clippy

      - name: Setup Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'
          key: ${{ matrix.target }}

      - name: Install system dependencies (Linux)
        if: matrix.platform == 'linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            build-essential \
            libwebkit2gtk-4.0-dev \
            libssl-dev \
            libgtk-3-dev \
            libayatana-appindicator3-dev \
            librsvg2-dev

      - name: Install cross-compilation tools (Linux ARM64)
        if: matrix.platform == 'linux' && matrix.arch == 'arm64'
        run: |
          sudo apt-get install -y gcc-aarch64-linux-gnu
          echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> $GITHUB_ENV

      - name: Install frontend dependencies
        run: npm ci

      - name: Run pre-build checks
        run: |
          npm run lint
          cd src-tauri
          cargo fmt --check
          cargo clippy --target ${{ matrix.target }} --all-features -- -D warnings

      - name: Run tests
        if: matrix.arch == 'x64'  # Skip tests for cross-compiled ARM64 builds
        run: |
          cd src-tauri
          cargo test --target ${{ matrix.target }} --all-features

      - name: Build application
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          # macOS signing
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: ${{ github.ref_name || github.event.inputs.version }}
          releaseName: 'MindLink ${{ github.ref_name || github.event.inputs.version }}'
          releaseBody: |
            ## 🚀 MindLink ${{ github.ref_name || github.event.inputs.version }}
            
            ### 📦 Downloads
            Choose the appropriate package for your platform:
            
            - **Windows**: `MindLink_*_x64_en-US.msi` or `MindLink_*_x64-setup.exe`
            - **macOS Intel**: `MindLink_*_x64.dmg` 
            - **macOS Apple Silicon**: `MindLink_*_aarch64.dmg`
            - **Linux x64**: `mindlink_*_amd64.deb`, `mindlink_*_amd64.rpm`, or `mindlink-*_amd64.AppImage`
            - **Linux ARM64**: `mindlink_*_arm64.deb`, `mindlink_*_aarch64.rpm`, or `mindlink-*_arm64.AppImage`
            
            ### 🔐 Verification
            All packages are cryptographically signed. Verify signatures before installation.
            
            ### 📋 System Requirements
            - **ChatGPT Plus or Pro subscription** (required)
            - **Windows**: Windows 10 or later
            - **macOS**: macOS 10.15 (Catalina) or later  
            - **Linux**: Ubuntu 18.04+ or equivalent
            
            See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.
          releaseDraft: true
          prerelease: false
          args: --target ${{ matrix.target }} --verbose

      - name: Calculate checksums
        shell: bash
        run: |
          cd src-tauri/target/${{ matrix.target }}/release/bundle/
          find . -type f \( -name "*.msi" -o -name "*.exe" -o -name "*.dmg" -o -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" \) -exec sha256sum {} \; > checksums-${{ matrix.target }}.txt

      - name: Upload checksums
        uses: actions/upload-artifact@v3
        with:
          name: checksums-${{ matrix.target }}
          path: src-tauri/target/${{ matrix.target }}/release/bundle/checksums-${{ matrix.target }}.txt
```

### Build Optimization

#### Rust Build Optimization

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-Wl,-dead_strip"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-Wl,-dead_strip"]

# Global optimization settings
[build]
jobs = 4  # Adjust based on CI runner capacity

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = 'abort'
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false
```

#### Frontend Build Optimization

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react-swc';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  
  // Build optimization
  build: {
    target: 'es2020',
    minify: 'terser',
    cssMinify: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom'],
          'tauri': ['@tauri-apps/api'],
        },
      },
    },
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
        pure_funcs: ['console.log', 'console.debug'],
      },
    },
  },
  
  // Development optimization
  server: {
    port: 1420,
    strictPort: true,
  },
  
  // Bundle analysis
  define: {
    __BUILD_VERSION__: JSON.stringify(process.env.npm_package_version),
    __BUILD_DATE__: JSON.stringify(new Date().toISOString()),
  },
});
```

## Platform-Specific Deployment

### Windows Deployment

#### MSI Package Creation

```toml
# src-tauri/tauri.conf.json (Windows section)
{
  "bundle": {
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256", 
      "timestampUrl": "",
      "wix": {
        "language": ["en-US"],
        "template": "templates/installer.wxs",
        "fragmentPaths": ["templates/fragments.wxs"],
        "componentRefs": ["MindLinkService"],
        "featureRefs": ["Complete"],
        "mergeRefs": [],
        "skipWebView": false,
        "enableElevatedUpdateTask": true
      },
      "allowDowngrades": false,
      "installMode": "perMachine"
    }
  }
}
```

#### Custom MSI Template

```xml
<!-- templates/installer.wxs -->
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" 
           Name="MindLink" 
           Language="1033" 
           Version="$(var.Version)" 
           Manufacturer="MindLink Team" 
           UpgradeCode="12345678-1234-1234-1234-123456789012">
    
    <Package InstallerVersion="200" 
             Compressed="yes" 
             InstallScope="perMachine"
             Description="Local LLM API bridge with Cloudflare tunneling" />

    <!-- Major upgrade rules -->
    <MajorUpgrade Schedule="afterInstallInitialize" 
                  DowngradeErrorMessage="A newer version of MindLink is already installed." 
                  AllowSameVersionUpgrades="yes" />

    <!-- Media -->
    <Media Id="1" Cabinet="MindLink.cab" EmbedCab="yes" />

    <!-- Directory structure -->
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <Directory Id="INSTALLDIR" Name="MindLink">
          <Component Id="MindLinkExecutable" Guid="*">
            <File Id="MindLinkExe" Source="$(var.SourceDir)\MindLink.exe" KeyPath="yes">
              <Shortcut Id="StartMenuShortcut"
                       Directory="ProgramMenuFolder"
                       Name="MindLink"
                       Description="Local LLM API bridge"
                       WorkingDirectory="INSTALLDIR" />
              <Shortcut Id="DesktopShortcut"
                       Directory="DesktopFolder" 
                       Name="MindLink"
                       Description="Local LLM API bridge"
                       WorkingDirectory="INSTALLDIR" />
            </File>
          </Component>
          
          <!-- Service component for auto-start -->
          <Component Id="MindLinkService" Guid="*">
            <RegistryKey Root="HKCU" Key="Software\Microsoft\Windows\CurrentVersion\Run">
              <RegistryValue Type="string" Name="MindLink" Value="&quot;[INSTALLDIR]MindLink.exe&quot; --minimized" />
            </RegistryKey>
          </Component>
        </Directory>
      </Directory>
      
      <Directory Id="ProgramMenuFolder" />
      <Directory Id="DesktopFolder" />
    </Directory>

    <!-- Features -->
    <Feature Id="Complete" Title="MindLink" Level="1">
      <ComponentRef Id="MindLinkExecutable" />
      <ComponentRef Id="MindLinkService" />
    </Feature>
  </Product>
</Wix>
```

#### Code Signing Script

```powershell
# scripts/sign-windows.ps1

param(
    [Parameter(Mandatory=$true)]
    [string]$CertificatePath,
    
    [Parameter(Mandatory=$true)]
    [string]$CertificatePassword,
    
    [Parameter(Mandatory=$true)]
    [string]$PackagePath
)

Write-Host "🔐 Signing Windows package: $PackagePath"

# Sign the package
$signResult = & signtool sign `
    /f $CertificatePath `
    /p $CertificatePassword `
    /t http://timestamp.digicert.com `
    /fd sha256 `
    /d "MindLink - Local LLM API Bridge" `
    /du "https://github.com/yourusername/mindlink" `
    $PackagePath

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Package signed successfully"
    
    # Verify signature
    Write-Host "🔍 Verifying signature..."
    & signtool verify /pa $PackagePath
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Signature verified"
    } else {
        Write-Error "❌ Signature verification failed"
        exit 1
    }
} else {
    Write-Error "❌ Code signing failed"
    exit 1
}
```

### macOS Deployment

#### App Bundle Configuration

```json
{
  "bundle": {
    "macOS": {
      "frameworks": [],
      "minimumSystemVersion": "10.15",
      "exceptionDomain": "",
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "providerShortName": "TEAM_ID",
      "entitlements": "entitlements.plist",
      "hardenedRuntime": true,
      "dmg": {
        "appPosition": {
          "x": 180,
          "y": 170
        },
        "applicationFolderPosition": {
          "x": 480,
          "y": 170
        },
        "windowSize": {
          "width": 660,
          "height": 400
        }
      }
    }
  }
}
```

#### Entitlements Configuration

```xml
<!-- entitlements.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.automation.apple-events</key>
  <true/>
  <key>com.apple.security.network.client</key>
  <true/>
  <key>com.apple.security.network.server</key>
  <true/>
  <key>com.apple.security.files.user-selected.read-write</key>
  <true/>
  <key>com.apple.security.cs.allow-jit</key>
  <false/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
  <false/>
  <key>com.apple.security.cs.allow-dyld-environment-variables</key>
  <false/>
</dict>
</plist>
```

#### macOS Build and Notarization

```bash
#!/bin/bash
# scripts/build-and-notarize-macos.sh

set -e

APP_NAME="MindLink"
BUNDLE_ID="com.mindlink.mindlink"
DEVELOPER_ID="Developer ID Application: Your Name (TEAM_ID)"
APPLE_ID="${APPLE_ID}"
APP_PASSWORD="${APPLE_PASSWORD}"
TEAM_ID="${APPLE_TEAM_ID}"

echo "🍎 Building and notarizing macOS application..."

# Build the application
npm run tauri build -- --target universal-apple-darwin

APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos/${APP_NAME}.app"
DMG_PATH="src-tauri/target/universal-apple-darwin/release/bundle/dmg/${APP_NAME}_*.dmg"

# Sign the app bundle
echo "🔐 Signing application bundle..."
codesign --force --options runtime --deep --sign "$DEVELOPER_ID" "$APP_PATH"

# Verify signature
codesign --verify --deep --strict "$APP_PATH"
spctl --assess --verbose "$APP_PATH"

# Create and sign DMG
echo "📦 Creating DMG..."
# DMG is created by Tauri automatically

# Sign DMG
DMG_FILE=$(ls $DMG_PATH | head -1)
codesign --force --sign "$DEVELOPER_ID" "$DMG_FILE"

# Notarize the DMG
echo "📋 Submitting for notarization..."
UPLOAD_RESULT=$(xcrun notarytool submit "$DMG_FILE" \
    --apple-id "$APPLE_ID" \
    --password "$APP_PASSWORD" \
    --team-id "$TEAM_ID" \
    --wait)

if echo "$UPLOAD_RESULT" | grep -q "status: Accepted"; then
    echo "✅ Notarization successful"
    
    # Staple the notarization to the DMG
    echo "📎 Stapling notarization..."
    xcrun stapler staple "$DMG_FILE"
    
    # Verify notarization
    xcrun stapler validate "$DMG_FILE"
    spctl --assess --type open --context context:primary-signature -v "$DMG_FILE"
    
    echo "✅ macOS deployment package ready: $DMG_FILE"
else
    echo "❌ Notarization failed"
    echo "$UPLOAD_RESULT"
    exit 1
fi
```

### Linux Deployment

#### Multi-Package Build

```bash
#!/bin/bash
# scripts/build-linux-packages.sh

set -e

VERSION=$(grep '^version = ' src-tauri/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
ARCH=$(uname -m)

echo "🐧 Building Linux packages for version $VERSION on $ARCH..."

# Build the application
npm run tauri build

BUNDLE_DIR="src-tauri/target/release/bundle"

# DEB package (generated by Tauri)
if [ -f "$BUNDLE_DIR/deb/mindlink_${VERSION}_${ARCH}.deb" ]; then
    echo "✅ DEB package created: $BUNDLE_DIR/deb/mindlink_${VERSION}_${ARCH}.deb"
    
    # Verify DEB package
    lintian "$BUNDLE_DIR/deb/mindlink_${VERSION}_${ARCH}.deb"
fi

# AppImage (generated by Tauri)
if [ -f "$BUNDLE_DIR/appimage/mindlink_${VERSION}_${ARCH}.AppImage" ]; then
    echo "✅ AppImage created: $BUNDLE_DIR/appimage/mindlink_${VERSION}_${ARCH}.AppImage"
    
    # Make AppImage executable
    chmod +x "$BUNDLE_DIR/appimage/mindlink_${VERSION}_${ARCH}.AppImage"
fi

# Create RPM package using alien (if DEB exists)
if command -v alien >/dev/null 2>&1 && [ -f "$BUNDLE_DIR/deb/mindlink_${VERSION}_${ARCH}.deb" ]; then
    echo "📦 Creating RPM package..."
    cd "$BUNDLE_DIR/deb"
    sudo alien --to-rpm --scripts "mindlink_${VERSION}_${ARCH}.deb"
    mv *.rpm "../rpm/"
    cd -
    echo "✅ RPM package created"
fi

# Create Snap package
if command -v snapcraft >/dev/null 2>&1; then
    echo "📦 Creating Snap package..."
    snapcraft
    echo "✅ Snap package created"
fi

echo "🎉 Linux package creation complete!"
```

#### Desktop Entry File

```ini
# assets/com.mindlink.mindlink.desktop
[Desktop Entry]
Version=1.0
Type=Application
Name=MindLink
Comment=Local LLM API bridge with Cloudflare tunneling
Comment[es]=Puente de API LLM local con túneles Cloudflare
Exec=mindlink
Icon=com.mindlink.mindlink
Terminal=false
Categories=Development;Network;
Keywords=LLM;API;OpenAI;ChatGPT;AI;
StartupNotify=true
StartupWMClass=MindLink
MimeType=application/x-mindlink-config;
```

#### Snap Package Configuration

```yaml
# snap/snapcraft.yaml
name: mindlink
version: git
summary: Local LLM API bridge with Cloudflare tunneling
description: |
  MindLink creates an OpenAI-compatible API server powered by your ChatGPT 
  Plus/Pro account. It automatically creates secure public tunnels via 
  Cloudflare, enabling seamless integration with third-party applications.

base: core20
confinement: strict
grade: stable

architectures:
  - build-on: amd64
  - build-on: arm64

apps:
  mindlink:
    command: bin/mindlink
    desktop: share/applications/com.mindlink.mindlink.desktop
    plugs:
      - network
      - network-bind
      - home
      - desktop
      - desktop-legacy
      - wayland
      - unity7

parts:
  mindlink:
    plugin: rust
    source: .
    build-packages:
      - pkg-config
      - libssl-dev
      - libgtk-3-dev
      - libwebkit2gtk-4.0-dev
      - libayatana-appindicator3-dev
    stage-packages:
      - libwebkit2gtk-4.0-37
      - libgtk-3-0
      - libayatana-appindicator3-1
```

## Distribution Strategies

### Primary Distribution: GitHub Releases

#### Automated Release Creation

```bash
#!/bin/bash
# scripts/create-github-release.sh

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

VERSION=$1
REPO="yourusername/mindlink"

echo "🚀 Creating GitHub release for version $VERSION..."

# Create release with GitHub CLI
gh release create "$VERSION" \
    --repo "$REPO" \
    --title "MindLink $VERSION" \
    --notes-file RELEASE_NOTES.md \
    --draft \
    --verify-tag

# Upload all platform packages
echo "📦 Uploading release assets..."

# Windows packages
find src-tauri/target -name "*.msi" -exec gh release upload "$VERSION" {} --repo "$REPO" \;
find src-tauri/target -name "*-setup.exe" -exec gh release upload "$VERSION" {} --repo "$REPO" \;

# macOS packages  
find src-tauri/target -name "*.dmg" -exec gh release upload "$VERSION" {} --repo "$REPO" \;

# Linux packages
find src-tauri/target -name "*.deb" -exec gh release upload "$VERSION" {} --repo "$REPO" \;
find src-tauri/target -name "*.rpm" -exec gh release upload "$VERSION" {} --repo "$REPO" \;
find src-tauri/target -name "*.AppImage" -exec gh release upload "$VERSION" {} --repo "$REPO" \;

# Upload checksums
find . -name "checksums-*.txt" -exec gh release upload "$VERSION" {} --repo "$REPO" \;

echo "✅ Release created successfully!"
echo "🌐 URL: https://github.com/$REPO/releases/tag/$VERSION"
```

### Package Repository Distribution

#### Debian/Ubuntu Repository

```bash
#!/bin/bash
# scripts/publish-to-apt-repo.sh

set -e

PACKAGE_FILE=$1
REPO_DIR="/var/www/apt-repo"
DISTRIBUTION="stable"
COMPONENT="main"
ARCHITECTURE="amd64"

echo "📦 Publishing to APT repository..."

# Copy package to repository
cp "$PACKAGE_FILE" "$REPO_DIR/pool/$COMPONENT/"

# Update package index
cd "$REPO_DIR"
dpkg-scanpackages pool/$COMPONENT /dev/null | gzip > "dists/$DISTRIBUTION/$COMPONENT/binary-$ARCHITECTURE/Packages.gz"

# Generate Release file
apt-ftparchive release "dists/$DISTRIBUTION" > "dists/$DISTRIBUTION/Release"

# Sign Release file
gpg --armor --detach-sign --sign "dists/$DISTRIBUTION/Release"

echo "✅ Package published to APT repository"
```

#### Homebrew Formula

```ruby
# Formula/mindlink.rb
class Mindlink < Formula
  desc "Local LLM API bridge with Cloudflare tunneling"
  homepage "https://github.com/yourusername/mindlink"
  version "1.0.0"
  
  if Hardware::CPU.arm?
    url "https://github.com/yourusername/mindlink/releases/download/v#{version}/MindLink_#{version}_aarch64.dmg"
    sha256 "sha256_for_arm64_dmg"
  else
    url "https://github.com/yourusername/mindlink/releases/download/v#{version}/MindLink_#{version}_x64.dmg"
    sha256 "sha256_for_x64_dmg"
  end

  depends_on macos: ">= :catalina"

  def install
    prefix.install Dir["*"]
    bin.install_symlink "#{prefix}/MindLink.app/Contents/MacOS/MindLink" => "mindlink"
  end

  service do
    run [opt_bin/"mindlink", "--minimized"]
    keep_alive true
    log_path var/"log/mindlink.log"
    error_log_path var/"log/mindlink.log"
  end

  test do
    system "#{bin}/mindlink", "--version"
  end
end
```

## Security and Code Signing

### Certificate Management

#### Windows Certificate Setup

```powershell
# scripts/setup-windows-certificate.ps1

param(
    [Parameter(Mandatory=$true)]
    [string]$CertificateFile,
    
    [Parameter(Mandatory=$true)]
    [string]$Password
)

Write-Host "🔐 Setting up Windows code signing certificate..."

# Import certificate to local machine store
$cert = Import-PfxCertificate -FilePath $CertificateFile -Password (ConvertTo-SecureString -String $Password -AsPlainText -Force) -CertStoreLocation Cert:\LocalMachine\My

Write-Host "✅ Certificate imported with thumbprint: $($cert.Thumbprint)"

# Verify certificate
$certs = Get-ChildItem -Path Cert:\LocalMachine\My | Where-Object { $_.Subject -like "*Your Company*" }
if ($certs.Count -eq 0) {
    Write-Error "❌ Certificate not found in store"
    exit 1
}

Write-Host "✅ Certificate verified and ready for signing"
```

#### macOS Certificate Setup

```bash
#!/bin/bash
# scripts/setup-macos-certificate.sh

set -e

CERTIFICATE_P12="${APPLE_CERTIFICATE_P12}"
CERTIFICATE_PASSWORD="${APPLE_CERTIFICATE_PASSWORD}"

echo "🔐 Setting up macOS code signing certificate..."

# Create temporary keychain
KEYCHAIN_PATH="$HOME/Library/Keychains/build.keychain"
KEYCHAIN_PASSWORD="temp-keychain-password"

# Create and unlock keychain
security create-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security unlock-keychain -p "$KEYCHAIN_PASSWORD" build.keychain
security set-keychain-settings -t 3600 -u build.keychain

# Import certificate
echo "$CERTIFICATE_P12" | base64 --decode > certificate.p12
security import certificate.p12 -k build.keychain -P "$CERTIFICATE_PASSWORD" -T /usr/bin/codesign

# Set keychain as default
security list-keychains -s build.keychain
security default-keychain -s build.keychain

# Verify certificate
security find-identity -v -p codesigning

echo "✅ Certificate setup complete"

# Clean up
rm certificate.p12
```

### Signature Verification

#### Verification Scripts

```bash
#!/bin/bash
# scripts/verify-signatures.sh

set -e

echo "🔍 Verifying package signatures..."

# Windows
for msi in src-tauri/target/*/release/bundle/msi/*.msi; do
    if [ -f "$msi" ]; then
        echo "Verifying Windows package: $(basename "$msi")"
        # Note: This would run on Windows
        # signtool verify /pa "$msi"
    fi
done

# macOS
for dmg in src-tauri/target/*/release/bundle/dmg/*.dmg; do
    if [ -f "$dmg" ]; then
        echo "Verifying macOS package: $(basename "$dmg")"
        codesign --verify --deep --strict "$dmg"
        spctl --assess --type open --context context:primary-signature "$dmg"
    fi
done

# Linux (check if packages can be installed)
for deb in src-tauri/target/*/release/bundle/deb/*.deb; do
    if [ -f "$deb" ]; then
        echo "Verifying Debian package: $(basename "$deb")"
        dpkg --info "$deb"
        lintian "$deb"
    fi
done

echo "✅ All signatures verified"
```

## Release Management

### Semantic Versioning

#### Version Bump Script

```bash
#!/bin/bash
# scripts/bump-version.sh

set -e

CURRENT_VERSION=$(grep '^version = ' src-tauri/Cargo.toml | sed 's/version = "\(.*\)"/\1/')
VERSION_TYPE=$1

if [ -z "$VERSION_TYPE" ]; then
    echo "Usage: $0 <major|minor|patch>"
    echo "Current version: $CURRENT_VERSION"
    exit 1
fi

echo "📈 Bumping $VERSION_TYPE version from $CURRENT_VERSION..."

# Parse current version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Calculate new version
case $VERSION_TYPE in
    "major")
        NEW_MAJOR=$((MAJOR + 1))
        NEW_MINOR=0
        NEW_PATCH=0
        ;;
    "minor") 
        NEW_MAJOR=$MAJOR
        NEW_MINOR=$((MINOR + 1))
        NEW_PATCH=0
        ;;
    "patch")
        NEW_MAJOR=$MAJOR
        NEW_MINOR=$MINOR
        NEW_PATCH=$((PATCH + 1))
        ;;
    *)
        echo "❌ Invalid version type. Use major, minor, or patch"
        exit 1
        ;;
esac

NEW_VERSION="$NEW_MAJOR.$NEW_MINOR.$NEW_PATCH"

echo "🎯 New version: $NEW_VERSION"

# Update all version files
echo "📝 Updating version in files..."
sed -i.bak "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
sed -i.bak "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" package.json
sed -i.bak "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json

# Update Cargo.lock
cd src-tauri && cargo check && cd ..

# Clean up backup files
rm -f src-tauri/Cargo.toml.bak package.json.bak src-tauri/tauri.conf.json.bak

echo "✅ Version bumped to $NEW_VERSION"
echo "📋 Don't forget to:"
echo "   1. Update CHANGELOG.md"
echo "   2. Commit changes: git commit -am 'chore: bump version to $NEW_VERSION'"
echo "   3. Create tag: git tag v$NEW_VERSION"
echo "   4. Push changes: git push && git push --tags"
```

### Changelog Generation

```bash
#!/bin/bash
# scripts/generate-changelog.sh

set -e

PREVIOUS_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
CURRENT_VERSION=$1

if [ -z "$CURRENT_VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "📝 Generating changelog for version $CURRENT_VERSION..."

if [ -n "$PREVIOUS_TAG" ]; then
    echo "📊 Changes since $PREVIOUS_TAG:"
    
    # Generate commit summary
    {
        echo "## [$CURRENT_VERSION] - $(date +%Y-%m-%d)"
        echo ""
        
        # Group commits by type
        echo "### Added"
        git log "$PREVIOUS_TAG"..HEAD --pretty=format:"- %s" --grep="^feat" | sed 's/^feat[^:]*: /- /'
        echo ""
        
        echo "### Changed"
        git log "$PREVIOUS_TAG"..HEAD --pretty=format:"- %s" --grep="^refactor\|^perf" | sed 's/^[^:]*: /- /'
        echo ""
        
        echo "### Fixed"
        git log "$PREVIOUS_TAG"..HEAD --pretty=format:"- %s" --grep="^fix" | sed 's/^fix[^:]*: /- /'
        echo ""
        
        echo "### Security"
        git log "$PREVIOUS_TAG"..HEAD --pretty=format:"- %s" --grep="^security" | sed 's/^security[^:]*: /- /'
        echo ""
        
    } > "CHANGELOG_$CURRENT_VERSION.md"
    
    echo "✅ Changelog generated: CHANGELOG_$CURRENT_VERSION.md"
else
    echo "⚠️ No previous tag found. Creating initial changelog entry."
    
    {
        echo "## [$CURRENT_VERSION] - $(date +%Y-%m-%d)"
        echo ""
        echo "### Added"
        echo "- Initial release of MindLink"
        echo "- OpenAI-compatible API server"
        echo "- ChatGPT Plus/Pro authentication"
        echo "- Cloudflare tunnel integration"
        echo "- Cross-platform desktop application"
        echo ""
    } > "CHANGELOG_$CURRENT_VERSION.md"
    
    echo "✅ Initial changelog created: CHANGELOG_$CURRENT_VERSION.md"
fi

echo "📋 Review the generated changelog and integrate it into CHANGELOG.md"
```

## Auto-Updates

### Update Mechanism

#### Tauri Updater Configuration

```json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://github.com/yourusername/mindlink/releases/latest/download/latest.json"
    ],
    "dialog": true,
    "pubkey": "your-public-key-here"
  }
}
```

#### Update Server Response

```json
{
  "version": "1.2.3",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2024-01-15T12:00:00Z",
  "platforms": {
    "darwin-x86_64": {
      "signature": "signature-here",
      "url": "https://github.com/yourusername/mindlink/releases/download/v1.2.3/MindLink_1.2.3_x64.app.tar.gz"
    },
    "darwin-aarch64": {
      "signature": "signature-here", 
      "url": "https://github.com/yourusername/mindlink/releases/download/v1.2.3/MindLink_1.2.3_aarch64.app.tar.gz"
    },
    "linux-x86_64": {
      "signature": "signature-here",
      "url": "https://github.com/yourusername/mindlink/releases/download/v1.2.3/mindlink_1.2.3_amd64.AppImage.tar.gz"
    },
    "windows-x86_64": {
      "signature": "signature-here",
      "url": "https://github.com/yourusername/mindlink/releases/download/v1.2.3/MindLink_1.2.3_x64-setup.exe.zip"
    }
  }
}
```

#### Update Generation Script

```bash
#!/bin/bash
# scripts/generate-update-manifest.sh

set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "🔄 Generating update manifest for version $VERSION..."

REPO="yourusername/mindlink"
BASE_URL="https://github.com/$REPO/releases/download/v$VERSION"

# Create manifest
cat > latest.json << EOF
{
  "version": "$VERSION",
  "notes": "See GitHub release notes for details",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
EOF

# Add platform entries
PLATFORMS=("darwin-x86_64" "darwin-aarch64" "linux-x86_64" "windows-x86_64")
for i in "${!PLATFORMS[@]}"; do
    PLATFORM="${PLATFORMS[$i]}"
    
    # Find corresponding file
    case $PLATFORM in
        "darwin-x86_64")
            FILE_PATTERN="*_x64.app.tar.gz"
            ;;
        "darwin-aarch64") 
            FILE_PATTERN="*_aarch64.app.tar.gz"
            ;;
        "linux-x86_64")
            FILE_PATTERN="*_amd64.AppImage.tar.gz"
            ;;
        "windows-x86_64")
            FILE_PATTERN="*_x64-setup.exe.zip"
            ;;
    esac
    
    # Generate signature (placeholder - implement actual signing)
    SIGNATURE="$(openssl dgst -sha256 -sign private.key -out /tmp/signature.sig "$FILE_PATTERN" && base64 -i /tmp/signature.sig | tr -d '\n')"
    
    cat >> latest.json << EOF
    "$PLATFORM": {
      "signature": "$SIGNATURE",
      "url": "$BASE_URL/$(basename $FILE_PATTERN)"
    }$([ $i -lt $((${#PLATFORMS[@]} - 1)) ] && echo "," || echo "")
EOF
done

cat >> latest.json << EOF
  }
}
EOF

echo "✅ Update manifest generated: latest.json"
```

## Monitoring and Analytics

### Deployment Metrics

#### Release Analytics

```bash
#!/bin/bash
# scripts/release-analytics.sh

REPO="yourusername/mindlink"
VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "📊 Fetching release analytics for $VERSION..."

# Get release data using GitHub API
RELEASE_DATA=$(gh api repos/$REPO/releases/tags/v$VERSION)

# Extract download counts
echo "📥 Download Statistics:"
echo "$RELEASE_DATA" | jq -r '.assets[] | "\(.name): \(.download_count) downloads"'

# Total downloads
TOTAL_DOWNLOADS=$(echo "$RELEASE_DATA" | jq -r '.assets[].download_count' | awk '{sum+=$1} END {print sum}')
echo "📈 Total downloads: $TOTAL_DOWNLOADS"

# Platform breakdown
echo ""
echo "🖥️ Platform Breakdown:"
echo "$RELEASE_DATA" | jq -r '.assets[] | select(.name | contains("windows")) | "\(.name): \(.download_count)"' | head -1 | sed 's/^/Windows: /'
echo "$RELEASE_DATA" | jq -r '.assets[] | select(.name | contains("darwin") or contains("mac")) | "\(.name): \(.download_count)"' | head -1 | sed 's/^/macOS: /'
echo "$RELEASE_DATA" | jq -r '.assets[] | select(.name | contains("linux") or contains("deb")) | "\(.name): \(.download_count)"' | head -1 | sed 's/^/Linux: /'
```

### Error Tracking

#### Sentry Integration

```rust
// src-tauri/src/error_reporting.rs
use sentry::{ClientOptions, protocol::Event};

pub fn init_error_reporting() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = sentry::init(ClientOptions {
        dsn: option_env!("SENTRY_DSN").map(|s| s.parse().ok()).flatten(),
        release: Some(env!("CARGO_PKG_VERSION").into()),
        environment: Some(if cfg!(debug_assertions) { 
            "development" 
        } else { 
            "production" 
        }.into()),
        traces_sample_rate: 0.1,
        ..Default::default()
    });
    
    Ok(())
}

pub fn report_deployment_event(version: &str, platform: &str) {
    sentry::add_breadcrumb(sentry::Breadcrumb {
        message: Some(format!("Application started - version: {}, platform: {}", version, platform)),
        level: sentry::Level::Info,
        ..Default::default()
    });
}
```

## Rollback Procedures

### Emergency Rollback

#### Automated Rollback Script

```bash
#!/bin/bash
# scripts/emergency-rollback.sh

set -e

CURRENT_VERSION=$1
ROLLBACK_VERSION=$2

if [ -z "$CURRENT_VERSION" ] || [ -z "$ROLLBACK_VERSION" ]; then
    echo "Usage: $0 <current-version> <rollback-version>"
    echo "Example: $0 1.2.3 1.2.2"
    exit 1
fi

echo "🚨 EMERGENCY ROLLBACK: $CURRENT_VERSION → $ROLLBACK_VERSION"
echo "⚠️  This will:"
echo "   1. Hide the problematic release"
echo "   2. Update auto-updater to point to stable version"
echo "   3. Notify users of the rollback"

read -p "Continue with rollback? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Rollback cancelled"
    exit 1
fi

REPO="yourusername/mindlink"

# 1. Mark current release as draft (hides it)
echo "📝 Marking v$CURRENT_VERSION as draft..."
gh release edit "v$CURRENT_VERSION" --repo "$REPO" --draft

# 2. Update latest.json to point to rollback version
echo "🔄 Updating auto-updater manifest..."
./scripts/generate-update-manifest.sh "$ROLLBACK_VERSION"

# 3. Upload new manifest
gh release upload "v$ROLLBACK_VERSION" latest.json --repo "$REPO" --clobber

# 4. Create rollback announcement
cat > rollback-announcement.md << EOF
# 🚨 Emergency Rollback Notice

We have rolled back MindLink from version $CURRENT_VERSION to $ROLLBACK_VERSION due to critical issues.

## What happened?
Version $CURRENT_VERSION contained issues that affected core functionality.

## What should you do?
- **Existing users**: Your application will auto-update to $ROLLBACK_VERSION within 24 hours
- **New users**: Download version $ROLLBACK_VERSION from our releases page
- **Manual update**: Download and install $ROLLBACK_VERSION immediately if experiencing issues

## When will the fix be available?
We are working on version $(echo "$CURRENT_VERSION" | awk -F. '{printf "%d.%d.%d", $1, $2, $3+1}') which will address these issues.

We apologize for any inconvenience caused.
EOF

echo "📢 Creating rollback announcement..."
gh release create "rollback-$CURRENT_VERSION-to-$ROLLBACK_VERSION" \
    --repo "$REPO" \
    --title "🚨 Rollback Notice: $CURRENT_VERSION → $ROLLBACK_VERSION" \
    --notes-file rollback-announcement.md \
    --prerelease

echo "✅ Emergency rollback complete!"
echo "🔗 Rollback announcement: https://github.com/$REPO/releases/tag/rollback-$CURRENT_VERSION-to-$ROLLBACK_VERSION"

# Clean up
rm rollback-announcement.md latest.json
```

### Gradual Rollback

#### Phased Rollback Strategy

```bash
#!/bin/bash
# scripts/phased-rollback.sh

set -e

PROBLEMATIC_VERSION=$1
STABLE_VERSION=$2
ROLLBACK_PERCENTAGE=${3:-10}

echo "📊 Starting phased rollback: $ROLLBACK_PERCENTAGE% of users"
echo "   From: $PROBLEMATIC_VERSION"
echo "   To:   $STABLE_VERSION"

# Create conditional update manifest
cat > latest-phased.json << EOF
{
  "version": "$STABLE_VERSION",
  "notes": "Stability improvements and bug fixes",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "rollback": {
    "from_version": "$PROBLEMATIC_VERSION",
    "percentage": $ROLLBACK_PERCENTAGE,
    "reason": "Critical stability issues"
  },
  "platforms": {
    // Platform definitions here
  }
}
EOF

# Monitor rollback progress
echo "📈 Monitoring rollback progress..."
echo "This would track:"
echo "  - Number of users rolled back"
echo "  - Error reports from both versions"
echo "  - Success rate of rollback process"

# Gradual increase over time
for percentage in 25 50 75 100; do
    echo "🔄 Increasing rollback to $percentage% (would be automated)"
    sleep 2  # In real scenario, this would be hours/days
done

echo "✅ Phased rollback complete"
```

This comprehensive deployment guide covers all aspects of building, distributing, and maintaining MindLink across multiple platforms. The automated scripts and workflows ensure consistent, secure, and reliable deployments while providing robust rollback mechanisms for handling issues in production.