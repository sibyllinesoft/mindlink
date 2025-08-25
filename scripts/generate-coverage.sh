#!/bin/bash
# Generate Code Coverage Report for MindLink Rust Application
# This script provides comprehensive coverage reporting with multiple output formats

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and project paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TAURI_DIR="$PROJECT_DIR/src-tauri"
COVERAGE_DIR="$PROJECT_DIR/coverage"

# Configuration
MIN_COVERAGE=80
TARGET_COVERAGE=85
TIMEOUT=120

echo -e "${BLUE}🔍 MindLink Code Coverage Generator${NC}"
echo "========================================"
echo ""

# Check if cargo-tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}⚠️  Installing cargo-tarpaulin...${NC}"
    cargo install cargo-tarpaulin
    echo ""
fi

# Verify we're in the correct directory
if [[ ! -f "$TAURI_DIR/Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Not in MindLink project directory${NC}"
    echo "Please run this script from the project root or scripts directory"
    exit 1
fi

# Clean previous coverage data
echo -e "${BLUE}🧹 Cleaning previous coverage data...${NC}"
rm -rf "$COVERAGE_DIR"
mkdir -p "$COVERAGE_DIR"

# Change to Tauri directory
cd "$TAURI_DIR"

# Run coverage with comprehensive options
echo -e "${BLUE}📊 Generating coverage report...${NC}"
echo "Target coverage: ${TARGET_COVERAGE}%"
echo "Minimum acceptable: ${MIN_COVERAGE}%"
echo ""

# Generate XML for CI integration (Codecov)
echo -e "${YELLOW}📋 Generating XML report for CI integration...${NC}"
cargo tarpaulin \
    --config ci \
    --verbose \
    --all-features \
    --workspace \
    --timeout $TIMEOUT \
    --target-dir target/tarpaulin \
    --exclude-files "src/tests/*" \
    --exclude-files "src/main.rs" \
    --exclude-files "tests/*" \
    --out xml \
    --output-dir "../coverage" \
    --fail-under $MIN_COVERAGE \
    2>&1 | tee "../coverage/tarpaulin.log"

# Generate HTML for local viewing
echo -e "${YELLOW}🌐 Generating HTML report for local viewing...${NC}"
cargo tarpaulin \
    --config ci \
    --verbose \
    --all-features \
    --workspace \
    --timeout $TIMEOUT \
    --target-dir target/tarpaulin \
    --exclude-files "src/tests/*" \
    --exclude-files "src/main.rs" \
    --exclude-files "tests/*" \
    --out html \
    --output-dir "../coverage" \
    2>&1 | tee -a "../coverage/tarpaulin.log"

# Generate JSON for programmatic access
echo -e "${YELLOW}📊 Generating JSON report for analysis...${NC}"
cargo tarpaulin \
    --config ci \
    --verbose \
    --all-features \
    --workspace \
    --timeout $TIMEOUT \
    --target-dir target/tarpaulin \
    --exclude-files "src/tests/*" \
    --exclude-files "src/main.rs" \
    --exclude-files "tests/*" \
    --out json \
    --output-dir "../coverage" \
    2>&1 | tee -a "../coverage/tarpaulin.log"

# Parse coverage percentage from the output
COVERAGE_PERCENT=$(grep -oP 'Coverage Results:\s+\K[\d.]+(?=%)' "../coverage/tarpaulin.log" | tail -1 || echo "0")

echo ""
echo "========================================"
echo -e "${BLUE}📈 Coverage Analysis Complete${NC}"
echo "========================================"
echo ""

# Display results with color coding
if (( $(echo "$COVERAGE_PERCENT >= $TARGET_COVERAGE" | bc -l) )); then
    echo -e "${GREEN}✅ Coverage: ${COVERAGE_PERCENT}% (Excellent - Above target of ${TARGET_COVERAGE}%)${NC}"
elif (( $(echo "$COVERAGE_PERCENT >= $MIN_COVERAGE" | bc -l) )); then
    echo -e "${YELLOW}⚠️  Coverage: ${COVERAGE_PERCENT}% (Good - Above minimum of ${MIN_COVERAGE}%)${NC}"
else
    echo -e "${RED}❌ Coverage: ${COVERAGE_PERCENT}% (Below minimum of ${MIN_COVERAGE}%)${NC}"
    echo -e "${RED}   Please add more tests to improve coverage${NC}"
fi

echo ""
echo "📁 Coverage Reports Generated:"
echo "   • XML (CI): $COVERAGE_DIR/cobertura.xml"
echo "   • HTML: $COVERAGE_DIR/tarpaulin-report.html"
echo "   • JSON: $COVERAGE_DIR/tarpaulin-report.json"
echo "   • Logs: $COVERAGE_DIR/tarpaulin.log"
echo ""

# Open HTML report in default browser (if running locally)
if [[ "${CI:-false}" != "true" ]] && command -v xdg-open &> /dev/null; then
    echo -e "${BLUE}🌐 Opening coverage report in browser...${NC}"
    xdg-open "$COVERAGE_DIR/tarpaulin-report.html" || true
elif [[ "${CI:-false}" != "true" ]] && command -v open &> /dev/null; then
    echo -e "${BLUE}🌐 Opening coverage report in browser...${NC}"
    open "$COVERAGE_DIR/tarpaulin-report.html" || true
fi

# Display coverage summary table
echo "📊 Coverage Summary:"
echo "   • Lines Covered: $(grep -oP 'lines:\s+\K\d+' "../coverage/tarpaulin.log" | tail -1 || echo "N/A")"
echo "   • Total Lines: $(grep -oP 'out of \K\d+' "../coverage/tarpaulin.log" | tail -1 || echo "N/A")"
echo ""

# Exit with appropriate code
if (( $(echo "$COVERAGE_PERCENT >= $MIN_COVERAGE" | bc -l) )); then
    echo -e "${GREEN}🎉 Coverage check passed!${NC}"
    exit 0
else
    echo -e "${RED}💥 Coverage check failed!${NC}"
    exit 1
fi