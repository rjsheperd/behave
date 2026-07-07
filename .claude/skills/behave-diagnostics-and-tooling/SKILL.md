---
name: behave-diagnostics-and-tooling
description: Measurement tooling to verify the Behave Rust port against C++ golden values; scripts for cross-validation, parity test navigation, and codebase drift tracking—use when debugging divergence between implementations, adding tests, or establishing baselines for the parallelization campaign.
---

# Behave Diagnostics and Tooling

Use this skill to MEASURE instead of eyeball. Three scripts + documentation for evidence-based debugging, test exploration, and benchmark methodology.

## Quick Start

All scripts live in `.claude/skills/behave-diagnostics-and-tooling/scripts/`. Call them from repo root.

```bash
# One-shot validation: C++ vs Rust in one command
./.claude/skills/behave-diagnostics-and-tooling/scripts/cross_validate.sh

# Find which test function owns a failing check
./.claude/skills/behave-diagnostics-and-tooling/scripts/parity_focus.sh "flame length"

# Health report: test count, parity checks, todo/panic/unwrap markers
./.claude/skills/behave-diagnostics-and-tooling/scripts/count_state.sh
```

---

## Script 1: cross_validate.sh

**Purpose:** One-shot validation combining C++ and Rust test results.

**What it does:**
1. Verifies cmake and cargo are installed
2. Builds `./build/testBehave` (C++ reference) if needed, else reuses existing
3. Runs C++ test suite, captures pass/fail counts from "Total tests passed: N" / "Total tests failed: M"
4. Runs `cargo test -p behave-run --test parity` (171 checks against C++ golden values)
5. Prints combined verdict; exits 0 if all pass, 1 otherwise

**Typical output:**
```
=== Cross-Validation: C++ vs Rust ===

[1/3] C++ reference already built (build/testBehave exists)

[2/3] Running C++ testBehave...
[... C++ test output ...]
Total tests passed: 171
Total tests failed: 0

[3/3] Running Rust parity suite (cargo test -p behave-run --test parity)...
[... Rust compiler output ...]
test parity_with_cpp_test_behave ... ok

=== VERDICT ===
C++:  171 passed, 0 failed
Rust: PASS (0)

✓ All cross-validation checks passed.
```

**When to use:**
- After landing a change to verify no parity regression
- To establish baseline for new feature before GPU work
- To validate that a fix actually fixes (end-to-end, not just unit tests)

**When NOT to use:** If you only need to run one test suite (use `cargo test` directly) or if you need selective parity checks (use `parity_focus.sh`).

---

## Script 2: parity_focus.sh

**Purpose:** Navigate large parity test files by finding which test function owns a failing check.

**Usage:**
```bash
./.claude/skills/behave-diagnostics-and-tooling/scripts/parity_focus.sh <pattern>
```

**What it does:**
- Searches `crates/behave-run/tests/parity.rs` for check names matching `<pattern>` (case-insensitive substring match)
- Prints the enclosing test function name for each match
- Enables rapid location of a failing check within parity.rs

**Examples:**
```bash
$ parity_focus.sh "scorch"
test_calculate_scorch_height
  ├─ scorch height: 80F, 5mph, 50 Btu/ft/s
test_calculate_scorch_height
  ├─ scorch height: 70F, 300 ft/min, 55 Btu/ft/s

$ parity_focus.sh "moisture"
test_surface_single_fuel_model
  ├─ surface: live moisture of extinction
test_surface_single_fuel_model
  ├─ surface: moisture scenario D1L1
... (35 more) ...

$ parity_focus.sh "spot"
test_spot_module
  ├─ spot: maximum spotting distance, uphill
test_spot_module
  ├─ spot: maximum spotting distance, downhill
... (more) ...
```

**Output format:**
- `test_function_name` followed by list of matching checks under that function
- Use to navigate parity.rs when `cargo test -p behave-run --test parity` fails
  (error message will say "observed X differs from expected Y"; search for the exact check name to find the assertion in parity.rs, then the function it lives in)

**When to use:**
- A parity check fails; you need to find it in parity.rs and understand its context
- You want to see all checks related to a subsystem (e.g., all "crown" checks)

---

## Script 3: count_state.sh

**Purpose:** One-liner health/drift report. Canonical record of codebase state.

