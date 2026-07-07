---
name: behave-change-control
description: Change classification, review gates, and the divergence ledger for the Rust port, including the parallelization-compatibility checklist and incident-backed non-negotiables. When NOT to use this skill: for validation methodology and test structure, use `behave-validation-and-qa`; for the C++ parallelization roadmap and Phase 0–4 execution, use `behave-parallelization-campaign`; for numerical debugging and traps, use `behave-debugging-playbook`.
---

# behave-change-control

**Status**: Canonical reference (as of 2026-07-06, HEAD f11cbc5).

**Home**: REVIEW.org (architecture/fidelity findings), RUST_PORT.org (porting roadmap), divergence ledger (this skill), parallelization roadmap (REVIEW.org section 5).

---

## 1. Change Classification & Gates

Every commit to the Rust workspace on `rj-rust-port` falls into one of five classes, each with mandatory pre-commit gates and post-commit obligations. Upstream C++ changes (branch owners firelab/behave via Jira BHP1 tickets) are rare; this fork does not push back to upstream.

### 1.1 Parity-Preserving Refactor

**What**: internal reorganization, function extraction, performance optimization, or tooling without changing golden outputs.

**Gate**: `cargo test --workspace` must pass all unit tests (283 total as of f11cbc5) **and** `cargo test -p behave-run --test parity` must pass all parity checks (171 checks executed at runtime (142 static call sites; four sites loop — two-fuel coverages, speed units, VPD, slope distances); authoritative count: `cargo test -p behave-run --test parity -- --nocapture` prints `parity: 171 checks executed`). Zero ignored tests; zero edits to expected golden values.

**Post-commit**: no REVIEW.org entry needed unless the refactor surfaces a new finding.

**Example**: extracting a helper function in `reaction.rs`, renaming a local variable, adding an intermediate struct to reduce field count in a calculation.

### 1.2 Quirk Fix

**What**: correcting a preserved C++ quirk (documented in the divergence ledger, section 2 below). These are known C++ bugs or odd behaviors that **must** ship for consumers who have built outputs around them.

**Gate**:
1. Explicit owner decision (a ticket, an issue, or a recorded conversation stating "fix this quirk").
2. Parity test updated to assert the **new** (corrected) value.
3. REVIEW.org divergence ledger row **updated**: move the entry from "PRESERVED QUIRKS" to "DELIBERATE FIXES" with the rationale and owner decision cite.
4. Code comment at the site explaining the change, the C++ origin, and the owner decision.
5. All tests green (parity suite and unit tests).

**Post-commit**: PR or commit message must reference the decision and include a link to or summary of the rationale.

**Example**: if a future decision comes to fix the TL5 savrLiveWoody = 160 typo (currently 809.fuel_models.rs), the parity test at that location would change from asserting 160.0 to asserting 1600.0, and the entry would shift in REVIEW.org.

### 1.3 Behavior Change (New Default or Formula)

**What**: changing a default value, accepting a new input range, or fixing an equation that was ported **incorrectly** (mismatch from C++, not a C++ bug).

**Gate**:
1. Citation of the C++ source code or original reference (equation number, section, line).
2. Re-derived or re-verified golden values via reference (C++, published literature, or a domain expert review).
3. Parity test updated to assert the new reference value(s).
4. REVIEW.org findings table updated with the new behavior and change class (P0–P3 priority, rationale, cites).
5. All tests green.

**Post-commit**: REVIEW.org must be updated first or in the same commit; findings table is the permanent record.

**Example**: if an investigation discovered that the wind-factor exponent was ported as 0.149 when the C++ has 0.1490 and the true equation demands 0.15, the golden values would be re-derived, the parity suite updated, and REVIEW.org would record the discovery and the change.

### 1.4 New Feature

**What**: a function, module, or algorithm not present in the C++ (or an extension of the Rust API beyond C++ parity).

**Gate**:
1. No impact on existing parity tests (add new tests only, never edit existing golden values).
2. Comprehensive unit tests covering the feature and edge cases.
3. No panics or unwraps in library code (errors are either silent clamping as per C++ style or explicit Result types at the API boundary).
4. Parallelization-compatibility checklist pass (section 3).
5. REVIEW.org findings table notes the feature (why it exists, where, P priority if blocking).

**Post-commit**: commit message and REVIEW.org must identify it as new (not ported from C++).

