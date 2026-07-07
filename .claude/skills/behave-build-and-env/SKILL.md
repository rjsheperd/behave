---
name: behave-build-and-env
description: Configure the Behave Rust-C++ build environment from scratch; known pitfalls (non-ASCII source bytes, CI runs only on master, Rust workspace has no CI coverage). Use when setting up a dev machine, troubleshooting build failures, or understanding environment constraints.
---

## Scope

This skill covers environment prerequisites, verified build commands (Rust + C++), CI reality, and platform-specific traps. It is the home for:
- Build command runbooks with actual test/source counts
- Prerequisite toolchain versions
- CMake options reference
- CI workflow reality (coverage, branch limits, mutation of docs/)
- Known environment pitfalls (non-ASCII bytes, slow full rebuilds, gitignore quirks)

**When NOT to use:** Running individual programs/tests in a live session → use `behave-run-and-operate`; understanding why a test failed → use `behave-debugging-playbook`; modifying CMake or build config → use `behave-config-and-toggles`.

---

## Prerequisite Toolchain (Verify Once)

### Rust
```bash
rustc --version  # Must be 1.83+ (Cargo.toml uses edition 2021)
cargo --version
```
**Verified (as of 2026-07-06):** rustc 1.91.0, cargo 1.91.0 ✓

### C++
```bash
cmake --version     # Must be >= 3.5
clang --version     # LLVM/Clang (not GCC; CI uses clang)
doxygen --version   # For docs generation
make --version      # GNU Make
```
**Verified (as of 2026-07-06):** cmake 4.1.2, doxygen 1.14.0, GNU Make 4.4.1 ✓

### Clone & Check Environment
```bash
git clone <repo-url>
cd behave
cargo --version
cmake --version
clang --version
doxygen --version
```

---

## Build Command Reference

### Rust: Full Workspace

**Build library (debug):**
```bash
cargo build --workspace
```
Output: `target/debug/` (gitignored via `.gitignore:21` pattern `target/*`)

**Run all unit tests + integration:**
```bash
cargo test --workspace
```
**Verified (as of 2026-07-06, branch rj-rust-port, HEAD f11cbc5):**
- **283 unit tests** (breakdown: 15 + 20 + 7 + 38 + 2 + 1 + 12 + 132 + 13 + 43 across 9 crates)
- **1 integration test** (parity suite, contains **171 assertions** internally; see `crates/behave-run/tests/parity.rs:1383 lines`)
- All passing ✓

**Parity test alone (Rust vs C++ golden):**
```bash
cargo test -p behave-run --test parity
```
Runs **1 test binary** with **171 internal checks** comparing Rust outputs to C++ golden values (tolerance 1e-6, VPD section 1e-3). See `behave-validation-and-qa` for parity suite anatomy.

---

### C++: Full Rebuild + Test

**Build C++ core:**
```bash
make compile
```
- Runs `cmake -B build && cmake --build build`
- Creates build/ (gitignored via `.gitignore:17`)
- Compiles **44 .cpp + 44 .h** source files (not 41 as historical refs may suggest; count via `find src/behave -name "*.cpp"`)
- Rebuilds **all 44 modules on every compile** (slow full rebuild by design — CMake target structure; optimized build unavailable)
- Output executables: `build/testBehave`, `build/testMortality`, `build/behave`

**Run C++ test suite:**
```bash
make test
```
**Verified (as of 2026-07-06):** **171 tests, 171 passed, 0 failed** ✓

**Run C++ mortality comparison (FOFEM parity):**
```bash
make test_mortality
```
- Requires input files: `src/testMortality/FOFEM_input.tre`, `src/testMortality/FOFEM_Mortality_Output.csv`
- Outputs: `results.csv` (gitignored via `.gitignore:202`)
- Compares Behave mortality logic to FOFEM reference

**Generate C++ Doxygen docs:**
```bash
make gendocs
```
- Runs `doxygen Doxyfile`
- Outputs: `docs/` (gitignored as generated)
- **WARNING:** CI bot auto-commits generated docs/ to master (see CI Reality section)

