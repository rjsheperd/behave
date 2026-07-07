---
name: behave-run-and-operate
description: Running the Behave C++ and Rust test suites, example clients, documentation pipeline, and artifact conventions. Use when running tests, building docs, checking 'is it working', or understanding where outputs go.
---

# behave-run-and-operate

Operational runbook for Behave testing, building, running clients, and artifact handling. HOME for: how to invoke each test/build command, expected output formats, where outputs land, and the one-command health check.

## C++ Build & Test

### Setup

```bash
make compile
```

Invokes CMake + clang to build all C++ executables into `build/` (gitignored). Produces:

- **./build/testBehave** — C++ golden suite (default ON)
- **./build/behave** — example client (default ON)
- **./build/testMortality** — mortality module test (default ON)
- **./build/compute_spot_distance_{pile,surface,trees}** — spot fire CLIs (default OFF; see CMake options below)
- **./build/behave-raws-batch** — RAWS data batch reader (default OFF)

### Run C++ Tests

```bash
make test
```

Runs `./build/testBehave`. Exit code 0 = all pass. Summary line format (verified 171 tests):

```
Total tests performed: 171
Total tests passed: 171
Total tests failed: 0
```

Color output (ANSI codes) included in stdout; strip with `| sed 's/\x1b\[[0-9;]*m//g'` if needed.

### Run Mortality Tests

```bash
make test_mortality
```

Invokes: `./build/testMortality src/testMortality//FOFEM_input.tre src/testMortality//FOFEM_Mortality_Output.csv results.csv`

Reads FOFEM reference data from `src/testMortality/FOFEM_input.tre` and `src/testMortality/FOFEM_Mortality_Output.csv`, writes results to `results.csv` (gitignored; safe to delete). Prints:

```
Starting tests with:
Input File:src/testMortality//FOFEM_input.tre
Output File:src/testMortality//FOFEM_Mortality_Output.csv
Result File:results.csv
```

### Example Client: ./build/behave

Hardcoded scenario: two-fuel models (FM1 + GS4/FM124) at various coverage percentages. Output format:

```
Wind and spread direction are in degrees clockwise relative to upslope

Spread rate for the two fuel models 1 and 124 with first fuel coverage 0%
is 1.09804 ch/hr
Flame length for the two fuel models 1 and 124 is 2.68981 ft
Direction of maximum spread is 180 degrees

[repeats for 10%, 20%, 30%, ... coverage]
```

Default behavior only; this is a hardcoded demo, not a configurable tool.

### CMake Build Options

Disable/enable via `-DFLAG=ON|OFF` during `cmake -B build`:

```bash
# Default ON — always built by "make compile"
TEST_BEHAVE       # ./build/testBehave
TEST_MORTALITY    # ./build/testMortality
EXAMPLE_APP       # ./build/behave

# Default OFF — opt in
RAWS_BATCH                  # ./build/behave-raws-batch
COMPUTE_SPOT_PILE           # ./build/compute_spot_distance_pile
COMPUTE_SPOT_SURFACE        # ./build/compute_spot_distance_surface
COMPUTE_SPOT_TORCHING_TREES # ./build/compute_spot_distance_trees
```

Example:

```bash
cmake -B build -DCOMPUTE_SPOT_PILE=ON -DCOMPUTE_SPOT_SURFACE=ON
cmake --build build
```

## Rust Build & Test

Rust workspace is in `crates/` with 9 crates (firelab-base, behave-surface, behave-crown, behave-spot, behave-contain, behave-ignite, behave-mortality, behave-weather, behave-run). Edition 2021, all v0.1.0, resolver=2. **No binary targets** — the facade (`crates/behave-run`) is a library only.

### Run All Tests

```bash
cargo test --workspace
```

Summary (verified 283 unit tests):

```
test result: ok. 283 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Runs unit tests across all 9 crates plus doc-tests (currently 0 doc tests). Typical runtime ~0.5s on M-series macOS.

### Run Tests per Crate

```bash
cargo test -p <crate-name>
```

Example: `cargo test -p behave-surface` → 132 tests in behave-surface lib.

### Run Parity Suite Only

```bash
cargo test -p behave-run --test parity
```

Golden-value test against C++ reference suite: `tests/parity.rs` (1 test function, 171 embedded checks). Replicates exact call sequence from C++ `src/testBehave/testBehave.cpp` against one shared `BehaveRun`. Tolerance 1e-6 (1e-3 for VPD). Exit code 0 = all 171 checks match C++ outputs.

```
running 1 test
test parity_with_cpp_test_behave ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### Run Single Parity Test (by Name)