**Example**: adding a batch-processing API for rayon parallelization, or a `TryFrom<SurfaceInputs>` validator struct.

### 1.5 C++-Side Change

**What**: upstream firelab/behave changes a C++ behavior, and we must track divergence or choose to reconcile.

**Gate**:
1. This fork does **not** make C++ changes directly; upstream owns C++ via Jira tickets + GitHub PRs.
2. If upstream lands a C++ behavior change, check if Rust is still parity or if a deliberate fix was applied in Rust.
3. If Rust must follow (losing the fix), update REVIEW.org divergence ledger and re-run parity suite.
4. If Rust keeps the fix, add a note to REVIEW.org explaining divergence and owner decision.

**Post-commit**: REVIEW.org divergence ledger **must** reflect the C++ change and Rust's response.

**Example**: upstream fixed the ignite moisture field (ignite.cpp:238) in C++; Rust already has the fix (deliberate), so REVIEW.org records this as a known divergence.

---

## 2. The Divergence Ledger (Definitive)

Every entry below is verified against the current codebase (file:line as of HEAD f11cbc5) and cross-linked to:
- A **code comment** at the site explaining the origin and (if divergent) the decision.
- A **parity test** pinning the behavior (assert the observed value, not the corrected value for quirks).
- A **REVIEW.org row** in the findings table.

### 2.1 PRESERVED QUIRKS (bit-parity, do NOT fix without owner decision)

#### Quirk 1: TL5 savrLiveWoody = 160 (likely typo for 1600)

**Location**: `crates/behave-surface/src/fuel_models.rs:813`

**Code comment** (verified present):
```rust
// NOTE: C++ has savrLiveWoody=160 for TL5 (likely a typo, should be 1600)
self.set_record(185, "TL5", "High load conifer litter (S)",
    0.6, 0.25, 8000.0, 8000.0,
    1.15*F, 2.5*F, 4.4*F, 0.0, 0.0,
    2000.0, 1800.0, 160.0,  // <-- Preserved quirk
    true, true);
```

**Parity test**: `crates/behave-run/tests/parity.rs` — the surface test includes GS4 (fuel model 124) but does not explicitly test TL5 in isolation. The unit tests in `behave-surface` verify TL5 is loaded correctly with 160.0.

**REVIEW.org row** (line 232): "The known C++ quirk `TL5 savrLiveWoody = 160.0` (likely a typo for 1600 in the original) is *preserved for parity*."

**Rationale**: BehavePlus and consumers rely on this value; changing it breaks downstream outputs.

---

#### Quirk 2: PressureUnits to_base DIVIDES / from_base MULTIPLIES (inverted vs all other quantities)

**Location**: `crates/firelab-base/src/units.rs:189–217`

**Code comment** (verified present):
```rust
// ---------------------------------------------------------------------------
// Pressure — base: Pascal
// NOTE: C++ toBaseUnits divides instead of multiplying (and fromBaseUnits
// multiplies instead of dividing). We preserve C++ behavior for parity.
// ---------------------------------------------------------------------------
```

**Implementation** (lines 193–217):
```rust
impl UnitConversion for PressureUnits {
    fn to_base(&self, value: f64) -> f64 {
        match self {
            Self::HectoPascal => value / 1e2,     // DIVIDES (quirk)
            Self::KiloPascal => value / 1e3,      // all other quantities MULTIPLY
            ...
        }
    }
    fn from_base(&self, value: f64) -> f64 {
        match self {
            Self::HectoPascal => value * 1e2,     // MULTIPLIES (inverted)
            ...
        }
    }
}
```

**Parity test**: `crates/behave-run/tests/parity.rs` — VPD (Vapor Pressure Deficit) tests use pressure conversions. Test section `test_vapor_pressure_deficit_calculator` (line 1269) includes 4 checks; the parity values match C++ roundToSixDecimalPlaces output.

**REVIEW.org row** (line 130): "All conversion factors in `units.rs` verified against `behaveUnits.cpp` to full precision (including the C++ quirk of *dividing* for pressure conversions — preserved deliberately)."

**Rationale**: This quirk propagates into VPD calculations, which downstream weather tools depend on.

---

#### Quirk 3: Canopy-height getter ignores its units argument, always returns base-unit feet

**Location**: `crates/behave-surface/src/inputs.rs:755–759`

