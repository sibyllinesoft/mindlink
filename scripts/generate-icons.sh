#!/bin/bash

# MindLink Icon Generation Script
# Generates all required icon sizes for Tauri app from the source SVG

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ICON_SOURCE="$PROJECT_ROOT/src-tauri/icons/brain-icon.svg"
ICON_DIR="$PROJECT_ROOT/src-tauri/icons"
ASSETS_DIR="$PROJECT_ROOT/assets"

# Ensure directories exist
mkdir -p "$ICON_DIR"
mkdir -p "$ASSETS_DIR"

# Check if ImageMagick is installed
if ! command -v magick &> /dev/null && ! command -v convert &> /dev/null; then
    echo "Error: ImageMagick is not installed. Please install it:"
    echo "Ubuntu/Debian: sudo apt-get install imagemagick"
    echo "macOS: brew install imagemagick"
    echo "Windows: Download from https://imagemagick.org/script/download.php"
    exit 1
fi

# Use magick if available (ImageMagick 7+), otherwise use convert (ImageMagick 6)
CONVERT_CMD="convert"
if command -v magick &> /dev/null; then
    CONVERT_CMD="magick"
fi

echo "🎨 Generating MindLink application icons..."

# Check if source SVG exists
if [[ ! -f "$ICON_SOURCE" ]]; then
    echo "Error: Source icon not found at $ICON_SOURCE"
    echo "Please ensure brain-icon.svg exists in the icons directory"
    exit 1
fi

echo "📁 Source: $ICON_SOURCE"
echo "📁 Output: $ICON_DIR"

# Tauri required icon sizes (PNG format)
declare -a SIZES=(
    "16"
    "24"
    "32"
    "48"
    "64"
    "96"
    "128"
    "256"
    "512"
    "1024"
)

# Generate PNG icons
echo "🔧 Generating PNG icons..."
for size in "${SIZES[@]}"; do
    output_file="$ICON_DIR/${size}x${size}.png"
    echo "  - Generating ${size}x${size}.png"
    $CONVERT_CMD -background transparent -size "${size}x${size}" "$ICON_SOURCE" "$output_file"
    
    # Optimize PNG
    if command -v optipng &> /dev/null; then
        optipng -quiet "$output_file"
    fi
done

# Generate specific named icons for Tauri
echo "🔧 Generating named PNG icons..."
$CONVERT_CMD -background transparent -size "32x32" "$ICON_SOURCE" "$ICON_DIR/32x32@2x.png"
$CONVERT_CMD -background transparent -size "128x128" "$ICON_SOURCE" "$ICON_DIR/128x128.png"
$CONVERT_CMD -background transparent -size "128x128" "$ICON_SOURCE" "$ICON_DIR/128x128@2x.png"
$CONVERT_CMD -background transparent -size "256x256" "$ICON_SOURCE" "$ICON_DIR/icon.png"

# Generate tray icon (smaller, optimized for system tray)
echo "🔧 Generating tray icons..."
$CONVERT_CMD -background transparent -size "16x16" "$ICON_SOURCE" "$ICON_DIR/tray-icon.png"
$CONVERT_CMD -background transparent -size "32x32" "$ICON_SOURCE" "$ICON_DIR/tray-icon@2x.png"

# macOS specific icons
echo "🍎 Generating macOS icons..."
if command -v iconutil &> /dev/null || [[ "$OSTYPE" == "darwin"* ]]; then
    # Create iconset directory
    ICONSET_DIR="$ICON_DIR/icon.iconset"
    mkdir -p "$ICONSET_DIR"
    
    # Generate all required macOS icon sizes
    $CONVERT_CMD -background transparent -size "16x16" "$ICON_SOURCE" "$ICONSET_DIR/icon_16x16.png"
    $CONVERT_CMD -background transparent -size "32x32" "$ICON_SOURCE" "$ICONSET_DIR/icon_16x16@2x.png"
    $CONVERT_CMD -background transparent -size "32x32" "$ICON_SOURCE" "$ICONSET_DIR/icon_32x32.png"
    $CONVERT_CMD -background transparent -size "64x64" "$ICON_SOURCE" "$ICONSET_DIR/icon_32x32@2x.png"
    $CONVERT_CMD -background transparent -size "128x128" "$ICON_SOURCE" "$ICONSET_DIR/icon_128x128.png"
    $CONVERT_CMD -background transparent -size "256x256" "$ICON_SOURCE" "$ICONSET_DIR/icon_128x128@2x.png"
    $CONVERT_CMD -background transparent -size "256x256" "$ICON_SOURCE" "$ICONSET_DIR/icon_256x256.png"
    $CONVERT_CMD -background transparent -size "512x512" "$ICON_SOURCE" "$ICONSET_DIR/icon_256x256@2x.png"
    $CONVERT_CMD -background transparent -size "512x512" "$ICON_SOURCE" "$ICONSET_DIR/icon_512x512.png"
    $CONVERT_CMD -background transparent -size "1024x1024" "$ICON_SOURCE" "$ICONSET_DIR/icon_512x512@2x.png"
    
    # Generate .icns file if on macOS
    if [[ "$OSTYPE" == "darwin"* ]] && command -v iconutil &> /dev/null; then
        echo "  - Generating app.icns"
        iconutil -c icns "$ICONSET_DIR" -o "$ASSETS_DIR/app.icns"
        iconutil -c icns "$ICONSET_DIR" -o "$ICON_DIR/app.icns"
    else
        echo "  - Skipping .icns generation (not on macOS or iconutil not available)"
    fi
    
    # Clean up iconset directory
    rm -rf "$ICONSET_DIR"
fi

# Windows ICO file
echo "🪟 Generating Windows icons..."
if command -v magick &> /dev/null; then
    echo "  - Generating app.ico"
    magick "$ICON_SOURCE" \( -clone 0 -resize 16x16 \) \( -clone 0 -resize 32x32 \) \( -clone 0 -resize 48x48 \) \( -clone 0 -resize 64x64 \) \( -clone 0 -resize 128x128 \) \( -clone 0 -resize 256x256 \) -delete 0 "$ASSETS_DIR/app.ico"
    cp "$ASSETS_DIR/app.ico" "$ICON_DIR/app.ico"
elif command -v convert &> /dev/null; then
    echo "  - Generating app.ico (ImageMagick 6)"
    convert "$ICON_SOURCE" -resize 256x256 \( -clone 0 -resize 128x128 \) \( -clone 0 -resize 64x64 \) \( -clone 0 -resize 48x48 \) \( -clone 0 -resize 32x32 \) \( -clone 0 -resize 16x16 \) "$ASSETS_DIR/app.ico"
    cp "$ASSETS_DIR/app.ico" "$ICON_DIR/app.ico"
fi

# Linux app icons
echo "🐧 Generating Linux icons..."
$CONVERT_CMD -background transparent -size "512x512" "$ICON_SOURCE" "$ASSETS_DIR/app.png"
$CONVERT_CMD -background transparent -size "128x128" "$ICON_SOURCE" "$ICON_DIR/app.png"

echo "✅ Icon generation complete!"
echo ""
echo "📊 Generated files:"
echo "   Tauri icons: $ICON_DIR/"
echo "   Platform assets: $ASSETS_DIR/"
echo ""
echo "🔍 Verify the icons look correct before building:"
echo "   - Check PNG files in $ICON_DIR"
echo "   - Test platform-specific formats (.icns, .ico)"
echo ""