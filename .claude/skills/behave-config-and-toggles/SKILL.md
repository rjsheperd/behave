---
name: behave-config-and-toggles
description: Catalog of every behavior axis in the Behave library—input-mode enums, behavior flags, and build options. Defines what is configurable, their variant values, defaults (verified against C++), and the parallelization compatibility checklist for new axes. Use when adding a configuration enum, checking an axis default, or verifying build options.
---

# behave-config-and-toggles

Definitive catalog of behavior configuration in the Behave library. "Configuration" here means:
- **Input-mode enums**: how users specify surface/crown/spot/ignite/contain data (wind height, moisture, two-fuel blending, chaparral type, etc.).
- **Behavior flags**: boolean toggles that change calculation behavior (wind speed limiter).
- **Build options (CMake + Cargo)**: compilation-time feature selection.

## Behavior-Axis Table (HOME)

**Format**: enumeration name, C++ source location, variants (with ordinal = C++), default, production/experimental flag, parity note.

### Wind & Orientation

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **WindHeightInputMode** | DirectMidflame (0) / TwentyFoot (1) / TenMeter (2) | DirectMidflame | Production | C++ ordinal-match required; user enters wind speed at different reference heights; 20-ft and 10-m paths require WAF calculations to convert to midflame. Ref: `firelab-base/src/enums.rs:9` |
| **WindAdjustmentFactorCalculationMethod** | UserInput (0) / UseCrownRatio (1) / DontUseCrownRatio (2) | UseCrownRatio | Production | Affects WAF calculation when wind height is 20-ft or 10-m. Matches C++ `surfaceInputEnums.h`. Ref: `firelab-base/src/enums.rs:33` |
| **WindAndSpreadOrientationMode** | RelativeToUpslope (0) / RelativeToNorth (1) | RelativeToUpslope | Production | Reference frame for wind direction and spread angles. Ordinal = C++. Ref: `firelab-base/src/enums.rs:22` |
| **Wind Speed Limit Flag** (is_wind_limit_enabled) | false / true | **false** | Production | When ON (true), caps effective wind speed at 0.9 × reaction intensity. Changed to OFF by default in BHP1-1367 (2026-02-??). Both C++ and Rust default OFF. Ref: `behave-surface/src/fire.rs:34, line 94` |

### Moisture

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **MoistureInputMode** | BySizeClass (0) / AllAggregate (1) / DeadAggregateAndLiveSizeClass (2) / LiveAggregateAndDeadSizeClass (3) / MoistureScenario (4) | BySizeClass | Production | Determines which moisture classes must be supplied by user. Ref: `firelab-base/src/enums.rs:46` |
| **Moisture class input requirement matrix** | (see section below) | N/A | Reference | Query via `SurfaceInputs::is_moisture_class_input_needed()` for decision table. |

### Moisture Class Input Requirements

Decision matrix for which classes are needed given `MoistureInputMode`:

| MoistureClassInput | BySizeClass | AllAggregate | DeadAgg…Live… | Live…DeadAgg… | MoistureScenario |
|-------------------|:-----------:|:------------:|:-------------:|:-------------:|:----------------:|
| OneHour | ✓ | - | - | ✓ | - |
| TenHour | ✓ | - | - | ✓ | - |
| HundredHour | ✓ | - | - | ✓ | - |
| LiveHerbaceous | ✓ | - | ✓ | - | - |
| LiveWoody | ✓ | - | ✓ | - | - |
| DeadAggregate | - | ✓ | ✓ | - | - |
| LiveAggregate | - | ✓ | - | ✓ | - |

*Note*: When `MoistureInputMode::MoistureScenario`, lookup against `MoistureScenarios` by name; no individual class input is needed. Ref: `behave-surface/src/inputs.rs:906` (is_moisture_class_input_needed).

### Surface Fire Outputs

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **SurfaceFireSpreadDirectionMode** | FromIgnitionPoint (0) / FromPerimeter (1) | FromIgnitionPoint | Production | Directs reporting of fire spread direction: from the ignition point vs. from the fire perimeter. Ref: `behave-surface/src/inputs.rs:20` |

### Two-Fuel Models

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **TwoFuelModelsMethod** | NoMethod (0) / Arithmetic (1) / Harmonic (2) / TwoDimensional (3) | NoMethod | Production | When using two overlapping fuel models, selects blending strategy. TwoDimensional is Finney's map-rotation method. Ref: `behave-surface/src/two_fuel_models.rs:18` |

### Special Fuels: Chaparral

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **ChaparralFuelType** | NotSet (0) / Chamise (1) / MixedBrush (2) | NotSet | Production | Vegetation type for age-based adjustments to fuel properties. Affects live moisture and heat of combustion. Ref: `behave-surface/src/chaparral.rs:14` |
| **ChaparralFuelLoadInputMode** | DirectFuelLoad (1) / FuelLoadFromDepthAndChaparralType (2) | DirectFuelLoad | Production | Method for specifying chaparral fuel loading: direct input vs. calculated from bed depth and type. **Note**: ordinals start at 1 (not 0). Ref: `behave-surface/src/chaparral.rs:24` |