**What it does:**
1. Runs `cargo test --lib --all` and prints last line (test count summary)
2. Counts `t.check(...)` + `t.check_bool(...)` calls in parity.rs
3. Counts `todo!`, `TODO` markers in non-test source code
4. Counts `panic!`, `.unwrap()`, `.expect(` in non-test lib code (approximation: does not distinguish from comments/strings)
5. Prints file counts (source vs test), line counts (rough)
6. Runs `cargo build --workspace` and prints last 3 lines (compiler status)

**Typical output:**
```
=== Behave Codebase Health Report ===
(as of 2026-07-06 18:47:10)

UNIT TESTS (cargo test --lib --all 2>&1 | tail -1):
[test result line from cargo]

PARITY CHECK CALL SITES (grep count in parity.rs; runtime count is 171 — loops):
  142 call sites

TODO!/TODO MARKERS (non-test lib code):
  1 markers

PANIC/UNWRAP/EXPECT (non-test lib code):
  4 occurrences (approximation: includes comments, strings)
  NOTE: Searches for literal panic!/, .unwrap(), .expect(—does not
  distinguish panic in tests (acceptable) vs lib code (problematic).

FILE COUNTS:
  52 source files (non-test)
  1 test files

LINE COUNTS (rough):
  ~19890 lines in source code
  ~1383 lines in test code

COMPILER STATUS (cargo build --workspace 2>&1 | tail -3):
warning: `behave-crown` (lib) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
```

**Interpretation:**
- Commit the output when establishing a baseline
- Re-run to detect drift: new TODOs added, new panics introduced, unplanned test count changes
- Panic/unwrap count is an approximation (rough heuristic; refine by reading the actual files)

**Known limitations:**
- Line counts use `wc -l` (not always accurate with mixed line endings; rough only)
- Panic count includes comments and strings (safe to over-count, but will have false positives)
- Does not measure per-crate; use `cargo build -p <crate>` or `cargo tree` for dependency analysis

**When to use:**
- Establish baseline at start of sprint/campaign
- Track deviations after landing large refactors
- Catch accidental TODOs or new panic paths

---

## Documentation: Interpreting Parity Failure Output

When `cargo test -p behave-run --test parity` fails, the output format is:

```
---- parity_with_cpp_test_behave stdout ----
thread 'parity_with_cpp_test_behave' panicked at '<N> of <M> parity checks failed:
<check_1>: observed <obs1> differs from expected <exp1> by more than <tol1>
<check_2>: observed <obs2> differs from expected <exp2> by more than <tol2>
...
', crates/behave-run/tests/parity.rs:<line>:<col>
```

**Interpretation strategy:**

1. **Count:** `<N> of <M>` tells you how many checks failed out of 171 total (runtime count; loops expand 142 call sites to 171 checks).
2. **Per-check detail:** Each line shows `check_name: observed X differs from expected Y by more than Z`
   - `observed`: the value the Rust code computed
   - `expected`: the C++ golden value (from testBehave.cpp)
   - `tolerance`: the allowed epsilon (typically 1e-6, or 1e-3 for VPD)
3. **Find the check in parity.rs:**
   ```bash
   grep "\"check_name\"" crates/behave-run/tests/parity.rs
   ```
   This gives the line number and context.
4. **Identify the function:**
   ```bash
   parity_focus.sh "check_name_substring"
   ```
   or manually scan backwards from the grep result to find `fn test_*` prefix.
5. **Root cause:** Common divergences are:
   - Unit conversion error (see `behave-change-control` for preserved quirks vs deliberate fixes)
   - Precision loss in f64 (rare at tolerance 1e-6)
   - Unimplemented feature (would be a `todo!()` panic, not a divergence)
   - Algorithm change not yet ported (check REVIEW.org for known gaps)

**Example workflow:**

```bash
$ cargo test -p behave-run --test parity
# Output: "4 of 171 parity checks failed: scorch height: 80F, ..."

$ parity_focus.sh "scorch"
test_calculate_scorch_height
  ├─ scorch height: 80F, 5mph, 50 Btu/ft/s
  ├─ scorch height: 70F, 300 ft/min, 55 Btu/ft/s

# Open crates/behave-run/tests/parity.rs, find test_calculate_scorch_height,
# inspect the inputs and the expected values from testBehave.cpp comments
```

---

## Documentation: Printing Intermediates (Fuelbed Getters)

