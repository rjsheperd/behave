---
name: behave-external-positioning
description: Ecosystem positioning, licensing strategy, upstream relationship, consumer landscape, and claims discipline for the Behave library (Rust port and C++ reference). Use when addressing release readiness, upstream bug reporting, licensing questions, performance claims vs FlamMap/ElmFire, or positioning the Rust port relative to C++ consumers. When NOT to use this skill: for change control gates and evidence standards, use `behave-change-control` and `behave-validation-and-qa`; for Rust/C++ architectural details, use `behave-architecture-contract`; for domain theory and equations, use `fire-behavior-reference`.
---

# Behave External Positioning

*Ecosystem, licensing, claims discipline, and upstream-relationship runbook for the Behave Rust port (canonical going forward, as of 2026-07-06) and C++ reference.*

## Licensing Strategy

### Status (as of 2026-07-06)

| Artifact | License | Citation | Obligation |
|----------|---------|----------|-----------|
| C++ library (`src/behave/`, 44 .h + 44 .cpp) | US public domain (17 USC 105, RMRS) | LICENCE.md | Preserve RMRS attribution in derived works |
| Rust port (`crates/`, 9-crate workspace) | **OPEN**: NO license fields in any Cargo.toml | — | Must be decided before crates.io release |
| Derived files (e.g., Rust→C++ porting docs) | Inherit public domain | LICENCE.md + code comments | Attribution notes, not license headers |

### Decision Point: Rust Crate Licensing

The Rust workspace (currently ~19,300 lines) is a derived work of public-domain C++ — derived works of public-domain software remain unrestricted. **Recommended crate license field format** (pending decision, not implemented):
- `license = "CC0-1.0 OR Unlicense"` (explicit waiver, maximizes reusability)
- OR `license = "Unlicense"` (simpler, permissive, standard in Rust ecosystem)
- OR `license = "CC0-1.0"` (Creative Commons zero assertion)

**Action**: before pushing to crates.io, add license field to workspace root `Cargo.toml` and each member crate matching the chosen strategy. Update the in-repo LICENCE.md to note Rust port coverage.

### Preservation Rule

C++ header attribution (7-line RMRS/17 USC 105 boilerplate) must survive in derived file comments if the source file is directly translated. Example in Rust source:
```rust
// C++ source: fuelbed.h / fuelbed.cpp
// RMRS public domain (17 USC 105), preserved as-is.
```

## Upstream Relationship

### Upstream & Fork Topology

- **Upstream** (origin): `https://github.com/firelab/behave` (firelab/behave, C++ master branch)
- **Fork** (this repo): `git@github.com:rjsheperd/behave.git` (rjsheperd, tracking branch `rj-rust-port`)
- **Integration**: Jira tickets `BHP1-####` in branch names → GitHub PRs → squash-merge to upstream master (no tags, no semver, no changelog as of 2026-07-06)

### Bug Reporting Protocol

Bugs found in C++ during the Rust port must be **reported upstream**, not silently forked. Reference the Rust port's parity suite and code review as evidence.

**Candidate upstream issues** (from the Rust divergence ledger; see `behave-change-control` for full details):

| Bug Category | Finding | C++ Location | Evidence | Upstream Status |
|--------------|---------|--------------|----------|-----------------|
| Dead code / variable shadowing | `isFuelDepthNeeded()` always returns false (shadowed variable) | ignite.cpp:310–320 | Rust parity test shows correct logic works | UNVERIFIED — not yet filed upstream |
| Field index error | `setMoistureHundredHour()` writes one-hour field [0] instead of hundred-hour [1] | ignite.cpp:238 | Rust separates fields correctly; parity test documents intent | UNVERIFIED — not yet filed upstream |
| Dead code / index error | `setLiveFuelHeatOfCombustion/Moisture()` write index [0] (dead fuel) instead of [1] (live fuel) | chaparralFuel.cpp | Rust indexes correctly; methods are unreachable in practice | UNVERIFIED — not yet filed upstream |
| Assertion-validation defect | `testBehave.cpp` compares observed-to-observed for moisture-class-needed and crown perimeter checks (~lines 379, 1104–1139) | testBehave.cpp:~379, ~1104–1139 | Rust parity suite asserts declared expected values instead; all pass | UNVERIFIED — test suite bug, low priority |

