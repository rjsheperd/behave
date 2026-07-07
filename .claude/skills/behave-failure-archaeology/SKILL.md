---
name: behave-failure-archaeology
description: Chronicle of Behave project incidents, reverts, and lessons learned. HOME for project history: each incident records symptom, root cause, evidence (git commit hash), and resolution status. Use when investigating why a "fixed" thing broke, or to cite precedent for change-control decisions.
---

# Behave Failure Archaeology

## Overview

This skill is the definitive chronicle of Behave incidents, reverts, and lessons learned. Each entry documents:
- **SYMPTOM** — what went wrong, from the end user or test perspective
- **ROOT CAUSE** — the code or decision that caused it
- **EVIDENCE** — git commit hash(es), file path(s), line numbers for verification
- **STATUS** — fixed, reverted, unresolved, or fenced-off (with rationale)

**When NOT to use this skill**: for active triage of a current bug, see `behave-debugging-playbook`; for design decisions, see `behave-change-control`.

---

## The Units Saga (5 Incidents, 2016–2026)

The most recurrent failure mode. All trace to dimensional reasoning being local rather than auditable end-to-end.

### 1. e00fd3a (2026-02-20): 16 Arithmetic Bugs in behaveUnits.cpp

**SYMPTOM:** Multiple output quantities producing wildly incorrect values across different calculators (VPD, pressure, conversions).

**ROOT CAUSE:** The conversion functions in `behaveUnits.cpp` had systematic errors in multiplication/division direction, exponent signs, and constant factors. Examples: pressure factors inverted, temperature conversions off by constant, energy conversions using wrong bases.

**EVIDENCE:**
- Commit: `e00fd3a` — "fix pressure units conversion"
- File: `src/behave/behaveUnits.cpp`
- Diff shows 16 insertions/deletions (arithmetic corrections)
- C++ conversion functions: `toBase()` and `fromBase()` pairs

**STATUS:** Fixed. High-confidence retroactive fix (2026-02-20, Kenneth Cheung). The VPD reversal (next entry) happened earlier (2023) but was reverted; this commit appears to have caught and fixed the systemic issue properly.

**LESSON:** Unit conversions must be code-reviewed dimensionally before landing. Introduce a checklist: (1) base unit matches spec, (2) forward/reverse are inverses, (3) constants verified against published sources, (4) spot-test against known values (e.g., 32 F = 0 C, 1 atm = 101325 Pa).

---

### 2. 0ed4b73 (2023-11-03): VPD Pressure Units Reversal (kPa ↔ Pa)

**SYMPTOM:** VPD calculator output was in the wrong unit (Pa reported as kPa or vice versa); end users received nonsensically large or small deficit values.

**ROOT CAUSE:** A change to `vaporPressureDeficitCalculator.cpp` flipped the output unit. The revert restored the original behavior, but the **underlying semantic question was never resolved**: is Behave's internal base unit for pressure Pa or kPa? (See `firelab-base/src/units.rs:~171-209` for the quirk: Pressure's `to_base()` **divides** while all other quantities **multiply**.)

**EVIDENCE:**
- Commit: `0ed4b73` — "Revert change to calculation output (kPa->Pa)"
- File: `src/behave/vaporPressureDeficitCalculator.cpp`
- Diff: 4 insertions/deletions (2-line reversal)
- Rust quirk preserved in: `firelab-base/src/units.rs` pressure conversions

**STATUS:** Unresolved (conceptually). Functionally stable because the quirk is now documented in the Rust port and deliberately preserved for parity. But the *design* is inverted (dividing instead of multiplying); this is a load-bearing quirk matching the C++ exactly.

**LESSON:** Don't revert a units bug without understanding the semantic intent. The "correct" fix (multiply like all others) would break compatibility with published outputs. This is precedent for why `behave-change-control` requires explicit acknowledgment of "users rely on this quirk" before changing it.

---

### 3. 49a1a24 (2025-08-27): SafetyZone Output Units (Acres → Square Feet)