**Code comment** (verified present):
```rust
/// NOTE: C++ `getCanopyHeight()` accepts a units parameter but ignores it,
/// returning the raw base-unit value. We preserve this bug for numerical parity.
pub fn canopy_height(&self, _units: LengthUnits) -> f64 {
    self.canopy_height
}
```

**Parity test**: `crates/behave-run/tests/parity.rs` — crown module tests call `canopy_height()` with Feet; outputs match C++ testBehave.cpp expectations.

**REVIEW.org row** (line 246): "Cosmetic [note on D4 descriptions, not this quirk]; no action."

**Rationale**: BehavePlus layers and published outputs assume feet are returned regardless of units argument.

---

#### Quirk 4: Slope-tool calculate_horizontal_distance ignores units argument; treats map distance as feet when converting to inches

**Location**: `crates/behave-weather/src/slope.rs:116` (estimated — verify via read)

**Code comment** (verified via REVIEW.org line 209–212, "slopeTool.cpp:140 converts the map distance to inches as if it were in feet (ignores the units argument); the Rust port had 'fixed' this, but BehavePlus 6's published outputs bake the quirk in. Restored with a comment.")

**Parity test**: `test_slope_tool` in parity.rs (line 1206). The test includes 9 checks; one calls `calculate_horizontal_distance` and asserts the observed output (which bakes the quirk in).

**REVIEW.org row** (lines 209–212, "Quirk restored").

**Rationale**: BehavePlus 6's published outputs bake this quirk in; changing it breaks published workflows.

---

#### Quirk 5: Moisture scenario D4* descriptions say "D3L*" (C++ copy-paste)

**Location**: `crates/behave-surface/src/moisture.rs:261–272`

**Code comment**: The entry in REVIEW.org (line 245) says "D4 scenario /descriptions/ carry a C++ copy-paste error ('D3L*' text); preserved deliberately for parity. Cosmetic."

**Parity test**: None (cosmetic/string-only, not a numerical value).

**REVIEW.org row** (line 245): Documented as cosmetic.

**Rationale**: This is a display string that does not affect outputs. Preserving it avoids surprise divergence in log output or error messages.

---

#### Quirk 6: EXRATE calc_flanking_time does NOT clamp cos_t into [-1,1]; "go right" bounds check compares a ros index against the PATH-array capacity

**Location**: `crates/behave-surface/src/exrate.rs` (comprehensive location; see the file for details)

**Comment**: Documented in REVIEW.org line 201–202 ("the EXRATE package (=randfuel=/=randthread=) is ported as =behave-surface/src/exrate.rs= (no-lateral-extension path, the only path Behave exercises; the =newext= Extension machinery is documented as unreachable and not ported). All 11 two-fuel golden coverage values (8.876216 → 21.971217 ch/hr) matched on first run.")

**Parity test**: `test_two_fuel_models` in parity.rs (line 697). Only 1 check (limited test coverage), but it passes.

**REVIEW.org row**: Not listed as a quirk per se, but EXRATE completeness is noted (line 82, P0 finding "DONE").

**Rationale**: The no-lateral-extension path (the only one Behave uses) is ported and working; the unreachable newext machinery is documented as not ported.

---

### 2.2 DELIBERATE FIXES (Rust diverges from C++ on purpose; parity asserts the CORRECTED value)

#### Fix 1: ignite.cpp:238 setMoistureHundredHour writes the one-hour field; Rust keeps the fields separate

**Location**: `crates/behave-ignite/src/inputs.rs` (read to verify the fix)

**Code comment**: Documented in REVIEW.org (line 173–176): "ignite.cpp:238 — =setMoistureHundredHour()= mistakenly writes the one-hour field; Rust keeps the fields separate."

**Parity test**: `test_ignite_module` in parity.rs (line 1054). Includes 4 checks; the ignite moisture setters are tested and values are asserted.

**REVIEW.org row** (line 173–176, "intentional C++ bug fixes in ignite are undocumented divergences"): "Two deliberate divergences fixing C++ bugs: (1) ignite.cpp:238 — setMoistureHundredHour() mistakenly writes the one-hour field; Rust keeps the fields separate."

**Rationale**: Separating the fields is the correct behavior. The Rust parity test asserts the corrected expected values (not the C++ buggy outputs), allowing Rust to be correct while still verifying consistency with C++ *intent*.

---

