---
name: behave-validation-and-qa
description: Defines evidence standards for the Behave Rust port — the golden-value parity suite as the primary validation authority (171 runtime golden checks replaying testBehave.cpp), per-crate unit tests, acceptance thresholds, and test-addition discipline. Use when adding tests, validating changes against golden values, defining new tolerances, or assessing whether a change meets evidence bar for landing. When NOT to use this skill: for Rust framework architecture and crate dependencies, use `behave-architecture-contract`; for per-crate unit test internals, use per-crate SKILL documentation; for evidence-gathering methodology, use `behave-research-methodology`.
---

# Behave Validation & QA

Evidence standards for the Behave Rust port (rj-rust-port branch). This skill is the authority on what counts as proof and how to structure tests.

## Evidence Hierarchy

Ranked by weight (most authoritative first):

1. **Golden-value parity suite** (crates/behave-run/tests/parity.rs, PRIMARY STANDARD)
   - 171 checks executed at runtime (142 static call sites; four sites loop — two-fuel coverages, speed units, VPD, slope distances), all passing, replaying testBehave.cpp against one shared BehaveRun in identical call order; the C++ suite itself reports "Total tests passed: 171", matching
   - Mirrors C++ error_tolerance = 1e-6 exactly (except VPD: 1e-3)
   - Designed for state-dependent tests: later checks assume results from earlier setups
   - Reports all failures together (fail-fast blocked) via TestInfo::finish()
   - Runs as a single test function; each check contributes to pass/fail decision
   - Runs via `cargo test -p behave-run --test parity`

