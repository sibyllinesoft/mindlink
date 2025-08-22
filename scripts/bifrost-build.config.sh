#!/bin/bash

# Bifrost Build Configuration
# This file contains build configuration options for the Bifrost source build

# Repository Configuration
export BIFROST_REPO_URL="https://github.com/maximhq/bifrost.git"
export BIFROST_BRANCH="main"
export BIFROST_REPO_DEPTH="1"  # Shallow clone for faster downloads

# Build Configuration
export BIFROST_BUILD_TYPE="release"  # or "debug"
export BIFROST_TARGET_ARCH="$(uname -m)"
export BIFROST_TARGET_OS="$(uname -s | tr '[:upper:]' '[:lower:]')"

# Paths Configuration
export BIFROST_BUILD_DIR="build"
export BIFROST_BINARIES_DIR="src-tauri/binaries"
export BIFROST_SOURCE_DIR="build/bifrost"

# Build Options
export BIFROST_CLEANUP_AFTER_BUILD="true"
export BIFROST_VERIFY_BINARY="true"
export BIFROST_PARALLEL_JOBS="$(nproc 2>/dev/null || echo 4)"

# Node.js Configuration (if building Node.js project)
export NODE_ENV="production"
export NPM_CONFIG_LOGLEVEL="warn"
export NPM_CONFIG_PROGRESS="false"

# Rust Configuration (if building Rust project)
export CARGO_BUILD_PROFILE="release"
export CARGO_TARGET_DIR="target"

# Platform-specific settings
case "$BIFROST_TARGET_OS" in
    "linux")
        export BIFROST_BINARY_NAME="bifrost"
        export BIFROST_STRIP_BINARY="true"
        ;;
    "darwin")
        export BIFROST_BINARY_NAME="bifrost"
        export BIFROST_STRIP_BINARY="true"
        ;;
    "windows"|"mingw"*|"msys"*|"cygwin"*)
        export BIFROST_BINARY_NAME="bifrost.exe"
        export BIFROST_STRIP_BINARY="false"
        ;;
    *)
        export BIFROST_BINARY_NAME="bifrost"
        export BIFROST_STRIP_BINARY="false"
        ;;
esac

# Validation function
validate_config() {
    local errors=()
    
    # Check if repository URL is valid
    if [[ -z "$BIFROST_REPO_URL" ]]; then
        errors+=("BIFROST_REPO_URL is not set")
    fi
    
    # Check if branch is valid
    if [[ -z "$BIFROST_BRANCH" ]]; then
        errors+=("BIFROST_BRANCH is not set")
    fi
    
    # Validate build type
    if [[ "$BIFROST_BUILD_TYPE" != "release" && "$BIFROST_BUILD_TYPE" != "debug" ]]; then
        errors+=("BIFROST_BUILD_TYPE must be 'release' or 'debug'")
    fi
    
    # Check for required directories
    if [[ -z "$BIFROST_BUILD_DIR" ]]; then
        errors+=("BIFROST_BUILD_DIR is not set")
    fi
    
    if [[ -z "$BIFROST_BINARIES_DIR" ]]; then
        errors+=("BIFROST_BINARIES_DIR is not set")
    fi
    
    # Report errors
    if [[ ${#errors[@]} -gt 0 ]]; then
        echo "Configuration validation failed:" >&2
        for error in "${errors[@]}"; do
            echo "  - $error" >&2
        done
        return 1
    fi
    
    return 0
}

# Print configuration
print_config() {
    echo "=== Bifrost Build Configuration ==="
    echo "Repository URL: $BIFROST_REPO_URL"
    echo "Branch: $BIFROST_BRANCH"
    echo "Build Type: $BIFROST_BUILD_TYPE"
    echo "Target OS: $BIFROST_TARGET_OS"
    echo "Target Architecture: $BIFROST_TARGET_ARCH"
    echo "Binary Name: $BIFROST_BINARY_NAME"
    echo "Build Directory: $BIFROST_BUILD_DIR"
    echo "Binaries Directory: $BIFROST_BINARIES_DIR"
    echo "Parallel Jobs: $BIFROST_PARALLEL_JOBS"
    echo "Cleanup After Build: $BIFROST_CLEANUP_AFTER_BUILD"
    echo "Verify Binary: $BIFROST_VERIFY_BINARY"
    echo "Strip Binary: $BIFROST_STRIP_BINARY"
    echo "================================="
}

# If this script is run directly, show the configuration
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    if validate_config; then
        print_config
    else
        exit 1
    fi
fi