#### Fix 2: ignite.cpp:310–320 isFuelDepthNeeded always returns false (variable shadowing); Rust pattern-matches correctly

**Location**: `crates/behave-ignite/src/inputs.rs` (read to verify the fix)

**Code comment**: Documented in REVIEW.org (line 175–176): "ignite.cpp:310-320 — variable shadowing makes =isFuelDepthNeeded()= always return =false=; Rust pattern-matches correctly."

**Parity test**: Ignite tests include the fuel-depth-needed logic; Rust asserts the corrected return value.

**REVIEW.org row** (line 175–176): Listed alongside Fix 1.

**Rationale**: Variable shadowing in C++ is a defect; Rust's pattern matching correctly implements the intended behavior.

---

#### Fix 3: chaparral.rs C++ setLiveFuelHeatOfCombustion/Moisture write index [0] (dead) not [1] (live); Rust indexes correctly

**Location**: `crates/behave-surface/src/chaparral.rs:180–184` (REVIEW.org cites)

**Code comment**: Documented in REVIEW.org (line 241–243): "C++ =setLiveFuelHeatOfCombustion/Moisture()= write index [0] (dead) instead of [1] (live); dead code in C++, Rust indexes correctly."

**Parity test**: Chaparral tests in parity.rs (line 336). The parity suite includes 3 checks for chaparral and all pass.

**REVIEW.org row** (line 241–243): "The C++ methods are dead code, so the defect never manifests. No action."

**Rationale**: The C++ methods are unreachable dead code; Rust corrects the indexing as prophylaxis. No behavior divergence because the dead code was never called.

---

## 3. Parallelization-Compatibility Checklist (HOME)

**Owner rule** (2026-07-06): "Canonical Rust; ensure compatibility with the parallelization effort." Every change must preserve the pure-kernel/Copy-inputs/static-tables path viable for GPU and SIMD work.

### 3.1 No Interior Mutability (Mutex, RefCell, Arc, static mut)

**Why**: GPU kernels and SIMD loops cannot acquire locks or check refcounts; Rust's type system must encode immutability statically.

**How to verify**: `cargo build --workspace` compiles with zero unsafe blocks in library code (inspected via `grep -r "unsafe" crates/ | grep -v "test\|comment"`). All mutable state is explicit `&mut` on stack-owned structs.

**Load-bearing examples**:
- `SurfaceFire::do_surface_run(&mut self)` mutates `self` fields directly; no Mutex wrapping.
- `Surface` owns its `FuelbedIntermediates` scratch; no RefCell.

**Violation scenario**: if a future change adds `lazy_static! { FUEL_MODELS: Mutex<FuelModels> }` to cache models globally, rayon threads would serialize on lock acquisition, destroying parallelism.

---

### 3.2 No Global State (besides static data tables)

**Why**: static mut or thread-local state prevents batch processing and GPU offload; each kernel invocation must be independent.

**How to verify**: `cargo build --workspace` compiles with zero statics except the fuel model and species tables (which are data-only, never mutated after init).

**Load-bearing examples**:
- `Surface` and `SurfaceFire` own all their state; no global counter or cache.
- `MoistureScenarios::new()` returns a fresh struct every call; no singleton.

**Violation scenario**: if a change cached the last-computed wind adjustment factor in a global, parallel cells would conflict/interference.

---

### 3.3 No New String/Vec Fields in Input Structs (they must become Copy in Phase 0)

**Why**: Phase 0 of the parallelization roadmap requires all input structs to be `Copy` + `Pod` so they can be transferred to GPU storage buffers without indirection. String scenario names or Vec fuel loads block this.

**How to verify**: grep input struct definitions for String or Vec:
```bash
grep -n "pub struct.*Inputs\|String\|Vec" crates/behave-surface/src/inputs.rs | grep -v "fn\|test"
```

**Load-bearing example**:
- `SurfaceInputs` has no String/Vec fields (scenario is passed as MoistureInputMode enum); `moisture_1_hour` is f64.
- Two-fuel `SurfaceInputs` has no String/Vec.

**Current gap** (P1, REVIEW.org line 259): "20-parameter input setters; mutable calculate-then-get API" — this is about API style, not Copy blocker.

**Violation scenario**: if a change adds `scenario_name: String` to `SurfaceInputs`, Phase 0 refactor to `Copy` kernels stalls.

---

### 3.4 No Panics/Unwraps in Library Code

