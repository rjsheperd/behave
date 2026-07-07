#!/bin/bash
# One-liner health/drift report: counts that change when code drifts.

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "=== Behave Codebase Health Report ==="
echo "(as of $(date '+%Y-%m-%d %H:%M:%S'))"
echo ""

# Cargo test count
echo "UNIT TESTS (cargo test --lib --all 2>&1 | tail -1):"
cargo test --lib --all 2>&1 | grep -E "^test result" | awk '{p+=$4; f+=$6} END {print "  " p " passed, " f " failed (across crates)"}' || echo "  (error running cargo test)"
echo ""

# Parity check CALL SITES (static grep). Runtime count is higher (171 as of
# 2026-07-06) because four sites loop; authoritative runtime count:
#   cargo test -p behave-run --test parity -- --nocapture 2>&1 | grep 'parity:'
PARITY_COUNT=$(grep -c "\.check\|\.check_bool" crates/behave-run/tests/parity.rs 2>/dev/null || echo 0)
echo "PARITY CHECK CALL SITES (grep in parity.rs; runtime count 171 — loops):"
echo "  $PARITY_COUNT call sites"
echo ""

# TODO/todo! count in source code
TODO_COUNT=$(find crates -name "*.rs" -type f ! -path "*/tests/*" -exec grep -h "todo!\|TODO" {} \; 2>/dev/null | wc -l || echo 0)
echo "TODO!/TODO MARKERS (non-test lib code):"
echo "  $TODO_COUNT markers"
echo ""

# Panic count in non-test lib code (approximation: searches for panic!, unwrap(), expect())
# Excludes test modules, panics in tests are acceptable
PANIC_COUNT=$(find crates -name "*.rs" -type f ! -path "*/tests/*" -exec grep -h "panic!\|\.unwrap()\|\.expect(" {} \; 2>/dev/null | wc -l || echo 0)
echo "PANIC/UNWRAP/EXPECT (non-test lib code):"
echo "  $PANIC_COUNT occurrences (approximation: includes comments, strings)"
echo "  NOTE: Searches for literal panic!/, .unwrap(), .expect(—does not"
echo "  distinguish panic in tests (acceptable) vs lib code (problematic)."
echo ""

# Source file count
SRC_FILES=$(find crates -name "*.rs" -type f ! -path "*/tests/*" | wc -l)
TEST_FILES=$(find crates -name "*.rs" -type f -path "*/tests/*" | wc -l)
echo "FILE COUNTS:"
echo "  $SRC_FILES source files (non-test)"
echo "  $TEST_FILES test files"
echo ""

# Line counts (rough)
SRC_LINES=$(find crates -name "*.rs" -type f ! -path "*/tests/*" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
TEST_LINES=$(find crates -name "*.rs" -type f -path "*/tests/*" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
echo "LINE COUNTS (rough):"
echo "  ~$SRC_LINES lines in source code"
echo "  ~$TEST_LINES lines in test code"
echo ""

# Compiler/warning check
echo "COMPILER STATUS (cargo build --workspace 2>&1 | tail -3):"
cargo build --workspace 2>&1 | tail -3 || echo "  (error building)"
