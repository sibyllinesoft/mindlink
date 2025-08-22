#!/bin/bash

# Test script for Bifrost source build integration
# This script helps test the build system before integration

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "=== Bifrost Source Build Test ==="
echo "Project Root: $PROJECT_ROOT"
echo "Script Directory: $SCRIPT_DIR"
echo ""

# Test 1: Configuration validation
echo "Test 1: Configuration validation"
if "$SCRIPT_DIR/bifrost-build.config.sh"; then
    echo "✅ Configuration is valid"
else
    echo "❌ Configuration validation failed"
    exit 1
fi
echo ""

# Test 2: Dependency check (dry run)
echo "Test 2: Checking build dependencies"
echo "Checking for required tools..."

check_tool() {
    if command -v "$1" &> /dev/null; then
        echo "✅ $1 is available"
        return 0
    else
        echo "❌ $1 is not available"
        return 1
    fi
}

all_deps_ok=true
check_tool "git" || all_deps_ok=false
check_tool "node" || all_deps_ok=false
check_tool "npm" || all_deps_ok=false

if command -v "cargo" &> /dev/null; then
    echo "✅ cargo is available (optional)"
else
    echo "⚠️  cargo not available (may be needed for Rust projects)"
fi

if [ "$all_deps_ok" = true ]; then
    echo "✅ All required dependencies are available"
else
    echo "❌ Some required dependencies are missing"
    echo "Please install missing tools and try again"
    exit 1
fi
echo ""

# Test 3: Script permissions and existence
echo "Test 3: Script permissions and existence"
scripts=(
    "build-bifrost.sh"
    "tauri-build-bifrost.sh"
    "bifrost-build.config.sh"
)

for script in "${scripts[@]}"; do
    script_path="$SCRIPT_DIR/$script"
    if [[ -f "$script_path" ]]; then
        if [[ -x "$script_path" ]]; then
            echo "✅ $script exists and is executable"
        else
            echo "⚠️  $script exists but is not executable, fixing..."
            chmod +x "$script_path"
            echo "✅ $script is now executable"
        fi
    else
        echo "❌ $script does not exist"
        exit 1
    fi
done
echo ""

# Test 4: Directory structure
echo "Test 4: Directory structure check"
dirs_to_check=(
    "src-tauri"
    "src-tauri/src"
    "src-tauri/src/managers"
)

for dir in "${dirs_to_check[@]}"; do
    dir_path="$PROJECT_ROOT/$dir"
    if [[ -d "$dir_path" ]]; then
        echo "✅ $dir exists"
    else
        echo "❌ $dir does not exist"
        exit 1
    fi
done

# Create binaries directory if it doesn't exist
binaries_dir="$PROJECT_ROOT/src-tauri/binaries"
if [[ ! -d "$binaries_dir" ]]; then
    echo "📁 Creating binaries directory: $binaries_dir"
    mkdir -p "$binaries_dir"
    echo "✅ Binaries directory created"
else
    echo "✅ Binaries directory exists"
fi
echo ""

# Test 5: Tauri configuration check
echo "Test 5: Tauri configuration check"
tauri_config="$PROJECT_ROOT/src-tauri/tauri.conf.json"
if [[ -f "$tauri_config" ]]; then
    echo "✅ Tauri configuration exists"
    
    # Check if beforeBuildCommand is set
    if grep -q "tauri-build-bifrost.sh" "$tauri_config"; then
        echo "✅ Tauri configuration includes Bifrost build script"
    else
        echo "⚠️  Tauri configuration may not include Bifrost build script"
        echo "Consider updating beforeBuildCommand in tauri.conf.json"
    fi
    
    # Check if externalBin is set
    if grep -q "binaries/bifrost" "$tauri_config"; then
        echo "✅ Tauri configuration includes Bifrost as external binary"
    else
        echo "⚠️  Tauri configuration may not include Bifrost as external binary"
        echo "Consider adding 'binaries/bifrost' to externalBin in tauri.conf.json"
    fi
else
    echo "❌ Tauri configuration not found"
    exit 1
fi
echo ""

# Test 6: Quick dry run (if --dry-run flag provided)
if [[ "${1:-}" == "--dry-run" ]]; then
    echo "Test 6: Dry run mode - checking build script without actually building"
    echo "Running: $SCRIPT_DIR/tauri-build-bifrost.sh --check-only"
    
    if "$SCRIPT_DIR/tauri-build-bifrost.sh" --check-only; then
        echo "✅ Build check passed (binary exists)"
    else
        echo "ℹ️  Build check indicates binary needs to be built"
        echo "This is normal if you haven't built Bifrost yet"
    fi
else
    echo "Test 6: Skipped (use --dry-run to test build check)"
fi
echo ""

# Summary
echo "=== Test Summary ==="
echo "✅ All basic tests passed!"
echo ""
echo "Next steps:"
echo "1. Run './scripts/build-bifrost.sh' to build Bifrost from source"
echo "2. Run './scripts/tauri-build-bifrost.sh --quick' to test Tauri integration"
echo "3. Run 'npm run dev' to test the full Tauri development workflow"
echo ""
echo "For more information, see: BIFROST_SOURCE_BUILD.md"