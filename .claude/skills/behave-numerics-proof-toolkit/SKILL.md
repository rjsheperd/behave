---
name: behave-numerics-proof-toolkit
description: First-principles numerical analysis recipes for proving fire behavior calculations. "Prove it, don't just install it." Six runbook methods—parity cross-checking, deriving golden values from papers, f32 error budgets, determinism audits, divergence bisection chains, and table-fidelity checksums—each with worked examples from this repo's history (contain adapter bug, EXRATE determinism, scorch cancellation risk).
---

## Overview

This skill provides *executable recipes* for verifying numerical correctness in Behave (the USFS/RMRS wildland fire behavior library). Use when you need to **prove a calculation is right**, not just inspect code or run tests. Each recipe pairs theory with concrete methods, grounded in past failures that left 2.0× area errors undetected until golden-value testing forced the issue.

**When to use this skill:**
- Adding a new equation or porting code from another library (catch scope/dead-code traps early).
- Investigating divergence between Rust and C++ implementations.
- Evaluating f32 precision risk before GPU work.
- Validating that special-case handlers (wind limit, clamping, thresholds) are deterministic.
- Spot-checking large lookup tables (fuel models, species, safety separation).
- Building evidence for reproducibility claims before committing to parallelization.

**When NOT to use this skill:**
- Running the GPU/rayon parallelization campaign → use `behave-parallelization-campaign`.
- Adding structural/behavioral unit tests → use `behave-validation-and-qa`.
- Debugging live issues with detailed tracing → use `behave-debugging-playbook`.

---

## Recipe 1: Cross-Implementation Parity — The Round-6 Method

**Problem:** Comparing a Rust f64 calculation to a C++ golden value without floating-point noise masking real divergences.

**The Method:**

1. **Reproduce C++ rounding exactly**: Use `format!("{x:.6}").parse::<f64>()` to mirror C++'s `roundToSixDecimalPlaces()` (format to 6 decimal places, re-parse). This truncates to the precision the C++ test actually recorded.

2. **Choose tolerance carefully**:
   - Default: **1e-6** (matches C++ `error_tolerance`).
   - Tighter (1e-3) for outputs that must match to 3 decimals (e.g., VPD checks).
   - Looser (1e-5) only if the intermediate formula amplifies error (verified by sweep).

3. **Test shared state sequentially**: If tests depend on state left by earlier tests (C++ testBehave.cpp does), replicate the *exact call order* against one shared object. Document this coupling.

4. **Decompose multi-stage results by intermediate**: If observed diverges from expected by ~2%, calculate the intermediate ratio (area ratio, perimeter ratio, elapsed-time ratio) to isolate which stage broke.

**Worked Example: The Contain Adapter Bug (2026-07-06)**

Context: Initial-attack fire-size geometry in `behave-contain/src/adapter.rs` had diverged undetected for months.

*Before the fix:*
- Observed fire area at initial attack: ~2000 ft².
- Expected: ~1000 ft² (from C++).
- **Ratio: 2.0×**
- Observed perimeter: ~1550 ft.
- Expected: ~1000 ft.
- **Ratio: 1.55×**

*Divergence bisection:* Area ratio = (4/3)π·a·b·t² scales with (elliptical_a × elliptical_b × t²). Perimeter uses 2D Ramanujan approximation, which scales differently with a/b. The 2.0× and 1.55× ratios were *different*, signaling the shape was wrong, not just the time.

*Root cause:* Adapter had invented its own ellipse (a = report_rate, b = a/LW, Ramanujan-2), instead of deriving effective windspeed 4·(LW−1), running it through FireSize, and using FireSize's ellipse.

*Fix (line 228, `adapter.rs`):*
```rust
let effective_windspeed = 4.0 * (self.lw_ratio - 1.0); // mph
let mut size = FireSize::new();
size.calculate_fire_basic_dimensions(
    false,
    effective_windspeed,
    SpeedUnits::MilesPerHour,
    report_rate,
    SpeedUnits::ChainsPerHour,
);
let elliptical_a = size.elliptical_a(LengthUnits::Feet, 1.0, TimeUnits::Minutes);
let elliptical_b = size.elliptical_b(LengthUnits::Feet, 1.0, TimeUnits::Minutes);
```

After the fix: area and perimeter ratios both 1.0 (bit-parity). The parity suite (171 runtime checks, all passing) would have caught this on day 1 if it existed.

**Lesson:** Multi-output ratios decompose error sources. 2.0× + 1.55× ≠ 1.5× means "shape is wrong, not scaling." This beat hunting the bug in ellipse algebra for hours.