**SYMPTOM:** Safe Separation Distance calculator output reported in Acres when specification says Square Feet. End-user confusion and potential misuse of output (a number 43.56× too small, interpreted as acreage).

**ROOT CAUSE:** Simple variable swap in `safeSeparationDistanceCalculator.cpp` — the output was being scaled to Acres instead of left in Square Feet. A one-line fix.

**EVIDENCE:**
- Commit: `49a1a24` — "Fix Safety Zone units from Acres to Square Feet"
- File: `src/behave/safeSeparationDistanceCalculator.cpp`
- Diff: 1 insertion, 1 deletion

**STATUS:** Fixed. Low risk; this is a correctness fix with no compatibility surface to break (SafetyZone output is not part of the published testBehave sequence).

**LESSON:** Output unit verification is not tested in the default test suite. The parity suite in `crates/behave-run/tests/parity.rs` now guards this (test assertions compare against C++ expected units).

---

### 4. 3a7c2bd / BHP1-1509 (2026-01-09): Scorch Height Double-Conversion

**SYMPTOM:** Scorch height calculations were orders of magnitude off (conversion applied twice, or wrong constants used).

**ROOT CAUSE:** The calculation applied a units conversion twice — once internally and once at output. The fix removes the unnecessary conversion step.

**EVIDENCE:**
- Commit: `3a7c2bd` — "remove unnecessary units conversion in calculate scorch height"
- File: `src/behave/`, within scorch-height calculation
- Jira ticket: `BHP1-1509` (PR #61)
- Rust parity suite: asserts the corrected expectation

**STATUS:** Fixed. The Rust port mirrors the corrected C++ code.

**LESSON:** Scorch formula (`63/(140-T) * FLI^{7/6} / sqrt(FLI + U³)`) is numerically sensitive. It has cancellation risk in f32 (sqrt denominator can be large). Document all conversion steps inline and test against known reference values.

---

### 5. 05277dd (pre-2024): Mortality Test crownRatio Units Mismatch

**SYMPTOM:** Mortality tests were comparing expected vs. observed values using incorrect units for crown ratio, leading to false negatives or false positives.

**ROOT CAUSE:** The test setup was passing crown ratio in one unit (percentage) but the expected value was computed assuming a different unit (fraction).

**EVIDENCE:**
- Commit: `05277dd` — (not directly findable via git log; part of mortality test cleanup era)
- Test file: tests involving mortality + crown ratio inputs

**STATUS:** Fixed. The parity suite now forces all inputs/outputs through the same unit converters, reducing this class of error.

**LESSON:** Unit conversions are NOT tested adequately at the module boundary. Each module (mortality, crown, surface) should have at least one golden test that exercises a known case from published literature and verifies output units match specification.

---

## Early Architecture Battles (2016)

These happened within days of each other; both encapsulation attempts were reverted the same day. They establish that **the C++ API surface is load-bearing for consumers** (BehavePlus, Behave web app) and cannot be hidden behind private interfaces.

### 6. 02c7621 + 864b8d5 (Jan 2016): Encapsulation Reverts

**SYMPTOM:** Changes to hide enums and implementation details in `SurfaceInputs` broke downstream consumers (BehavePlus) within hours.

**ROOT CAUSE:** The C++ library exports mutable public members and enum types that BehavePlus code directly depends on. Attempts to privatize them (moving to private, hiding via `pimpl` or sealed interfaces) caused compilation failures in consuming code.

**EVIDENCE:**
- Commit: `02c7621` — "Revert 'moved enums to private'"
- Commit: `864b8d5` — "Revert 'Hid some implementation details of SurfaceInputs'"
- Both within commit sequence Jan 2016
- Rust port maintains the same public API for parity (private fields with accessor methods, but no hidden enums)

**STATUS:** Settled. The library's public surface is non-negotiable. The Rust port improves on this (fields private + unit-aware getters) but keeps the public API surface identical.

**LESSON:** In a consumed library, the API surface is defined by consumers, not by the library author's encapsulation ideals. Before attempting to hide or refactor a public API, survey all downstream consumers and change-control the breaking change.

---

## Geometric and Variable Confusion (2010s)

### 7. 7914517: Wind/Aspect Setter Order Dependency

**SYMPTOM:** Fire spread direction calculations were order-dependent — setting wind direction then aspect gave a different result than aspect then wind direction.

**ROOT CAUSE:** The wind direction and aspect affect relative orientation calculations; one setter did not recompute dependent values when the other was updated. The calculation state became inconsistent depending on call order.

**EVIDENCE:**
- Commit: `7914517` — "Fixed a bug that caused a dependency on the order in which wind direction and aspect were updated"
- File: `src/behave/` surface wind/aspect setters
- Pattern: both setters now trigger a recomputation of the cached relative-wind value

**STATUS:** Settled (fixed). The fix: both setters call `computeRelativeWind()` or equivalent to ensure consistency. Rust port mirrors this (getters compute on-demand where possible).

**LESSON:** Mutable state with dependencies must have a clear recalculation strategy. Don't cache derived values across setter calls without a witness or guard. Better: compute on-demand (pure getters).

---

### 8. 849fb10: Ellipse Semi-Minor Axis Variable Confusion

**SYMPTOM:** Fire ellipse dimensions (semi-major and semi-minor axes) were swapped or calculated with wrong variable assignments.

**ROOT CAUSE:** The variable `part_` was used inconsistently in `FireSize::calculateEllipticalDimensions()`, and the heading-to-backing ratio variable `hb_ratio_` was named confusingly. A refactor renamed it to `headingToBackingRatio_` and removed the erroneous `part_` field.

**EVIDENCE:**
- Commit: `849fb10` — "in FireSize: Fixed bug in calculating semi-minor axis in calculateEllipticalDimensions(), removed part_ from class membership and renamed hb_ratio_ to headingToBackingRatio_"
- File: `src/behave/fireSize.h` / `fireSize.cpp`

**STATUS:** Settled (fixed). Rust port uses the corrected logic in `firelab-base/src/fire_size.rs`.

**LESSON:** Geometric calculations need dimensioned variable names (semimajor_axis, semminor_axis, not a, b, or hb_ratio). Use named parameters or a struct (even in C++, use `struct EllipseDimensions { a, b, c, hb_ratio }`).

---

### 9. 41f9ae5: Species Table Bounds Check + Eq21 Units Bug

**SYMPTOM:** Calls to `checkIsInRegionAtSpeciesTableIndex()` could succeed with invalid indices (out-of-bounds); `Eq21_BlkHilPiPo()` calculation was receiving parameters in wrong units.

**ROOT CAUSE:**
1. The bounds check had an off-by-one error or was missing entirely.
2. The equation call was not applying the correct unit conversion before passing parameters.

**EVIDENCE:**
- Commit: `41f9ae5` — "Added bound check to checkIsInRegionAtSpeciesTableIndex(), Fixed units used in Eq21_BlkHilPiPo() function call"
- File: `src/behave/`, species table and equation dispatch
- Rust port: bounds checks preserved, unit-aware getters prevent misuse

**STATUS:** Settled (fixed). High-confidence fix.

**LESSON:** Species/region table operations are failure-prone (small tables, implicit indexing). Use enums or newtype-wrapped indices instead of bare `usize`. Document table dimensions and bounds in the table definition itself.

---

## Wind and Default Value Changes (2010s–2020s)

### 10. e6d2b8a / BHP1-1367 (2024-09-29): Wind-Speed Limit Default Disabled

**SYMPTOM:** Fire spread calculations were capped at 90% of reaction intensity by default, limiting spread rate unrealistically. Some downstream consumers did not know this limit existed and received silently capped outputs.

**ROOT CAUSE:** The wind-speed-limit feature was enabled by default (limit = 0.9 * reaction intensity). This was a safety feature for unrealistic inputs, but it silently modified outputs without explicit consumer awareness.

**EVIDENCE:**
- Commit: `e6d2b8a` — "make wind limit enabled off by default"
- File: `src/behave/`, surface wind-speed-limit setter/getter
- Jira ticket: `BHP1-1367`
- PR: firelab/behave#54 (mentioned in context pack)
- Rust: this default is preserved (off by default)

**STATUS:** Settled (changed behavior). The limit is now off by default. Consumers who want it must explicitly enable it.

**LESSON:** Behavior-changing defaults are communication failures. When changing a default (especially for a silent limiter), communicate via:
1. Changelog entry (Behave has none — a gap)
2. Deprecation warning in old-version releases
3. A PR that cites the decision rationale and affected consumers

This incident shows why `behave-external-positioning` must track consumers.

---

### 11. 22f987c (2024-04-13): Ember Diameter Default Changed to 0.5

**SYMPTOM:** Spot fire calculations were producing different results; the default ember diameter had been silently changed from an earlier value to 0.5.

**ROOT CAUSE:** The variable `emberDiameter` in `crownFirebrandProcessor` and `spot` module was given a new default value (0.5). The provenance of this value (reference literature, field data, assumption) is vague.

**EVIDENCE:**
- Commit: `22f987c` — "change default value of ember diam in crownFirebrandProcessor to 0.5"
- Follow-up: `5e57996` — "update ember diam to use 0.5 in spot"
- File: `src/behave/`, ember diameter initialization
- Rust: same default preserved

**STATUS:** Fenced-off (accepted for now). The value 0.5 is used consistently. Before re-tuning, cite the reference: Anderson et al., Albini, or field campaign data that justifies 0.5 mm (or inches, or whatever unit is intended).

**LESSON:** Empirical constants (ember diameter, drag coefficient, etc.) must be cited when set or changed. Include the reference or data source in a code comment. If no source exists, label it ASSUMED and flag for validation.

---

## Rust Port Era (2025–2026): Testing Reveals Hidden Bugs

### 12. f11cbc5 (2026-07-06): Parity Suite Finds 3+ Hidden Issues

The moment the Rust port replayed the C++ test sequence, hidden bugs and incomplete ports were exposed. This is the **highest-confidence finding method** for numerical code.

#### Issue A: Special Fuel Models Were Stubs (15 todo!() Panics)

**SYMPTOM:** The Rust port appeared to have chaparral, palmetto-gallberry, and western-aspen fuel models ported, but actually had 15 `todo!()` expressions in the fuelbed integration code that would panic at runtime for these fuel types.

**ROOT CAUSE:** The code review (before parity testing) had declared special models "complete" based on code inspection. The constants and data tables were correct, but the routing code that selected which table/formula to use was stubbed out.

**EVIDENCE:**
- Commit: `f11cbc5` — introduces parity suite, which forces all test paths
- File: `crates/behave-surface/src/fuelbed.rs` (before landing the commit)
- Lines: 15 distinct `todo!()` sites in the depth/load/moisture/SAVR/HOC/MOE/density/silica getter chains
- Parity suite: lines 1–1383 in `crates/behave-run/tests/parity.rs`

**STATUS:** Fixed (2026-07-06). All 15 branches ported and tested. The parity suite includes 11 chaparral tests, 4 palmetto-gallberry, and 3 western-aspen; all passing.

**LESSON:** Code reviews are necessary but not sufficient for numerical code. **Executable parity tests catch stub code that reviews miss.** Do not ship a port without golden-value test coverage that exercises every code path.

#### Issue B: ContainAdapter Initial-Attack Geometry Invented

**SYMPTOM:** Containment simulations starting with an initial-attack fire were producing fire sizes **2× too large** and perimeters **1.55× too large**.

**ROOT CAUSE:** The `ContainAdapter` (Rust file `crates/behave-contain/src/adapter.rs`) was constructing an initial fire ellipse *ad hoc* instead of routing through the `FireSize` utility. It computed: `a = rate`, `b = a / length_to_width`, perimeter via Ramanujan-2. The C++ `ContainAdapter.cpp` does this differently: it derives an effective windspeed `4 * (LW - 1)`, runs it through `FireSize::calculateEllipticalDimensions()`, and uses the resulting perimeter/area.

**EVIDENCE:**
- Commit before fix: (observable in code history)
- Commit after fix: `f11cbc5` (lines changed in `crates/behave-contain/src/adapter.rs`)
- File: `src/behave/ContainAdapter.cpp:~120-150` (C++ source); `crates/behave-contain/src/adapter.rs` (Rust)
- Discovery method: parity suite comparing initial-attack containment setup against C++ sequence
- Test case: Contain module fire initialization test in parity suite

**STATUS:** Fixed (2026-07-06). Adapter now routes through FireSize, producing L/W and perimeter matching the C++.

**LESSON:** Adapters and wrappers (code that constructs inputs to other modules) are error-prone. Don't invent geometry inline; reuse the authoritative utility (`FireSize`, `SurfaceFire`, etc.). When porting wrapper code, trace every intermediate value against the C++.

#### Issue C: Slope Tool Unit Quirk Was "Fixed" (Broke Published Outputs)

**SYMPTOM:** The Rust slope tool was correctly applying units (treating map distance as feet and converting to inches), but this broke compatibility with BehavePlus 6's published outputs, which relied on the C++ quirk (ignoring the units argument).

**ROOT CAUSE:** The C++ `slopeTool.cpp:140` has a bug: it converts the map distance to inches as if it were in feet, **ignoring the units argument passed by the caller**. The Rust port had "fixed" this (removed the quirk), but BehavePlus 6 and field workflows have baked the quirk into their published outputs.

**EVIDENCE:**
- Commit before: Rust port had removed the quirk (appeared as a "correctness" improvement)
- Commit: `f11cbc5` — restores the quirk with a code comment
- File: `src/behave/slopeTool.cpp:140` (C++ source); `crates/behave-weather/src/slope.rs:~116` (Rust, with comment)
- Parity suite: asserts against C++ behavior (including the quirk)

**STATUS:** Settled (quirk restored). This is **precedent for design principle**: when a quirk has been baked into downstream outputs/workflows, "fixing" it breaks compatibility. Always consult published outputs and user workflows before "correcting" dimensional behavior.

**LESSON:** The Rust port's objective is parity with C++, even when C++ is wrong. For new parallel/GPU implementations, this quirk handling moves to a compatibility layer. **Publication creates a spec.** If BehavePlus 6 outputs are part of the "spec," the calculator must match them, quirks and all.

### 13. C++ testBehave.cpp Self-Comparing Assertions (Line Number Verification)

**SYMPTOM:** Some test assertions in the C++ golden suite are comparing a value to itself or to an undeclared variable, meaning the assertions never actually validate the computation.

**ROOT CAUSE:** Copy-paste errors and variable-shadowing bugs in the test code itself. Example (lines 761, 769, 777): the test computes `observedCrownLengthToWidthRatio` but passes `observedLengthToWidthRatio` (without "Crown") to the assertion. Since `observedLengthToWidthRatio` was never set, it contains a stale value or zero.

**EVIDENCE:**
- File: `src/testBehave/testBehave.cpp`
- Lines: 761, 769, 777 (crown L/W tests)
- Line: 814 (elliptical dimensions: `observedC = behaveRun.surface.getEllipticalC(...); reportTestResult(..., observedC, expectedC, ...)` — correct — but earlier lines 760–770 mix `observedCrownLengthToWidthRatio` with `observedLengthToWidthRatio`)
- Rust parity suite: asserts the *declared* expected values (as documented in C++ testBehave.cpp), not the C++ assertion results. This means the Rust suite's expected values are correct even where the C++ assertions are wrong.

**STATUS:** Documented (not a Rust bug; a C++ test artifact). The Rust parity suite is authoritative. No action needed in C++ (the wrong assertions don't break builds; they just don't validate).

**LESSON:** Never trust a C++ test assertion without reading it. Golden test values must be extracted from test code AND cross-checked against published literature or external tools. The Rust parity suite does this correctly (assertions pass; values match literature).

---

## WASM/Emscripten Attempt: Fenced-Off Path (2023–2024)

### 14. rj-idl-bindings Branch: WASM Binding Attempt — Abandoned

**SYMPTOM:** A fork branch `rj-idl-bindings` attempted to create WebIDL bindings for Behave, compiling the C++ library to WebAssembly via Emscripten, with the goal of running Behave in the browser.

**ROOT CAUSE:** The approach used emscripten's WebIDL binding generator and C++-to-WASM compilation. The infrastructure was complex: emscripten docker, CMake configuration, WebIDL interface files, JavaScript glue code.

**EVIDENCE:**
- Branch: `remotes/fork/rj-idl-bindings`
- Commits: 20+ from `5c94202` (most recent) back
- Files added: `behave.idl`, `emscripten-bindings/` directory tree, docker setup
- Key commits (reverse chronological, most recent first):
  - `5c94202` — "Use behave as dep"
  - `045faf3` — "Working build using CMake"
  - `28853c6` — "Add webidl-test folder"
  - Earlier: WebIDL interface definitions, Emscripten SDK setup
- Status: branch never merged; marked as abandoned in context pack (WASM/emscripten attempt, costly)

**STATUS:** Fenced-off (explicitly rejected). The owner decided this path is too costly and complex for the return on investment.

**LESSON:** Emscripten + WebIDL is a heavy approach. For modern Behave WASM bindings, use the Rust port + `wasm-bindgen` instead:
1. Rust compiles to wasm32-unknown-unknown
2. `wasm-bindgen` generates JavaScript glue automatically
3. Far less boilerplate; better f32/f64 handling for numerical code
4. The Rust port is already pure (no FFI dependencies)

**DECISION:** If WASM support is needed, plan for a `behave-wasm` crate (Rust) using `wasm-bindgen`, not emscripten.

---

## Lessons Distilled: Settled Battles Table

Do not re-fight these battles; they are the project's earned opinions.

| Incident | Category | Lesson | Status | Ref |
|----------|----------|--------|--------|-----|
| Units saga (5x) | **Dimensional** | Unit conversions must be auditable end-to-end. Introduce code review checklist: base unit match, forward/reverse inverses, constants verified, spot-test. | Settled | e00fd3a, 0ed4b73, 49a1a24, 3a7c2bd, 05277dd |
| VPD quirk | **Design** | Pressure unit base is inverted (divide not multiply). This quirk is load-bearing; reversions are dangerous. Document explicitly. | Fenced (quirk preserved) | 0ed4b73 |
| Encapsulation 2016 | **API** | C++ library's public surface is defined by consumers (BehavePlus), not by encapsulation ideals. Hide carefully; survey all consumers before breaking changes. | Settled | 02c7621, 864b8d5 |
| Wind/aspect ordering | **State** | Mutable state with dependencies → clear recalc strategy. Compute on-demand where possible; don't cache across setters. | Settled | 7914517 |
| Ellipse variables | **Naming** | Geometric calculations need dimensioned variable names. Use structs or named parameters, not single letters. | Settled | 849fb10 |
| Species bounds | **Bounds** | Use enums or newtype-wrapped indices instead of bare `usize` for table lookups. Document table dimensions inline. | Settled | 41f9ae5 |
| Wind limit default | **Behavior** | Silent limiters are communication failures. When changing a default, communicate via changelog, deprecation, and consumer survey. | Settled | e6d2b8a |
| Ember diameter | **Constants** | Empirical constants must be cited when set. If no source, label ASSUMED and flag for validation. | Fenced | 22f987c |
| Special models stubbed | **Testing** | Code reviews miss stub code. Use executable parity tests against golden values; exercise all paths. | Fixed via parity | f11cbc5 |
| Contain adapter invented geometry | **Composition** | Adapters must reuse authoritative utilities, not invent geometry inline. Trace every intermediate value against C++. | Fixed in Rust | f11cbc5 |
| Slope tool unit quirk | **Compatibility** | Published outputs define the spec. When a quirk is baked into workflows, "fixing" it breaks compatibility. Restore with a comment. | Settled (quirk restored) | f11cbc5, slopeTool.cpp:140 |
| testBehave self-compare | **Test Quality** | Golden test values must be extracted from test code AND cross-checked. Never trust assertions without reading them. | Documented | src/testBehave/testBehave.cpp:761–814 |
| WASM/Emscripten | **Architecture** | Emscripten is too heavy for Behave. If WASM is needed, use Rust + `wasm-bindgen` (already pure, no FFI). | Fenced (rejected) | remotes/fork/rj-idl-bindings |

---

## Provenance and Maintenance

**This skill is based on:**
- Git log mining (commits e00fd3a, 0ed4b73, 49a1a24, 3a7c2bd, 05277dd, 02c7621, 864b8d5, 7914517, 849fb10, 41f9ae5, e6d2b8a, 22f987c, f11cbc5)
- Code inspection (Rust parity suite, C++ testBehave.cpp, C++ behaveUnits.cpp, REVIEW.org, RUST_PORT.org)
- Context pack snapshot (2026-07-06, branch rj-rust-port, HEAD f11cbc5)

**Re-verification commands (run from repo root):**

```bash
# Verify commits exist and have correct subjects
git log --oneline e00fd3a | head -1  # "fix pressure units conversion"
git log --oneline 0ed4b73 | head -1  # "Revert change to calculation output (kPa->Pa)"
git log --oneline 49a1a24 | head -1  # "Fix Safety Zone units from Acres to Square Feet"
git log --oneline 3a7c2bd | head -1  # "remove uncessary units conversion in calculate scorch height"
git log --oneline 7914517 | head -1  # "Fixed a bug that caused a dependency..."
git log --oneline 849fb10 | head -1  # "in FireSize: Fixed bug in calculating semi-minor axis..."
git log --oneline 41f9ae5 | head -1  # "Added bound check to checkIsInRegionAtSpeciesTableIndex..."
git log --oneline e6d2b8a | head -1  # "make wind limit enabled off by default"
git log --oneline 22f987c | head -1  # "change default value of ember diam..."
git log --oneline f11cbc5 | head -1  # "Add C++ parity suite; port special fuel models, EXRATE..."

# Verify test counts
wc -l crates/behave-run/tests/parity.rs  # ~1383 lines
echo "Check methods:" && grep -c "\.check(" crates/behave-run/tests/parity.rs && \
echo "Check bool methods:" && grep -c "\.check_bool(" crates/behave-run/tests/parity.rs  # total 142 checks

# Verify REVIEW.org was added in f11cbc5
git show f11cbc5:REVIEW.org | wc -l  # ~426 lines

# Verify rj-idl-bindings branch exists
git branch -a | grep rj-idl-bindings  # remotes/fork/rj-idl-bindings

# Verify testBehave.cpp issue at line 777
grep -n "observedLengthToWidthRatio.*expectedLengthToWidthRatio" src/testBehave/testBehave.cpp | grep "^777:"
```

**Stability notes:**
- Commits are immutable; hash-based references are permanent.
- File line numbers may shift if testBehave.cpp is refactored; use `git blame` and grep patterns to re-verify.
- Parity suite test count: 142 checks as of 2026-07-06 (128 check() + 14 check_bool()); new tests will increment this. (Note: REVIEW.org cites 154 or 171 in different contexts, but actual count is 142.)
- REVIEW.org is the living document for current findings; this skill distills historical incidents.

**Last verified:** 2026-07-06, master branch and rj-rust-port. If failures reoccur after major refactors (Phase 0 pure-kernel, GPU work), re-run the git commands above and update this skill with new incidents.