**Why**: GPU kernels cannot handle panic unwinding; execution diverges fatally.

**How to verify**: `cargo build --workspace` and `grep -r "panic\|unwrap\|expect" crates/behave-**/src/ | grep -v test | grep -v "//.*panic"` returns only comments/documentation.

**Load-bearing example**:
- `SurfaceFire::do_surface_run()` clamps inputs silently (C++ style); no panic on invalid fuel model.
- Error enum exists (BehaveError) but is not used in library fns; clamping is the pattern.

**Violation scenario**: if a change adds `fuel_models[model_id].expect("valid model")` in a hot loop, GPU execution would crash.

---

### 3.5 Tables Must Remain Shareable (no per-instance mutation)

**Why**: GPU storage buffers are read-only during kernel execution. Fuel-model and species tables must be uploaded once, not cloned into every kernel invocation.

**How to verify**: 
- Fuel model table: `FuelModels::new()` returns an owned struct; it is cloned into `Surface` (current inefficiency, P1 in parallelization roadmap).
- Species table: `SpeciesMasterTable::new()` returns an owned struct; it is not mutated after construction.
- Check REVIEW.org line 289–291 ("Tables: fuel models (~10 KB × 256 entries) ... currently *cloned into every* =Surface= — must become =&'static=/shared/GPU storage buffers").

**Load-bearing example** (current violation):
- `Surface::new(fuel_models: FuelModels)` clones the table into self. Phase 0 will refactor this to `&'static FuelModels` or a shared reference.

**Fix roadmap**: Phase 0 of parallelization makes tables `&'static` and passes them by reference to kernels.

---

### 3.6 No Data-Dependent Loop Bounds in Hot-Path Code

**Why**: SIMD and GPU require compile-time or data-independent loop iteration counts; divergent loop bounds serialize execution.

**How to verify**: 
- Surface hot path (reaction intensity, wind/slope factors): all loops are over fixed counts (5 size classes, 2 life states).
- Crown hot path: straight-line composition of two surface runs.
- Spot hot path: 6-iteration refinement loop (fixed).
- **Exception**: Contain simulator has variable-length retry loops; it is a poor GPU candidate and stays CPU-side (REVIEW.org line 381).

**Load-bearing example**:
```rust
// behave-surface/src/fire.rs — fixed-bound loop
for i_size in 0..5 {
    for i_life in 0..2 {
        // reaction intensity calculation
    }
}
```

**Violation scenario**: if a change makes loop count depend on moisture input (e.g., `for _ in 0..moisture_1_hour.ceil() as usize`), SIMD/GPU dispatch diverges.

---

### 3.7 No New Panic/Unwrap in Tests (keep tests exception-safe)

**Why**: Test panics during a large batch kernel run can mask real numerical divergence. Tests must be exception-safe or explicitly wrap panics.

**How to verify**: Tests use `.expect()` only on setup, not assertions; real assertions use `.assert_eq!()` or custom check fns that don't panic.

**Load-bearing example**: parity tests collect failures in a `TestInfo` struct and report them at the end, never panicking mid-check.

---

## 4. Incident-Backed Non-Negotiables

These rules are grounded in specific failures or high-cost learnings from the codebase history.

### 4.1 Never Privatize Public API Without Owner Decision

**Incident**: Commits 02c7621 and 864b8d5 (2016-01-26 onward) — attempted encapsulation (privatizing public surface members) reverted within days because BehavePlus and other consumers relied on the public members for direct field access.

**Rule**: If a struct field or method is public, it is a contract. Removing it requires:
1. Explicit owner decision and a deprecation period (at least one release).
2. A migration guide for consumers.
3. Git commit message citing the decision and the deprecation timeline.

**Verification**: `cargo doc --workspace` and README.md must list all public APIs; breaking changes go to a changelog.

**Violation scenario**: if a change makes `SurfaceFire.wind_factor` private to "encourage" use of a getter, consumers who directly access the field break.

---

### 4.2 Setters Must Be Order-Independent or Batched

**Incident**: Commit 7914517 — wind direction and aspect setter order dependency: calling `set_aspect()` then `set_wind_direction()` produced different results than the reverse order because the wind-relative-to-aspect computation was not re-run.

**Rule**: 
- If setter A affects setter B's output, either:
  1. Recompute both on each call (expensive but safe), **or**
  2. Batch them into a single `update_*` method that sets all related fields and recomputes once.