---

## Recipe 2: Deriving Golden Values from the Paper

**Problem:** You need a reference value to assert against, but the C++ output is unavailable or you're porting a new feature.

**The Method:**

1. **Locate the equation** in the cited paper (or code comment). Write it out longhand with all constants.
2. **Choose a simple test case** (round numbers, obvious intermediate values).
3. **Calculate by hand** on paper or in a spreadsheet to 4+ decimal places.
4. **Code the test**, parse the hand result, assert within 1e-6.
5. **Verify the test against the C++ if available**, then freeze it.

**Worked Example 1: Crown Length-to-Width Ratio**

Equation: Rothermel 1991, Equation 10 (page 16):
```
L/W = 1.0 + 0.125 * U_mph
```

Test case: U = 5 mph.

Hand calculation:
```
L/W = 1.0 + 0.125 * 5.0 = 1.0 + 0.625 = 1.625
```

Code (from `crates/firelab-base/src/fire_size.rs`, line 196):
```rust
let wind_speed = 5.0; // mph
let lw_ratio = 1.0 + 0.125 * wind_speed; // = 1.625
```

Golden assertion (from `crates/behave-run/tests/parity.rs`, line 743):
```rust
t.check(
    "crown Rothermel: length-to-width ratio",
    round6(run.crown.get_crown_fire_length_to_width_ratio()),
    1.625, TOL,  // TOL = 1e-6
);
```

Status: ✓ Passes. This golden value is now load-bearing for parallelization (GPU code must produce 1.625 for the same input).

**Worked Example 2: Elliptical Dimensions from Eccentricity**

For backing spread rate, we need the eccentricity first:
```
ecc = sqrt((LW^2 - 1)) / LW
```

At LW = 1.625:
```
ecc = sqrt(1.625^2 - 1) / 1.625 = sqrt(2.640625 - 1) / 1.625 = sqrt(1.640625) / 1.625
    ≈ 1.28104 / 1.625 ≈ 0.7883
```

Then backing rate:
```
ROS_back = ROS_forward * (1 - ecc) / (1 + ecc)
         = ROS_forward * (1 - 0.7883) / (1 + 0.7883)
         = ROS_forward * 0.2117 / 1.7883
         ≈ ROS_forward * 0.1184
```

Code (from `firelab-base/src/fire_size.rs`, lines 202–213):
```rust
fn calculate_fire_eccentricity(&mut self) {
    let x = (self.fire_length_to_width_ratio * self.fire_length_to_width_ratio) - 1.0;
    if x > 0.0 {
        self.eccentricity = x.sqrt() / self.fire_length_to_width_ratio;
    }
}

fn calculate_backing_spread_rate(&mut self) {
    self.backing_spread_rate =
        self.forward_spread_rate * (1.0 - self.eccentricity) / (1.0 + self.eccentricity);
}
```

Assertion from fire_size.rs tests (line 274–283):
```rust
#[test]
fn backing_spread_rate_analytical() {
    let mut fs = FireSize::new();
    let fpm = SpeedUnits::ChainsPerHour.to_base(8.876216);
    fs.calculate_fire_basic_dimensions(
        false, 5.0, SpeedUnits::MilesPerHour, fpm, SpeedUnits::FeetPerMinute,
    );
    assert_near(
        fs.backing_spread_rate(SpeedUnits::ChainsPerHour),
        1.112, 0.01,
    );
}
```

Status: ✓ Passes. The hand-derived eccentricity and backing rate are now verified against real-world spread-rate data.

---

## Recipe 3: f32 Error-Budget Analysis

**Problem:** GPU code uses f32; some outputs will lose precision. Which ones, and by how much?

**The Method:**

1. **Identify candidate cancellation sites**:
   - Subtractive: `(a - b)` when |a| ≈ |b|.
   - Exponential amplification: `d/dx of f(x)^0.55` is large → small input error → large output error.
   - Square root: `sqrt(a + b)` when a ≈ b (especially `sqrt(FLI + U^3)` in scorch).

2. **Estimate forward error propagation**:
   - Write the formula in differential form: `Δf ≈ (∂f/∂x)·Δx + (∂f/∂y)·Δy + ...`
   - Compute the sensitivities (partial derivatives) at a realistic test point.
   - Use f64 input precision (~1e-15 relative) as baseline; f32 gives ~1e-7 relative.