**Dev target (docs + tags):**
```bash
make dev
```
Runs both `gendocs` and `gentags` (creates TAGS file for Emacs/Vim)

**Clean build artifacts:**
```bash
make clean
```
Removes `build/` and `docs/`

---

## CMake Options (C++ Build Configuration)

Read: `.github/workflows/ci.yml` and `CMakeLists.txt:9-55` for source truth. Options control which executables link into testBehave/testMortality.

| Option | Default | Meaning |
|--------|---------|---------|
| `TEST_BEHAVE` | ON | Compiles `build/testBehave` (C++ surface/crown/contain tests) |
| `TEST_MORTALITY` | ON | Compiles `build/testMortality` (FOFEM mortality parity) |
| `EXAMPLE_APP` | ON | Compiles `build/behave` (example client binary) |
| `RAWS_BATCH` | OFF | RAWS data batch reader CLI |
| `COMPUTE_SPOT_PILE` | OFF | Pile spot distance calculator |
| `COMPUTE_SPOT_SURFACE` | OFF | Surface spot distance calculator |
| `COMPUTE_SPOT_TORCHING_TREES` | OFF | Torching-tree spot distance calculator |

**To rebuild with custom options:**
```bash
cmake -B build -DTEST_BEHAVE=ON -DRAWS_BATCH=ON
cmake --build build
```

---

## CI Reality (as of 2026-07-06)

### C++ CI (Enabled)
- **File:** `.github/workflows/ci.yml`
- **Trigger:** Push to master or pull_request against master
- **Runner:** ubuntu-latest
- **Compiler:** clang (not GCC)
- **Steps:**
  1. Checkout
  2. CMake configure/build (threeal/cmake-action v1.3.0)
  3. `make test` (runs `build/testBehave`, 171 tests)
- **Note:** Master branch only; no branch flexibility

### Docs CI (Enabled, Mutating)
- **File:** `.github/workflows/docs.yml`
- **Trigger:** Push to master only (no PRs)
- **Runner:** ubuntu-latest
- **Steps:**
  1. Checkout
  2. Run `doxygen` (mattnotmitt/doxygen-action v1.9.8)
  3. **Git add/commit/push docs/ as bot commit** (user: Doxygen, email: doxygen@users.noreply.github.com)
- **⚠️ WARNING:** Pushing to master automatically updates docs/ via bot. Never hand-edit docs/ — regenerate via `make gendocs` locally, push to branch, docs/ will be auto-committed on merge.

### Rust CI (Not Implemented)
- **Status:** OPEN ITEM — Rust workspace has zero CI coverage
- **What a correct Rust CI job would run (candidate):**
  ```yaml
  - name: Rust tests
    run: cargo test --workspace --all-features
  
  - name: Rust parity
    run: cargo test -p behave-run --test parity
  
  - name: Rust clippy
    run: cargo clippy --workspace -- -D warnings
    # Note: ignore known accepted lints per build output
  ```

---

## Known Environment Traps

### 1. Non-ASCII Bytes in C++ Source
Some `src/behave/*.cpp` files contain non-ISO extended-ASCII (e.g., `surfaceFire.cpp`).
```bash
file src/behave/surfaceFire.cpp
# Output: C source, Non-ISO extended-ASCII text
```
**Impact:** Piping through standard grep fails silently.

**Solution:** Use `grep -a` (search binary as text):
```bash
grep -a "some_pattern" src/behave/surfaceFire.cpp
```

### 2. Full Rebuilds on Every C++ Compile
CMake target structure compiles **all 44 .cpp modules** on every build (no incremental linking optimization).
- Each `make compile` takes ~1-2 min on modern hardware
- Workaround: Develop in Rust when possible (incremental, ~1 sec)