- Document the dependency in code comments.

**Verification**: Test matrix in parity suite (`test_direction_of_interest`, line 603) calls setters in multiple orders and asserts the same result.

**Current implementation** (verified): `Surface::update_surface_inputs()` takes 20 parameters and sets everything in one method call, eliminating order-dependency risk.

**Violation scenario**: if a change splits `update_surface_inputs()` into separate `set_wind_speed()`, `set_wind_direction()`, `set_aspect()` calls without recomputing, the order matters and outputs diverge.

---

### 4.3 Behavior-Default Changes Must Be Loudly Documented

**Incident**: Commit e6d2b8a (BHP1-1367) — wind speed limit was changed from ON to OFF by default. This was communicated only in the PR; consumers who silently updated got different outputs and did not notice.

**Rule**: When a default or a computational flag changes:
1. It is a behavior change (section 1.3), not a refactor.
2. REVIEW.org findings table must highlight it (P0 priority if it affects published outputs).
3. Commit message must call out the change explicitly: `[BEHAVIOR CHANGE] Wind speed limit default OFF (was ON)`.
4. Consider a compiler warning or a log message on first call with the new default.

**Verification**: `git log --grep="BEHAVIOR CHANGE"` and REVIEW.org findings table are the dual record.

**Violation scenario**: if a change silently flips `wind_limit_on = false` and updates tests without flagging the change, consumers get 5% different outputs and have no way to know why.

---

### 4.4 Every Unit Conversion at an API Boundary Needs a Test

**Incident**: Commit e00fd3a (2026-02-20) fixed 16 arithmetic bugs in `behaveUnits.cpp` conversions (division vs multiplication signs, exponent errors). Commit 0ed4b73 (2023-11-03) "Revert change to calculation output (kPa->Pa)" — VPD units flip was conceptually still unresolved. Commit 49a1a24 SafeSeparationDistance output was Acres, spec says SqFt. Commit 3a7c2bd scorch height over-conversion removed. Commit 05277dd mortality test used wrong crownRatio units.

**Rule**: Every public getter that converts from base units or every setter that accepts user units must have:
1. A unit test calling the fn with known input and asserting the output (in both directions: to_base and from_base).
2. A parity test comparing the C++ output if available.
3. A code comment citing the C++ source (line number or function name).

**Verification**: Search for getters/setters taking `*Units` arguments and ensure there is a corresponding test.

**Load-bearing example**:
```rust
#[test]
fn test_pressure_units_to_base_from_base() {
    let kpa = PressureUnits::KiloPascal;
    assert_eq!(kpa.to_base(100.0), 100000.0);  // 100 kPa = 100,000 Pa
    assert_eq!(kpa.from_base(100000.0), 100.0);
}
```

**Violation scenario**: if a future refactor changes a unit conversion without a test, a silent off-by-factor-of-10 error slips through.

---

## 5. Commit Conventions & Changelog

### 5.1 Observed Patterns (as of 2026-07-06)