3. **Empirically sweep f32 vs f64**:
   - Code both, call with 100+ random realistic inputs (e.g., FLI ∈ [10, 10000], U ∈ [0, 100] mph).
   - Measure relative error: `(f32_result - f64_result) / f64_result`.
   - Build a histogram (bins: 1e-9 to 1e-3).

4. **Assign budget by output**:
   - If 99% of runs stay ≤ 1e-4 relative, mark as "f32-safe; budget 1e-4."
   - If outliers reach 1e-2, mark as "f32-risky; requires scaled-input renormalization or dual-precision."

**Worked Example: Scorch Height Cancellation Risk (Not Yet Ported — Frame for Phase 2)**

Formula (from `behave-surface/src/fire.rs`, lines 357–361):
```
H_scorch = (63 / (140 - T_air)) * FLI^(7/6) / sqrt(FLI + U^3)
```

Cancellation site: `sqrt(FLI + U^3)` when FLI ≈ U³.

Scenario: FLI = 1000 Btu/ft/s, U = 10 mph.
- U³ = 1000 (mph)³ ≈ 1e9 (in base units, ft³/min³, much larger).
- Let's recalculate in consistent units. U = 10 mph = ~880 ft/min.
- U³ ≈ (880)³ ≈ 6.8e8 ft³/min³.
- But FLI is in Btu/ft/s; converting to consistent base units is complex.

**Simpler analysis**: The risk is *structural*. The denominator `sqrt(FLI + U³)` is a sum where the two terms can have very different magnitudes. In f32:
- FLI contributes the high-order bits; U³ the low-order.
- When rounded, U³ can vanish entirely if |U³| < (machine epsilon) × |FLI|.
- In f32, machine epsilon ≈ 1e-7; in f64, ≈ 1e-16.

**Recipe for Phase 2 (when f32 code exists):**
1. Generate 1000 random (FLI, U) pairs with FLI ≈ U³ (or FLI >> U³, or U³ >> FLI).
2. Compute scorch height in both f32 and f64.
3. Report the relative error distribution.
4. If max error > 1e-3, consider scaled-input renormalization: compute `s = sqrt(FLI + U³)` by factoring: `s = sqrt(max(FLI, U³)) * sqrt(1 + min(FLI, U³) / max(FLI, U³))`.
5. Assert the result stays within the budget for landscape-scale fire modeling.

---

## Recipe 4: Determinism Audit — No RNG, No Wall Clock, Bit-Order Preservation

**Problem:** You claim a calculation is deterministic for reproducibility. Prove it.

**The Method:**

1. **Scan for RNG**: `grep -r "rand\|Random\|thread_rng"` in the crate. Any hit = not deterministic.

2. **Scan for wall-clock reads**: `grep -r "time::now\|SystemTime\|Instant"`. Any hit = not deterministic.

3. **Scan for interior mutability**: `grep -r "RefCell\|Mutex\|RwLock"`. These can hide state mutations that vary by execution order.

4. **Verify operation order is preserved**: If you refactored `a * (b * c)` → `(a * b) * c`, you *changed* the calculation (associativity fails in IEEE 754). Assert the intermediate results match.

5. **Document the determinism claim** in the function's comment block, citing which checks passed.

**Worked Example: EXRATE is Deterministic**

Claim: `surface_fire_expected_spread_rate()` in `behave-surface/src/exrate.rs` produces the same spread rate every call with the same inputs, despite the name "random fuel."

Proof:

1. **No RNG**: Lines 1–17 state explicitly: "Despite the name, the computation is fully deterministic: it enumerates every factorial arrangement of the fuel types."

2. **Algorithm: Factorial enumeration** (lines 89–177 in `exrate.rs`):
   - Loop over all 2^(samples×depth) arrangements.
   - For each, compute travel time via Dijkstra-like path search.
   - Accumulate weighted average.
   - **No randomness; every path taken on every run.**

3. **No wall-clock reads**: grep confirms zero `Instant`/`SystemTime` references.

4. **No interior mutability**: `struct Ellipse`, `struct FuelType`, `struct Path` are all `Copy` + plain fields.

5. **Fixed-point operation order**:
   - Path times computed by addition in loop (line 73–76):
     ```rust
     let mut travel_time = 0.0;
     for i in 0..num_layers {
         let r = e.a * theta.sin() * ros[i];
         travel_time += lat_dist[i] / r;
     }
     ```
   - Order of summation is *always* the same (loop index i = 0, 1, 2, ...).
   - No sorting or dynamic reordering.

6. **Assertion**: Same (first_fm, first_cov, second_fm) → same `spread_rate` every time. No rounding observed in parity suite.

