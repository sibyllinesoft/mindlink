#!/bin/bash
# MindLink E2E Test Runner
# 
# This script runs end-to-end tests for the MindLink application.
# It handles application startup, test execution, and cleanup.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LOG_DIR="$PROJECT_ROOT/target/e2e-logs"
TEST_TIMEOUT=300  # 5 minutes

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test environment..."
    
    # Kill any running MindLink processes
    pkill -f "mindlink" || true
    
    # Kill any processes on test ports
    lsof -ti:3001 | xargs -r kill -9 || true
    
    log_info "Cleanup completed"
}

# Trap to ensure cleanup runs on exit
trap cleanup EXIT

# Create log directory
mkdir -p "$LOG_DIR"

log_info "Starting MindLink E2E Test Suite"
log_info "Project root: $PROJECT_ROOT"
log_info "Log directory: $LOG_DIR"

# Step 1: Build the application
log_info "Building MindLink application..."
cd "$PROJECT_ROOT"

if ! cargo build --release > "$LOG_DIR/build.log" 2>&1; then
    log_error "Failed to build application. Check $LOG_DIR/build.log for details."
    exit 1
fi

log_success "Application built successfully"

# Step 2: Check if tauri-driver is available
if ! command -v tauri-driver &> /dev/null; then
    log_warning "tauri-driver not found. Installing..."
    if ! cargo install tauri-driver > "$LOG_DIR/driver-install.log" 2>&1; then
        log_error "Failed to install tauri-driver. E2E tests may fail."
    else
        log_success "tauri-driver installed successfully"
    fi
fi

# Step 3: Run unit tests first
log_info "Running unit tests before E2E tests..."
if ! cargo test --lib > "$LOG_DIR/unit-tests.log" 2>&1; then
    log_error "Unit tests failed. Check $LOG_DIR/unit-tests.log for details."
    exit 1
fi
log_success "Unit tests passed"

# Step 4: Run integration tests
log_info "Running integration tests..."
if ! cargo test --test '*' > "$LOG_DIR/integration-tests.log" 2>&1; then
    log_warning "Some integration tests failed. Check $LOG_DIR/integration-tests.log"
    # Don't exit - E2E tests might still work
fi

# Step 5: Run E2E tests
log_info "Starting E2E tests..."

# Test categories to run
E2E_TEST_PATTERNS=(
    "test_dashboard_ui_loads"
    "test_service_control_workflow"
    "test_api_testing_interface"
    "test_realtime_status_updates"
    "test_application_lifecycle"
    "test_complete_user_workflow"
    "test_server_health_endpoint"
    "test_chat_completions_endpoint"
    "test_streaming_completions"
    "test_models_endpoint"
    "test_api_error_handling"
    "test_cors_headers"
    "test_complete_api_workflow"
)

# Track test results
PASSED_TESTS=0
FAILED_TESTS=0
TOTAL_TESTS=${#E2E_TEST_PATTERNS[@]}

log_info "Running $TOTAL_TESTS E2E test categories..."

# Run each test category
for test_pattern in "${E2E_TEST_PATTERNS[@]}"; do
    log_info "Running test: $test_pattern"
    
    if timeout "$TEST_TIMEOUT" cargo test "$test_pattern" -- --nocapture > "$LOG_DIR/${test_pattern}.log" 2>&1; then
        log_success "✅ $test_pattern PASSED"
        ((PASSED_TESTS++))
    else
        log_error "❌ $test_pattern FAILED (see $LOG_DIR/${test_pattern}.log)"
        ((FAILED_TESTS++))
    fi
done

# Step 6: Generate test report
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
REPORT_FILE="$LOG_DIR/e2e-test-report.md"

cat > "$REPORT_FILE" << EOF
# MindLink E2E Test Report

**Generated:** $TIMESTAMP  
**Total Tests:** $TOTAL_TESTS  
**Passed:** $PASSED_TESTS  
**Failed:** $FAILED_TESTS  
**Success Rate:** $(( (PASSED_TESTS * 100) / TOTAL_TESTS ))%

## Test Results

EOF

# Add individual test results to report
for test_pattern in "${E2E_TEST_PATTERNS[@]}"; do
    if [ -f "$LOG_DIR/${test_pattern}.log" ]; then
        if grep -q "test result: ok" "$LOG_DIR/${test_pattern}.log"; then
            echo "✅ **$test_pattern** - PASSED" >> "$REPORT_FILE"
        else
            echo "❌ **$test_pattern** - FAILED" >> "$REPORT_FILE"
        fi
    else
        echo "⚠️ **$test_pattern** - NO LOG" >> "$REPORT_FILE"
    fi
done

cat >> "$REPORT_FILE" << EOF

## Log Files

All detailed test logs are available in: \`$LOG_DIR/\`

- Build log: \`build.log\`
- Unit tests: \`unit-tests.log\`
- Integration tests: \`integration-tests.log\`
- Individual E2E test logs: \`{test_name}.log\`

## Notes

- E2E tests require the application to be built and may start their own instances
- Some tests may fail if authentication is required but not available
- Network-dependent tests may fail in restricted environments
- UI tests require a display and may fail in headless environments

EOF

log_info "Test report generated: $REPORT_FILE"

# Step 7: Summary
echo
log_info "=== E2E Test Suite Summary ==="
log_info "Total Tests: $TOTAL_TESTS"
log_success "Passed: $PASSED_TESTS"

if [ "$FAILED_TESTS" -gt 0 ]; then
    log_error "Failed: $FAILED_TESTS"
    log_error "Some E2E tests failed. Check individual logs for details."
    exit 1
else
    log_success "All E2E tests passed!"
    exit 0
fi