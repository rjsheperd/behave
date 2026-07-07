---
name: behave-architecture-contract
description: Load-bearing design decisions, crate dependency graph, invariants (HOME), API-style rationale, parallelization-compatibility contract, and known weak points in the Rust port. Trigger when reviewing crate architecture, API boundaries, invariant violations, or justifying why purity/Copy-inputs/static-tables matter.
---

## Crate Dependency Graph (as of 2026-07-06)

```
firelab-base (no deps)
├── behave-surface
│   ├── behave-crown
│   │   └── behave-spot
│   └── behave-run
├── behave-weather
│   └── behave-run
├── behave-ignite
│   └── behave-run
├── behave-contain
│   └── behave-run
├── behave-mortality
│   └── behave-run
└── behave-run [aggregates all]
    └── depends on all seven domain crates
```

**Boundary rationale** (verified from crates/*/Cargo.toml):

| Crate | Purpose | Why this boundary |
|-------|---------|-------------------|
| `firelab-base` | Units, cross-crate enums, error types | Shared foundation; zero external deps; no computations |
| `behave-surface` | Rothermel 1972 surface fire (11 files) | Core engine; longest chain of computations; highest reuse |
| `behave-crown` | Rothermel 1991 crown fire & containment | Runs TWO Surface instances (Scott & Reinhardt 1998 eq 20, eq 11-13) to cross-check critical open wind speed; **owns Surface inputs**, not surface — crown is caller |
| `behave-spot` | Spot fire distance (firebrand trajectories) | Depends on crown (flame height, energy release), not surface directly (uses dev-dep only) |
| `behave-weather` | Wind/slope/VPD/RH tools | Independent; no fire physics; utility belt; consumed by facade and consumers |
| `behave-ignite` | Lightning/fuel ignition probability | Standalone physics; no fire-size/spread dependencies |
| `behave-contain` | Containment simulation | Standalone (originally expected to depend on surface, does not); adapter pattern wraps simulation |
| `behave-mortality` | Tree survival & biomass mortality | Standalone physics; FOFEM-compatible; uses species lookup tables |
| `behave-run` | Top-level facade | Aggregates all; mirrors C++ `BehaveRun` public fields (mutable setters); library clients call through this |

**Key insight**: Crown owns the two-surface pattern (surface instance reuse), not surface itself. Spot depends on crown output (flame geometry), not surface spread. This enforces data flow: surface spread → crown flame → spot distance.

---

## API Style Contract (Phase-1 Parity Artifact)

The Rust port deliberately mirrors the C++ mutable-setter → do_*_run → getter pattern as a PHASE-1 design. This is **not idiomatic Rust** and will be replaced in Phase 0 (pure-kernel refactor).

### Pattern (verified in crates/behave-surface/src/facade.rs and crates/behave-run/src/lib.rs)

```rust
// Setter: mutates struct field via &mut self
pub fn set_fuel_model_number(&mut self, model: i32) { /* ... */ }

// Run: triggers computation, mutates internal state via &mut self
pub fn do_surface_run_in_direction_of_max_spread(&mut self) { /* ... */ }

// Getter: reads result via &self
pub fn spread_rate(&self, units: SpeedUnits) -> f64 { /* ... */ }
```

### Why this is load-bearing until Phase 0

1. **Parity checkpoint**: Rust and C++ must produce bit-identical results at test boundaries. Mutable setters allow side-by-side validation of each setter/getter.
2. **C++ consumer compatibility**: BehavePlus desktop, IFTDSS, and Behave web app expect this call sequence. The port must accept their code paths without change.
3. **Facade fields are pub**: `BehaveRun.surface`, `BehaveRun.crown`, etc. are public (verified in crates/behave-run/src/lib.rs:29-41) to mirror C++ and allow consumers direct field access.
4. **Blocks trivial batching until Phase 0**: This API shape (mutable state inside modules) makes parallel rayon batching awkward; pure-kernel refactor (consuming Copy inputs, returning outputs) fixes this.

### What NOT to do

- **Do NOT "idiomize" this piecemeal** into `Builder` or `new_with_*` patterns. That breaks parity validation and consumer compatibility.
- **Do NOT make fields private** without C++ side coordination (they are access points for consumers).
- **Do NOT add `Option<_>` or `Result<_>` wrapper layers** — C++ returns f64, Rust returns f64, even for edge cases (return 0.0 for invalid inputs).

**Sanctioned replacement** (cross-ref `behave-parallelization-campaign`): Phase 0 pure-kernel refactor. New public entry points like `pub fn run_surface_fire(inputs: &SurfaceFireInputs) -> SurfaceFireOutputs` (all Copy inputs, struct outputs) will coexist; old mutable-setter API will be marked `#[deprecated]` and wrapped around the kernel.

