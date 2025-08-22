#!/bin/bash

# Bifrost Source Build Script
# This script clones, builds, and installs the Bifrost binary from source
# for integration with the MindLink Tauri application.

set -euo pipefail  # Exit on error, undefined vars, pipe failures

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_DIR="$PROJECT_ROOT/build"
BIFROST_DIR="$BUILD_DIR/bifrost"
BIFROST_REPO="https://github.com/maximhq/bifrost.git"
BIFROST_BRANCH="main"  # or "stable" if preferred
TARGET_ARCH="$(uname -m)"
TARGET_OS="$(uname -s | tr '[:upper:]' '[:lower:]')"

# Output directories
BINARIES_DIR="$PROJECT_ROOT/src-tauri/binaries"
BINARY_NAME="bifrost"

# Add .exe extension for Windows (when cross-compiling or running on Windows)
if [[ "$TARGET_OS" == "mingw"* ]] || [[ "$TARGET_OS" == "msys"* ]] || [[ "$TARGET_OS" == "cygwin"* ]]; then
    BINARY_NAME="bifrost.exe"
fi

# Logging functions
log_info() {
    echo "[INFO] $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo "[ERROR] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

log_warn() {
    echo "[WARN] $(date '+%Y-%m-%d %H:%M:%S') - $1" >&2
}

# Check if required tools are available
check_dependencies() {
    log_info "Checking build dependencies..."
    
    local missing_deps=()
    
    # Check for Git
    if ! command -v git &> /dev/null; then
        missing_deps+=("git")
    fi
    
    # Check for Node.js
    if ! command -v node &> /dev/null; then
        missing_deps+=("node")
    fi
    
    # Check for npm
    if ! command -v npm &> /dev/null; then
        missing_deps+=("npm")
    fi
    
    # Check for Rust/Cargo if Bifrost uses Rust
    if ! command -v cargo &> /dev/null; then
        log_warn "Cargo not found - may be needed if Bifrost uses Rust"
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_error "Please install the missing dependencies and try again."
        exit 1
    fi
    
    log_info "All required dependencies found"
}

# Create necessary directories
setup_directories() {
    log_info "Setting up build directories..."
    
    mkdir -p "$BUILD_DIR"
    mkdir -p "$BINARIES_DIR"
    
    log_info "Build directories created"
}

# Clone or update Bifrost repository
setup_repository() {
    log_info "Setting up Bifrost repository..."
    
    if [[ -d "$BIFROST_DIR" ]]; then
        log_info "Bifrost repository exists, updating..."
        cd "$BIFROST_DIR"
        
        # Fetch latest changes
        if ! git fetch origin; then
            log_error "Failed to fetch updates from remote repository"
            return 1
        fi
        
        # Reset to clean state and pull latest
        git reset --hard HEAD
        git clean -fd
        
        if ! git pull origin "$BIFROST_BRANCH"; then
            log_error "Failed to pull latest changes"
            return 1
        fi
        
        log_info "Repository updated successfully"
    else
        log_info "Cloning Bifrost repository..."
        
        if ! git clone --branch "$BIFROST_BRANCH" --depth 1 "$BIFROST_REPO" "$BIFROST_DIR"; then
            log_error "Failed to clone Bifrost repository"
            return 1
        fi
        
        log_info "Repository cloned successfully"
    fi
}

# Detect build system and build the project
build_bifrost() {
    log_info "Building Bifrost from source..."
    
    cd "$BIFROST_DIR"
    
    # Check what kind of project this is and build accordingly
    if [[ -f "Cargo.toml" ]]; then
        log_info "Detected Rust project, building with Cargo..."
        build_rust_project
    elif [[ -f "package.json" ]]; then
        log_info "Detected Node.js project, building with npm..."
        build_nodejs_project
    elif [[ -f "Makefile" ]]; then
        log_info "Detected Makefile, building with make..."
        build_make_project
    elif [[ -f "CMakeLists.txt" ]]; then
        log_info "Detected CMake project, building with cmake..."
        build_cmake_project
    else
        log_error "Unknown build system - no recognized build files found"
        log_error "Please check the Bifrost repository for build instructions"
        return 1
    fi
}

# Build Rust project
build_rust_project() {
    log_info "Installing Rust dependencies and building..."
    
    # Build in release mode for performance
    if ! cargo build --release; then
        log_error "Cargo build failed"
        return 1
    fi
    
    # Find the built binary
    local binary_path="target/release/$BINARY_NAME"
    if [[ ! -f "$binary_path" ]]; then
        # Try without .exe extension
        binary_path="target/release/bifrost"
        if [[ ! -f "$binary_path" ]]; then
            log_error "Built binary not found at expected location"
            return 1
        fi
    fi
    
    log_info "Rust build completed successfully"
    BUILT_BINARY_PATH="$BIFROST_DIR/$binary_path"
}

# Build Node.js project
build_nodejs_project() {
    log_info "Installing Node.js dependencies..."
    
    # Install dependencies
    if ! npm ci --only=production; then
        log_warn "npm ci failed, trying npm install..."
        if ! npm install --only=production; then
            log_error "npm install failed"
            return 1
        fi
    fi
    
    # Check if there's a build script
    if npm run --silent 2>/dev/null | grep -q "build"; then
        log_info "Running build script..."
        if ! npm run build; then
            log_error "npm run build failed"
            return 1
        fi
    fi
    
    # Check if there's a package script to create binary
    if npm run --silent 2>/dev/null | grep -q "package"; then
        log_info "Running package script..."
        if ! npm run package; then
            log_error "npm run package failed"
            return 1
        fi
    fi
    
    # Look for the binary in common locations
    local possible_paths=(
        "bin/bifrost"
        "dist/bifrost"
        "build/bifrost"
        "out/bifrost"
        "bifrost"
        "bin/bifrost.exe"
        "dist/bifrost.exe"
        "build/bifrost.exe"
        "out/bifrost.exe"
        "bifrost.exe"
    )
    
    for path in "${possible_paths[@]}"; do
        if [[ -f "$path" ]]; then
            BUILT_BINARY_PATH="$BIFROST_DIR/$path"
            log_info "Found binary at: $path"
            break
        fi
    done
    
    if [[ -z "${BUILT_BINARY_PATH:-}" ]]; then
        log_error "Could not find built binary in expected locations"
        log_error "Please check the Bifrost build output for the correct binary location"
        return 1
    fi
    
    log_info "Node.js build completed successfully"
}

# Build with Makefile
build_make_project() {
    log_info "Building with make..."
    
    if ! make; then
        log_error "Make build failed"
        return 1
    fi
    
    # Look for binary in common locations
    local possible_paths=(
        "bifrost"
        "bin/bifrost"
        "build/bifrost"
        "out/bifrost"
    )
    
    for path in "${possible_paths[@]}"; do
        if [[ -f "$path" ]]; then
            BUILT_BINARY_PATH="$BIFROST_DIR/$path"
            log_info "Found binary at: $path"
            break
        fi
    done
    
    if [[ -z "${BUILT_BINARY_PATH:-}" ]]; then
        log_error "Could not find built binary"
        return 1
    fi
    
    log_info "Make build completed successfully"
}

# Build with CMake
build_cmake_project() {
    log_info "Building with CMake..."
    
    mkdir -p build
    cd build
    
    if ! cmake ..; then
        log_error "CMake configuration failed"
        return 1
    fi
    
    if ! make; then
        log_error "CMake build failed"
        return 1
    fi
    
    # Look for binary
    if [[ -f "bifrost" ]]; then
        BUILT_BINARY_PATH="$BIFROST_DIR/build/bifrost"
    else
        log_error "Could not find built binary"
        return 1
    fi
    
    log_info "CMake build completed successfully"
}

# Copy binary to target location
install_binary() {
    log_info "Installing Bifrost binary..."
    
    if [[ -z "${BUILT_BINARY_PATH:-}" ]]; then
        log_error "No built binary path set"
        return 1
    fi
    
    if [[ ! -f "$BUILT_BINARY_PATH" ]]; then
        log_error "Built binary not found at: $BUILT_BINARY_PATH"
        return 1
    fi
    
    local target_path="$BINARIES_DIR/$BINARY_NAME"
    
    # Copy the binary
    if ! cp "$BUILT_BINARY_PATH" "$target_path"; then
        log_error "Failed to copy binary to target location"
        return 1
    fi
    
    # Make it executable
    chmod +x "$target_path"
    
    log_info "Binary installed successfully at: $target_path"
    
    # Verify the binary works
    if "$target_path" --version &>/dev/null || "$target_path" --help &>/dev/null; then
        log_info "Binary verification successful"
    else
        log_warn "Binary verification failed - binary may not be working correctly"
    fi
}

# Cleanup build artifacts (optional)
cleanup() {
    if [[ "${CLEANUP_BUILD:-true}" == "true" ]]; then
        log_info "Cleaning up build artifacts..."
        
        # Keep the source but clean build artifacts
        cd "$BIFROST_DIR"
        
        if [[ -f "Cargo.toml" ]]; then
            cargo clean &>/dev/null || true
        elif [[ -f "package.json" ]]; then
            rm -rf node_modules/.cache &>/dev/null || true
        fi
        
        log_info "Cleanup completed"
    fi
}

# Print build information
print_build_info() {
    log_info "=== Bifrost Build Information ==="
    log_info "Source repository: $BIFROST_REPO"
    log_info "Branch: $BIFROST_BRANCH"
    log_info "Target OS: $TARGET_OS"
    log_info "Target Architecture: $TARGET_ARCH"
    log_info "Binary location: $BINARIES_DIR/$BINARY_NAME"
    log_info "Build directory: $BIFROST_DIR"
    
    if [[ -f "$BINARIES_DIR/$BINARY_NAME" ]]; then
        local binary_size=$(stat -c%s "$BINARIES_DIR/$BINARY_NAME" 2>/dev/null || stat -f%z "$BINARIES_DIR/$BINARY_NAME" 2>/dev/null || echo "unknown")
        log_info "Binary size: $binary_size bytes"
        
        # Try to get version info
        local version_output=""
        if version_output=$("$BINARIES_DIR/$BINARY_NAME" --version 2>&1); then
            log_info "Binary version: $version_output"
        fi
    fi
    
    log_info "================================="
}

# Handle errors and cleanup
handle_error() {
    local exit_code=$?
    log_error "Build failed with exit code: $exit_code"
    
    # Additional error context
    if [[ -n "${BIFROST_DIR:-}" ]] && [[ -d "$BIFROST_DIR" ]]; then
        log_error "Build was attempted in: $BIFROST_DIR"
        
        # Show recent git commits for context
        if [[ -d "$BIFROST_DIR/.git" ]]; then
            log_info "Recent commits in source repository:"
            cd "$BIFROST_DIR"
            git log --oneline -5 || true
        fi
    fi
    
    exit $exit_code
}

# Main execution function
main() {
    log_info "Starting Bifrost source build process..."
    
    # Set up error handling
    trap handle_error ERR
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --branch)
                BIFROST_BRANCH="$2"
                shift 2
                ;;
            --no-cleanup)
                CLEANUP_BUILD="false"
                shift
                ;;
            --help)
                echo "Usage: $0 [options]"
                echo "Options:"
                echo "  --branch BRANCH    Use specific git branch (default: main)"
                echo "  --no-cleanup       Don't clean build artifacts after build"
                echo "  --help             Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Execute build steps
    check_dependencies
    setup_directories
    setup_repository
    build_bifrost
    install_binary
    cleanup
    print_build_info
    
    log_info "Bifrost build process completed successfully!"
}

# Run main function with all arguments
main "$@"