**Filing procedure** (when deciding to escalate):
1. Clone firelab/behave locally; reproduce the bug in C++ from minimal test case.
2. Reference Rust parity suite evidence: cite exact line numbers in `crates/behave-run/tests/parity.rs`.
3. File GitHub issue on firelab/behave (or Jira if firelab maintains internal backlog).
4. Do NOT push divergent C++ behavior to master branch; document as "preserved quirk" in Rust code with comments.
5. Once upstream fixes, backport the C++ fix and remove the Rust workaround.

## Consumers & Competitive Landscape

### Direct Consumers of C++ Library

| Consumer | Role | Integration | Notes |
|----------|------|-----------|-------|
| BehavePlus desktop | Primary GUI; landscape & fire design tool | Links against C++ `libBehave.a` (static) | Published 2010–2023; BehavePlus 6 outputs are golden reference for quirk compatibility (e.g., slope-tool units bug preserved for parity). |
| Behave web app | Browser-based fire simulator | Links against Rust port (planned) or C++ via WASM (legacy) | Not yet live; considered candidate for first Rust consumer. |
| IFTDSS (Integrated Forest Threat Decision Support System) | USFS decision-support framework | Embedded C++ library | Operational use; federal wildland-fire planning tool. |

### Competitive / Reference Implementations

| Engine | Type | Scope | Relevance |
|--------|------|-------|-----------|
| **FlamMap** | Landscape-scale fire behavior simulator | Batch evaluation of cell-by-cell fire spread | Competitive frame for GPU goal: Rust port aims to beat FlamMap performance at landscape scale via batching + GPU. |
| **ElmFire** (formerly Wildland-Urban Interface Fire Dynamics Simulator) | Fire behavior + atmosphere coupling | Coupled fire–atmosphere simulation on landscapes | Similar competitive class to FlamMap; GPU port must match or exceed. |
| **FARSITE** (Fire Area Simulator) | Landscape fire growth and behavior | Historical fire-growth prediction tool | Parallel-compute candidate; not a direct competitor (focuses on fire perimeter growth, not real-time behavior). |
| **FOFEM** (Fire and Fuels Extension to the Forest Vegetation Simulator) | Mortality & emissions from fire effects | Post-fire vegetation mortality (species-specific) | Reference standard for mortality validation; Behave `behave-mortality` crate calibrated against FOFEM output (see `behave-validation-and-qa`). |

## Claims Discipline

### Evidence Standards for Public Statements

**RULE**: Do NOT claim "numerically identical to BehavePlus/C++" or "beats FlamMap" without citing concrete, reproducible evidence. All claims must trace to one of these:

#### 1. **Parity Suite (Golden Reference)**

Claim template: *"Numerically identical to C++ Behave reference suite for [specific outputs]"*

**Supporting evidence**: `crates/behave-run/tests/parity.rs` (171 checks, all passing as of 2026-07-06)
- Mirrors `src/testBehave/testBehave.cpp` call sequence exactly.
- Tolerance: 1e-6 absolute (matches C++ `error_tolerance`).
- Outputs covered by exact parity:
  - Surface spread rate (ROS, ft/min) for 100+ fuel models and special fuels (chaparral, palmetto, aspen).
  - Flame length, fire size (area, perimeter).
  - Crown fire (Rothermel + Scott & Reinhardt).
  - Spot fire (all 4 sources, 14 species).
  - Ignition probability (lightning + firebrand).
  - Containment (Fried & Fried adapter).
  - Mortality (species master table, 197 species).
  - Weather tools (VPD, fine-dead-fuel, slope conversions).