```bash
cargo test -p behave-run --test parity <test_name> -- --exact
```

Parity suite has no sub-tests (single `#[test]` function); all 171 checks run together. To filter by output, use grep:

```bash
cargo test -p behave-run --test parity 2>&1 | grep -i "surface"
```

### Ignored Tests

```bash
cargo test --workspace -- --ignored
```

Currently: **0 ignored tests**. No gap tests marked `#[ignore]` yet.

### Generate Rust API Docs

```bash
cargo doc --workspace --no-deps
```

Outputs to `target/doc/`. Warning: unresolved link in behave-surface chaparral.rs (non-fatal). Open `target/doc/firelab_base/index.html` in browser.

## Documentation Pipeline

### Build Docs Locally

```bash
make gendocs
```

Runs Doxygen (reads `Doxyfile`, outputs HTML to `docs/`). Generated files are tracked in git (consequence: docs/ branch churns on master pushes). Safe to run anytime; output is deterministic given source.

### CI Docs Auto-Commit

On every push to master, GitHub Actions (`.github/workflows/docs.yml`):

1. Checks out repo
2. Runs `mattnotmitt/doxygen-action` (v1.9.8)
3. Commits docs/ with message: `Generated Docs (commit: <HEAD_SHA>)`
4. Pushes back to master

**Consequence**: docs/ will diverge from local changes before you push. Avoid rebasing over master after a docs commit unless you `git pull` first. Safe to rebase a feature branch onto a pre-docs commit if you know HEAD hash.

## Artifacts & Data Files

### Ignored (safe to delete)

- **build/** — C++ CMake output (gitignored)
- **target/** — Rust cargo build output (gitignored)
- **results.csv** — mortality test output (gitignored); regenerated by `make test_mortality`

### Tracked-But-Generated (don't hand-edit)

- **docs/** — Doxygen HTML output. CI auto-commits on master push. Local changes vanish when you `git pull` after a docs commit. Safe to build locally for preview.

### Reference Data (tracked, not auto-generated)

- **src/testMortality/FOFEM_input.tre** — FOFEM mortality reference dataset (read-only for tests)
- **src/testMortality/FOFEM_Mortality_Output.csv** — expected mortality results (baseline for comparison)

## One-Command Health Check

```bash
make compile && make test && cargo test --workspace && cargo test -p behave-run --test parity
```

Takes ~5s (single M-series). Expected output:

```
Total tests performed: 171
Total tests passed: 171
Total tests failed: 0
[...C++ ANSI colors...]

[Rust unit test summary, 283 tests, all pass]
test result: ok. 283 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 1 test
test parity_with_cpp_test_behave ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Exit code 0 = fully healthy. Any non-zero exit or FAILED in any line = broken.

## When NOT to Use This Skill

- **Environment setup** (clang, cmake, cargo, Doxygen versions) → `behave-build-and-env`
- **Interpreting a failing test** (symptom diagnosis) → `behave-debugging-playbook`
- **Adding a new test** → `behave-validation-and-qa`
- **Understanding the parity suite internals** → `behave-validation-and-qa`
- **Parallelization roadmap** → `behave-parallelization-campaign`

## Provenance and Maintenance

**Verified against repo (as of 2026-07-06):**

- C++ test count: `make test` → 171 tests (all pass on commit f11cbc5)
- Rust test count: `cargo test --workspace` → 283 unit tests (all pass)
- Parity suite: 171 embedded checks, exact call order from C++ testBehave.cpp
- CMakeLists.txt: 6 build targets; TEST_BEHAVE/TEST_MORTALITY/EXAMPLE_APP default ON; spot CLIs + RAWS_BATCH default OFF
- Makefile: `make compile`, `make test`, `make test_mortality`, `make gendocs` verified
- CI: `.github/workflows/ci.yml` runs on ubuntu, clang, `make test` (C++ only, no Rust)
- Docs: `.github/workflows/docs.yml` auto-commits on master push with HEAD hash
- .gitignore: build/, target/*, results.csv confirmed ignored; docs/ tracked

**Re-verify by running:**

```bash
# C++ test count
make test 2>&1 | grep "^Total tests passed"

# Rust test count (sum unit tests only)
cargo test --workspace 2>&1 | grep "^test result:" | head -10

# Parity test
cargo test -p behave-run --test parity 2>&1 | grep -E "^(running|test result)"

# Build options
grep "OPTION(" CMakeLists.txt

# Docs workflow
cat .github/workflows/docs.yml | grep -E "doxygen-action|commit -m"
```
