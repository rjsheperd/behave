---
name: behave-docs-and-writing
description: Maintain docs of record (REVIEW.org, RUST_PORT.org, ARCH.org, README.md, LICENCE.md); apply org-mode conventions, Rust doc-comment house style (//! module headers citing C++ sources, method comments with C++ method names), C++ Doxygen/public-domain headers, and commit message conventions (imperative summary + bulleted body + Co-Authored-By trailer).
---

# behave-docs-and-writing

Skill for maintaining the written record of the Behave (BehaveCore) library: documentation files, code comments, commit messages. HOME for doc-related facts and conventions; do not duplicate beyond one-line cross-references in other skills.

When NOT to use: external claims/papers/licensing questions → `behave-external-positioning`; verification of test/build facts → `behave-build-and-env` or `behave-validation-and-qa`; change control gates → `behave-change-control`; parallelization specifics → `behave-parallelization-campaign`.

## Docs of Record

All five are authoritative and held together by cross-references; update the one that owns each fact when state changes.

| Document | Path | Role | Update condition |
|----------|------|------|-------------------|
| REVIEW.org | `/REVIEW.org` (branch root) | **HOME** for current architecture state, fidelity review findings table (P0–P3 priority), divergence ledger (preserved quirks + deliberate fixes), and parallelization roadmap (5 phases + readiness matrix). Owned by code review closure. | After fidelity findings close; when divergence is discovered/documented; when parallelization phase completes or roadmap shifts. |
| RUST_PORT.org | `/RUST_PORT.org` | Historical record of the 2026-03-05 porting roadmap; largely superseded by REVIEW.org (which it cross-references). DO NOT update — it's a historical snapshot. | Never. Archive, do not mutate. |
| ARCH.org | `/ARCH.org` | Map of C++ architecture: the big-picture diagram (BehaveRun facade + modules), table of key C++ file locations, zone descriptions (surface/crown/spot/etc.). HOME for C++ component organization. | When C++ files are added/removed or logical zones reorganize. |
| README.md | `/README.md` | C++ build/test entry point: dependencies, `make compile` / `make test` / `make dev`, build artifacts to gitignore. | When build commands, environment setup, or outputs change. |
| LICENCE.md | `/LICENCE.md` | Public-domain (RMRS Missoula, US Gov work, title 17 §105) header block, copied verbatim into every C++ .h/.cpp file. | Never. Public domain is invariant. |

### REVIEW.org section structure

- **Executive Summary**: fidelity verdict + gap inventory + parallelization outlook + single sentence readiness statement.
- **Review Methodology**: baseline test counts + review agent list.
- **Architecture Review**: crate DAG + completeness inventory (C++↔Rust mapping table) + API design + error handling + test strategy gaps.
- **Fidelity Review by Crate**: per-crate status (✓, ⚠), constants verified, tables spot-checked, special cases noted (e.g., the TL5 typo, PressureUnits inverted quirk).
- **Parity Suite Results**: the golden-value test suite (behave-run/tests/parity.rs) findings — what it revealed and fixed.
- **Prioritized Findings**: P0–P3 table (finding, file/line, status — DONE or open). P0 blocks shipping; P1/P2 are medium; P3 is deferred.
- **Parallelization Roadmap**: target workload (landscape batch), readiness facts (purity, loops, transcendentals, tables, precision), phase-by-phase plan (Phase 0 pure-kernel refactor → Phase 1 rayon → Phase 2 SIMD → Phase 3 GPU).
- **Appendix**: baseline fact (branch name, build/test command, test count, reference C++ commit).

### Findings-row template (REVIEW.org findings table)

```
| P<0-3> | <one-line summary of finding> | <file> =<line(s) or anchor>= or =<ident>= |
```

Example from table:
```
| P0 | DONE — golden parity suite (171 checks, 0 ignored) landed | =behave-run/tests/parity.rs= |
| P1 | =behave-run= facade is a skeleton (no full input/output plumbing) | =behave-run/src/lib.rs= |
| P3 | Bare =f64= (no unit newtypes); silent clamping instead of =Result= | workspace-wide |
```

### Divergence-ledger entry template (REVIEW.org, Appendix or dedicated section)

Each C++↔Rust divergence belongs in REVIEW.org, with rationale, and gets a code comment (see "Rust doc-comment house style" below).

```
**[Category]** — [one-line summary]

C++ behavior: [what the C++ does, file:line if known]
Rust change: [what Rust does differently, file:line]
Rationale: [why (quirk preserved, C++ bug fixed, forward-compat)]
Test anchor: [the parity test name or condition that validates this]
Status: [PRESERVED (bit-parity requirement) | FIXED (deliberate divergence)]
```

Example from shared context:
```
**TL5 savrLiveWoody = 160.0 (likely typo for 1600)**
C++ behavior: fuel_models.cpp:809 hardcodes 160.0
Rust: preserved in fuel_models.rs:~809
Rationale: bit-parity; changing it would alter landscape-scale outputs
Test anchor: parity suite does not exercise this directly (FM TL5 not in testBehave)
Status: PRESERVED
```

## Org-mode Conventions

Files in `/` (root) and `/crates/*/` (rust) use org-mode for long-form docs. Conventions observed:

### Headings & TODO keywords (from RUST_PORT.org usage)

- `#+TITLE:`, `#+AUTHOR:`, `#+DATE: YYYY-MM-DD` (always ISO date).
- `#+STARTUP: overview` (default folded).
- `#+OPTIONS: toc:<N> num:nil` (table of contents up to N levels; no section numbers).
- `#+TODO: TODO IN-PROGRESS | DONE SKIP` (optional; define if the file has task tracking).
- Headings: `*`, `**`, `***` (1–3 levels typical; use 4+ sparingly).
- Code blocks: `` `#+begin_src <lang>` ``, e.g. `` `#+begin_src rust` ``, `` `#+begin_example` `` for pseudocode.
- Inline literals: `=code=` (monospace, for filenames, identifiers, commands, code fragments).
- Cross-references: `[[file:path/to/file.org][link text]]` (org-internal) or `[[*Section Name][link text]]` (within-doc).
- Links to code: `[[file:crates/X/src/Y.rs][src/Y.rs]]` (prefer this over inline line numbers if the file is stable).
- Table syntax: `| col1 | col2 |` with `|-` separator row below the header.

### Section templates

**Roadmap section** (from RUST_PORT.org):
```org
* Overview
** Scope
[description]

** Architecture: C++ → Rust Mapping
| C++ | Rust |

** Guiding Principles
1. [principle]
2. [principle]

* Phase Name
** Deliverables
** Timeline / Prerequisites
```

**Review section** (from REVIEW.org):
```org
* Review Methodology
- Baseline: [command] → N/N tests pass
- Review agents: [list of review types]

* Finding Area
** Crate or component name — ✓ / ⚠ / ✗ verdict
[detailed findings]

| Finding | Status |
|---------|--------|
```

## Rust Doc-Comment House Style

All Rust source modules (52 files under `crates/*/src/**/*.rs`) follow a uniform comment pattern. Read the actual file before writing; do not guess.

### Module header (//! comment at file top)

**Pattern:**
```rust
//! One-line summary of the module's purpose.
//!
//! Optionally: a paragraph or two of additional context.
//!
//! C++ source: file.h / file.cpp
```

**Verified examples:**
- `firelab-base/src/units.rs`: "Unit conversion types for the BehavePlus fire behavior library. … C++ source: behaveUnits.h / behaveUnits.cpp"
- `behave-surface/src/fire.rs`: "Surface fire spread rate, flame length, and related outputs. This is the core Rothermel surface fire spread calculator. … C++ source: surfaceFire.h / surfaceFire.cpp"
- `behave-crown/src/fire.rs`: "Crown fire spread rate, transition, and activity calculations. The Crown struct owns two Surface instances internally (composition pattern matching the C++ design). Implements both Rothermel (1991) and Scott & Reinhardt (2001) methods. C++ source: crown.h / crown.cpp"
- `behave-mortality/src/safety.rs`: "Safety zone calculations. C++ source: safety.h / safety.cpp. Calculates safety zone size, separation distance, and radius based on flame height and number of personnel/equipment."

**Rule:** always cite the C++ source file pair (header + implementation). If the module has no exact C++ counterpart (e.g., all Rust-original), say "No direct C++ counterpart" or cite the nearest logical parent.

### Struct/type doc comments (/// comment above type definition)

**Pattern:**
```rust
/// One-line description.
///
/// Optional: longer description.
///
/// C++ class: `ClassName`
pub struct MyType { … }
```

**Example (fire.rs:24–26):**
```rust
/// Core Rothermel surface fire spread calculator.
///
/// C++ class: `SurfaceFire`
pub struct SurfaceFire { … }
```

### Method doc comments (/// comment above function/method)

**Pattern:**
```rust
/// One-line summary of what the method returns or does.
///
/// Optional: parameters, side effects, panics.
///
/// C++ method: `methodName`
pub fn calculate_forward_spread_rate(&mut self, …) -> f64 { … }
```

**Example (fire.rs:171–176):**
```rust
/// Main entry point: calculate forward spread rate and all derived outputs.
///
/// Returns the spread rate in ft/min — either in direction of max spread
/// or in direction of interest if `has_direction_of_interest` is true.
///
/// C++ method: `calculateForwardSpreadRate`
pub fn calculate_forward_spread_rate(…) -> f64 { … }
```

### Inline comments for quirks and deliberate divergences

**Pattern for preserved C++ quirks:**
```rust
// C++ quirk preserved (file:line): [description]
// [rationale or parity note]
```

**Pattern for deliberate divergences:**
```rust
// DIVERGENCE: [description]
// C++ does [X]; Rust does [Y] because [reason].
// Test anchor: [test name or condition].
```

**Verified example (exrate.rs:110–116):**
```rust
/// The C++ quirks are preserved deliberately:
/// - the "go right" bounds check compares a flat ros index against
///   `num_alloc - 1` (the *path array* capacity, samples^depths), not the
///   ros array length;
/// - `spread_rates[0]` is populated by the forward pass and reused by the
///   left/right flanking passes without being rewritten;
/// - the flanking layer count truncates `separation / cell_size`.
```

### Numeric values from tables

Always cite the C++ source or constant name:
```rust
let constant = 0.1147;  // Anderson 1983, fire_size ellipse formula exp coefficient
let savr = 2580.0;      // FM3, from fuelModels.cpp:line
```

## C++ Documentation Style

All C++ files (41 headers + 41 implementations in `src/behave/`) use Doxygen and carry a public-domain header block.

### Public-domain header block (every .h and .cpp file)

**Pattern (MUST NOT be stripped during porting):**
```cpp
//------------------------------------------------------------------------------
/*! \file FileName.h
    \author Copyright (C) YEAR by Author Name.
    \license This is released under the GNU Public License 2.
    \brief One-line purpose.
  
    Longer description (optional).
 */

#ifndef _FILENAME_H_INCLUDED_
#define _FILENAME_H_INCLUDED_
```

**Example (Contain.h:1–9):**
```cpp
//------------------------------------------------------------------------------
/*! \file Contain.h
    \author Copyright (C) 2006 by Collin D. Bevins.
    \license This is released under the GNU Public License 2.
    \brief An implementation of Freid and Fried (\ref friedfried1995)
    wildfire containment model.
  
    This is a fire containment algorithm …
 */
```

### Doxygen comments (/*! … */)

Used for class, enum, method documentation. Examples:
```cpp
/*! \class Contain Contain.h
    \brief Fire flank (half-a-fire) containment object.
  
    Longer description.
 */

/*! \enum ContainTactic
    \brief Identifies the possible fire containment tactic.
 */
enum ContainTactic {
    HeadAttack = 0,     //!< Containment forces attack fire head
    RearAttack = 1      //!< Containment forces attack fire rear
};
```

## Commit Message Conventions

Followed in all Rust port commits (as of 2026-07-06). Observed in `git log` and verified in commit `f11cbc5`.

### Format

```
<imperative one-line summary (50–72 chars)>

<blank line>

<bulleted body; each bullet is 1–2 complete sentences; line-wrap at ~80 chars>
<each bullet starts with "- " and a capital letter>

<blank line>

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
```

### Example (commit f11cbc5)

```
Add C++ parity suite; port special fuel models, EXRATE, and fix contain adapter

- Add golden-value parity suite (behave-run/tests/parity.rs): replays the full
  testBehave.cpp sequence against one shared BehaveRun — 171 checks, all passing.
- Port special-model fuelbed integration (was 15 todo!()s in fuelbed.rs):
  chaparral, palmetto-gallberry, and western aspen depth/load/moisture/SAVR/
  heat-of-combustion/MOE/density/silica branches, mirroring
  surfaceFuelbedIntermediates.cpp; add the aspen mortality chain
  (fuelbed -> SurfaceFire -> Surface::get_aspen_mortality).
- Port EXRATE (randfuel/randthread, no-extension path) as exrate.rs and wire
  the two-fuel-models TwoDimensional method to it; newext Extension machinery
  is unreachable from Behave and documented as not ported.
- Fix ContainAdapter initial-attack geometry to match ContainAdapter.cpp
  (effective windspeed 4*(LW-1) through FireSize); was 2x off on fire size.
- Restore slopeTool.cpp unit quirk in calculate_horizontal_distance (BehavePlus
  outputs bake it in) and align the unit test with testBehave.cpp expectations.
- Add missing Surface::backing/flanking_spread_distance facade getters.
- Add REVIEW.org: architecture/fidelity review and parallelization roadmap.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
```

### Rules

- **Imperative voice**: "Add", "Fix", "Port", "Restore", not "Adding", "Added", "Fixed".
- **One-line summary first**: complete sentence or noun phrase; no period. Omit ticket numbers in the summary; include them in the body if relevant.
- **Body bullets**: each describes one logical change; reference files, line numbers, or C++ methods when precision matters.
- **Trailer**: `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` on every port/review commit (convention for AI-assisted work).

## Skill Maintenance Note Template

When adding a new skill to this crate ecosystem, include a "Provenance and maintenance" section at the end:

```markdown
## Provenance and maintenance

This skill is based on: [list of sources, e.g. "REVIEW.org (2026-07-06)", "git log (commits f11cbc5–ba9b6bd)", "reading 5 Rust modules"]

Re-verify these facts periodically (command to run or manual check):
- **Test count** (should be ~279 unit + 171 parity): `cargo test --workspace -- --list | grep "test result:" `
- **REVIEW.org sections**: read `/REVIEW.org` for changes to findings table, divergence ledger, phase status
- **Rust doc-comment patterns**: spot-check 2–3 new modules when they land for consistency with //! module header + C++ source citation
- **Commit message convention**: check latest 3 commits with `git log --format=%B -3` for imperative voice + Co-Authored-By trailer
- **C++ file count**: `find src/behave -name "*.h" -o -name "*.cpp" | wc -l` (should be ~82 files)
- **Org-mode date stamps**: verify `/REVIEW.org` #+DATE field is current (update when facts change)
```

## Quick Reference Checklist

Before writing docs, code comments, or commit messages:

- [ ] **File naming**: .org for long-form docs (REVIEW.org, RUST_PORT.org, ARCH.org); .md for building (README.md); .rs for code
- [ ] **Org-mode files**: include #+TITLE, #+AUTHOR, #+DATE (ISO), #+STARTUP:overview, cross-references with `[[file:…]]` and `[[*…]]`
- [ ] **Rust module header**: //! + one-line summary + "C++ source: X.h / X.cpp"
- [ ] **Rust method comments**: /// summary + "C++ method: methodName"
- [ ] **Rust quirk/divergence**: inline comment with file:line, rationale, and test anchor
- [ ] **C++ files**: keep public-domain header block (\file, \author, \license, \brief); Doxygen /*! … */ for classes/methods
- [ ] **Commit message**: imperative summary (50–72 chars) + body bullets (1–2 sentences each) + Co-Authored-By trailer
- [ ] **Findings table**: P0–P3 rows with one-line summary, file/line reference, status
- [ ] **Divergence ledger**: always document in REVIEW.org with C++ behavior, Rust change, rationale, test anchor, status

## Provenance and maintenance

This skill is based on: REVIEW.org (2026-07-06), RUST_PORT.org (2026-03-05 snapshot), ARCH.org, git log (commits f11cbc5 and earlier), reading 5+ Rust modules (units.rs, fire.rs, safety.rs, crown/fire.rs, exrate.rs), C++ public-domain headers (Contain.h, ContainResource.h), parity.rs test structure, and README.md / LICENCE.md.

Re-verify these facts:
- **Docs of record table**: verify all 5 files exist and read their #+TITLE/first section — `ls -la /REVIEW.org /RUST_PORT.org /ARCH.org /README.md /LICENCE.md`
- **REVIEW.org sections**: `grep "^\\* " /REVIEW.org | head -20` (should list Exec Summary, Review Methodology, Architecture, Fidelity, Parity, Findings, Parallelization)
- **Rust module doc count**: `grep -l "^//!" crates/*/src/**/*.rs | wc -l` (should be ~40+)
- **Parity suite test count**: `grep "fn test_" crates/behave-run/tests/parity.rs | wc -l` (test functions; multiply by 10–20 for actual assertion count)
- **Commit message convention**: `git log -1 --format=%B | grep "Co-Authored-By"` (should appear)
- **C++ public-domain header**: `head -10 src/behave/Contain.h | grep "license"` (should match LICENCE.md verbatim)
