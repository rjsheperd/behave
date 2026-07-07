---
name: behave-research-methodology
description: The discipline for turning a hunch into an accepted result in Behave—evidence bar, hypothesis prediction, the lifecycle from hypothesis to adoption, historical source patterns, and anti-patterns. Use when designing experiments, validating changes, proposing new features, or investigating whether a discovery warrants action. HOME for research methodology, idea validation, and the institutional pattern that keeps the codebase honest. NOT for active campaign-level parallelization work (see behave-parallelization-campaign) or incident postmortems (see behave-failure-archaeology).
---

# Behave Research Methodology

**Status**: Canonical reference (as of 2026-07-06, HEAD f11cbc5).

**Scope**: The discipline and evidence bar for turning hunches into accepted results in Behave. Covers the evidence bar, hypothesis structure and prediction, the idea lifecycle, historical source patterns for good ideas, and anti-patterns that historically led to failures. This skill is the institutional memory of **how we know what we know**.

**When NOT to use this skill**: for active parallelization campaign execution, use `behave-parallelization-campaign`; for incident postmortems and historical patterns, use `behave-failure-archaeology`; for validation gates and parity suite mechanics, use `behave-validation-and-qa`; for specific numerical debugging traps, use `behave-debugging-playbook`.

---

## 1. The Evidence Bar: One Mechanism Must Explain ALL Observations

The foundational principle: **accept no hypothesis that leaves any observation unexplained**.

### 1.1 Pattern: The Contain Adapter Diagnosis (2026-07-06)

A worked example from this repo that demonstrates the bar in action.

**Observations** (from parity suite failure before fix):
1. Fire size at initial attack was 2.0× too large
2. Fire perimeter was 1.55× too large  
3. Final-containment calculations passed (matched C++ golden values)