### Special Fuels: Western Aspen

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **AspenFireSeverity** | Low (0) / Moderate (1) | Low | Production | Severity classification affecting mortality calculations. Ref: `behave-surface/src/western_aspen.rs:12` |

### Containment

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **ContainTactic** | HeadAttack (0) / RearAttack (1) | N/A (caller specifies) | Production | Attack strategy: head (toward fire front) vs. rear (toward fire back). Ref: `behave-contain/src/algorithm.rs:16` |

### Spot Fires

| Axis | Variants | Default | Status | Notes |
|------|----------|---------|--------|-------|
| **SpotTreeSpecies** | 14 variants (ordinals 0–13) | EngelmannSpruce (0) | Production | Tree species affecting firebrand loft height in torching and crowning scenarios. See full list below. Ref: `behave-spot/src/inputs.rs:13` |
| **SpotFireLocation** | MidslopeWindward (0) / ValleyBottom (1) / MidslopeLeeward (2) / RidgeTop (3) | MidslopeWindward | Production | Terrain location where spot fire ignites. Affects downwind canopy interaction. Ref: `behave-spot/src/inputs.rs:34` |
| **SpotDownWindCanopyMode** | Closed (0) / Open (1) | Closed | Production | Canopy condition downwind of ignition for spot distance calculation. Ref: `behave-spot/src/inputs.rs:45` |

#### SpotTreeSpecies Full List

```
0: EngelmannSpruce
1: DouglasFir
2: SubalpineFir
3: WesternHemlock
4: PonderosaPine
5: LodgepolePine
6: WesternWhitePine
7: GrandFir
8: BalsamFir
9: SlashPine
10: LongleafPine
11: PondPine
12: ShortleafPine
13: LoblollyPine
```

## CMake Build Options (HOME for C++ builds)

All options in `CMakeLists.txt`. Defaults are per lines 19–27:

| Option | Default | Impact |
|--------|---------|--------|
| TEST_BEHAVE | ON | Builds `testBehave` executable (C++ parity tests). |
| TEST_MORTALITY | ON | Builds mortality module tests (compares against FOFEM). |
| EXAMPLE_APP | ON | Builds example client application. |
| RAWS_BATCH | OFF | Builds RAWS data batch reader CLI. |
| COMPUTE_SPOT_PILE | OFF | Builds pile spot fire distance calculator CLI. |
| COMPUTE_SPOT_SURFACE | OFF | Builds surface spot fire distance calculator CLI. |
| COMPUTE_SPOT_TORCHING_TREES | OFF | Builds torching tree spot fire distance calculator CLI. |

Each option adds a `-D<OPTION_NAME>` C preprocessor definition when enabled.

## Cargo Features (HOME)

**As of 2026-07-06**: **No Cargo feature flags exist** across the workspace (`crates/*/Cargo.toml`).

- No `[features]` section in any crate.
- Rust build is monolithic (`cargo build --workspace`) or per-crate (`cargo build -p <crate>`).
- Future GPU/parallelization code may introduce features like `gpu`, `simd`, `rayon-batch`, but none are planned yet.

## How to Add a New Behavior Axis

**Checklist for adding a configuration enum or behavior flag to the Rust port** (ensures compatibility with ongoing parallelization and parity testing):

1. **Define the enum** in the appropriate crate:
   - Core/cross-crate axes → `firelab-base/src/enums.rs`
   - Surface-only axes → `behave-surface/src/inputs.rs`
   - Spot-only → `behave-spot/src/inputs.rs`
   - Contain-only → `behave-contain/src/algorithm.rs`
   - Weather-only → `behave-weather/src/lib.rs`
   
2. **Match C++ ordinals exactly** if axis crosses C++↔Rust parity tests:
   - Read the C++ source (e.g., `surfaceInputEnums.h`) and verify enum variant values are `= N`.
   - Parity test (`crates/behave-run/tests/parity.rs`) may serialize/deserialize ordinals; mismatch breaks parity.
   - If no parity dependency, ordinals are free to choose (but consistency with C++ is preferred).

3. **Set the default value** to match C++ **exactly**:
   - Find the default initialization in the C++ class (e.g., `surfaceInputs.cpp` constructor or `new()`).
   - Apply same default in the Rust `new()` or `initialize()` method (e.g., `SurfaceInputs::new()` line 104–166).
   - **Verify against parity suite** before landing (run `cargo test -p behave-run --test parity`).

4. **Add to SurfaceInputs struct and initialize** (if applicable):
   - Add field to struct definition.
   - Initialize in `SurfaceInputs::new()`.
   - Add setter and getter methods following the `set_*()` / `*()` pattern.
   - Add unit conversion in getters if needed (use `UnitConversion` trait from `firelab-base`).

5. **Ensure Parallelization Compatibility** (critical for Phase 0–GPU):
   - Enum must `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` (no `String`, no interior mutability).
   - Never use heap-allocated types (Vec, String, Box) in input structs.
   - Enum must be sendable across thread boundaries (no `Rc`, no `RefCell`).
   - Verify no new global state or static tables; all lookups must be pure functions with Copy inputs.