When debugging divergence, you often need to inspect intermediate values (e.g., characteristic SAVR, heat source, reaction intensity) computed by the surface module.

**Available getters** (from `crates/behave-surface/src/facade.rs`):

- **Spread & Flame:**
  - `spread_rate(units) -> f64` — rate of spread in specified units
  - `flame_length_output(units) -> f64` — flame length
  - `fireline_intensity(units) -> f64` — Byram intensity
  - `heat_per_unit_area(units) -> f64` — HPUA

- **Wind & Reaction:**
  - `midflame_wind_speed(units) -> f64` — wind after WAF adjustment
  - `reaction_intensity(units) -> f64` — Rothermel reaction intensity
  - `slope_factor() -> f64` — dimensionless slope multiplier

- **Fuelbed Properties:**
  - `characteristic_savr(units) -> f64` — effective SAV ratio
  - `bulk_density(units) -> f64` — fuel bulk density
  - `heat_sink(units) -> f64` — heat sink (effective sink)
  - `heat_source(units) -> f64` — effective heat source
  - `packing_ratio() -> f64` — β/β_op
  - `relative_packing_ratio() -> f64` — β relative to maximum

- **Ellipse & Dimensions:**
  - `fire_length_to_width_ratio() -> f64` — L/W
  - `fire_eccentricity() -> f64` — eccentricity (0-1)
  - `elliptical_a(units) -> f64`, `elliptical_b(units) -> f64`, `elliptical_c(units) -> f64` — ellipse semi-axes
  - `fire_length(units) -> f64`, `max_fire_width(units) -> f64` — dimensions
  - `fire_perimeter(units) -> f64`, `fire_area(units) -> f64` — integrated ellipse measures

- **Moisture:**
  - `characteristic_moisture_by_life_state(life_state) -> f64` — weighted moisture for dead/live
  - `live_fuel_moisture_of_extinction(units) -> f64` — Mc_live

**Example: print intermediates in a test**

Add to `crates/behave-run/tests/parity.rs` after a `do_surface_run_in_direction_of_max_spread()` call:

```rust
eprintln!("DEBUG: characteristic_savr = {}", run.surface.characteristic_savr(SurfaceAreaToVolumeUnits::SquareFeetPerCubicFoot));
eprintln!("DEBUG: bulk_density = {}", run.surface.bulk_density(DensityUnits::PoundsPerCubicFoot));
eprintln!("DEBUG: reaction_intensity = {}", run.surface.reaction_intensity(HeatSourceAndReactionIntensityUnits::BtusPerSquareFoot));
eprintln!("DEBUG: midflame_wind_speed = {}", run.surface.midflame_wind_speed(SpeedUnits::FeetPerMinute));
```

Then run:
```bash
cargo test -p behave-run --test parity -- --nocapture 2>&1 | grep DEBUG
```

This prints the intermediates without failing the test. Compare the Rust output against C++ (print equivalent values in testBehave.cpp using `std::cerr`).

---

## Documentation: Benchmark Methodology for Parallelization Campaign

**Current state (as of 2026-07-06):**
- No criterion benches exist (`benches/` directory does not exist)
- Rust workspace builds with `cargo build --release`
- Single-threaded baseline must be established BEFORE any parallelization (rayon, GPU, etc.)

**Benchmark requirements (Phase 0 → Phase 1 roadmap):**

1. **Baseline: Single-Threaded Surface Run**
   - What: Time to compute ONE surface fire spread for a given fuel model / scenario
   - Measurement unit: nanoseconds per run
   - Scenario: FM124 (GS4 — grass-shrub, mid-range complexity), low-moisture scenario (D1L1)
   - Example code (pseudo-Rust):
     ```rust
     let mut run = BehaveRun::new(...);
     run.surface.update_surface_inputs(...); // GS4, low moisture
     
     let start = std::time::Instant::now();
     for _ in 0..10_000 {
       run.surface.do_surface_run_in_direction_of_max_spread();
     }
     let elapsed = start.elapsed();
     println!("10k runs: {:.2} ns/run", elapsed.as_nanos() as f64 / 10_000.0);
     ```
   - Expected baseline (rough estimate from C++): 1–2 microseconds per run (~1,000–2,000 ns)
   - Later: measure other scenarios (FM1, TL181, chaparral)