**Candidate Hypotheses**:
- **H1**: Wrong elapsed time (would scale all dimensions uniformly; rejected because perimeter and area scale differ, and parity would fail consistently across different inputs)
- **H2**: Wrong length-to-width ratio (would change perimeter/area ratio; rejected because the ratio doesn't match the observed 2.0× area × 1.55× perimeter disproportionality)
- **H3**: Adapter's ellipse geometry invented ad hoc (a = rate, b = a/LW, Ramanujan-2 perimeter formula), diverged from C++'s FireSize path (a = rate, b = rate/LW where LW comes from effective windspeed 4·(LW−1) run through FireSize's Anderson formula) — **ACCEPTED**

**Why H3 alone survived**: it predicts BOTH the 2.0× area error AND the 1.55× perimeter error simultaneously; explains why final-containment (which uses time-integrated behavior, not initial ellipse geometry alone) still passes. Different L/W from different inputs → C++ gets LW_c via Anderson(effective_wind), adapter gets LW_a via input wind directly → area scales with both rate and LW, perimeter scales with their geometric mean → compounds differently.

**Evidence**: 
- Code: `crates/behave-contain/src/adapter.rs` (Rust divergence vs C++ `contain.cpp`)
- Parity test: `test_contain_module()` in `crates/behave-run/tests/parity.rs` (now passing after fix; 6 checks)
- Fix commit: PR #[unknown] (verify in git log when skill is updated)
- Lesson: A favorable code review missed this (reviews are hypotheses; tests are experiments)

### 1.2 The Bar in Practice

When you observe divergence from expected behavior:

1. **List all observations** (including negatives: things that DID match, things that diverged by how much)
2. **Generate candidate mechanisms** that could explain each observation independently
3. **Find the ONE mechanism that explains ALL observations simultaneously** (Occam's razor, not "most observations")
4. **Predict new observations** from that mechanism; design experiments to test them
5. **Only accept the hypothesis if new predictions match** or if no experiment can falsify it

**Anti-pattern**: "This looks right to me" or "spot-checking a few values" — these are not experiments. See Anti-patterns section 5.1.

### 1.3 Institutional Rule: Independent Adversarial Review

Every significant finding is subject to assigned adversarial refutation:
- Code review task: "Review this diff for correctness bugs; your job is to REFUTE the author's claim, not rubber-stamp it"
- Parity suite task: "These 171 checks are the ground truth; if any diverges, explain why or reject the diff"
- Publication standard (beyond this repo): peer review with explicit adversarial framing ("try to find a counterexample")

The lesson from the Rust port: a thorough human code review claimed 15 modules were "complete" but asserted no golden-value tests. The parity suite landing revealed 15 `todo!()` panics in those exact modules. The review was a hypothesis; the test was an experiment.

---

## 2. Hypothesis-Predicts-Numbers-Before-Running

You do not understand a change until you can predict its output before observing it.

### 2.1 Pattern: Crown Fire L/W Prediction (Scott & Reinhardt 1991)

**Hypothesis**: Crown fire length-to-width ratio follows L/W = 1 + 0.125 × U_20mph, where U_20mph is wind speed at 20 feet in miles per hour.

**Setup** (from parity suite): set wind speed to 5 mph at 20 feet.

**Prediction** (calculated BEFORE running code):
- L/W = 1 + 0.125 × 5 = 1 + 0.625 = 1.625

**Observed** (parity.rs lines 530–534):
```rust
t.check(
    "crown L/W: 5mph (scenario default)",
    round6(run.crown.get_crown_fire_length_to_width_ratio()),
    1.625, TOL,
);
```
Result: **PASS** (observed = 1.625 within tolerance).

**Why this matters**: if you predict 1.625 and observe 1.625, you have validated your understanding of:
- The equation coefficients (1.0 and 0.125 exactly)
- The input-to-calculation path (wind speed flows through correctly)
- The output getter (returning the right field, not a cached stale value)
- The rounding/precision behavior (f64 arithmetic in Rust matches C++ to 1e-6)

### 2.2 Pattern: EXRATE Coverage Endpoints

**Hypothesis**: EXRATE deterministic fire coverage blend (Finney algorithm) reaches exactly two ROS values at the endpoints: 0% coverage → ROS of the second fuel model alone, 100% coverage → ROS of the first fuel model alone.

**Setup** (two-fuel model scenario, parity.rs lines 73–86):
- Fuel model 1 (primary)
- Fuel model 124 (GS4, grass-shrub)  
- Coverage blend at 0% and 100%

**Predictions** (from EXRATE lookup tables, species model endpoints):
- At 0% coverage (second model dominates): ROS = 8.876216 ch/hr
- At 100% coverage (first model dominates): ROS = 21.971217 ch/hr

**Observed** (parity.rs lines 702–712):
```rust
fn test_two_fuel_models(run: &mut BehaveRun, t: &mut TestInfo) {
    // ... setup ...
    t.check("two-fuel 0% coverage", run.surface.get_spread_rate_chaining_per_hour(), 8.876216, TOL);
    // ...
    t.check("two-fuel 100% coverage", run.surface.get_spread_rate_chaining_per_hour(), 21.971217, TOL);
}
```
Result: **PASS** (both endpoints match).

**Why this matters**: the EXRATE algorithm deterministically interpolates between two ROS values; if you predict the endpoints, you've validated:
- The lookup tables are correct (they yield exactly these ROS values for these fuel/moisture combos)
- The blend formula is correct (it's monotonic, reaches the exact endpoints, not approximations)
- The integration is correct (coverage parameter flows through to the right table lookup)

### 2.3 Prediction Discipline

Before you run ANY experiment:

1. **State your hypothesis** in plain language (not code, not math symbols alone)
2. **Derive or calculate the predicted value** using independent means (published equation, reference implementation, hand calculation with a calculator)
3. **Write down the tolerance** you expect (1e-6 for parity, more lenient for new experiments)
4. **Document the prediction** in a branch, an experiment flag, or a test comment (so you cannot later claim you "always expected the observed value")
5. **Run the experiment** without looking at the output
6. **Compare**: if prediction matches, you've learned something; if it diverges, you've learned something else (but you MUST investigate why)

**Anti-pattern**: "I'll run the code and see what makes sense" — this is confirmation bias, not science. You will unconsciously accept the first answer and rationalize it.

---

## 3. The Idea Lifecycle: From Hunch to Adopted Result

Every change to Behave (beyond bug fixes on the critical path) passes through this lifecycle.

### 3.1 Stage 1: Hunch

You suspect something. Examples:
- "The wind-factor exponent might be wrong"
- "SIMD parallelization could be 10× faster"
- "The scorch formula cancels numerically in f32"

**Action**: Write it down. No action yet; it's an observation.

### 3.2 Stage 2: Written Hypothesis with Predicted Numbers

Formalize the hunch into a testable statement. Examples:
- "The wind-factor exponent is 0.1490 in the C++ code but should be 0.15 per Rothermel's original paper; if true, the ROS for FM 124 at 5 mph should be 19.677 ch/hr (vs observed C++ value 19.673 ch/hr)"
- "A rayon batch-parallelization of 1000 surface-fire runs should complete in <100ms on an 8-core CPU (sequential: 5000ms)"
- "f32 scorch heights differ from f64 by >5% when FLI + U³ < 1e-6 (cancellation-prone numerator denominator)"

**Action**: Document in a branch, experiment ticket, or REVIEW.org section. State the falsifiable prediction explicitly.

### 3.3 Stage 3: Experiment Behind a Branch/Flag (Never on the Parity Path)

**Critical rule**: No change lands on `rj-rust-port` (the parity path) until it has passed gate 3.5.

- Create a feature branch (`rj-investigate-wind-factor-exponent`, `rj-f32-scorch-study`, etc.)
- Implement the hypothesis
- Add comprehensive unit tests for the new path (do not edit existing golden values)
- Run the parity suite — it must still pass (zero edits to golden expectations)
- Document all findings (good and bad) in the branch's commit messages or a scratch document

Example workflow:
```bash
git checkout -b rj-investigate-scorch-f32
# Edit scorch formula to use f32 arithmetic
# Add unit tests with random inputs spanning the cancellation-prone region
cargo test -p behave-mortality --lib
# Write up findings: "At FLI=0.5, U=0.1, f32 diverges by 6.2% from f64; outside this narrow region, divergence <0.1%"
# Document in commit message and push to origin (branch only, not main)
```

### 3.4 Stage 4: Parity + Unit Gates

Before landing on `rj-rust-port`:

1. **Parity suite must pass 100%** (`cargo test -p behave-run --test parity` returns zero failures)
   - If the change is a fix (not a new feature), update the golden expected values in the parity suite AND cite the owner decision in REVIEW.org
   - If the change is a new feature, do NOT edit existing parity checks (only add new unit tests)

2. **No panics/unwraps in library code** (panics must be confined to test code)

3. **No interior mutability, no global state** — these break the parallelization path (see behave-parallelization-campaign)

4. **Parallelization-compatibility checklist** (see behave-change-control section 3):
   - Copy-semantics inputs (no heap fields, no String, no Arc)
   - Static tables only (no mutable caches)
   - No branch divergence beyond fuel-model kind (which is handled by facade dispatch)

### 3.5 Stage 5: Change Classification & REVIEW.org Entry

Classify the change (see `behave-change-control` section 1 for full taxonomy):
- **Parity-preserving refactor** — gates: parity suite + unit tests pass, zero edits to golden values
- **Quirk fix** — gates: owner decision + updated parity assertion + REVIEW.org divergence entry
- **Behavior change (formula/equation fix)** — gates: citation of C++ source or reference + re-derived golden values + parity updated + REVIEW.org findings entry
- **New feature** — gates: no parity edits + comprehensive unit tests + parallelization checklist + REVIEW.org findings entry
- **C++-side change** — gates: track divergence in REVIEW.org divergence ledger

**Action**: Update REVIEW.org findings table or divergence ledger (depending on class). Each row:
- What changed, where, why
- Owner decision cite (if applicable)
- Test coverage (parity line range or unit test file)
- Priority (P0–P3, if a finding; or N/A, if a refactor)

### 3.6 Stage 6: Adopt (Landing Commit) or Retire (Close Branch)

**To adopt**: Create a PR into `rj-rust-port` with all gates passed. Commit message must reference the change class, REVIEW.org entry, and any owner decision cite.

**To retire**: If the experiment proves the hypothesis false, close the branch and document the learning in `behave-failure-archaeology` (see section 5.4).

---

## 4. Where Good Ideas Historically Came From (Verifiable Instances)

### 4.1 Executable Parity Suite Exposing Review-Invisible Gaps

**Instance**: The chaparral/palmetto-gallberry/western-aspen special fuel models integration (2026-07-06).

- **Observation**: Code review declared 15 modules "complete"; the Rust port fidelity review (agent-based, line-matched code inspection) found no defects.
- **Experiment**: Land the parity suite (`crates/behave-run/tests/parity.rs`, 171 checks).
- **Discovery**: The suite immediately failed on the first call to special-model integration — 15 `todo!()` panics in `behave-surface/src/fuelbed.rs` revealed the modules were stubs, not complete.
- **Action**: Code review task marked "fix integration paths"; human review would never have caught this (it requires running the full sequence).
- **Result**: Integrated all 15 special models; parity suite now passes 171/171 checks.

**Lesson**: Executable tests are hypotheses about behavior; human reviews are hypotheses about code. Only execution answers whether the hypotheses are compatible.

---

### 4.2 Scoping Before Porting (EXRATE Newext Discovery)

**Instance**: EXRATE lateral-extension machinery (firelab-behave, 2024–2026).

- **Observation**: EXRATE.cpp (`src/behave/randfuel.cpp`, `randthread.cpp`, `newext.cpp`) totals ~1650 lines.
- **Hypothesis**: All 1650 lines are reachable and must be ported to Rust for completeness.
- **Experiment**: Read the C++ caller tree (how EXRATE is invoked from the test suite, Behave wrapper, BehavePlus GUI).
  - Call sites: `testBehave.cpp` (spot checks, no extension calls), `spotBehaviorCalculator.cpp` (documented to use no-extension path only)
  - Configuration: CMake option `COMPUTE_SPOT_DISTRIBUTION` (default OFF; requires explicit enable; never used in CI)
- **Discovery**: Of ~1650 lines, only the "no lateral extension" path is reachable under default behavior. The lateral extension (newext) machinery — 416 dead lines — is unreachable unless you explicitly compile and call a CLI tool that is not shipped.
- **Action**: Document the newext path as unreachable (code comment in `crates/behave-surface/src/exrate.rs`); do not port it.
- **Result**: Saved ~416 lines of porting effort; zero test coverage lost; EXRATE parity suite passes (171 checks, zero ignored).

**Lesson**: Before porting a foreign library, map its reachable parameter space. Published line count and "we might need this someday" are not sufficient justification.

---

### 4.3 Reading the C++ Caller to Determine Reachable Parameter Space

**Instance**: EXRATE samples and depth parameters (spot fire generation, 2026).

- **Observation**: EXRATE has configurable parameters: `samples` (number of random trials) and `depth` (iteration depth in the factorial grid).
- **Hypothesis**: The parameters can take any value in a wide range.
- **Experiment**: Read the call sites:
  - `spotBehaviorCalculator.cpp:getTorchingTreesFirebrandCriteria()` → calls EXRATE with hardcoded `samples=2, depth=2`
  - No other call sites in the shipped C++ code
- **Discovery**: Only (samples=2, depth=2) is reachable; the algorithm supports larger grids (samples^depths can be huge), but the caller never uses them.
- **Action**: Unit tests for (2,2) only; document the parameter constraint in the Rust struct comment.
- **Result**: Avoided over-generalizing; parity suite confirms (2,2) behavior; no wasted test coverage.

**Lesson**: Reachable parameter space is determined by callers, not by API design. Read the callers.

---

### 4.4 Treating Published BehavePlus Outputs as Spec (Over Dimensional Correctness)

**Instance**: Slope tool map-distance-to-inches conversion (2026-07-06).

- **Observation**: `slopeTool.cpp:140` converts a map distance (input unit is parameter) to inches by:
  - If the input unit is feet, it treats the value as-is (correct)
  - If the input unit is meters or miles, it **still treats the value as if it were feet**, then converts to inches
- **Hypothesis**: This is a bug (units parameter is ignored; all inputs are treated as feet).
- **Experiment**: Check BehavePlus 6 published outputs (from the desktop app, publicly available) for slope calculations.
- **Discovery**: BehavePlus 6's published outputs **incorporate the bug**. If you fix the Rust port to respect the units argument, outputs diverge from BehavePlus and consumers see "broken" values.
- **Action**: Restore the bug in Rust; document it with a code comment pinning it to slopeTool.cpp:140.
- **Result**: Parity with BehavePlus maintained; dimensional correctness sacrificed for consumer compatibility (owner decision, documented in REVIEW.org).

**Lesson**: Published outputs (especially from a shipping product like BehavePlus 6) are sometimes the de facto spec, even when dimensionally wrong. Change only with owner approval.

---

## 5. Anti-Patterns Ledger

### 5.1 Trusting Spot-Check Reviews for Completeness

**Anti-pattern**: "I reviewed the code; I checked a few values; it looks complete."

**Why it fails**: Human eyes cannot execute code; reviewers can miss panics (like the 15 `todo!()` stubs), state dependencies (like a getter depending on a calculation not yet run), or type mismatches (like expecting a reference but getting a copy).

**Institutional memory**: The Rust port code review declared 15 special-fuel modules "complete"; the parity suite landing revealed all 15 were stubs. A code review is a hypothesis; only execution falsifies it.

**Mitigation**: Reserve judgment until the parity suite runs. Treat code reviews as "looks plausible" not "is correct."

---

### 5.2 "Fixing" Quirks Encountered Mid-Task

**Anti-pattern**: While implementing a feature, you notice a C++ quirk (like the pressure units divide-instead-of-multiply). You "fix" it in Rust without owner approval, thinking you're improving the code.

**Why it fails**: Consumers (BehavePlus, Behave web app, IFTDSS) may have built their own output around the quirk. Fixing it breaks their downstream calculations silently.

**Institutional memory**: The C++ pressure-units reversal (0ed4b73, 2023) was reverted within weeks. The slope-tool units bug (slopeTool.cpp:140) was "fixed" in an early Rust draft, then restored after parity analysis revealed BehavePlus 6 relies on it.

**Mitigation**: Document all preserved quirks in code comments with cites to C++ source (see `behave-change-control` divergence ledger). If you want to fix one, file an issue and await owner decision.

---

### 5.3 Porting a Foreign Library Wholesale Before Mapping Reachable Subset

**Anti-pattern**: You find a reference implementation (e.g., EXRATE lateral-extension machinery, 416 lines) and port all of it to Rust because "it's part of the library."

**Why it fails**: Dead code adds complexity, test burden, and maintenance cost. The 416 unreachable EXRATE lines would have required porting ~100 LOC of infrastructure (random number generation, factorial grid machinery) that the Behave codebase never calls.

**Institutional memory**: EXRATE newext discovery (section 4.2) saved this effort. Reading the callers first revealed the unreachable path.

**Mitigation**: Before porting, map the C++ call tree. Use static analysis (grep for function names, read test harnesses) to determine what's actually reachable. Document unreachable paths in code comments.

---

### 5.4 Changing Two Variables Per Experiment

**Anti-pattern**: You hypothesize "wind speed AND slope cause fire spread rate to diverge from C++." You change both simultaneously in your experiment.

**Why it fails**: If the experiment diverges, you cannot isolate which variable caused it. Confounded experiments are useless.

**Institutional memory**: Not yet in this repo (the parity suite limits single-variable changes). But this is a textbook statistics failure.

**Mitigation**: Design experiments with one independent variable. Hold all else constant. Separate experiments for wind, slope, and fuel-moisture if you want to test them.

---

### 5.5 Relying on "It Compiles" as Evidence of Correctness

**Anti-pattern**: The Rust code compiles (no errors); Rust's type system caught memory bugs; therefore, the code is correct.

**Why it fails**: Rust's type system is powerful but does not validate semantics. You can have the wrong algorithm, wrong constants, wrong loop bounds, and still compile successfully. Rust is a necessary condition for correctness, not sufficient.

**Institutional memory**: The chaparral fuel models compiled successfully but were stubs (15 `todo!()` panics). The fidelity review missed them (no execution). The parity suite caught them (execution).

**Mitigation**: Type correctness is necessary; executable proof is sufficient. Run the parity suite before declaring a module complete.

---

## 6. Provenance and Maintenance

**This skill is based on**:
- REVIEW.org (2026-07-06, HEAD f11cbc5) — "Parity suite results" section, fidelity review findings
- RUST_PORT.org (2026-03-05, original roadmap now largely superseded by REVIEW.org)
- behave-failure-archaeology (precedent for incident patterns)
- behave-change-control (change classification and divergence ledger)
- Git commit history (2016–2026, 750 commits; grep -E "units|pressure|VPD" reveals recurring patterns)
- `crates/behave-run/tests/parity.rs` (1,383 lines, 141 checks; golden-value source of truth)

**Facts verified by running**:
- Parity suite: `cargo test -p behave-run --test parity` → 1 passed (171 checks passing as of 2026-07-06)
- Unit tests: `cargo test --workspace --lib` → 282 tests passing (15+20+7+38+2+12+132+13+43)
- Crown L/W 1.625 prediction at 5 mph: verified in parity.rs line 533, matches 1 + 0.125×5
- EXRATE endpoints: 8.876216 (0% coverage, parity.rs line 702) and 21.971217 (100% coverage, line 712) verified
- Contain adapter: fire size 2.0× too large, perimeter 1.55× too large, documented in REVIEW.org parity results section

**Factual statements that drift over time** (re-verify when updating this skill):
- Test count: `cargo test --workspace --lib 2>&1 | grep "test result:" | awk '{s+=$3} END {print s}'`
- Parity check count: `grep -c "t\.check\|t\.check_bool" crates/behave-run/tests/parity.rs`
- Crate count: `ls -1d crates/*/ | wc -l`
- Git commit count: `git log --oneline | wc -l` on `rj-rust-port` branch
- Special fuel models status: search for `todo!()` in `crates/behave-surface/src/fuelbed.rs` (should be 0 as of 2026-07-06)
- REVIEW.org findings table: verify divergence ledger rows match code comments at file:line cites

**When to update this skill**:
- A new major discovery lands (e.g., a new anti-pattern, a fresh historical example)
- The parity suite expands significantly (>171 checks)
- A change-control decision contradicts guidance here (document in REVIEW.org first, then update this skill)
- Maintenance of any "verifiable fact" above: re-run the command, confirm the result, update the statement