**Scope limitations**: Do NOT claim parity for:
- Outputs NOT covered by parity (facade I/O plumbing, partial 10-m wind paths — see `behave-architecture-contract` for gaps).
- f32 GPU variants (see error bounds section below).

#### 2. **f32 / GPU Results (Error Bounds Discipline)**

Claim template: *"f32 GPU results match f64 Rust baseline to within [X]% error (median [Y]%, max [Z]%) on [dataset]"*

**Required before claiming GPU correctness**:
- Run controlled experiment on published benchmark dataset (e.g., Rothermel SurfaceFire + 40-fuel-model sweep).
- Compare f32 output vs f64 baseline (Rust reference).
- Report:
  - **Median absolute error** (% or absolute units, e.g., ft/min ROS).
  - **Max absolute error** (worst-case single output).
  - **Error distribution** (histogram or percentile table).
  - **Transcendental hotspots** (where f32 divergence is highest; e.g., scorch height formula has cancellation risk).
- Pinned toolchain, seed-free determinism (f32 is deterministic across architectures if no dynamic dispatch).

**Gap** (as of 2026-07-06): f32 study is not yet run. Do NOT claim GPU fidelity until it is.

#### 3. **Performance Claims vs FlamMap/ElmFire**

Claim template: *"Rust port [batched|GPU] variant processes N cells/sec, [X]× faster than FlamMap on [dataset] with [hardware]"*

**Minimum reproducibility bar**:
- Pinned C/C++ FlamMap/ElmFire binary versions + commit SHAs.
- Published landscape dataset (e.g., LANDFIRE, SNF test grid, or cite your own with dimensions + cell count).
- Hardware specification: CPU model, cores, RAM, GPU (if applicable), thermal environment.
- Fair-play assumptions clearly stated:
  - Same fuel model map, weather grid, and time step as FlamMap.
  - Clock measurement methodology (wall-clock, CPU time, GPU-only, or incl. I/O?).
  - Whether Behave is single-threaded, multi-threaded, or GPU-accelerated in the test.
  - Compiler flags, optimization levels (e.g., `-O3` vs `-Ofast`).
- Result reproducibility: provide enough detail that a peer can re-run the benchmark.

**Gap** (as of 2026-07-06): GPU variant does not exist yet. Any performance claim is speculative until Phase 1 (rayon batch) lands and is benchmarked.

### Framing Rules (What NOT to claim)

| Claim | Status | Why | Correct Framing |
|-------|--------|-----|-----------------|
| "Rust port is a drop-in replacement for C++" | ❌ Never | API is intentionally mutable-state-style (phase 1 parity, not production). Facade is a skeleton. | "Rust port is bit-for-bit parity-compatible with C++ for core fire behavior calculations; API surface TBD for production. |
| "GPU version is ready for production" | ❌ Never (until Phase 2+ complete) | f32 study not done, WebGPU divergence handling unproven, no real landscape benchmarks. | "GPU roadmap (Phase 2+) is planned; feasibility study underway." |
| "Beats FlamMap at landscape scale" | ❌ Until benchmarked | Speculative; competitive comparison requires same-dataset, same-hardware test. | "GPU research target: match or exceed FlamMap throughput on [specific landscape, hardware config]." |
| "No numerical divergence from C++" | ⚠️ Qualified only | The 171-parity-check suite confirms for the outputs it covers. But facade I/O, f32, and GPU path divergences are expected. | "Core fire behavior calculations are numerically identical to C++ (171 parity checks); facade I/O and GPU paths under development." |
| "Preserves all BehavePlus behavior" | ⚠️ Qualified only | Rust port preserves all known C++ quirks (TL5 SAVR, slope-tool units, etc.) tested & documented. But Rust may gain new features. | "Rust port preserves C++ behavior for all 171 parity-test outputs (fuel models, special fuels, wind, crown, spot, mortality, contain, weather tools); new features tracked separately." |

