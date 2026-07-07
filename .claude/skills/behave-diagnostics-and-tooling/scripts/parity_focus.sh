#!/bin/bash
# Find which test function owns a parity check by name pattern.
# Usage: parity_focus.sh <pattern>
# Example: parity_focus.sh "scorch"

if [ $# -eq 0 ]; then
  echo "Usage: parity_focus.sh <pattern>"
  echo ""
  echo "Searches crates/behave-run/tests/parity.rs for check names matching"
  echo "<pattern> (case-insensitive) and prints the surrounding test function."
  echo ""
  echo "Example:"
  echo "  parity_focus.sh scorch"
  echo "  parity_focus.sh 'moisture'"
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
PARITY_FILE="$REPO_ROOT/crates/behave-run/tests/parity.rs"

if [ ! -f "$PARITY_FILE" ]; then
  echo "ERROR: $PARITY_FILE not found."
  exit 1
fi

PATTERN="$1"

# Extract check names from patterns like:
#   t.check(
#       "name",
# or: t.check_bool("name",
# then filter by pattern and print with function context
grep -B 200 "\.check(" "$PARITY_FILE" | grep -B 200 "\"" | awk -v pat="$PATTERN" '
  # Track function names as we parse
  /^fn test_/ {
    current_fn = $0
    gsub(/^fn /, "", current_fn)
    gsub(/\(.*$/, "", current_fn)
  }

  # Look for quoted strings that appear right after t.check or t.check_bool
  /\.check\(/ || /\.check_bool\(/ {
    # If the quote is on the same line as .check, extract it
    if (match($0, /"([^"]*)"/, arr)) {
      check_name = arr[1]
      if (tolower(check_name) ~ tolower(pat)) {
        printf "%s\n  ├─ %s\n", current_fn, check_name
      }
    }
  }

  # Also check the next line if .check( is alone
  /^[[:space:]]*"([^"]*)"/ && prev_was_check {
    if (match($0, /"([^"]*)"/, arr)) {
      check_name = arr[1]
      if (tolower(check_name) ~ tolower(pat)) {
        printf "%s\n  ├─ %s\n", current_fn, check_name
      }
    }
    prev_was_check = 0
  }

  /\.check\($/ { prev_was_check = 1 }
'