---

## INVARIANTS (Home Authority)

Verify each invariant with the command in the **Verification** row. Violations are load-bearing.

### Invariant 1: All Math in f64 (IEEE Double)

**Rationale**: C++ behaveUnits.cpp uses double; consumers (BehavePlus, web) rely on consistent 6-7 decimal places. f32 causes catastrophic cancellation (scorch height formula, wind-factor exponentials); ruled out until Phase 3 f32-study.

**Command to verify**:
```bash
grep -rn 'f32' crates/behave-*/src/ --include='*.rs' | grep -v '//' | head -5
```
**Expected**: Empty (no f32 in source code; only in Phase-3 candidate lists).

### Invariant 2: Base Units per Quantity (Defined in firelab-base/src/units.rs)

**Rationale**: All internal storage and inter-crate calls use base units. Getters/setters convert on boundaries. This prevents the [5+ unit-conversion incidents](https://github.com/firelab/behave/issues) that plagued C++.

**Base unit table** (verified from firelab-base/src/units.rs lines 20–609):

| Quantity | Base Unit | Impl Lines | Notes |
|----------|-----------|------------|-------|
| Area | SquareFeet (ft²) | 24–58 | 6 variants |
| BasalArea | SquareFeetPerAcre (ft²/ac) | 64–87 | forestry input |
| Length | Feet (ft) | 94–133 | 8 variants; 1 chain = 66 ft |
| Loading | PoundsPerSquareFoot (lb/ft²) | 140–167 | fuel bed mass |
| Pressure | Pascal (Pa) | 175–218 | **INVERTED**: to_base divides, from_base multiplies (C++ quirk, line 171-172 comment) |
| SAVR | SquareFeetOverCubicFeet (ft²/ft³) | 225–252 | surface-area-to-volume |
| Speed | FeetPerMinute (ft/min) | 259–295 | 7 variants; chains/hour = 1/10 conversion |
| Fraction | Fraction [0,1] | 302–323 | 0% = 0.0, 100% = 1.0 |
| Slope | Degrees | 330–351 | atan-based conversion from percent |
| Density | PoundsPerCubicFoot (lb/ft³) | 358–379 | fuel bed density |
| HeatOfCombustion | BtusPerPound (Btu/lb) | 386–407 | ~8000 for wood |
| HeatSink | BtusPerCubicFoot (Btu/ft³) | 414–435 | unused in surface, used in mortality |
| HeatPerUnitArea (FLI) | BtusPerSquareFoot (Btu/ft²) | 442–466 | byram fireline intensity |
| HeatSourceAndReactionIntensity | BtusPerSquareFootPerMinute | 473–503 | reaction intensity (\*Gamma* in eq 27) |
| FirelineIntensity | BtusPerFootPerSecond (Btu/ft/s) | 510–542 | line intensity (*I* in Byram) |
| Temperature | Fahrenheit (°F) | 549–572 | NO zero-passthrough (unlike others) |
| Time | Minutes | 579–609 | 5 variants |

**Command to verify**:
```bash
grep 'fn to_base\|fn from_base' crates/firelab-base/src/units.rs | wc -l
```
**Expected**: `36` (2 trait definitions + 2 per enum × 17 enums = 34 implementations).

### Invariant 3: Zero Unsafe Code

**Rationale**: Library must be sound for parallel/GPU execution. No ptr casts, no transmute, no unguarded FFI. C++ linkage happens at crate boundaries only (if at all).

**Command to verify**:
```bash
grep -rn 'unsafe' crates/ --include='*.rs' 2>/dev/null | wc -l
```
**Expected**: `0` (no matches).

### Invariant 4: Zero Panics/Unwraps in Library Code

**Rationale**: Panics abort the process; library must fail gracefully (return 0.0, NaN, or signal via output). Unwraps are hidden panics.

**Exceptions** (documented):
- One documented assert in `crates/behave-surface/src/exrate.rs:456` (laterals-not-ported guard; execution path guarded by callers).

**Command to verify**:
```bash
grep -rn 'panic!\|\.unwrap()\|\.expect(' crates/behave-*/src/ 2>/dev/null | grep -v '#\[cfg(test)\]' | grep -v 'mod tests'
```
**Expected output** (as of 2026-07-06):
```
crates/behave-mortality/src/species.rs:471:        let last = t.records.last().unwrap();
crates/behave-mortality/src/canopy.rs:125:        let r = t.get_by_index(1).unwrap();
crates/behave-mortality/src/canopy.rs:135:        let r = t.get_by_index(15).unwrap();
crates/behave-mortality/src/canopy.rs:145:        let r = t.get_by_index(39).unwrap();
```

**Status**: Known gap. These are in table lookups with static indices (hardcoded 1, 15, 39); parity test `crates/behave-run/tests/parity.rs` validates correctness. Phase 1 acceptable; refactor to `get_unchecked` or Result-wrapping in Phase 2.

### Invariant 5: Fuel Model Tables Bit-Exact to C++

**Rationale**: Scott & Burgan 40-model table is USFS canonical; any divergence breaks downstream consumers (BehavePlus displays, published FVD). Rothermel 13-model also critical.

**Verification approach** (parity suite is the authority):
- Parity test runs Surface over all 40 models with fixed inputs (fuel moisture, wind, slope, aspect).
- Outputs (ROS, flame length, etc.) compared to C++ `testBehave` output with 6-decimal-place tolerance.
- Tables verified at test start-time by iterating model numbers 1–40 and checking defined/dynamic flags.

**Command to check table sizes**:
```bash
grep -c 'const.*MODEL' crates/behave-surface/src/fuel_models.rs
```
**Expected**: ~50 constants (model definitions + metadata).

**Parity test command**:
```bash
cargo test -p behave-run --test parity 2>&1 | tail -3
```
**Expected**: `test result: ok. 1 passed; 0 failed` (single test with 171 internal assertions, all passing).

### Invariant 6: No Interior Mutability or Global State

**Rationale**: Interior mutability (`RefCell`, `Mutex`, `Cell`) blocks parallelization (data races in rayon batches). Globals (`static mut`, `lazy_static`, `OnceLock` with mutation) are non-shareable across threads. Pure-kernel approach requires immutable shared state.

**Command to verify**:
```bash
grep -rn 'RefCell\|Mutex\|Cell\|static mut\|lazy_static\|OnceLock' crates/ --include='*.rs' 2>/dev/null
```
**Expected**: Empty.

**Note**: `OnceLock` for read-only initialization is safe (e.g., precomputed lookup tables); grep result will include those. Manual review required if any hits; as of 2026-07-06, none exist.

### Invariant 7: Enum Ordinals Match C++ Enums

**Rationale**: Parity test compares ordinal values at boundaries (e.g., `FuelLifeState::Dead as i32 == 0`). C++ consumers may cast integers to enums or vice versa. Divergence breaks FFI compatibility.

**Example** (verified in crates/firelab-base/src/enums.rs):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuelLifeState {
    Dead = 0,
    Live = 1,
}
```
Maps to C++ `enum FuelLifeState { DEAD = 0, LIVE = 1 }`.

**Verification** (parity test validates indirectly; direct check via source comparison):
```bash
grep -A 5 'pub enum FuelLifeState\|pub enum WindHeightInputMode' \
  crates/firelab-base/src/enums.rs | head -30
```

### Invariant 8: Clone Everywhere (for future per-thread use)

**Rationale**: Phase 1.5 (rayon batching) will clone inputs per task; Phase 3 (GPU) will clone to device memory. All public structs must be `Clone` or explicitly document why they cannot be.

**Verification**:
```bash
grep -rn 'pub struct' crates/behave-*/src/lib.rs crates/behave-*/src/facade.rs | while read line; do
  file=$(echo "$line" | cut -d: -f1)
  struct=$(echo "$line" | sed 's/.*pub struct //' | cut -d' ' -f1)
  grep -q "derive.*Clone" "$file" || echo "MISSING Clone: $struct in $file"
done
```
**Expected**: Empty (all pub structs have `#[derive(Clone)]` or are wrapped in Arc/Rc).

---

## Parallelization-Compatibility Contract

**Summary** (verify entire text in `behave-parallelization-campaign` skill):

The Rust port locks in the parallelization roadmap: pure-kernel (Phase 0) → rayon batch (Phase 1.5) → f32 study (Phase 2) → SoA+SIMD (Phase 2.5) → WebGPU/WGSL (Phase 3).

**Mandatory for each change**:

1. **Copy inputs, no interior mutability**: Every input struct in public APIs must be `Copy` or movable without state changes. No `Cell`, `RefCell`, `Mutex`.
2. **Static tables shareable**: Fuel model tables (currently ~10 KB, cloned per Surface instance) must stay `&'static` or boxed once in Phase 0.
3. **Output as struct, not &mut**: Instead of `pub fn spread_rate(&self) -> f64` + mutable side effects, future API: `pub fn run(inputs: &Inputs) -> Outputs` where `Outputs` is Copy.
4. **f64 holds until Phase 2 ends**: f32 risk spots (scorch-height sqrt cancellation, wind-factor exponentials) must not go live until f32-study validation.
5. **No panics in compute loops**: Panics in GPU shaders are undefined. Graceful degradation (return 0.0 or NaN) required.
6. **Enum dispatch stays narrow**: `facade.rs` switch on fuel-model-kind (lines 61–102 verified) is the ONLY fat dispatch; all other branches must be thin (inline-cost < 50 cycles).

**Checklist for PRs** (cross-ref `behave-change-control`):
- [ ] No new `static`, `RefCell`, `Mutex`, or unsafe code.
- [ ] All new public structs are `Clone` and/or `Copy`.
- [ ] Fuel model table unchanged or refactored to eliminate per-instance clones.
- [ ] No new conditional panics in paths marked for GPU (currently all surface/crown fire).

---

## Known Weak Points (Verified 2026-07-06)

### 1. Behave-Run Facade Skeleton (134 lines, incomplete I/O plumbing)

**File**: crates/behave-run/src/lib.rs:44–106 (lines verified).

**Status**: Facade aggregates all modules but lacks:
- No serialization (JSON/CSV input → BehaveRun config).
- No output bundling (BehaveRun → results struct with all metrics).
- Only fuel-model delegation methods; no surface/crown/spot result getters.
- Consumers still call through Surface/Crown directly; facade is assembly only.

**Impact**: Low. Clients work around facade; parity test calls individual module methods, not facade.

**Phase 0 target**: Scaffold full I/O facade with input/output builders.

### 2. Wind.rs 10-m Conversion Paths Partial

**File**: crates/behave-surface/src/wind.rs:151–155 (verified).

**Status**: `WindSpeedUtility::ten_meter_to_twenty_foot` exists (line 153) but no direct 10-m → midflame path. Consumers must call:
```rust
let waf = wind_adj.calculate_with_crown_ratio(...);
let midflame = WindSpeedUtility::ten_meter_to_twenty_foot(wind_10m) * waf;
```

**Impact**: Low. Clients implement wrapper; parity test does not exercise 10-m input.

**Phase 1 target**: Add `Surface::set_wind_speed_input_mode(WindHeightInputMode::TenMeter)` to abstract conversion.

### 3. Behave-Surface Fire-Size Stub

**File**: crates/behave-surface/src/fire_size.rs (6 lines verified; re-exports from firelab-base).

**Status**: Unnecessary re-export; crowns owns FireSize lifetime and passes &mut to calculations. Stub serves only for `use behave_surface::FireSize`.

**Impact**: Negligible. Clutter; no functional bug.

**Phase 1 target**: Remove; clients import from firelab-base directly (or rethink ownership in Phase 0 pure-kernel).

### 4. Fuel Model Table Cloned Per Surface Instance (~10 KB × N instances)

**File**: crates/behave-surface/src/facade.rs:35 (Surface owns FuelModels).

**Status**: FuelModels struct (Scott & Burgan 40-model table) is cloned into each Surface instance. Memory: ~10 KB × (num surfaces in rayon batch) = potential ×1000 overhead.

**Impact**: Medium for GPU/large-batch scenarios. Negligible for single-run desktop app.

**Phase 0 target**: Refactor to `Surface<'a>` with `&'a FuelModels`, or lazy-static-init wrapper with &'static reference.

### 5. No Rust CI

**File**: .github/workflows/ci.yml (C++ tests only; Rust has zero coverage).

**Status**: CI runs `make test` (C++ `testBehave`). Rust parity test (171 checks) runs only locally or in developer pre-commit hooks. Regressions can land on master.

**Impact**: Medium. Parity suite is comprehensive (full C++ replay), but no gate before merge.

**Phase 0 target**: Add GitHub Actions job: `cargo test --workspace && cargo test -p behave-run --test parity`.

### 6. No License Metadata in Cargo.toml

**File**: crates/*/Cargo.toml (all 9 crates lack `license` field).

**Status**: Public-domain intent (per repo README), but Cargo.toml entries empty. Consumers cannot auto-audit licensing.

**Impact**: Low. Metadata only; no functional issue.

**Phase 1 target**: Add `license = "CC0-1.0"` (or appropriate) to all crate Cargo.tomls.

### 7. Mutable API Blocks Trivial Batching

**File**: crates/behave-surface/src/facade.rs (entire Surface API uses `&mut self`).

**Status**: Each batch task needs its own Surface instance (cannot share), then clone FuelModels into it (invariant 4 violation). Blocks `rayon::scope { |s| for task in batch { s.spawn(||  surface.do_run()) } }` until Phase 0 pure-kernel.

**Impact**: High for parallelization roadmap; negligible for current desktop use.

**Mitigation** (temporary): Phase 1.5 will wrap old mutable API in pure-kernel functions: `pub fn run_surface(inputs: &Inputs) -> Outputs { let mut s = Surface::new(); s.apply_inputs(inputs); s.do_run(); s.get_outputs() }`.

---

## When NOT to Use This Skill

- **For domain equations or fire behavior theory**: Use `fire-behavior-reference` (models, equations, units, conventions).
- **For making changes or PR gates**: Use `behave-change-control` (change classes, incident ledger, compatibility checklist).
- **For debugging or triage**: Use `behave-debugging-playbook` (symptom → experiments).
- **For research or open problems**: Use `behave-research-frontier` (candidate ideas, evidence bar).

---

## Provenance and Maintenance

**Based on**:
- Cargo.toml inspection: crates/*/Cargo.toml (9 crates verified).
- Units table: crates/firelab-base/src/units.rs lines 1–609 (verified; 17 enums, 34 conversion methods).
- Facade audit: crates/behave-run/src/lib.rs (134 lines) + crates/behave-surface/src/facade.rs (first 100 lines).
- Parity test: crates/behave-run/tests/parity.rs (171 checks, all passing as of 2026-07-06).
- REVIEW.org findings table (open items section).
- Context pack (git archaeology 2016–2026, incident ledger, parallelization roadmap).

**To re-verify volatile facts**:

1. **Crate DAG and dependency chain**:
   ```bash
   # (run from repo root)
   for c in firelab-base behave-surface behave-crown behave-spot behave-weather \
            behave-ignite behave-contain behave-mortality behave-run; do
     echo "=== $c ===" && grep -A10 '\[dependencies\]' crates/$c/Cargo.toml | head -15
   done
   ```

2. **Base unit count**:
   ```bash
   grep -c 'pub enum.*Units' crates/firelab-base/src/units.rs
   ```
   Expected: `17`.

3. **No unsafe, no panics in library**:
   ```bash
   grep -rn 'unsafe' crates/behave-*/src/ && echo "FAIL: unsafe found" || echo "OK: no unsafe"
   grep -rn 'panic!\|unwrap()\|expect(' crates/behave-*/src/ | grep -v '#\[cfg(test)\]' | wc -l
   ```

4. **Parity suite status**:
   ```bash
   cargo test -p behave-run --test parity 2>&1 | tail -3
   ```
   Expected: `test result: ok. 1 passed; 0 failed` (one test containing 171 assertions).

5. **Facade line count**:
   ```bash
   wc -l crates/behave-run/src/lib.rs
   ```
   Expected: `134`.

6. **Fire-size stub presence**:
   ```bash
   find crates -name fire_size.rs -type f
   ```
   Expected: two files (firelab-base, behave-surface stub).

**Last verified**: 2026-07-06, branch `rj-rust-port`, HEAD commit f11cbc5 (REVIEW.org, parity suite passing).
