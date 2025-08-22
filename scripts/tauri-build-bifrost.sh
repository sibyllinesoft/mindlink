#!/bin/bash

# Tauri Integration Script for Bifrost Source Build
# This script is designed to be called by the Tauri build process
# to ensure Bifrost is available before building the application

set -euo pipefail

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Source configuration
source "$SCRIPT_DIR/bifrost-build.config.sh"

# Logging functions
log_info() {
    echo "[TAURI-BUILD] $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo "[TAURI-BUILD] $(date '+%Y-%m-%d %H:%M:%S') - ERROR: $1" >&2
}

log_warn() {
    echo "[TAURI-BUILD] $(date '+%Y-%m-%d %H:%M:%S') - WARN: $1" >&2
}

# Check if Bifrost binary already exists and is valid
check_existing_binary() {
    local binary_path="$PROJECT_ROOT/$BIFROST_BINARIES_DIR/$BIFROST_BINARY_NAME"
    
    if [[ -f "$binary_path" ]]; then
        log_info "Found existing Bifrost binary at: $binary_path"
        
        # Verify the binary works
        if "$binary_path" --version &>/dev/null || "$binary_path" --help &>/dev/null; then
            log_info "Existing binary is functional, skipping build"
            return 0
        else
            log_warn "Existing binary failed verification, will rebuild"
            rm -f "$binary_path"
            return 1
        fi
    else
        log_info "No existing Bifrost binary found"
        return 1
    fi
}

# Quick build mode - only update if repository has changes
check_for_updates() {
    local source_dir="$PROJECT_ROOT/$BIFROST_SOURCE_DIR"
    
    if [[ ! -d "$source_dir" ]]; then
        log_info "Source directory not found, full build required"
        return 1
    fi
    
    cd "$source_dir"
    
    # Fetch latest changes
    if ! git fetch origin &>/dev/null; then
        log_warn "Failed to fetch updates, proceeding with existing source"
        return 0
    fi
    
    # Check if local is behind remote
    local local_commit=$(git rev-parse HEAD)
    local remote_commit=$(git rev-parse origin/$BIFROST_BRANCH)
    
    if [[ "$local_commit" != "$remote_commit" ]]; then
        log_info "Updates available, rebuild required"
        return 1
    else
        log_info "Source is up to date"
        return 0
    fi
}

# Force rebuild mode
force_rebuild() {
    log_info "Force rebuilding Bifrost..."
    
    # Remove existing binary
    local binary_path="$PROJECT_ROOT/$BIFROST_BINARIES_DIR/$BIFROST_BINARY_NAME"
    rm -f "$binary_path"
    
    # Run the build script
    "$SCRIPT_DIR/build-bifrost.sh"
}

# Quick build mode - check for updates first
quick_build() {
    log_info "Running quick build check..."
    
    if check_existing_binary; then
        if check_for_updates; then
            log_info "No rebuild necessary"
            return 0
        fi
    fi
    
    log_info "Rebuild required, starting build process..."
    "$SCRIPT_DIR/build-bifrost.sh"
}

# Verify the final binary is in place
verify_installation() {
    local binary_path="$PROJECT_ROOT/$BIFROST_BINARIES_DIR/$BIFROST_BINARY_NAME"
    
    if [[ ! -f "$binary_path" ]]; then
        log_error "Bifrost binary not found after build: $binary_path"
        return 1
    fi
    
    if [[ ! -x "$binary_path" ]]; then
        log_error "Bifrost binary is not executable: $binary_path"
        return 1
    fi
    
    log_info "Bifrost binary successfully installed and verified"
    
    # Print binary info
    local binary_size=$(stat -c%s "$binary_path" 2>/dev/null || stat -f%z "$binary_path" 2>/dev/null || echo "unknown")
    log_info "Binary size: $binary_size bytes"
    
    return 0
}

# Create minimal build status file for Tauri
create_build_status() {
    local status_file="$PROJECT_ROOT/$BIFROST_BINARIES_DIR/bifrost-build-status.json"
    local binary_path="$PROJECT_ROOT/$BIFROST_BINARIES_DIR/$BIFROST_BINARY_NAME"
    
    mkdir -p "$(dirname "$status_file")"
    
    cat > "$status_file" << EOF
{
  "buildTime": "$(date -Iseconds)",
  "binaryPath": "$binary_path",
  "binaryName": "$BIFROST_BINARY_NAME",
  "sourceRepo": "$BIFROST_REPO_URL",
  "sourceBranch": "$BIFROST_BRANCH",
  "targetOS": "$BIFROST_TARGET_OS",
  "targetArch": "$BIFROST_TARGET_ARCH",
  "buildType": "$BIFROST_BUILD_TYPE"
}
EOF

    log_info "Build status saved to: $status_file"
}

# Main execution
main() {
    log_info "Starting Tauri Bifrost build integration..."
    
    # Parse command line arguments
    local build_mode="quick"  # default mode
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --force)
                build_mode="force"
                shift
                ;;
            --quick)
                build_mode="quick"
                shift
                ;;
            --check-only)
                build_mode="check"
                shift
                ;;
            --help)
                echo "Usage: $0 [options]"
                echo "Options:"
                echo "  --force       Force rebuild even if binary exists"
                echo "  --quick       Quick build (check for updates first) [default]"
                echo "  --check-only  Only check if binary exists, don't build"
                echo "  --help        Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
    
    # Validate configuration
    if ! validate_config; then
        log_error "Configuration validation failed"
        exit 1
    fi
    
    # Execute based on mode
    case "$build_mode" in
        "force")
            force_rebuild
            ;;
        "quick")
            quick_build
            ;;
        "check")
            if check_existing_binary; then
                log_info "Binary check passed"
                exit 0
            else
                log_error "Binary check failed"
                exit 1
            fi
            ;;
    esac
    
    # Verify installation
    if ! verify_installation; then
        log_error "Installation verification failed"
        exit 1
    fi
    
    # Create build status
    create_build_status
    
    log_info "Tauri Bifrost build integration completed successfully"
}

# Run main with all arguments
main "$@"