### 3. .gitignore Pattern: `target/*` (not `target/`)
Rust cargo output ignored by `.gitignore:21` as `target/*` (with trailing `/*`).
```bash
cat .gitignore | grep -E "^target"
# Output: target/*
```
**Impact:** `target/` directory itself may show as untracked if the wildcard doesn't match some edge case. Solution: No action needed for normal Rust builds; files within target/ are ignored.

### 4. docs/ Directory is Generated Output
- **Never hand-edit** files in `docs/`
- Doxygen auto-regenerates on every `make gendocs` or CI master push
- If you hand-edit `docs/index.html`, next gendocs or CI run will overwrite it
- Solution: Edit Doxyfile or source code comments instead

### 5. Docs/ Mutations on Master Pushes
Pushing to master triggers docs.yml, which auto-commits generated docs/ via bot.
```bash
git push origin master
# CI runs immediately → docs.yml generates docs/ → bot commits "Generated Docs (commit: ...)"
# Your local master is now behind by 1 bot commit
```
**Solution:** After pushing to master, pull to sync the bot commit:
```bash
git push origin master && git pull origin master
```

### 6. Build on Non-macOS Platforms
- **CI runs on ubuntu-latest** (Linux, not macOS)
- Local dev observed on macOS (Apple Silicon aarch64-apple-darwin)
- **Impact:** File paths, line endings (CRLF vs LF), compiler differences
- **Solution:** Test locally, then verify CI passes (or use Docker for Linux testing)

---

## From-Scratch Smoke Sequence (Verify Environment Good)

After clone, run this to confirm a working build:

```bash
# 1. Rust build + test
cargo test --workspace

# 2. C++ compile + test
make compile
make test

# Expected outcome: both green
# Rust: 283 unit tests + 1 parity (171 assertions) all passing
# C++: 171 tests all passing
```

If both pass, environment is good.

---

## Crate Workspace Structure (for reference)

Cargo.toml workspace members (9 crates, all v0.1.0, edition 2021):

| Crate | Role | Key Dependencies |
|-------|------|------------------|
| `firelab-base` | Units, enums, errors, fire size | none |
| `behave-surface` | Rothermel surface spread | firelab-base |
| `behave-crown` | Crown fire, firebrands | firelab-base, behave-surface |
| `behave-spot` | Spot fire distance | firelab-base, behave-crown |
| `behave-ignite` | Ignition probability | firelab-base |
| `behave-contain` | Containment simulation | firelab-base |
| `behave-mortality` | Tree mortality | firelab-base |
| `behave-weather` | Weather tools (slope, etc.) | firelab-base |
| `behave-run` | Facade, integration tests | all above |

---

## Provenance and Maintenance

**Based on:**
- Makefile (`.github/workflows/ci.yml`, `CMakeLists.txt`, `src/behave/*.cpp/.h` directory scans)
- Verified by running: `cargo test --workspace`, `make compile`, `make test`, `cargo test -p behave-run --test parity`
- Rust toolchain inspection: `rustc --version`, `cargo --version`
- C++ toolchain: `cmake --version`, `clang --version`, `doxygen --version`

**Re-verification commands (if context pack diverges):**
```bash
# Rust test count
cargo test --workspace 2>&1 | grep "^test result:" | wc -l

# C++ test count
make test 2>&1 | grep "Total tests"

# Source file count
find src/behave -name "*.cpp" | wc -l

# Crate count
ls -d crates/*/ | wc -l

# CMake version requirement (in CMakeLists.txt:9)
grep CMAKE_MINIMUM_REQUIRED CMakeLists.txt

# CI files exist and state
ls .github/workflows/*.yml
```

**Last verified:** 2026-07-06 (branch rj-rust-port, HEAD f11cbc5, commit 800f566 "Rust Port WIP" with parity suite)

**Known discrepancies from earlier docs:**
- Context pack stated "41 .cpp + 41 .h"; actual count: **44 each**
- Rust test documentation mentioned 289 tests; actual: **283 unit + 1 integration (171 assertions)**
- No Rust CI coverage is an open item; C++ CI only
