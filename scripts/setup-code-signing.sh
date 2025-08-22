#!/bin/bash

# MindLink Code Signing Setup Guide
# Provides instructions for setting up code signing on macOS and Windows

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🔐 MindLink Code Signing Setup Guide"
echo "======================================"
echo ""

cat << 'EOF'
Code signing ensures that users can trust your application and prevents
security warnings during installation.

📋 REQUIREMENTS BY PLATFORM:

🍎 macOS Code Signing:
   Prerequisites:
   - Apple Developer Account ($99/year)
   - Developer ID Application Certificate
   - Access to macOS machine for signing

   Setup Steps:
   1. Enroll in Apple Developer Program
   2. Generate Developer ID Application Certificate
   3. Download certificate to keychain
   4. Set APPLE_CERTIFICATE_IDENTITY environment variable
   5. Optionally notarize for Gatekeeper compatibility

   Environment Variables:
   export APPLE_CERTIFICATE_IDENTITY="Developer ID Application: Your Name"
   export APPLE_SIGNING_IDENTITY="Your Name"
   export APPLE_ID="your@email.com"
   export APPLE_PASSWORD="app-specific-password"
   export APPLE_TEAM_ID="TEAM12345"

🪟 Windows Code Signing:
   Prerequisites:
   - Code signing certificate (from CA like DigiCert, Sectigo)
   - Windows machine or cross-platform signing tool

   Setup Steps:
   1. Purchase code signing certificate
   2. Install certificate or use Azure Key Vault
   3. Set certificate thumbprint in tauri.conf.json
   4. Configure timestamp server

   Environment Variables:
   export WINDOWS_CERTIFICATE_THUMBPRINT="abcdef1234567890..."
   export WINDOWS_CERTIFICATE_PASSWORD="certificate-password"

🐧 Linux Code Signing:
   - Linux AppImages don't require code signing
   - GPG signing can be used for additional verification
   - Distribution through repositories provides trust

🔧 CURRENT CONFIGURATION STATUS:
EOF

echo ""
echo "📁 Checking current configuration..."

# Check tauri.conf.json for signing configuration
if [[ -f "$PROJECT_ROOT/src-tauri/tauri.conf.json" ]]; then
    echo "✅ tauri.conf.json exists"
    
    # Check macOS signing config
    if grep -q '"signingIdentity": null' "$PROJECT_ROOT/src-tauri/tauri.conf.json"; then
        echo "⚠️  macOS signing identity not configured"
    else
        echo "✅ macOS signing identity configured"
    fi
    
    # Check Windows certificate config
    if grep -q '"certificateThumbprint": null' "$PROJECT_ROOT/src-tauri/tauri.conf.json"; then
        echo "⚠️  Windows certificate thumbprint not configured"
    else
        echo "✅ Windows certificate thumbprint configured"
    fi
else
    echo "❌ tauri.conf.json not found"
fi

echo ""
echo "🚀 NEXT STEPS:"
echo ""
echo "1. For macOS signing:"
echo "   - Get Apple Developer Account"
echo "   - Generate certificate"
echo "   - Update tauri.conf.json signingIdentity"
echo ""
echo "2. For Windows signing:"
echo "   - Get code signing certificate"
echo "   - Update tauri.conf.json certificateThumbprint"
echo ""
echo "3. Test signing:"
echo "   npm run build"
echo ""
echo "4. For CI/CD, store certificates securely:"
echo "   - Use GitHub Secrets for certificate data"
echo "   - Use environment variables for passwords"
echo ""

cat << 'EOF'
📖 ADDITIONAL RESOURCES:

macOS:
- https://tauri.app/v1/guides/distribution/sign-macos
- https://developer.apple.com/developer-id/

Windows:
- https://tauri.app/v1/guides/distribution/sign-windows
- Code signing certificate providers:
  * DigiCert: https://www.digicert.com/code-signing/
  * Sectigo: https://sectigo.com/ssl-certificates-tls/code-signing

Cross-platform:
- https://tauri.app/v1/guides/building/cross-platform

⚠️  IMPORTANT NOTES:
- Keep certificates and private keys secure
- Never commit certificates to version control
- Use environment variables for sensitive data
- Test signing process before release
- Budget for certificate renewal costs

EOF