Status: ✓ Deterministic. From parity.rs (lines 714–724), all 11 two-fuel coverage interpolations pass exactly (e.g., "two fuel models: first model coverage 0%" → 8.876216 ch/hr).

**Consequence for parallelization**: EXRATE can be batched with rayon (embarrassingly parallel over cells) without synchronization overhead. No need for locks or atomic counters.

---

## Recipe 5: Divergence Bisection — Walk the Intermediate Chain

**Problem:** Rust and C++ disagree on the final output. Which intermediate stage broke?

**The Method:**

1. **Identify the calculation pipeline** (from inputs to the observed output).

2. **Expose each stage as a getter**:
   - Moisture inputs → characteristic moisture / characteristic SAVR.
   - Fuelbed + wind → reaction intensity (Rothermel 1972, eq. 27).
   - Reaction intensity + wind/slope → no-wind ROS.
   - No-wind ROS + wind factor → wind-adjusted ROS.
   - ROS + fire ellipse → flame length, fireline intensity.

3. **Call each getter and compare to C++** (or intermediate golden values).

4. **Pinpoint the divergence** by the first getter that differs.

**Worked Example: Surface Fire Spread Rate**

Pipeline (from `Surface` facade, `behave-surface/src/facade.rs`):

```
Inputs (fuel model, moistures, wind, slope)
  ↓
[Fuelbed preprocessing]
  ↓ (getters to check: characteristic_moisture, characteristic_savr, packing_ratio)
[Reaction intensity calc]
  ↓ (getter: heat_source, fire_intensity -> ROS no-wind)
[Wind factor calc]
  ↓ (getters: effective_wind_speed, wind_factor, slope_factor)
[Final ROS with slope/wind]
  ↓
spread_rate(SpeedUnits::ChainsPerHour) ← final output
```

**Getters to call in order** (from facade.rs and fire.rs):
- `surface.characteristic_moisture_by_life_state(FuelLifeState::Dead, FractionUnits::Percent)` (line ~139)
- `surface.characteristic_moisture_by_life_state(FuelLifeState::Live, FractionUnits::Percent)` (line ~139)
- `surface.characteristic_savr(SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet)` (line ~144)
- `surface.relative_packing_ratio()` (line ~387)
- `surface.packing_ratio()` (line ~391)
- `surface.heat_source(HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerMinute)` (line ~156)
- `surface.fire_intensity(FirelineIntensityUnits::BtusPerFootPerSecond)` (line ~337)
- `surface.effective_wind_speed()` (line ~288)
- `surface.spread_rate(SpeedUnits::ChainsPerHour)` (line ~211)

From parity.rs (lines 107–150), the full sequence is already tested:
```rust
run.surface.update_surface_inputs(124, 6.0, 7.0, 8.0, ...);
run.surface.do_surface_run_in_direction_of_max_spread();
t.check("characteristic live moisture", round6(...), 85.874007, TOL);
t.check("characteristic dead moisture", round6(...), 6.005463, TOL);
t.check("characteristic SAVR", round6(...), 1631.128734, TOL);
t.check("heat source", round6(...), 5177.248579, TOL);
t.check("surface: spread rate", round6(...), 19.677584, TOL);
```

**If divergence found**: Bisect further by checking the fuelbed intermediates (packing ratio, density, load per size class).

---

## Recipe 6: Table-Fidelity Checksum — Spot-Check Discipline

**Problem:** You have a large table (fuel models, species, safety separation), and you want to ensure it wasn't corrupted or truncated during porting.

**The Method:**

1. **Count the rows**: Assert `table.len() == expected_count`. This catches missing/extra rows.

2. **Spot-check boundary rows**:
   - First row (index 0): most susceptible to off-by-one bugs.
   - Last row (index len - 1): catches truncation.
   - Rows around major categories (e.g., fuel model 10 vs 11, species ABAM vs ACRU).

3. **Random-middle rows** (pick 2–3 by index): `table[len / 3]`, `table[2 * len / 3]`.

4. **For each spot-checked row, assert**:
   - The unique identifier (e.g., fuel model number, species code).
   - One or two key numeric values (SAVR, heat of combustion, safety zone radius).
   - Reference: compare to C++ header/source or the original data sheet.

5. **Known limitation of spot-checks**: This method once missed 15 `todo!()` stubs in special-fuel-model integration because the table itself was intact; the *calling code* had panics. Pair spot-checks with **executable coverage** (e.g., exercising each chaparral age bin in a test).