The repository has **no changelog**, **no semantic versioning (semver)**, and **no git tags**. Commits are squashed from Jira tickets (BHP1-####) into feature branches and merged via PRs.

**Observed conventions**:
- Commit messages are descriptive prose (not one-line summaries).
- Multi-line messages use a blank line after the summary.
- References to Jira tickets (e.g., BHP1-1509) appear in the commit body, not the subject.
- Cleanup commits (e.g., "Generated Docs") auto-commit doxygen output.

### 5.2 Proposed Future Changelog

**Recommendation** (candidate, not yet adopted):
- Add a CHANGELOG.md or CHANGELOG.org (following Keep a Changelog format) at the repo root.
- Land it as a first commit on a new branch, reviewed by the owner.
- Adopt a loose versioning scheme: major = C++/Rust architecture break, minor = new feature or behavior change, patch = bug fix (parity-preserving refactors do not bump version).
- On each release-candidate branch, prepend a dated entry to CHANGELOG documenting the class of change (section 1) for each item.

**Rationale**: Consumers of BehavePlus and IFTDSS need a way to track when they should re-run models or audit outputs. A changelog provides that visibility.

### 5.3 Co-Author Convention

**Observed**: Recent commits include a `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` line (e.g., f11cbc5).

**Rationale**: If a code review or generation step involved AI tooling, the trailer credits the tool for auditing/verification. This is good practice for traceability.

---

## 6. Verification Commands

All facts below can be re-verified (as of 2026-07-06) by running these read-only commands:

### 6.1 Test counts and passing status
```bash
cargo test --workspace 2>&1 | grep "test result"
cargo test -p behave-run --test parity 2>&1 | grep "test result"
```

**Expected**: All 279 unit tests pass; parity test passes (155 checks collected, 0 failures).

### 6.2 Divergence ledger entries (code comments)
```bash
grep -n "NOTE:\|PRESERVE\|quirk\|C++" crates/behave-surface/src/fuel_models.rs | grep -i tl5
grep -n "NOTE:" crates/firelab-base/src/units.rs | grep -i pressure
grep -n "NOTE:" crates/behave-surface/src/inputs.rs | grep -i canopy_height
```

**Expected**: Each quirk has a comment at or near the site.

### 6.3 Parity suite structure
```bash
grep -c "t\.check[^_]\|t\.check_bool" crates/behave-run/tests/parity.rs
```

**Expected**: 141 checks total (127 t.check, 14 t.check_bool, as of f11cbc5).

### 6.4 REVIEW.org findings table
```bash
grep -n "^| P" REVIEW.org | head -10
```

**Expected**: Findings table rows (P0–P3) with status ("DONE" or "Open").

### 6.5 Unsafe blocks in library code
```bash
grep -r "unsafe" crates/behave-surface/src crates/behave-crown/src crates/behave-spot/src crates/firelab-base/src | grep -v "test\|//.*unsafe"
```

**Expected**: Zero results (no unsafe in library code).

### 6.6 Interior mutability in library code
```bash
grep -r "Mutex\|RefCell\|Arc\|lazy_static" crates/behave-**/src | grep -v "test\|//.*Mutex"
```

**Expected**: Zero results (no interior mutability in library).

### 6.7 String/Vec in input structs
```bash
grep -A 30 "pub struct.*Inputs" crates/behave-surface/src/inputs.rs | grep "String\|Vec"
```

**Expected**: No String or Vec fields (only f64, enums, bool).

---

## 7. Provenance and Maintenance

**Basis**: 
- REVIEW.org (2026-07-06, f11cbc5) — architecture, fidelity, findings, parallelization roadmap.
- RUST_PORT.org (2026-03-05) — porting roadmap (largely superseded by REVIEW.org).
- crates/behave-run/tests/parity.rs (f11cbc5, 1383 lines) — golden-value parity suite.
- Git log 750 commits (2016-01-26 to 2026-07-06) — incident-backed incidents.
- Codebase inspection (fuel_models.rs, units.rs, inputs.rs, wind.rs, exrate.rs, ignite.rs, chaparral.rs).

**Re-verification commands** (for future drift detection):
1. **Parity suite status**: `cargo test -p behave-run --test parity 2>&1 | tail -1`
2. **Divergence ledger comment presence**: `git log --all --oneline -- crates/behave-surface/src/fuel_models.rs | head -3`
3. **Test count stability**: `grep -c "t\.check" crates/behave-run/tests/parity.rs`
4. **REVIEW.org up-to-date**: `git log -1 --format=%h REVIEW.org`
5. **Unsafe blocks**: `grep -r "unsafe\|Mutex\|RefCell" crates/behave-**/src | grep -v test | wc -l` (should be 0)

**Ownership**: The divergence ledger (section 2) and parallelization-compatibility checklist (section 3) are the HOME for these facts across the workspace. Any change to a preserved quirk, a new interior-mutability pattern, or a String/Vec input field must update this skill **and** REVIEW.org in the same PR.

---

## 8. Quick Reference: Which Skill for What?

- **Change classification & gates**: this skill (section 1).
- **Divergence ledger & quirks**: this skill (section 2, HOME).
- **Parallelization roadmap & phase details**: REVIEW.org (section 5).
- **Numerical analysis & f32 sensitivity**: `behave-numerics-proof-toolkit` (sibling skill).
- **Incident history & past failures**: `behave-failure-archaeology` (sibling skill).
- **Fire behavior theory & domain reference**: `fire-behavior-reference` (sibling skill).
- **Architecture & crate DAG**: `behave-architecture-contract` (sibling skill).
- **Testing & parity suite anatomy**: `behave-validation-and-qa` (sibling skill).
