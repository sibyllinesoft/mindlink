#!/bin/bash

# MindLink Updater Setup Script
# Generates cryptographic keys for Tauri auto-updater

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
KEYS_DIR="$PROJECT_ROOT/.updater-keys"

# Create keys directory (should be in .gitignore)
mkdir -p "$KEYS_DIR"

echo "🔐 Setting up MindLink auto-updater..."

# Check if tauri CLI is available
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install Rust first."
    exit 1
fi

# Generate updater keys
echo "🔑 Generating updater keys..."
cd "$PROJECT_ROOT"

# Use tauri CLI to generate updater keys
if command -v tauri &> /dev/null; then
    tauri signer generate -w "$KEYS_DIR/private.key" -p "$KEYS_DIR/public.key"
elif npx tauri signer generate --help &> /dev/null; then
    npx tauri signer generate -w "$KEYS_DIR/private.key" -p "$KEYS_DIR/public.key"
else
    echo "Error: Tauri CLI not found. Please install it:"
    echo "npm install -g @tauri-apps/cli"
    exit 1
fi

# Read the public key
if [[ -f "$KEYS_DIR/public.key" ]]; then
    PUBLIC_KEY=$(cat "$KEYS_DIR/public.key")
    echo "✅ Keys generated successfully!"
    echo ""
    echo "📋 Public key:"
    echo "$PUBLIC_KEY"
    echo ""
    echo "🔧 Next steps:"
    echo "1. Update tauri.conf.json with this public key"
    echo "2. Keep private key secure (in $KEYS_DIR/private.key)"
    echo "3. Add *.key files to .gitignore"
    echo ""
    
    # Update tauri.conf.json with the public key
    echo "🔧 Updating tauri.conf.json..."
    if command -v jq &> /dev/null; then
        # Use jq to properly update JSON
        jq --arg pubkey "$PUBLIC_KEY" '.plugins.updater.pubkey = $pubkey' src-tauri/tauri.conf.json > tmp.json && mv tmp.json src-tauri/tauri.conf.json
        echo "✅ tauri.conf.json updated with public key"
    else
        echo "⚠️  jq not found. Please manually update the pubkey in tauri.conf.json:"
        echo "   Replace 'TO_BE_GENERATED' with: $PUBLIC_KEY"
    fi
else
    echo "❌ Failed to generate keys"
    exit 1
fi

# Add keys to .gitignore
echo "🔧 Updating .gitignore..."
if [[ ! -f "$PROJECT_ROOT/.gitignore" ]]; then
    touch "$PROJECT_ROOT/.gitignore"
fi

if ! grep -q "\.updater-keys" "$PROJECT_ROOT/.gitignore"; then
    echo "" >> "$PROJECT_ROOT/.gitignore"
    echo "# Updater keys (keep private)" >> "$PROJECT_ROOT/.gitignore"
    echo ".updater-keys/" >> "$PROJECT_ROOT/.gitignore"
    echo "*.key" >> "$PROJECT_ROOT/.gitignore"
    echo "✅ Added updater keys to .gitignore"
fi

echo ""
echo "🎉 Updater setup complete!"
echo ""
echo "⚠️  IMPORTANT SECURITY NOTES:"
echo "   - Keep $KEYS_DIR/private.key secure and never commit it"
echo "   - Only the public key goes in tauri.conf.json"
echo "   - Private key is used to sign releases on CI/CD"
echo ""