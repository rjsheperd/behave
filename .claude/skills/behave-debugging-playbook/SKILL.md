---
name: behave-debugging-playbook
description: Symptom-to-root-cause triage table for debugging real Behave failures. Maps observed phenomena to specific modules, C++ quirks, and diagnostic experiments. HOME for the parity suite state-carryover trap and unit-conversion factor lookup.
---

# Behave Debugging Playbook

Triage guide for the most common, real failure modes in the Rust port and C++ library.
Built from 750 commits of git archaeology, the parity suite, and post-mortem incident
reconstruction.

## Quick Symptom Lookup

| Symptom | Root cause | Diagnosis | Fix strategy |
|---------|-----------|-----------|--------------|
| Parity check `test_foo` fails | Failure is in `foo` OR in an earlier section whose state it depends on | Isolate: comment out all tests except `test_foo` and section it depends on (see below) | Run `cargo test -p behave-run --test parity -- --nocapture` with reduced test sequence; binary-search which predecessor fails |
| Value off by clean factor (2×, 12×, 66×, 43560×, 5280×, 88×, 1.1×) | Unit conversion mismatch | Compare factor to lookup table (see [Unit Conversion Factor Lookup](#unit-conversion-factor-lookup)) | Trace the getter/setter unit argument; verify `to_base()` / `from_base()` applied in correct direction |
| Value off in 6th decimal place (tolerance 1e-6 violated) | Float rounding; `format!("{x:.6}").parse()` vs C++ `stringstream` difference | Print intermediates via Rust test (add `.check()` calls); compare C++ and Rust; check if reordered operations change precision | Operator precedence or intermediate magnitude; may require algorithmic restructure for later parallel work |
| Direction/spread wrong only in `RelativeToNorth` mode | Wind direction or aspect not handled in coordinate frame conversions | Check `wind.rs` aspect subtraction; verify `update_wind_direction()` and `update_aspect()` call order (BHP1-1367 legacy) | Look for missing recompute after aspect or wind direction change; aspect must apply to BOTH before next wind calc |
| Test passes alone, fails in suite (or vice versa) | Parity state carryover — earlier test mutated `BehaveRun`; next test inherits that state | Reproduce by running the exact sequence from start up to failing test | Extract the setup sequence; rebuild `BehaveRun` to clean state between test sections (requires new test function) |
| Chaparral / palmetto-gallberry / aspen output wrong | Special-model dispatch in `facade.rs` is branching but passing wrong parameters or the special-fuel module has wrong constants | Check `facade.rs:81-83` for the conditional; verify the fuel module (chaparral.rs, palmetto_gallberry.rs, western_aspen.rs) constant tables | Compare module constants to C++; run `cargo test -p behave-surface --test chaparral_*` (if exists) or extract from parity suite `test_chaparral()` |
| Two-fuel TwoDimensional spread rate wrong | EXRATE (exrate.rs) is using wrong fuel model's L/W ratio or coverage normalization is broken | Verify `two_fuel_models.rs:~184` uses `lb_ratio = self.length_to_width_ratio[SECOND]` (second model only) | Print `ros[FIRST]`, `ros[SECOND]`, `coverage[FIRST]`, `coverage[SECOND]` before `exrate::surface_fire_expected_spread_rate()` call |
| Contain initial-attack perimeter / fire-size 2× too large | ContainAdapter bypasses `FireSize` and invents its own ellipse geometry | Check `contain/src/adapter.rs` — must compute effective windspeed = `4.0 * (lw_ratio - 1.0)` and route through `FireSize` (not hand-rolled ellipse) | The 2026-07-06 fix routes through `FireSize`; verify line-matched to C++ (perimeter 1.55× larger before fix) |
| C++ `grep` finds a function but Rust search doesn't | File contains non-ASCII bytes (e.g., directional quote, copyright symbol); grep defaults to binary mode | Use `grep -a` to force text mode | Verify with `file src/behave/surfaceFire.cpp` — will show "Non-ISO extended-ASCII" |
| C++ testBehave asserts X is expected | The assertion may compare observed-to-observed (a C++ bug), not expected-to-observed | Read the full assertion code, not just the failure message | Known C++ assertion bugs: moisture-class-needed checks (~line 379), crown L/W checks (~line 761-777), crown perimeter checks (~line 1104) — all compare observed to observed; Rust parity suite asserts the declared expected values instead |

## Unit Conversion Factor Lookup

When a computed value is off by a clean factor:

| Factor | Conversion pair | Direction hint | C++ incident(s) |
|--------|-----------------|-----------------|-----------------|
| 2× | Unknown | One `to_base()` or `from_base()` call missing | e00fd3a (2026-02-20): 16 arithmetic bugs in behaveUnits.cpp |
| 12× | Feet ↔ Inches | Check `LengthUnits::to_base()` multiplies by 0.08333... or reciprocal | Slope tool inches/feet confusion |
| 66× | Feet ↔ Chains | `LengthUnits::Chains.to_base()` = 66.0; `.from_base()` = 0.01515... | Containment time-to-fireline conversions (ch/h → ft/min) |
| 43560× | Square feet ↔ Acres | `AreaUnits::Acres.to_base()` = 43560.00216... | 49a1a24: SafeSeparationDistance output was Acres, spec says Square Feet; 3a7c2bd: scorch height over-conversion |
| 5280× | Feet ↔ Miles | `LengthUnits::Miles.to_base()` = 5280.0 | Spotting distance or max-travel-distance unit mismatch |
| 88× | Feet per minute ↔ Miles per hour | 1 mph = 88 fpm (5280 ft/mi ÷ 60 min/hr) | Wind speed conversions |
| 1.1× | Various (context-specific) | Often rounding or precision loss in multi-step conversion | Scorch formula near-zero denominator (sqrt(FLI + U³) cancellation risk in f32) |
| Reciprocal (0.5×) | **Pressure only** | **INVERTED QUIRK: C++ divides in `to_base()`, multiplies in `from_base()`** | 0ed4b73 (2023-11-03): VPD kPa→Pa flip unresolved conceptually; preserved for parity at `firelab-base/src/units.rs:171-217` |

### How to test a conversion factor hypothesis

1. Isolate the failing output value (e.g., flame length = 42 ft instead of 21 ft, a 2× error)
2. Identify which unit conversions surround the calculation:
   - Trace the getter that returns the value (e.g., `get_flame_length(LengthUnits::Feet)`)
   - Check if `from_base()` was called; verify the unit argument
3. Compare C++ and Rust conversion factors side-by-side:
   ```bash
   # C++: grep "to_base\|from_base" src/behave/behaveUnits.cpp | grep -A2 "Feet\|Inches"
   # Rust: grep -A10 "impl UnitConversion for LengthUnits" crates/firelab-base/src/units.rs
   ```
4. Check git history for the same module's conversion:
   ```bash
   git log --all --oneline -- src/behave/behaveUnits.cpp | head -10
   ```

## Float Precision & Rounding

### The round-6 contract

- C++ uses `stringstream` (default formatting) → rounding behavior is locale-dependent and opaque
- Rust mirrors this with `format!("{x:.6}").parse::<f64>().unwrap()` (round-half-even)
- Tolerance: 1e-6 (checked by parity suite)
- **VPD section is special**: VPD calc uses Tetens coefficients; tolerance relaxed to 1e-3 in `crates/behave-run/tests/parity.rs:test_vapor_pressure_deficit_calculator()`

### Common 6th-decimal failures

| Root cause | Where | Fix |
|-----------|-------|-----|
| Operator precedence (e.g., `a / b * c` vs `a / (b * c)`) | Rothermel surface spread rate (eq. 27), scorch formula | Reorder operations; add intermediate variables; check C++ source for the order it uses |
| Cancellation in division (e.g., `(x - y) / (x + y)` when x≈y) | Scorch height formula `sqrt(FLI + U³)` denominator (f32 risk) | Check if intermediate was rounded to 6 decimal places mid-calculation |
| Trigonometric precision (sin, cos, atan2) | Direction-of-max-spread calculations (ellipse L/W → aspect) | Compare `(observed - expected).abs() < 1e-6` and if barely failing, check for sin/cos series truncation |

**Reproducing a 6th-decimal failure:**
1. Add `.check()` call with observed and expected, then re-run: `cargo test -p behave-run --test parity -- --nocapture`
2. Extract the output value from C++ `./build/testBehave` and format with `printf "%.15f\n"` to see all digits
3. Format Rust value the same way and binary-search the first mismatching digit

## Direction of Spread & Wind Orientation

### RelativeToNorth mode quirks

- **Wind direction** = direction the wind blows **from** (compass convention)
- **Aspect** = direction the slope faces (uphill direction)
- **Spread direction** = direction fire advances to (sum of wind and slope vectors)
- **RelativeToNorth mode** applies aspect subtraction: `wind_dir_actual = input_wind_dir - aspect` (converts to upslope-relative frame)

### When direction is wrong only in RelativeToNorth

1. Check `wind.rs` update order:
   - `update_wind_direction()` must be called
   - `update_aspect()` must be called *after* (or both must trigger a recompute)
   - There is a known BHP1-1367 order dependency (setter order matters)
2. Verify `SurfaceInputs::calculate_all_wind()` is called after both setters (or in the input update method)
3. Test the isolated direction calculation:
   ```bash
   # Rust test: set wind_dir=90°, aspect=30°, mode=RelativeToNorth
   # Expected result_dir = 90 - 30 = 60° (relative to upslope)
   # If result_dir = 90, aspect was not subtracted
   ```

## Parity Suite State Carryover

### How the suite is structured

- **File**: `crates/behave-run/tests/parity.rs:~1400 lines`
- **Strategy**: ONE shared `BehaveRun` instance, 21 test functions called in sequence
- **Why**: C++ `testBehave.cpp` does the same — state from test N affects test N+1
- **Example**: `test_surface_single_fuel_model()` calls `do_surface_run_in_direction_of_max_spread()`, mutating `BehaveRun.surface.fire`; then `test_chaparral()` re-uses that same fire object

### Test function sequence (in order)

```
1. test_surface_single_fuel_model         (line 107)  ← sets up initial surface fire
2. test_chaparral                          (line 336)  ← uses GS4 state from #1
3. test_calculate_scorch_height            (line 392)  
4. test_palmetto_gallberry                 (line 410)
5. test_western_aspen                      (line 434)
6. test_length_to_width_ratio              (line 463)
7. test_elliptical_dimensions              (line 555)
8. test_direction_of_interest              (line 603)
9. test_fireline_intensity                 (line 680)
10. test_two_fuel_models                    (line 697)
11. test_crown_module_rothermel             (line 727)
12. test_crown_module_scott_and_reinhardt   (line 808)
13. test_spot_module                        (line 926)
14. test_speed_unit_conversion              (line 1019)
15. test_ignite_module                      (line 1054)
16. test_safety_module                      (line 1094)
17. test_contain_module                     (line 1116)
18. test_fine_dead_fuel_moisture_tool       (line 1169)
19. test_slope_tool                         (line 1206)
20. test_vapor_pressure_deficit_calculator  (line 1269)
21. test_simple_surface                     (line 1295)
```

### Debugging a carryover failure

If test N fails in the full suite but passes alone:

1. **Identify what N depends on**: check test N's first few calls
   - Does it call `run.surface.do_surface_run_in_direction_of_max_spread()`?
   - Does it call `run.crown.*`?
   - If no explicit setup, it inherits state from test N-1
2. **Isolate by prefix replay**:
   ```bash
   # Create a temp test file with calls 1..N-1 + N only
   # cargo test -p behave-run --test parity
   ```
3. **OR**: binary-search which test in the sequence breaks N:
   - Remove test M (where M < N)
   - Re-run full suite
   - If N passes, M was the culprit
4. **Fix**: either re-initialize `BehaveRun` state in test N, or fix the state-mutation in test M

### Known carryover hazards

- `surface.fire` object is mutated by `do_surface_run()` calls; subsequent tests use the new ROS/fireline intensity
- `inputs` setters accumulate; calling `update_surface_inputs()` again with different parameters changes all 20+ fields at once
- Crown tests call `surface.do_surface_run_in_direction_of_max_spread()` internally; order matters

## Special Fuel Models (Chaparral, Palmetto, Aspen)

### Dispatch location

**File**: `crates/behave-surface/src/facade.rs:81-83`

```rust
let is_special = self.inputs.is_using_palmetto_gallberry()
    || self.inputs.is_using_western_aspen()
    || self.inputs.is_using_chaparral();
```

If any flag is true, the calculation takes the special-model path via `SurfaceFire::calculate_forward_spread_rate()`.

### Special fuel model modules

| Fuel type | Module | Constants verified against C++ | Status |
|-----------|--------|---------|--------|
| Chaparral | `crates/behave-surface/src/chaparral.rs` | Age-depth exponentials, live/dead heat of combustion (8800/7350), MOE tables (0.30-0.85) | ✓ parity checks pass; 15 branches (depth, load, moisture, SAVR, HOC, MOE, density, silica) ported 2026-07-06 |
| Palmetto-Gallberry | `crates/behave-surface/src/palmetto_gallberry.rs` | Rough age regressions, HOC 8300, MOE 0.40, depth = 2/3 * understory_height | ✓ parity checks pass |
| Western Aspen | `crates/behave-surface/src/western_aspen.rs` | 5 curing models x DBH, flame-length-to-char-height mortality ratio | ✓ parity checks pass; aspen mortality implemented via fuelbed.calculate_western_aspen_mortality; parity asserts 0.267093 |

### Debugging special-model failures

1. Verify the flag is being set correctly:
   ```bash
   # Check that SurfaceInputs setter is called
   # grep "set_chaparral\|set_palmetto\|set_aspen" crates/behave-run/tests/parity.rs
   ```
2. Confirm the special-model fuelbed load/depth calculation:
   - Chaparral: depth = f(age); load = f(depth, type)
   - Palmetto: depth = 2/3 understory_height; load = f(age, rough)
   - Aspen: check curing% and DBH-based model selection
3. Check if the module constants are bit-exact to C++:
   ```bash
   # Chaparral coefficients: grep -A5 "CHAMISE\|MIXED_BRUSH" crates/behave-surface/src/chaparral.rs
   # Palmetto: grep -A5 "LONGLEAF\|LOBLOLLY" crates/behave-surface/src/palmetto_gallberry.rs
   ```
4. If fuelbed is correct but output wrong, check if the module's parameters (density, SAVR, MOE, HOC) are correctly inserted into `Fuelbed`:
   - Look for the branch in `fuelbed.rs` that handles the special type
   - Verify all 5 size classes get the correct values (not zero-initialized)

## Two-Fuel Models & EXRATE

### TwoDimensional method dispatch

**File**: `crates/behave-surface/src/two_fuel_models.rs:~184`

```rust
TwoFuelModelsMethod::TwoDimensional => {
    let lb_ratio = self.length_to_width_ratio[SECOND];  // ← second model only
    let samples = 2;
    let depth = 2;
    let laterals = 0;
    self.spread_rate = crate::exrate::surface_fire_expected_spread_rate(
        &self.ros, &self.coverage, lb_ratio, samples, depth, laterals,
    );
}
```

### Critical facts

- **L/W ratio comes from the SECOND fuel model** (the minority model); this matches BehavePlus
- **Coverage order**: first_coverage (normalized to 0–1), second_coverage = 1 − first_coverage
- **EXRATE path**: the `crate::exrate` module implements Finney's deterministic expected-spread-rate factorial enumeration
  - `samples=2, depth=2, laterals=0` hardcoded (from behavePlus.xml)
  - Only the no-extension path is used (the `newext` machinery is documented as unreachable and not ported)

### Debugging two-fuel failures

1. Confirm the method enum is correct:
   ```bash
   # Check parity suite call:
   # grep -A5 "TwoDimensional" crates/behave-run/tests/parity.rs
   ```
2. Print the intermediate ROS values and L/W:
   ```bash
   # Add temporary .check() calls in test_two_fuel_models:
   // t.check("TwoFuel ROS first", ros[FIRST], expected_ros_1, TOL);
   // t.check("TwoFuel ROS second", ros[SECOND], expected_ros_2, TOL);
   // t.check("TwoFuel LW ratio", lb_ratio, expected_lw, TOL);
   ```
3. Verify coverage normalization:
   - Sum of both coverages must equal 1.0
   - Check order: is first_coverage being used for the first model or flipped?
4. If the EXRATE calculation is suspect, compare against C++ by printing the `samples, depth, laterals` parameters passed to the C++ `expected_spread_rate()` function

## Containment Adapter Geometry

### The 2026-07-06 fix (fire-size 2.0× too large before)

**File**: `crates/behave-contain/src/adapter.rs`

**Problem**: ContainAdapter was computing initial-attack fire geometry using a hand-rolled ellipse (`a = rate, b = a/LW`), instead of routing through `FireSize` (which applies the Anderson ellipse formula and clamping).

**Solution**: Compute effective windspeed = `4.0 * (lw_ratio - 1.0)`, pass to `FireSize`, and use the resulting perimeter/area.

**Evidence**: Before fix, fire size at initial attack was 2.0× too large; perimeter was 1.55× too large. Parity check now passes.

### Debugging contain geometry issues

1. Check if initial-attack perimeter / area are off by 2× or 1.55×:
   - Likely the hand-rolled ellipse path is being used instead of `FireSize`
2. Verify the effective-windspeed calculation:
   ```bash
   grep -n "4.0.*lw_ratio\|4.*LW" crates/behave-contain/src/adapter.rs
   ```
3. Confirm `FireSize` is being called:
   ```bash
   grep -n "FireSize::new()" crates/behave-contain/src/adapter.rs
   ```

## C++ File Encoding Quirks

### Non-ASCII bytes blocking grep

**Symptom**: `grep "some_function"` finds nothing in a C++ file, but the function visibly appears in your editor.

**Root cause**: The file contains non-ASCII bytes (directional quote, copyright symbol, special dash). `grep` defaults to treating the file as binary.

**Files affected**:
- `src/behave/surfaceFire.cpp` (verified: `file` command shows "Non-ISO extended-ASCII")
- Possibly others in `src/behave/`

**Fix**: Use `grep -a` (force text mode) or `grep -i` (case-insensitive):
```bash
grep -a "your_search" src/behave/surfaceFire.cpp
```

## C++ Test Suite Assertion Bugs

### Broken C++ assertions in testBehave.cpp

The C++ test suite has several assertions that compare **observed-to-observed** (a logic bug), not expected-to-observed.

| Location | What it checks | Issue | Rust parity workaround |
|----------|----------------|-------|----------------------|
| ~line 379 | Moisture class needed (1-hr, 10-hr, 100-hr) | Compares `class1 == class2` where both are extracted from the same object | Parity suite asserts the declared C++ expected values instead |
| ~line 761-777 | Crown L/W ratio | Compares computed L/W against itself after intermediate rounding | Check the C++ source; Rust asserts the published BehavePlus value |
| ~line 1104 | Crown fire perimeter | Similar observed-to-observed pattern | Rust parity suite overrides with the correct expected value |

### How to handle these in Rust

When porting a C++ test:
1. **Read the C++ assertion code**, not just the failure message
2. If both sides of `==` are derived from the same object, it's likely comparing observed-to-observed
3. Instead, use the **declared expected value** from the test setup (if present) or the published output from BehavePlus
4. Document the C++ bug in a comment in the parity suite

## Discriminating Experiments

Use these workflows to narrow down the root cause.

### Experiment 1: Isolate the failing test section

**Goal**: Determine if the failure is in test N or inherited from N-1.

**Steps**:
1. Comment out all test functions except the failing one:
   ```bash
   # In crates/behave-run/tests/parity.rs, around line 1358:
   # Keep only test_two_fuel_models(&mut t, &mut run);
   ```
2. Ensure the setup is still called (`set_surface_inputs_for_gs4_low_moisture`):
   ```bash
   # This is called before the loop at line 1356, so it still runs
   ```
3. Run: `cargo test -p behave-run --test parity -- --nocapture`
4. If it passes, the failure is a **carryover** (inherited state). If it fails, the failure is **local**.

### Experiment 2: Compare C++ and Rust intermediate values

**Goal**: Identify where the paths diverge.

**For C++**:
```bash
# (run from repo root)
make compile
./build/testBehave 2>&1 | grep "test_name" | head -20
# (No direct output filtering; redirect to file and grep)
```

**For Rust**:
1. Add `.check()` calls with intermediate values in the Rust test:
   ```rust
   let ros = run.surface.get_spread_rate(SpeedUnits::ChainsPerHour);
   t.check("ros_intermediate", ros, c_value, TOL);
   ```
2. Run: `cargo test -p behave-run --test parity -- --nocapture 2>&1 | grep "ros_intermediate"`

### Experiment 3: Binary-search a stalled calculation

**Goal**: Identify which component (wind, slope, moisture) is wrong.

**Steps**:
1. Start with all inputs applied (the full scenario from the parity suite)
2. Zero out one variable (e.g., set wind speed to 0)
3. Re-run and check if the output becomes obviously wrong or stays the same
4. Repeat for each input variable: wind, slope, moisture
5. **Hypothesis**: if zeroing wind has no effect, the wind path is broken; if zeroing slope has no effect, slope is broken

### Experiment 4: Precision loss in f64 arithmetic

**Goal**: Determine if the error is a precision loss or an algorithmic bug.

**Steps**:
1. Compute the value in Rust with high precision (no rounding):
   ```rust
   let x = some_calculation;  // No rounding
   println!("raw: {:.15}", x);
   ```
2. Compute with C++ rounding (6 decimal places):
   ```rust
   let x_rounded = format!("{:.6}", x).parse::<f64>().unwrap();
   println!("rounded: {:.15}", x_rounded);
   ```
3. If they differ significantly, **precision is the issue**. If they're the same, the **algorithm is wrong**.

## When NOT to Use This Skill

- **For adding new features or modules**: use `behave-change-control`
- **For the history of why a trap exists or when it was introduced**: use `behave-failure-archaeology`
- **For domain theory (models, equations, constants)**: use `fire-behavior-reference`
- **For understanding the C++ or Rust architecture at a high level**: use `behave-architecture-contract`
- **For build/environment issues or test commands**: use `behave-build-and-env`

---

## Provenance and Maintenance

**Source basis**:
- 750-commit git history (2016-01-26 to 2026-07-06); recurring incident patterns extracted
- `REVIEW.org` (2026-07-06 code review synthesis; 154→171 check parity suite; convergence findings)
- `crates/behave-run/tests/parity.rs` (1400 lines; 21 test functions; C++ state carryover documented)
- Unit conversion incidents: e00fd3a, 49a1a24, 0ed4b73, 3a7c2bd (commits verified)
- Contain geometry fix: 2026-07-06 parity suite validation (2.0× fire size, 1.55× perimeter before fix)
- C++ file encoding: `file src/behave/surfaceFire.cpp` confirmed "Non-ISO extended-ASCII"

**Re-verification commands** (run these if facts drift):

```bash
# Parity test count:
grep "\.check\|\.check_bool" crates/behave-run/tests/parity.rs | wc -l

# Test function sequence:
grep -n "^fn test_" crates/behave-run/tests/parity.rs | head -25

# Special-model dispatch:
grep -n "is_using_palmetto\|is_using_western\|is_using_chaparral" crates/behave-surface/src/facade.rs

# Two-fuel L/W source (SECOND model):
grep -A2 "TwoDimensional =>" crates/behave-surface/src/two_fuel_models.rs | grep "lb_ratio"

# Pressure unit quirk (DIVIDE in to_base):
grep -A10 "impl UnitConversion for PressureUnits" crates/firelab-base/src/units.rs | grep "divide"

# Non-ASCII in surfaceFire.cpp:
file src/behave/surfaceFire.cpp | grep -i "ascii"

# Contain geometry fix presence (4.0 * lw_ratio):
grep -n "4.0.*lw_ratio" crates/behave-contain/src/adapter.rs

# Parity suite running (171 checks passing):
cargo test -p behave-run --test parity 2>&1 | grep -E "test result|passed"
```

**Last verified**: 2026-07-06 (brand new — every fact cross-checked against running codebase)

**Maintenance**: update this skill when:
- New recurring failure modes emerge (add to triage table with incident reference)
- Unit conversion bugs are fixed (update factor lookup table; add commit hash)
- C++ file encoding issues spread to other files (add to list)
- Parity suite structure changes (update test function sequence)