6. **Update parity test** if axis affects outputs:
   - If axis changes any calculated value, add a test case to `crates/behave-run/tests/parity.rs`.
   - Replay the axis in the C++ test sequence and assert the Rust result matches the C++ golden value.
   - Mark test with attribute if comparing against a known C++ bug: `// C++ divergence: explain why`.

7. **Document in REVIEW.org**:
   - Add entry to this skill (behave-config-and-toggles) **in the Behavior-Axis Table**.
   - If axis changes public API surface, add line to REVIEW.org "Open Items" section.
   - If axis touches the parallelization roadmap (e.g., adds branching), note impact in REVIEW.org "Parallelization Outlook".

8. **No feature flags** (unless parallelization phase gate is approved by ownership):
   - Do NOT add a Cargo `[features]` section without consensus.
   - Configuration enums are the default mechanism for user-facing toggles.
   - Feature gates are reserved for internal refactoring phases (e.g., `gpu`, `batch-simd`).

## Re-Verification Commands

**One-liner commands to validate facts in this skill against the repo:**

### Enum Variants & Ordinals
```bash
# WindHeightInputMode
grep -A 6 "pub enum WindHeightInputMode" crates/firelab-base/src/enums.rs

# MoistureInputMode
grep -A 5 "pub enum MoistureInputMode" crates/firelab-base/src/enums.rs

# TwoFuelModelsMethod
grep -A 4 "pub enum TwoFuelModelsMethod" crates/behave-surface/src/two_fuel_models.rs

# All spot enums
grep -A 10 "pub enum Spot" crates/behave-spot/src/inputs.rs

# ContainTactic
grep -A 2 "pub enum ContainTactic" crates/behave-contain/src/algorithm.rs
```

### Defaults
```bash
# SurfaceInputs defaults
sed -n '104,166p' crates/behave-surface/src/inputs.rs

# Wind limit default
grep "is_wind_limit_enabled:" crates/behave-surface/src/fire.rs | head -2

# SpotInputs defaults
sed -n '75,92p' crates/behave-spot/src/inputs.rs
```

### Moisture Input Requirements Matrix
```bash
# is_moisture_class_input_needed implementation
sed -n '906,930p' crates/behave-surface/src/inputs.rs
```

### CMake Options
```bash
grep "^OPTION(" CMakeLists.txt | head -7
```

### Cargo Features
```bash
# Verify no [features] section exists anywhere
grep -r "\[features\]" crates/*/Cargo.toml
# (Should return empty if none defined)
```

### Parity Test Coverage
```bash
# Count individual parity checks in test
grep -E "\.check\(|\.check_bool\(" crates/behave-run/tests/parity.rs | wc -l
# (As of 2026-07-06: 171 checks executed at runtime; 142 static call sites)
```

## Audience & When to Use This Skill

**Load this skill when**:
- You need to verify what defaults a configuration axis has, or confirm its variants.
- You are adding a new configuration enum (see "How to Add" checklist above).
- You are troubleshooting a user-submitted scenario that hinges on a behavior axis (e.g., wind height mode, moisture input mode).
- You are reviewing a pull request that touches configuration enums and need to verify ordinal parity with C++.
- You are authoring the parallelization campaign and need to verify input struct field compatibility.

**Do NOT use this skill for**:
- Understanding the *physical meaning* of a mode (e.g., "what does RelativeToNorth actually do?" → use `fire-behavior-reference`).
- Build environment setup or CI configuration → use `behave-build-and-env`.
- Changing build options or CMake configuration → that is managed in `CMakeLists.txt` directly, not a skill.

## When NOT to use this skill

- To understand what a mode MEANS physically (what is midflame wind, what does a moisture scenario represent) -> `fire-behavior-reference`.
- To set up the build environment or run tests -> `behave-build-and-env` / `behave-run-and-operate`.
- To change a default or add a new axis that alters outputs -> the gates in `behave-change-control` apply; this skill only catalogs the axes.

## Provenance and Maintenance

**Source facts**:
- Enum definitions, variants, and ordinals: read from crate source files (verified 2026-07-06).
- Defaults: verified against `SurfaceInputs::new()`, `SpotInputs::new()`, fire.rs initializers (verified 2026-07-06).
- CMake options: read from `CMakeLists.txt` lines 19–27 (verified 2026-07-06).
- Cargo features: grep across `crates/*/Cargo.toml` for `[features]` section (verified 2026-07-06, none found).
- Parity suite reference: 171 runtime checks in `crates/behave-run/tests/parity.rs` (verified 2026-07-06 via `cargo test -p behave-run --test parity -- --nocapture`; 142 static call sites, four of which loop).

**Re-verification frequency**: After every enum addition or default-value change. Run commands above to catch drift.

**Owner**: Rust port team. Maintainer: behave-change-control skill (coordinates all config changes across C++↔Rust parity).
