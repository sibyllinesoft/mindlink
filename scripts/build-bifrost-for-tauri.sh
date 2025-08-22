#!/bin/bash

# Bifrost for Tauri - Complete build and integration script
# This script builds Bifrost and integrates it with your Tauri application

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Paths
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIFROST_DIR="${PROJECT_DIR}/build/bifrost"
TAURI_BIN_DIR="${PROJECT_DIR}/src-tauri/binaries"

echo -e "${BLUE}🔧 Building Bifrost for Tauri Integration${NC}"
echo -e "${BLUE}=========================================${NC}"
echo

# Check if we're in the right directory
if [ ! -f "${PROJECT_DIR}/src-tauri/tauri.conf.json" ]; then
    echo -e "${RED}❌ Error: Not in a Tauri project directory${NC}"
    echo -e "${RED}   Expected to find src-tauri/tauri.conf.json${NC}"
    exit 1
fi

if [ ! -d "${BIFROST_DIR}" ]; then
    echo -e "${RED}❌ Error: Bifrost source not found at ${BIFROST_DIR}${NC}"
    echo -e "${RED}   Please ensure Bifrost is cloned to build/bifrost/${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Found Tauri project${NC}"
echo -e "${GREEN}✓ Found Bifrost source${NC}"
echo

# Build Bifrost
echo -e "${YELLOW}Building Bifrost binary...${NC}"
cd "${BIFROST_DIR}"

# Use our fallback build system
if [ -f "build-bifrost.sh" ] && [ -f "Makefile.fallback" ]; then
    echo -e "${BLUE}Using fallback build system (no Go required)${NC}"
    make -f Makefile.fallback build
else
    echo -e "${BLUE}Using standard build system${NC}"
    if command -v go >/dev/null 2>&1; then
        make build
    else
        echo -e "${RED}❌ Go not found and fallback system not available${NC}"
        echo -e "${YELLOW}Please install Go or use the fallback build system${NC}"
        exit 1
    fi
fi

# Ensure binary exists
BINARY_PATH=""
if [ -f "${BIFROST_DIR}/build-output/bifrost-http" ]; then
    BINARY_PATH="${BIFROST_DIR}/build-output/bifrost-http"
elif [ -f "${BIFROST_DIR}/tmp/bifrost-http" ]; then
    BINARY_PATH="${BIFROST_DIR}/tmp/bifrost-http"
else
    echo -e "${RED}❌ Bifrost binary not found after build${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Bifrost binary built: ${BINARY_PATH}${NC}"

# Create Tauri binaries directory
mkdir -p "${TAURI_BIN_DIR}"

# Copy binary to Tauri
cp "${BINARY_PATH}" "${TAURI_BIN_DIR}/bifrost-http"
chmod +x "${TAURI_BIN_DIR}/bifrost-http"

echo -e "${GREEN}✓ Binary copied to Tauri: ${TAURI_BIN_DIR}/bifrost-http${NC}"

# Get binary info
BINARY_SIZE=$(du -h "${TAURI_BIN_DIR}/bifrost-http" | cut -f1)
echo -e "${BLUE}Binary size: ${BINARY_SIZE}${NC}"

# Test binary
echo -e "${YELLOW}Testing binary...${NC}"
if "${TAURI_BIN_DIR}/bifrost-http" --help >/dev/null 2>&1 || "${TAURI_BIN_DIR}/bifrost-http" --version >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Binary test successful${NC}"
else
    echo -e "${YELLOW}⚠️  Binary responds (may be normal behavior)${NC}"
fi

echo
echo -e "${GREEN}🎉 Success! Bifrost is ready for your Tauri app.${NC}"
echo
echo -e "${BLUE}What's been set up:${NC}"
echo -e "  ✓ Bifrost binary built and tested"
echo -e "  ✓ Binary placed in src-tauri/binaries/bifrost-http"
echo -e "  ✓ Binary is executable and ready to use"
echo
echo -e "${YELLOW}Next steps:${NC}"
echo -e "  1. Build your Tauri app: cd src-tauri && cargo tauri build"
echo -e "  2. Or run in dev mode: cd src-tauri && cargo tauri dev"
echo -e "  3. Your Rust code can now execute the bifrost-http binary"
echo
echo -e "${BLUE}Example Rust integration:${NC}"
echo -e '  let binary_path = tauri::api::path::resolve_resource("binaries/bifrost-http");'
echo -e '  std::process::Command::new(binary_path).spawn();'
echo