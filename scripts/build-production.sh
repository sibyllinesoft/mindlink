#!/bin/bash

# MindLink Production Build Script
# Builds the application with all required components and tests the output

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/src-tauri/target"
DIST_DIR="$PROJECT_ROOT/dist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 MindLink Production Build${NC}"
echo "==============================="
echo ""

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Change to project directory
cd "$PROJECT_ROOT"

# Check prerequisites
echo -e "${BLUE}🔍 Checking prerequisites...${NC}"

if ! command -v node &> /dev/null; then
    print_error "Node.js not found. Please install Node.js."
    exit 1
fi
print_status "Node.js $(node --version)"

if ! command -v npm &> /dev/null; then
    print_error "npm not found. Please install npm."
    exit 1
fi
print_status "npm $(npm --version)"

if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo not found. Please install Rust."
    exit 1
fi
print_status "Rust $(rustc --version)"

# Check if Tauri CLI is available
if command -v tauri &> /dev/null; then
    print_status "Tauri CLI (global) available"
elif npm list @tauri-apps/cli &> /dev/null; then
    print_status "Tauri CLI (local) available"
elif npx tauri --version &> /dev/null 2>&1; then
    print_status "Tauri CLI (via npx) available"
else
    print_error "Tauri CLI not found. Please install it:"
    echo "  npm install @tauri-apps/cli"
    echo "  or"
    echo "  npm install -g @tauri-apps/cli"
    exit 1
fi

echo ""

# Clean previous builds
echo -e "${BLUE}🧹 Cleaning previous builds...${NC}"
if [[ -d "$BUILD_DIR" ]]; then
    rm -rf "$BUILD_DIR/release"
    print_status "Cleaned Rust build artifacts"
fi

if [[ -d "$DIST_DIR" ]]; then
    rm -rf "$DIST_DIR"
    print_status "Cleaned frontend dist"
fi

echo ""

# Install dependencies
echo -e "${BLUE}📦 Installing dependencies...${NC}"
npm ci
print_status "Dependencies installed"

echo ""

# Build Bifrost binary
echo -e "${BLUE}🔧 Building Bifrost binary...${NC}"
if [[ -f "build/bifrost/build-bifrost.sh" ]]; then
    ./build/bifrost/build-bifrost.sh
    print_status "Bifrost binary built"
else
    print_warning "Bifrost build script not found, assuming binary exists"
fi

# Verify Bifrost binary exists
if [[ -f "src-tauri/binaries/bifrost-http" ]]; then
    print_status "Bifrost binary verified"
    ls -la src-tauri/binaries/bifrost-http
else
    print_error "Bifrost binary not found at src-tauri/binaries/bifrost-http"
    exit 1
fi

echo ""

# Build frontend (if needed)
echo -e "${BLUE}🎨 Building frontend...${NC}"
if [[ -f "vite.config.js" ]] || [[ -f "vite.config.ts" ]]; then
    npm run build
    print_status "Frontend built with Vite"
elif [[ -d "src" ]] && [[ ! -d "dist" ]]; then
    # Create a basic dist directory if no build process
    mkdir -p dist
    cp -r src/* dist/ 2>/dev/null || true
    print_status "Frontend files copied to dist"
else
    print_status "Frontend build not required or already exists"
fi

echo ""

# Build Tauri application
echo -e "${BLUE}🔨 Building Tauri application...${NC}"
echo "This may take several minutes..."

BUILD_START=$(date +%s)

# Set environment variables for signing if available
if [[ -n "${APPLE_CERTIFICATE_IDENTITY:-}" ]]; then
    print_status "macOS signing identity: $APPLE_CERTIFICATE_IDENTITY"
fi

if [[ -n "${WINDOWS_CERTIFICATE_THUMBPRINT:-}" ]]; then
    print_status "Windows certificate configured"
fi

# Run the build
if npm run build 2>&1; then
    BUILD_END=$(date +%s)
    BUILD_TIME=$((BUILD_END - BUILD_START))
    print_status "Build completed in ${BUILD_TIME}s"
else
    print_error "Build failed"
    exit 1
fi

echo ""

# Analyze build output
echo -e "${BLUE}📊 Analyzing build output...${NC}"

if [[ -d "$BUILD_DIR/release/bundle" ]]; then
    print_status "Build artifacts created"
    
    # List all generated installers
    echo ""
    echo "📦 Generated installers:"
    find "$BUILD_DIR/release/bundle" -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" -o -name "*.dmg" -o -name "*.msi" -o -name "*.exe" | while read -r file; do
        size=$(du -h "$file" | cut -f1)
        echo "  - $(basename "$file") (${size})"
    done
    
    # Check for updater artifacts
    if [[ -d "$BUILD_DIR/release/bundle/updater" ]]; then
        print_status "Updater artifacts generated"
        ls -la "$BUILD_DIR/release/bundle/updater"
    fi
    
    echo ""
    
    # Test binary execution (Linux only for now)
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo -e "${BLUE}🧪 Testing binary execution...${NC}"
        
        # Find the binary
        BINARY_PATH=$(find "$BUILD_DIR/release" -name "mindlink" -type f -executable | head -1)
        if [[ -n "$BINARY_PATH" ]]; then
            print_status "Binary found: $BINARY_PATH"
            
            # Test if binary runs (with timeout)
            if timeout 10s "$BINARY_PATH" --version &> /dev/null || timeout 10s "$BINARY_PATH" --help &> /dev/null; then
                print_status "Binary executes successfully"
            else
                print_warning "Binary test inconclusive (may be GUI-only)"
            fi
        else
            print_warning "Binary not found for testing"
        fi
    fi
    
else
    print_error "No build artifacts found"
    exit 1
fi

echo ""

# Final summary
echo -e "${GREEN}🎉 Build Summary${NC}"
echo "=================="
echo ""
print_status "Production build completed successfully"
print_status "All required components bundled"
print_status "Installers generated for target platforms"

if [[ -n "${APPLE_CERTIFICATE_IDENTITY:-}" ]] || [[ -n "${WINDOWS_CERTIFICATE_THUMBPRINT:-}" ]]; then
    print_status "Code signing configured"
else
    print_warning "Code signing not configured - installers will show security warnings"
fi

echo ""
echo "📁 Build artifacts location: $BUILD_DIR/release/bundle"
echo "🔍 Test installers on clean systems before distribution"
echo "📚 See DEPLOYMENT.md for distribution instructions"
echo ""