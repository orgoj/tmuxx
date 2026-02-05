#!/bin/bash
# Full test suite for tmuxx - runs format, lint, unit tests, and fixture tests
# Usage: ./scripts/check-all.sh [--fix]
#   --fix: Auto-fix formatting issues instead of just checking

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

FIX_MODE=false
if [[ "$1" == "--fix" ]]; then
    FIX_MODE=true
fi

echo "=========================================="
echo "  tmuxx Full Test Suite"
echo "=========================================="
echo ""

# Step 1: Format check
echo "1/4 Checking formatting..."
if $FIX_MODE; then
    cargo fmt
    echo "    [FIXED] Formatting applied"
else
    if ! cargo fmt --check; then
        echo ""
        echo "    [FAIL] Formatting issues found. Run './scripts/check-all.sh --fix' to auto-fix."
        exit 1
    fi
fi
echo "    [PASS] Formatting OK"
echo ""

# Step 2: Clippy lint
echo "2/4 Running clippy..."
if ! cargo clippy -- -D warnings 2>&1; then
    echo ""
    echo "    [FAIL] Clippy found issues"
    exit 1
fi
echo "    [PASS] Clippy OK"
echo ""

# Step 3: Unit tests
echo "3/4 Running unit tests..."
if ! cargo test --quiet; then
    echo ""
    echo "    [FAIL] Unit tests failed"
    exit 1
fi
echo "    [PASS] Unit tests OK"
echo ""

# Step 4: Fixture regression tests
echo "4/4 Running fixture tests..."
# Build release for faster fixture tests
cargo build --release --quiet
if ! ./target/release/tmuxx test 2>&1 | tee /tmp/fixture-results.txt | tail -5; then
    echo ""
fi

# Check if all passed
if grep -q "0 Failed" /tmp/fixture-results.txt; then
    echo "    [PASS] Fixture tests OK"
else
    FAILED_COUNT=$(grep -oP '\d+ Failed' /tmp/fixture-results.txt | grep -oP '\d+')
    echo ""
    echo "    [FAIL] $FAILED_COUNT fixture test(s) failed"
    echo "    Run 'cargo run -- test -d' for details"
    exit 1
fi

echo ""
echo "=========================================="
echo "  All checks passed!"
echo "=========================================="