2. **Batch Throughput: Cells/Second**
   - What: After rayon parallelization (Phase 1), measure landscalse-scale cell throughput
   - Measurement unit: cells per second
   - Scenario: 100×100 grid of landscape cells, each with a surface run (10k runs total)
   - Example expectation: 10M cells/sec on a 4-core machine (if 1M base throughput × speedup ~4)
   - Later: WGPu baseline (GPU cells/sec)

3. **Profiling for Hot Spots**
   - Use `cargo build --release && perf` (on Linux) or Instruments (macOS) to identify bottlenecks
   - Focus on: transcendental functions (exp, ln, sqrt), reaction.rs Eq 27, wind factor calculation
   - Validate that f32 precision loss is acceptable (run parity suite with f32, measure error vs tolerance)

4. **Regression Testing**
   - After each parallelization phase, re-run cross_validate.sh to confirm parity holds
   - Add criterion bench suite under `crates/behave-run/benches/surface_benchmark.rs` to track regression
   - Criterion output format: compares against previous run automatically

**Setting up the benchmark (when ready to formalize):**

```bash
# Add to Cargo.toml
[[bench]]
name = "surface"
harness = false

# Create crates/behave-run/benches/surface.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use behave_run::BehaveRun;
use behave_surface::fuel_models::FuelModels;

fn bench_gs4_surface_run(c: &mut Criterion) {
    let mut run = BehaveRun::new(FuelModels::new(), ...);
    // Setup GS4, low moisture
    
    c.bench_function("gs4_single_run", |b| {
        b.iter(|| {
            run.surface.do_surface_run_in_direction_of_max_spread()
        });
    });
}

criterion_group!(benches, bench_gs4_surface_run);
criterion_main!(benches);
```

Then run:
```bash
cargo bench -p behave-run --bench surface
```

**Baseline capture workflow:**

1. Run `count_state.sh` to establish current health
2. Run `cargo build --release` (creates optimized binary)
3. Run ad-hoc benchmark (pseudo-code above) and record ns/run
4. Commit result to a file (e.g., `BENCHMARKS.md`) with date/machine/Rust version
5. After GPU phase, re-run and compare

---

## When NOT to Use This Skill

- **Adding new tests:** Use `behave-validation-and-qa` (covers unit test authoring, test location strategy)
- **Campaign phase gates / parallelization roadmap:** Use `behave-parallelization-campaign` (tracks milestones, decision gates)
- **Understanding C++ divergence in depth:** Use `behave-change-control` (ledger of preserved quirks vs deliberate fixes, incident analysis)
- **Compiler warnings or build failures:** Use `behave-build-and-env` (build commands, CI reality, env traps)
- **Debugging specific algorithm bugs:** Use `behave-debugging-playbook` (symptom→triage, traps, discriminating experiments)

---

## Provenance and Maintenance

**Based on:** Git history (commits ba9b6bd, 800f566, f11cbc5), parity.rs structure, C++ testBehave output format, cross_validate.sh pragmatism.

**Re-verification commands (run quarterly or after refactors):**

- Parity check call sites: `grep -cE "t\.check(_bool)?\(" crates/behave-run/tests/parity.rs` (142 as of 2026-07-06); runtime count: `cargo test -p behave-run --test parity -- --nocapture 2>&1 | grep "parity:"` (must print 171 executed, or update docs)
- C++ test count: `./build/testBehave 2>&1 | grep "Total tests passed:"` (must be 171, or update cross_validate.sh parsing)
- Facade getter names: `grep "pub fn " crates/behave-surface/src/facade.rs | wc -l` (must be ~50; if add new, update "Available getters" list)
- Script runtime: run all three scripts; confirm no stale paths, no missing dependencies, no hardcoded line numbers
- Cargo test count drift: `cargo test --lib --all -- --list 2>&1 | tail -1` (record baseline, flag if changes unexpectedly)
- Benchmark readiness: `find . -name "benches" -type d` (must remain empty until Phase 1; when criterion lands, update Section 4)

**Known gaps:**
- Panic/unwrap count in count_state.sh is approximate (includes comments); safe but not precise. Refine if become a gate criterion.
- Cross_validate.sh parsing assumes specific `Total tests passed:` format from testBehave; brittle if C++ test format changes.
- No per-crate breakdown in health report; add if cross-crate refactoring becomes common.