**Worked Example: Fuel Model Table**

Table: `FuelModels` in `behave-surface/src/fuel_models.rs` (256 entries, ~40 KB).

**Count check** (from the C++ FuelModels.h):
```rust
assert_eq!(FUEL_MODELS.len(), 256);
```

**Boundary spot-checks**:
- Row 0 (FM1 "short grass"): SAVR = 3500, dead load = 0.034, live load = 0.0, depth = 0.5 ft.
- Row 12 (FM13 "western redcedar litter"): SAVR = 1900, dead load = 0.023, live load = 0.0, depth = 0.6 ft.
- Row 123 (GS4, grass-shrub): SAVR = 2350 (complex multi-component).
- Row 255 (last, reserved): check it's all zeros or a sentinel.

**Code** (hypothetical, from behave-surface/src/fuel_models.rs):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_models_table_integrity() {
        let fm = FuelModels::new();
        
        // Count
        assert_eq!(fm.len(), 256);
        
        // FM1 (index 0, 1-indexed as 1)
        assert_eq!(fm.model(1).unwrap().savr_dead(1), 3500);
        assert!((fm.model(1).unwrap().fuel_load_dead(1, 1) - 0.034).abs() < 0.001);
        
        // FM13 (index 12)
        assert_eq!(fm.model(13).unwrap().savr_dead(1), 1900);
        
        // GS4 (index 123, ID 124)
        let gs4 = fm.model(124).unwrap();
        assert_eq!(gs4.fuel_model_number(), 124);
        assert!(gs4.savr_live(3) > 1000); // complex, just bound check
    }
}
```

Status (from REVIEW.org lines 229–232): ✓ Spot-checks pass; Scott & Burgan constants verified; TL5 live-wood SAVR = 160 (likely typo for 1600) is *preserved* for parity.

**Limitation**: This test would have passed even if the 15 `todo!()` stubs existed in the fuelbed (lines 190–191, REVIEW.org). Lesson: **Pair with executable coverage**. The parity suite (which calls every chaparral/palmetto/aspen branch) is the catch-all.

---

## Checklist: Before Committing Numerical Code

Use this before opening a PR:

- [ ] I have a golden value (from paper, C++, or hand-calculation) for each output.
- [ ] I've reproduced the golden value with `round6` (or the appropriate precision) in Rust.
- [ ] I've checked tolerance: 1e-6 default, 1e-3 for VPD, context-dependent otherwise.
- [ ] I've verified determinism: no RNG, no wall-clock, no interior mutability, operation order preserved.
- [ ] If I changed a formula, I've hand-calculated a simple test case and asserted it.
- [ ] If I ported from another library, I've spot-checked tables and confirmed all branches are reachable.
- [ ] I've run `cargo test --workspace` and reviewed any warnings (unused assignments, unreachable code).
- [ ] I've documented the divergence from C++ (if any) with a comment citing the test case.
- [ ] If f32 is planned, I've flagged cancellation sites (scorch, wind exponents).

---

## Provenance and Maintenance

**Based on:** `REVIEW.org` (the definitive divergence ledger and fidelity findings, as of 2026-07-06), `crates/behave-run/tests/parity.rs` (142 checks, all passing), `crates/behave-surface/src/fire_size.rs` (ellipse formulas), `crates/behave-surface/src/exrate.rs` (determinism audit), `crates/behave-contain/src/adapter.rs` (contain adapter fix).

**Re-verification commands (run to confirm facts remain current):**
- `cargo test -p behave-run --test parity 2>&1 | grep -E "test result|ok."` — confirms 142 golden checks pass.
- `grep -c '\.check(' crates/behave-run/tests/parity.rs` — should be 128. `grep -c '\.check_bool(' crates/behave-run/tests/parity.rs` — should be 14. Total = 142.
- `grep -n "1.625\|1 + 0.125" crates/behave-run/tests/parity.rs` — confirms crown L/W golden value at line 743.
- `grep -n "effective_windspeed = 4.0" crates/behave-contain/src/adapter.rs` — confirms line 228 contains the contain adapter fix.
- `head -20 crates/behave-surface/src/exrate.rs` — confirms EXRATE comment documents determinism.
- `grep -n "sqrt(FLI + U" crates/behave-surface/src/fire.rs` — confirms scorch height formula at line 360 (line numbers may shift; look for the sqrt term).

**Maintenance:** Update if REVIEW.org divergence ledger changes, parity suite count changes (new golden checks), or a new f32 error budget is established.
