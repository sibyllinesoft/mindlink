# Scripts Directory

This directory contains build and automation scripts for the MindLink project.

## Bifrost Source Build Scripts

### Core Scripts

- **`build-bifrost.sh`** - Main script that builds Bifrost from source
- **`tauri-build-bifrost.sh`** - Tauri integration wrapper
- **`bifrost-build.config.sh`** - Configuration file
- **`test-bifrost-build.sh`** - Test and validation script

### Usage

```bash
# Test the build system setup
npm run bifrost:test

# Build Bifrost from source
npm run bifrost:build

# Force rebuild (ignore existing binary)
npm run bifrost:build:force

# Test with dry run
npm run bifrost:test:dry
```

### Manual Usage

```bash
# Basic build
./scripts/build-bifrost.sh

# Build specific branch
./scripts/build-bifrost.sh --branch stable

# Tauri integration (quick check)
./scripts/tauri-build-bifrost.sh --quick

# Force rebuild
./scripts/tauri-build-bifrost.sh --force
```

## Script Details

### build-bifrost.sh

Main build script that:
- Clones/updates Bifrost repository
- Detects build system (Cargo, npm, Make, CMake)
- Builds the project for current platform
- Installs binary to `src-tauri/binaries/`
- Includes comprehensive error handling and logging

### tauri-build-bifrost.sh

Tauri integration script that:
- Provides integration with Tauri build process
- Supports quick builds (checks for updates first)
- Handles build verification and status tracking
- Used by `beforeBuildCommand` in tauri.conf.json

### bifrost-build.config.sh

Configuration file with:
- Repository settings (URL, branch)
- Build options (type, parallel jobs)
- Platform-specific settings
- Environment variable defaults

### test-bifrost-build.sh

Validation script that:
- Checks dependencies and permissions
- Validates configuration
- Tests directory structure
- Provides setup guidance

## Integration

The scripts are integrated with:

1. **Tauri Build Process** - Automatically builds Bifrost before Tauri builds
2. **npm Scripts** - Convenient commands in package.json
3. **Binary Manager** - Rust code can find the built binary
4. **Git Workflow** - Build artifacts properly ignored

## Configuration

Customize the build by editing `bifrost-build.config.sh` or setting environment variables:

```bash
export BIFROST_BRANCH="stable"
export BIFROST_BUILD_TYPE="debug"
export BIFROST_CLEANUP_AFTER_BUILD="false"
```

## Troubleshooting

1. Run the test script: `npm run bifrost:test`
2. Check dependencies are installed (git, node, npm)
3. Verify repository access and branch availability
4. Use force rebuild: `npm run bifrost:build:force`
5. Check logs for detailed error information

For more information, see `../BIFROST_SOURCE_BUILD.md`.