2. **Per-crate unit tests** (file-local #[cfg(test)] modules, 282 total)
   - 15 in firelab-base, 20 in behave-surface, 7 in behave-weather, 38 in behave-crown
   - 2 in behave-ignite, 12 in behave-contain, 132 in behave-run (includes parity + unit)
   - 13 in behave-mortality, 43 in behave-spot
   - Runs via `cargo test --workspace --lib` (unit tests only, no integration tests)
   - Covers unit-level invariants, isolated APIs, edge cases
   - Use exact-match assertions (f64 bit-for-bit) for deterministic pure functions

3. **Observed behavior vs. published outputs**
   - BehavePlus 6 published outputs are the spec even when dimensionally wrong
   - Example: slope-tool map-distance-to-inches quirk preserved because BehavePlus bakes it in
   - See `behave-change-control` skill for the C++ divergence ledger and incident history

4. **Randomized differential corpus** (OPEN: not yet implemented)
   - Future: C++↔Rust parity suite over broader input ranges, property-test style
   - Will extend parity checks beyond the golden testBehave.cpp sequence

## Parity Suite Anatomy

**File**: crates/behave-run/tests/parity.rs (1,383 lines)

### Structure & Design

```
TestInfo struct (lines 33–52):
  - passed: count of successful checks
  - failures: Vec of human-readable divergence messages
  - check() method: compares (observed, expected, tol), collects failure
  - check_bool() method: casts bool to u8 then calls check()

Scenario helpers (lines 59–194):
  - set_surface_inputs_for_gs4_low_moisture() → init state for cascading tests
  - set_surface_inputs_for_two_fuel_models_low_moisture()
  - set_crown_inputs_low_moisture()
  All mirror C++ setSurfaceInputsFor*() naming and parameter order

Test sections (lines 107–1340):
  - test_surface_single_fuel_model(): lines 107–335 (32 checks)
  - test_chaparral(): 336–391 (11 checks)
  - test_calculate_scorch_height(): 392–409 (3 checks)
  - test_palmetto_gallberry(): 410–433 (5 checks)
  - test_western_aspen(): 434–462 (6 checks)
  - test_length_to_width_ratio(): 463–554 (9 checks)
  - test_elliptical_dimensions(): 555–602 (6 checks)
  - test_direction_of_interest(): 603–679 (10 checks)
  - test_fireline_intensity(): 680–696 (2 checks)
  - test_two_fuel_models(): 697–726 (5 checks)
  - test_crown_module_rothermel(): 727–807 (16 checks)
  - test_crown_module_scott_and_reinhardt(): 808–925 (23 checks)
  - test_spot_module(): 926–1018 (18 checks)
  - test_speed_unit_conversion(): 1019–1053 (6 checks)
  - test_ignite_module(): 1054–1085 (7 checks)
  - test_safety_module(): 1086–1114 (6 checks)
  - test_contain_module(): 1115–1186 (6 checks)
  - test_fine_dead_fuel_moisture_tool(): 1187–1213 (5 checks)
  - test_slope_tool(): 1214–1283 (9 checks)
  - test_vapor_pressure_deficit_calculator(): 1284–1326 (4 checks, TOL=1e-3)
  - test_simple_surface(): 1327–1340 (3 checks)

Main test driver (lines 1351–1382):
  parity_with_cpp_test_behave() → new_behave_run() + call all test sections
  finish() → panic if t.failures.is_empty() == false
```

### Why Shared State & Call Order Matter

C++ testBehave.cpp runs all tests *sequentially against ONE shared BehaveRun*, with later tests depending on state left by earlier ones. Example:
- test_surface_single_fuel_model() sets up surface inputs and runs calculations
- test_length_to_width_ratio() immediately follows and queries L/W ratios (no fresh input setup)
- This tests the *continuity contract* of the API

**The Rust suite replicates this exactly** to catch bugs where a getter wrongly depends on a field not yet initialized or earlier calculations silently corrupted state.

### round6() — The Precision Ceremony

```rust
fn round6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap()
}
```

Mirrors C++ `roundToSixDecimalPlaces()` (line 25 in testBehave.cpp):
1. Format to 6 decimal places (string rep)
2. Re-parse as f64

This captures the finite precision of C++'s output stream and matches the tolerance-testing intention: "does the Rust kernel round to the same display value?"

### Tolerance & Special Cases

```rust
const TOL: f64 = 1e-6;  // global default (matches C++ error_tolerance)
```

**Exception**: Vapor Pressure Deficit (VPD) calculator uses `TOL = 1e-3` (lines 1317, 1324). VPD involves atmospheric physics with lower precision requirements.

**How to read a check**:
```rust
t.check("name", observed_value, expected_value, 1e-6);
// Passes if |observed - expected| < 1e-6
```

### Deliberate Divergences (Cross-Ref: behave-change-control)

The Rust port **intentionally fixes two C++ bugs** that the parity suite accepts:

1. **behave-ignite:inputs.rs — setMoistureHundredHour bug** (ignite.cpp:238)
   - C++: mistakenly writes to the one-hour field; Rust keeps fields separate
   - Parity suite asserts Rust's corrected values (parity lines 1068–1072)

2. **behave-ignite:inputs.rs — isFuelDepthNeeded variable shadowing** (ignite.cpp:310–320)
   - C++: shadowed variable makes isFuelDepthNeeded() always return false
   - Rust: pattern-matched correctly
   - Parity suite asserts Rust's corrected behavior (parity lines 1074–1080)

These are listed in crates/behave-ignite/src/inputs.rs with code comments pinning them to lines in the C++ source.

### Preserved Quirks (Bit-for-Bit, NOT to be "fixed")

The Rust port preserves C++ quirks for parity. Documented in `behave-change-control`:

1. TL5 savrLiveWoody = 160.0 (likely typo for 1600, preserved in fuel_models.rs:~809)
2. PressureUnits to_base DIVIDES / from_base MULTIPLIES (inverted vs all others, firelab-base/src/units.rs:~171–209)
3. Canopy-height getter ignores units argument, always returns feet (behave-surface/src/inputs.rs:~1165)
4. Slope tool calculate_horizontal_distance treats map distance as feet when converting to inches; units arg ignored (behave-weather/src/slope.rs:~116)
5. Moisture scenario D4* descriptions say "D3L*" (C++ copy-paste, moisture.rs:~261)
6. EXRATE calc_flanking_time does not clamp cos_t into [-1,1]; "go right" bounds check compares a ros index against PATH-array capacity (exrate.rs)

All listed with test coverage or code comments. Change any of these only via behave-change-control (owner decision required).

## Per-Crate Unit Test Conventions

### File Structure

Tests live in file-local `#[cfg(test)]` modules at the end of source files:

```rust
// In src/fuel_models.rs (or any module)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fm1_properties() {
        let c = FuelModels::new();
        assert_eq!(c.fuel_code(1), "FM1");
        // ... more assertions
    }
}
```

**Not** in separate `tests/` directories (those are integration tests; we don't use them for library internals).

### Test Naming

- `<thing>_<aspect>` or `<thing>_<aspect>_test` — e.g., `fm1_properties`, `slope_tool_basic_conversions`
- Predicate-style (ending in `?`): NOT preferred for test names; use `_is_*` or `_should_*` (e.g., `is_valid_fuel_model_number`)

### What to Assert

1. **Exact match** (f64 bit-for-bit): pure, deterministic functions with no floating-point error sources
   ```rust
   assert_eq!(some_table_lookup(5), 123.456);
   ```

2. **Tolerance-based**: any computed result (e.g., sqrt, exp, division)
   ```rust
   assert!((computed - expected).abs() < 1e-10, "observed {}, expected {}", computed, expected);
   ```

3. **Parity-style checks** for inputs that interact with the parity suite:
   ```rust
   let observed = round6(some_calculation());
   let expected = 123.456789;  // From C++ golden output
   assert!((observed - expected).abs() < 1e-6);
   ```

### Where Goldens Come From

Every new golden value needs ONE of:
1. **C++ source output** — run testBehave.cpp and capture the value; cite the line
2. **Paper derivation** — reference the equation + constants; use `behave-numerics-proof-toolkit` for verification
3. **BehavePlus 6 reference run** — customer-published output (lower bar for quirks, higher for algorithms)

Document the source in a nearby comment:

```rust
#[test]
fn scorch_height_calculation() {
    // Golden value from testBehave.cpp line 456 (GS4 case)
    // Equation: Byram scorch height formula, inputs flame_length=12.5, wind_speed=5.2
    let observed = round6(run.surface.scorch_height(...));
    assert!((observed - 28.493721).abs() < 1e-6);
}
```

## Adding Tests

### When to Add a Parity Section

**Always** when:
- Porting a new module or fixing a bug in an existing module
- Adding a golden value from testBehave.cpp

Steps:
1. Read the relevant C++ test function (e.g., testChaparral in testBehave.cpp)
2. Identify the input setup (state dependencies)
3. Add a new test section function in parity.rs:
   ```rust
   fn test_my_new_feature(t: &mut TestInfo, run: &mut BehaveRun) {
       // Set inputs if needed (or rely on prior test)
       run.my_feature.do_something();
       
       t.check("my feature: output A", round6(run.my_feature.get_a()), 123.456, TOL);
       t.check("my feature: output B", round6(run.my_feature.get_b()), 789.012, TOL);
   }
   ```
4. Call the new function from parity_with_cpp_test_behave() in the correct sequence
5. Run `cargo test -p behave-run --test parity` to verify all checks pass

### When to Add a Unit Test

**For**:
- Boundary conditions (0, very large, negative inputs)
- Error cases (invalid fuel model, out-of-range moistures)
- Table lookups and conversions
- Internal helper functions not directly tested by parity suite

**Steps**:
1. Open the source file (e.g., src/fuel_models.rs)
2. Add to the `#[cfg(test)]` module at the end:
   ```rust
   #[test]
   fn my_boundary_case() {
       let result = function_under_test(edge_case_input);
       assert_eq!(result, expected_value);
   }
   ```
3. Run `cargo test --workspace --lib` to verify

### Re-running After Changes

**REQUIRED after ANY change to crates/**:
```bash
cargo test --workspace --lib           # Unit tests (282 tests)
cargo test -p behave-run --test parity # Parity suite (171 runtime checks)
```

Both must pass. The parity suite is particularly important because it exercises all crates together and catches integration bugs (e.g., state leaking between modules, incorrect final precision).

## Acceptance Thresholds (Tolerance Budget)

| Change Class | Tolerance | Why | Example |
|---|---|---|---|
| Refactor (no math change) | Bit-exact (1 ULP) | Must reproduce C++ behavior identically | Reordering pure function calls |
| C++ parity port | 1e-6 (TOL constant) | Mirrors C++ testBehave tolerance | Porting Rothermel engine |
| f32 deployment study | DEFINED in behave-parallelization-campaign (PENDING) | Must be approved explicitly per-output | GPU implementation decision |
| Optimization (math-preserving rearrangement) | Tolerance stated in PR + parity suite | Document why rearrangement is safe | Compensated summation in reaction_intensity |

**NEVER silently increase tolerance**. Tolerance changes require:
1. A comment in code citing the reason
2. An entry in the PR description
3. Approval from the crate owner (RJ Sheperd, typically)

## Golden Inventory & Provenance

All golden values in parity.rs are copied from testBehave.cpp constants or computed outputs, documented inline with line numbers.

### Categories:

1. **Surface single-fuel-model outputs** (parity lines 107–335)
   - Source: testBehave.cpp lines 67–430 (testSurfaceSingleFuelModel)
   - Inputs: GS4, low moisture, 5 mph wind, 30% slope
   - 32 checks covering spread rate, flame length, ROS in multiple units, vector quantities

2. **Special fuel models** (parity lines 336–462)
   - Chaparral, palmetto-gallberry, western aspen
   - Source: testBehave.cpp special-model sections (chaparral ~line 432+, etc.)
   - Note: Aspen mortality IS implemented (fuelbed -> SurfaceFire -> Surface::get_aspen_mortality); parity asserts 0.267093 in the aspen section (as of 2026-07-06)

3. **Crown fire** (parity lines 727–925)
   - Rothermel and Scott & Reinhardt initiation/spread
   - Source: testBehave.cpp testCrownModuleRothermel/testCrownModuleScottAndReinhardt
   - Feeds on surface layer results; state-dependent

4. **Spot fire** (parity lines 926–1018)
   - Four spotting sources (firebrand lofting, rolling debris, direct flame, plume buoyancy)
   - 14 species-specific trajectories
   - Source: testBehave.cpp testSpotModule

5. **Tools** (parity lines 1187–1326)
   - Fine dead fuel moisture, slope tool, VPD calculator
   - Each has its own input domain and precision requirements

### Tracking Coverage

When backporting a new C++ test section:
1. Grep testBehave.cpp for the function name
2. Read the entire test function (all checks)
3. Port EVERY check to Rust parity.rs
4. Record the line range in the parity.rs comment block above each section
5. Run parity suite — ZERO ignores

**Currently**: 171/171 runtime checks active, 0 ignored (as of 2026-07-06; all formerly-gapped sections — chaparral, palmetto-gallberry, western aspen incl. mortality, two-fuel TwoDimensional — are folded into the main sequence).

## Debugging Failed Tests

### Parity Suite Failure

When `cargo test -p behave-run --test parity` fails:

1. **Read the panic message** — lists failed checks with observed vs. expected + tolerance
   ```
   3 of 171 parity checks failed:
   surface single-fuel: spread rate: observed 12.34 differs from expected 12.35 by more than 0.000001
   ```

2. **Check call order dependency**
   - Find the failed check in parity.rs
   - Look at the test section calling it
   - Does this section depend on state from an earlier test?
   - If yes, did that earlier test run and pass?

3. **Reproduce in isolation** (if the check seems to be unit-level)
   ```rust
   // Add a standalone unit test in the relevant crate
   #[test]
   fn debug_isolated_case() {
       let mut run = BehaveRun::new(...);
       run.surface.update_surface_inputs(...);
       let observed = run.surface.spread_rate(...);
       eprintln!("Observed: {}", observed);
       assert!((observed - expected).abs() < 1e-6);
   }
   ```
   Then `cargo test --lib -p behave-run -- --nocapture debug_isolated_case`.

4. **Check precision loss**
   - Print the unrounded value: `eprintln!("raw: {}, rounded: {}", raw, round6(raw));`
   - Is round6() truncating a value that should stay above/below the golden?
   - Check units: are you comparing the same units? (common error: chains/hr vs ft/min)

5. **Verify the golden value**
   - Re-run `make test_mortality` (if it's mortality-related) or `make test` (C++) to regenerate golden
   - Did the C++ source change?
   - Is the golden value even correct in the C++ test? (See "C++ test-suite bugs" below)

### Unit Test Failure

When `cargo test --workspace --lib` fails in a specific crate:

1. **Check the test output** for the exact assertion
2. **Inspect the function under test** — add debug prints
3. **Consider tolerance** — if comparing floats, did you use `.abs() < eps`?
4. **Refactor tests that are too broad** — break into smaller tests per case

### C++ Test-Suite Bugs (Documented)

The parity suite corrects three known bugs in C++ testBehave.cpp (all documented in REVIEW.org):

1. **Moisture-class-needed checks** (~line 379 in testBehave.cpp)
   - The C++ test compares observed-to-observed (never assigns to expected), so the check always passes
   - Rust parity suite asserts the *declared* expected value instead
   - Rust value is correct

2. **Crown L/W checks** (~lines 761–777 in testBehave.cpp)
   - C++ test compares observed-to-observed
   - Rust asserts correct expected value

3. **Crown perimeter check** (~line 1104 in testBehave.cpp)
   - Same issue
   - Rust correct

**Implication**: If you backport a new C++ test to the parity suite, READ THE C++ TEST CAREFULLY. It may not actually be asserting what it looks like.

## Open QA Items (Candidate Future Work)

All PENDING, NOT blocking the current state:

1. **Randomized differential corpus** — property-test style C++↔Rust parity over wider input ranges
   - Would replace ad-hoc "I'll test this case" with systematic coverage
   - Implementation: `proptest` crate + a hypothesis-driven input generator

2. **Rust CI** — .github/workflows/ci.yml currently runs C++ tests only (master branch)
   - Add a Rust test step: `cargo test --workspace`
   - Add parity suite: `cargo test -p behave-run --test parity`

3. **Property tests** — beyond randomized corpus; invariant-based
   - Example: spread_rate() ≥ 0 always; reaction_intensity() ≥ 0 always
   - Example: spread_rate(mph) * 88 = spread_rate(ft/sec) (unit conversion is a ring homomorphism)
   - Would catch bugs in conversions and arithmetic

4. **Per-output tolerance studies** — for f32 GPU deployment (CROSS-REF: behave-parallelization-campaign)
   - Parity suite rerun with f32 kernel
   - Measure relative error distribution per output
   - Accept/reject per-output decision

## NOT Covered by This Skill

- **Performance measurement** → `behave-diagnostics-and-tooling` (benchmarking, profiling, flamegraph)
- **Change control & incident tracking** → `behave-change-control` (C++ divergence ledger, decision gates, Jira tickets)
- **Domain knowledge** (equations, fuel models, constants) → `fire-behavior-reference` (models, equations, units, conventions)

## Provenance and Maintenance

**Last verified**: 2026-07-06 on branch rj-rust-port (HEAD f11cbc5)

Re-verify these facts (all read-only, safe):

```bash
# Parity suite structure (run from repo root)
wc -l crates/behave-run/tests/parity.rs                    # Should be ~1383
grep -c "\.check(" crates/behave-run/tests/parity.rs      # Should be 128+ calls
cargo test -p behave-run --test parity 2>&1 | grep "ok"   # Should be "1 passed"

# Unit test counts per crate
cargo test --workspace --lib 2>&1 | grep "test result:"   # Collect 9 lines
# Expected: 15 (base) + 20 (surface) + 7 (weather) + 38 (crown) + 2 (ignite) 
#         + 12 (contain) + 132 (run) + 13 (mortality) + 43 (spot) = 282 total

# Verify round6() implementation
grep -A 3 "fn round6" crates/behave-run/tests/parity.rs

# Check TOL constant
grep "^const TOL:" crates/behave-run/tests/parity.rs       # Should be 1e-6

# Verify no ignored tests in parity suite
grep "#\[ignore\]" crates/behave-run/tests/parity.rs       # Should be empty
```

**Golden source authority**: testBehave.cpp (C++ master / HEAD a4dc3a0 lineage) + REVIEW.org "Parity suite results" section + per-check inline comments in parity.rs.

**Change control**: This skill is documentation. To change evidence standards or acceptance thresholds, file a decision with the owner (RJ Sheperd) and update this skill + the relevant crate/test.