## Release Readiness Gaps for crates.io

### Pre-Release Checklist (P0 blocking)

- [ ] **License fields** — add `license = "CC0-1.0 OR Unlicense"` (or chosen variant) to workspace root and each crate Cargo.toml.
- [ ] **crate descriptions** — audit and finalize short descriptions for each crate in Cargo.toml ([description] field, <100 chars).
- [ ] **docs.rs metadata** — verify `[package.metadata.docs.rs]` settings for each public crate (e.g., enable all features, exclude tests if applicable).
- [ ] **Versioning policy** — decide semver (0.1.0 as-is? 1.0.0 for parity-tested release? feature-based 0.x.y?) and document in CONTRIBUTING or CHANGELOG.
- [ ] **README** — each public crate should have module-level docs (visible on docs.rs); top-level README should exist or link to docs.
- [ ] **CONTRIBUTING** — define PR acceptance criteria, parity-test requirement, code-review process.

### Pre-Release Checklist (P1 strongly recommended)

- [ ] **CI coverage for Rust** — add `.github/workflows/rust-ci.yml` (cargo test --workspace, cargo clippy, cargo fmt check).
- [ ] **Changelog** — adopt a schema (CHANGELOG.md, Markdown; or keep in git tags/releases). Document the parity-suite landing and special-fuel integration.
- [ ] **Semver tags** — tag the first release (e.g., `v0.1.0-parity-release`) and push to crates.io.
- [ ] **FOFEM calibration note** — in behave-mortality README, cite the comparison dataset and tolerance (see `behave-validation-and-qa`).

### Post-Release Operations

- **Update upstream README.md** — note Rust port availability and link to `crates.io` / `docs.rs`.
- **Consumer outreach** — notify BehavePlus, Behave web app, IFTDSS teams when release is ready.
- **Monitoring** — watch GitHub Issues for consumer bug reports; triage against C++ divergence ledger.

## Related Skills

- `behave-change-control` — C++ divergence ledger (HOME for details), parallelization-compatibility checklist, incident-backed non-negotiables.
- `behave-validation-and-qa` — parity suite anatomy, adding tests, FOFEM comparison methodology.
- `behave-docs-and-writing` — house style, docs-of-record locations (REVIEW.org, RUST_PORT.org, ARCH.org).
- `fire-behavior-reference` — domain glossary (FlamMap, ElmFire, FARSITE, FOFEM context).
- `behave-parallelization-campaign` — GPU roadmap phases, performance targets, the "beyond-SOTA" goal.

---

## Provenance and Maintenance

**Skill based on**: LICENCE.md (license boilerplate), REVIEW.org (2026-07-06 findings/parity suite), Cargo.toml workspace structure, firelab/behave remote (upstream topology), `crates/behave-run/tests/parity.rs` (171-check suite), consumer references in REVIEW.org.

**Re-verification commands** (run from repo root):

```bash
# Verify license file
head -20 LICENCE.md

# Count license fields in Cargo.toml
grep -c "^license" Cargo.toml crates/*/Cargo.toml || echo "0 license fields found"

# Verify upstream remote
git remote -v | grep firelab

# Count parity checks (search for check calls)
grep "\.check(" crates/behave-run/tests/parity.rs | wc -l

# Run parity suite and confirm pass
cargo test -p behave-run --test parity 2>&1 | grep "test result:"

# Verify divergence ledger comments in source
grep -r "NOTE: C++" crates/behave-surface/src/fuel_models.rs
grep -r "ignite.cpp" crates/behave-ignite/src/inputs.rs

# Confirm no file-by-file checklist exists yet for crates.io readiness
[ -f CRATES_IO_RELEASE_CHECKLIST.md ] && echo "Checklist exists" || echo "Checklist does NOT exist (expected gap)"
```

**Last updated**: 2026-07-06 by Claude Code.
**Next review trigger**: before any crates.io push, after upstream PR activity, or when f32 study lands.
