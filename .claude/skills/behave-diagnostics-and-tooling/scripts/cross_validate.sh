#!/bin/bash
# Cross-validate C++ and Rust implementations.
# Builds the C++ reference (if needed), runs tests on both, prints combined verdict.

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "=== Cross-Validation: C++ vs Rust ==="
echo ""

# Check for required tools
if ! command -v cmake &> /dev/null; then
  echo "ERROR: cmake not found. Install cmake and try again."
  exit 1
fi

if ! command -v cargo &> /dev/null; then
  echo "ERROR: cargo not found. Install Rust and try again."
  exit 1
fi

# Build C++ if needed
if [ ! -f build/testBehave ]; then
  echo "[1/3] Building C++ reference (make compile)..."
  make compile
else
  echo "[1/3] C++ reference already built (build/testBehave exists)"
fi
echo ""

# Run C++ tests
echo "[2/3] Running C++ testBehave..."
CPP_OUTPUT=$(./build/testBehave 2>&1 || true)
# Extract the summary line: "Total tests passed: N" / "Total tests failed: N"
CPP_PASS=$(echo "$CPP_OUTPUT" | grep "Total tests passed:" | awk '{print $NF}' || echo "?")
CPP_FAIL=$(echo "$CPP_OUTPUT" | grep "Total tests failed:" | awk '{print $NF}' || echo "?")
echo "$CPP_OUTPUT" | tail -20
echo ""

# Run Rust parity tests
echo "[3/3] Running Rust parity suite (cargo test -p behave-run --test parity)..."
RUST_OUTPUT=$(cargo test -p behave-run --test parity 2>&1 || true)
if echo "$RUST_OUTPUT" | grep -q "test result: ok"; then
  RUST_PASS="PASS"
  RUST_FAIL="0"
elif echo "$RUST_OUTPUT" | grep -q "failures:"; then
  RUST_PASS="FAIL"
  RUST_FAIL=$(echo "$RUST_OUTPUT" | grep "failures:" | head -1)
else
  RUST_PASS="UNKNOWN"
  RUST_FAIL="?"
fi
echo "$RUST_OUTPUT" | tail -30
echo ""

# Summary
echo "=== VERDICT ==="
echo "C++:  $CPP_PASS passed, $CPP_FAIL failed"
echo "Rust: $RUST_PASS ($RUST_FAIL)"
echo ""

if [ "$RUST_PASS" = "PASS" ] && [ "$CPP_FAIL" -eq 0 ]; then
  echo "✓ All cross-validation checks passed."
  exit 0
else
  echo "✗ Cross-validation FAILED. Review output above."
  exit 1
fi
