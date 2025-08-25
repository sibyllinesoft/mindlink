#!/bin/bash
# Cloudflared Installation Script
# Generated on: 2025-08-25
# Review this script before execution

set -euo pipefail  # Exit on any error

echo "🔍 Checking system requirements..."

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ Error: This script is designed for Linux systems only"
    echo "   Detected OS: $OSTYPE"
    exit 1
fi

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        CLOUDFLARED_ARCH="amd64"
        ;;
    aarch64|arm64)
        CLOUDFLARED_ARCH="arm64"
        ;;
    armv7l)
        CLOUDFLARED_ARCH="arm"
        ;;
    i386|i686)
        CLOUDFLARED_ARCH="386"
        ;;
    *)
        echo "❌ Error: Unsupported architecture: $ARCH"
        echo "   Supported: x86_64, aarch64/arm64, armv7l, i386/i686"
        exit 1
        ;;
esac

echo "✅ System check passed"
echo "   OS: Linux"
echo "   Architecture: $ARCH (cloudflared: $CLOUDFLARED_ARCH)"

# Check for required tools
echo "🔍 Checking required tools..."
for tool in curl sudo; do
    if ! command -v "$tool" &> /dev/null; then
        echo "❌ Error: $tool is required but not installed"
        exit 1
    fi
done
echo "✅ Required tools available"

# Check if cloudflared is already installed
if command -v cloudflared &> /dev/null; then
    CURRENT_VERSION=$(cloudflared version 2>/dev/null | head -n1 || echo "unknown")
    echo "⚠️  Warning: cloudflared is already installed"
    echo "   Current version: $CURRENT_VERSION"
    echo "   Location: $(which cloudflared)"
    read -p "   Continue with installation? (y/N): " -r
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled"
        exit 0
    fi
fi

echo "📦 Installing cloudflared..."

# Create temporary directory
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Get latest release info from GitHub API
echo "🔍 Fetching latest release information..."
LATEST_RELEASE_URL="https://api.github.com/repos/cloudflare/cloudflared/releases/latest"
RELEASE_INFO=$(curl -s "$LATEST_RELEASE_URL")

if [[ -z "$RELEASE_INFO" ]] || echo "$RELEASE_INFO" | grep -q "rate limit"; then
    echo "❌ Error: Failed to fetch release information from GitHub API"
    echo "   This might be due to rate limiting or network issues"
    exit 1
fi

# Extract download URL for our architecture
DOWNLOAD_URL=$(echo "$RELEASE_INFO" | grep -o "https://github.com/cloudflare/cloudflared/releases/download/[^\"]*linux-$CLOUDFLARED_ARCH" | head -n1)

if [[ -z "$DOWNLOAD_URL" ]]; then
    echo "❌ Error: Could not find download URL for linux-$CLOUDFLARED_ARCH"
    echo "   Available assets:"
    echo "$RELEASE_INFO" | grep -o "https://github.com/cloudflare/cloudflared/releases/download/[^\"]*" | head -10
    exit 1
fi

VERSION=$(echo "$DOWNLOAD_URL" | grep -o '/[0-9][^/]*/' | tr -d '/')
echo "📥 Downloading cloudflared $VERSION for linux-$CLOUDFLARED_ARCH..."
echo "   URL: $DOWNLOAD_URL"

# Download cloudflared
cd "$TEMP_DIR"
if ! curl -L -o cloudflared "$DOWNLOAD_URL"; then
    echo "❌ Error: Failed to download cloudflared"
    exit 1
fi

# Verify download
if [[ ! -f "cloudflared" ]] || [[ ! -s "cloudflared" ]]; then
    echo "❌ Error: Downloaded file is missing or empty"
    exit 1
fi

# Make executable
chmod +x cloudflared

# Test the binary works
echo "🔍 Testing downloaded binary..."
if ! ./cloudflared --version &> /dev/null; then
    echo "❌ Error: Downloaded binary is not working"
    echo "   This might indicate a corrupted download or incompatible architecture"
    exit 1
fi

DOWNLOADED_VERSION=$(./cloudflared --version 2>/dev/null | head -n1 || echo "unknown")
echo "✅ Binary test passed: $DOWNLOADED_VERSION"

# Install to system
echo "🔧 Installing to /usr/local/bin/cloudflared..."
if ! sudo cp cloudflared /usr/local/bin/cloudflared; then
    echo "❌ Error: Failed to install cloudflared to /usr/local/bin"
    echo "   Make sure you have sudo privileges"
    exit 1
fi

# Set proper permissions
sudo chmod +x /usr/local/bin/cloudflared

# Verify system installation
echo "✅ Verifying installation..."
if ! command -v cloudflared &> /dev/null; then
    echo "❌ Error: cloudflared not found in PATH after installation"
    echo "   You may need to add /usr/local/bin to your PATH"
    echo "   Or restart your shell session"
    exit 1
fi

INSTALLED_VERSION=$(cloudflared --version 2>/dev/null | head -n1 || echo "unknown")
INSTALL_LOCATION=$(which cloudflared)

echo "🎉 Installation complete!"
echo ""
echo "📋 Installation Summary:"
echo "   Version: $INSTALLED_VERSION"
echo "   Location: $INSTALL_LOCATION"
echo "   Architecture: $ARCH ($CLOUDFLARED_ARCH)"
echo ""
echo "🚀 Usage Instructions:"
echo ""
echo "1. Basic tunnel creation:"
echo "   cloudflared tunnel create my-tunnel"
echo ""
echo "2. Run a quick tunnel (temporary):"
echo "   cloudflared tunnel --url http://localhost:8080"
echo ""
echo "3. Login to Cloudflare (required for persistent tunnels):"
echo "   cloudflared tunnel login"
echo ""
echo "4. Configure a persistent tunnel:"
echo "   cloudflared tunnel create my-app"
echo "   cloudflared tunnel route dns my-app my-app.example.com"
echo "   cloudflared tunnel run my-app"
echo ""
echo "5. View help and more options:"
echo "   cloudflared --help"
echo "   cloudflared tunnel --help"
echo ""
echo "📖 Documentation:"
echo "   https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/"
echo ""
echo "⚠️  Security Note:"
echo "   Cloudflared creates secure tunnels to expose local services."
echo "   Always verify what services you're exposing and configure"
echo "   appropriate access controls in your Cloudflare dashboard."