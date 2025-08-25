#!/bin/bash

# MindLink CI/CD Pipeline Validation Script
# This script validates the local environment matches CI/CD requirements

set -euo pipefail

echo "🔍 MindLink CI/CD Pipeline Validation"
echo "===================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check functions
check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "✅ ${GREEN}$1${NC} is installed"
        return 0
    else
        echo -e "❌ ${RED}$1${NC} is not installed"
        return 1
    fi
}

check_cargo_tool() {
    if cargo "$1" --help &> /dev/null; then
        echo -e "✅ ${GREEN}cargo $1${NC} is available"
        return 0
    else
        echo -e "❌ ${RED}cargo $1${NC} is not available"
        return 1
    fi
}

# Counter for failed checks
FAILED_CHECKS=0

echo ""
echo "📦 Checking Core Dependencies..."

# Core tools
check_command "cargo" || ((FAILED_CHECKS++))
check_command "rustc" || ((FAILED_CHECKS++))
check_command "node" || ((FAILED_CHECKS++))
check_command "npm" || ((FAILED_CHECKS++))
check_command "git" || ((FAILED_CHECKS++))

echo ""
echo "🛠️ Checking Rust Toolchain..."

# Rust components
if rustup component list --installed | grep -q "rustfmt"; then
    echo -e "✅ ${GREEN}rustfmt${NC} component installed"
else
    echo -e "❌ ${RED}rustfmt${NC} component not installed"
    ((FAILED_CHECKS++))
fi

if rustup component list --installed | grep -q "clippy"; then
    echo -e "✅ ${GREEN}clippy${NC} component installed" 
else
    echo -e "❌ ${RED}clippy${NC} component not installed"
    ((FAILED_CHECKS++))
fi

echo ""
echo "🔧 Checking Cargo Tools..."

# Cargo tools used in CI
CARGO_TOOLS=("audit" "deny" "outdated")

for tool in "${CARGO_TOOLS[@]}"; do
    check_cargo_tool "$tool" || echo -e "  ${YELLOW}Install with:${NC} cargo install cargo-$tool"
done

# Check if tarpaulin is available (optional)
if check_cargo_tool "tarpaulin"; then
    echo -e "  ${GREEN}Coverage tool ready${NC}"
else
    echo -e "  ${YELLOW}Install with:${NC} cargo install cargo-tarpaulin"
fi

echo ""
echo "📁 Checking Project Structure..."

# Check critical files exist
REQUIRED_FILES=(
    "src-tauri/Cargo.toml"
    "src-tauri/tauri.conf.json"
    "src-tauri/deny.toml"
    "src-tauri/tarpaulin.toml"
    ".github/workflows/release.yml"
    "package.json"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        echo -e "✅ ${GREEN}$file${NC} exists"
    else
        echo -e "❌ ${RED}$file${NC} missing"
        ((FAILED_CHECKS++))
    fi
done

echo ""
echo "🧪 Running Quick Validation Tests..."

# Test formatting
cd src-tauri
if cargo fmt --check > /dev/null 2>&1; then
    echo -e "✅ ${GREEN}Code formatting${NC} is correct"
else
    echo -e "❌ ${RED}Code formatting${NC} issues found"
    echo -e "  ${YELLOW}Fix with:${NC} cargo fmt"
    ((FAILED_CHECKS++))
fi

# Test basic clippy (quick check)
echo "🔍 Running basic clippy check..."
if timeout 30s cargo clippy --quiet --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    echo -e "✅ ${GREEN}Basic clippy${NC} passes"
else
    echo -e "❌ ${RED}Clippy warnings${NC} found"
    echo -e "  ${YELLOW}Check with:${NC} cargo clippy --all-targets --all-features -- -D warnings"
    ((FAILED_CHECKS++))
fi

# Test if tests compile (don't run them, just check compilation)
echo "🔍 Checking test compilation..."
if timeout 30s cargo test --no-run --quiet > /dev/null 2>&1; then
    echo -e "✅ ${GREEN}Tests compile${NC} successfully"
else
    echo -e "❌ ${RED}Test compilation${NC} failed"
    echo -e "  ${YELLOW}Check with:${NC} cargo test --no-run"
    ((FAILED_CHECKS++))
fi

cd ..

echo ""
echo "🔐 Checking Security Configuration..."

# Check deny.toml exists and is valid
if [[ -f "src-tauri/deny.toml" ]]; then
    cd src-tauri
    if cargo deny --help > /dev/null 2>&1; then
        if timeout 30s cargo deny check --config deny.toml > /dev/null 2>&1; then
            echo -e "✅ ${GREEN}Security policy${NC} validation passes"
        else
            echo -e "⚠️  ${YELLOW}Security policy${NC} has warnings/errors"
            echo -e "  ${YELLOW}Check with:${NC} cargo deny check"
        fi
    else
        echo -e "⚠️  ${YELLOW}cargo deny${NC} not installed (optional for local dev)"
    fi
    cd ..
fi

echo ""
echo "📊 Final Results"
echo "==============="

if [[ $FAILED_CHECKS -eq 0 ]]; then
    echo -e "🎉 ${GREEN}All validation checks passed!${NC}"
    echo -e "✅ Your environment is ready for the CI/CD pipeline"
    echo ""
    echo "🚀 Ready to:"
    echo "  - Create pull requests (will trigger quality gates)"
    echo "  - Push to main (will run full pipeline + benchmarks)"
    echo "  - Create release tags (will build and deploy)"
    echo ""
    echo "📝 To run a full local test similar to CI:"
    echo "  ./scripts/generate-coverage.sh"
    echo "  cargo clippy --all-targets --all-features -- -D warnings"
    echo "  cargo test --all-features"
    exit 0
else
    echo -e "⚠️  ${YELLOW}$FAILED_CHECKS validation check(s) failed${NC}"
    echo -e "🔧 Please address the issues above before using CI/CD pipeline"
    echo ""
    echo "🚀 Quick fixes:"
    echo "  rustup component add rustfmt clippy"
    echo "  cargo install cargo-audit cargo-deny cargo-tarpaulin"
    echo "  cargo fmt"
    echo ""
    exit 1
fi