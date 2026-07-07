---
name: behave-orchestrator
description: Entry-point skill for ANY work session on the Behave repo (Rust port of the USFS fire behavior library). Load this FIRST when starting a task of any kind — debugging a wrong number, adding or changing a feature or flag, porting C++ code, running the parallelization campaign, validating changes, writing docs, making release or licensing decisions, or doing research. It contains zero project facts; it is the routing layer — which sibling skill to load when and in what order, the session bootstrap and landing checklists, the multi-agent orchestration patterns proven on this project, context-budget discipline, and the escalation rules for decisions only the owner may make.
---

# behave-orchestrator

You are an Opus-level agent starting a session on the Behave repo with zero
prior context. The 16 sibling skills under `.claude/skills/` hold the project's
knowledge; this skill tells you how to drive them. It deliberately contains
**no project facts** — every fact has exactly one home in a sibling skill
(one-home-per-fact rule). If you catch yourself copying a fact into this file,
you are violating that rule: add it to the owning sibling instead.

## 1. Session bootstrap (run before ANY task)

Execute in order. Do not skip steps because the task "looks small" — the
project's costliest recorded incidents were one-liners
(`behave-failure-archaeology`).

1. **Orient**:
   ```bash
   git status --short && git log --oneline -3 && git branch --show-current
   ```
   Expected branch (as of 2026-07-06): `rj-rust-port`. Any other branch, or a
   dirty tree you did not create → stop, report, and ask before proceeding.
2. **Baseline the health of the tree you inherited**:
   ```bash
   bash .claude/skills/behave-diagnostics-and-tooling/scripts/count_state.sh
   ```
   For any task touching numerics, also run the full cross-validation
   (C++ and Rust suites must both pass BEFORE you change anything):
   ```bash
   bash .claude/skills/behave-diagnostics-and-tooling/scripts/cross_validate.sh
   ```
   **Never start work on an unverified baseline.** You cannot attribute a
   failure you did not baseline: if the suite is red after your change, you
   must know it was green before it.
3. **Read the state of record**: open `REVIEW.org` and read its priority
   findings table. Treat narrative prose sections as time-stamped snapshots,
   not current truth — a stale narrative bullet in that file has misled agents
   before (see `behave-failure-archaeology`). Tables and dated entries win
   over prose.
4. **Drift-check the facts you are about to rely on**: every sibling skill
   ends with a "Provenance and maintenance" section of one-line re-verification
   commands. Run the ones relevant to your task. A skill fact with a
   re-verification command is a claim; the command's output is the truth.

## 2. Task routing table

Classify the incoming task by its shape, load the listed skills **in order**,
and stop when the listed gate passes. Load lazily — start with the first skill
listed; pull the next only when you hit its territory.

| Task shape (trigger phrases) | Load in order | Done when |
|---|---|---|
| "X computes the wrong number" / parity failure / divergence from C++ | `behave-debugging-playbook` → `behave-numerics-proof-toolkit` (divergence bisection) → `behave-failure-archaeology` (has this been fought before?) | One mechanism explains ALL observations, including the negative ones; full parity suite green |
| Add/change a feature, flag, default, enum, or table | `behave-change-control` (classify the change FIRST — §1 of that skill) → `behave-config-and-toggles` (if it is a behavior axis) → `behave-validation-and-qa` (evidence for landing) | The gate for its change class passes |
| Parallelization / performance / SIMD / GPU / batch work | `behave-parallelization-campaign` (find the current phase and its gate) → `behave-architecture-contract` (the compatibility contract) → `behave-diagnostics-and-tooling` (measurement) | The current campaign phase's gate, measured — never eyeballed |
| Port code from C++ (this repo's C++ or a foreign package) | `behave-failure-archaeology` (the porting lessons — read BEFORE writing code) → `behave-validation-and-qa` (goldens first) → `behave-change-control` (divergence ledger discipline) | The new parity section is green; any C++ bug found is ledgered, not silently fixed |
| Build/env broken, fresh machine, CI question | `behave-build-and-env` → `behave-run-and-operate` | Bootstrap smoke sequence green |
| Run something / "is it working?" / where do outputs go | `behave-run-and-operate` | The documented health check passes |
| Write/update docs, commit messages, REVIEW.org | `behave-docs-and-writing` | Docs of record updated per its templates |
| Release, licensing, upstream contact, public claims, benchmarks vs other tools | `behave-external-positioning` → `behave-validation-and-qa` (claims must cite evidence) | Claim carries its parity/benchmark citation |
| "Why is it designed like this?" / architecture review | `behave-architecture-contract` → `behave-failure-archaeology` | — (informational) |
| Domain question (what IS this quantity/model/convention?) | `fire-behavior-reference` | — (informational) |
| Open-ended improvement idea / "what should we work on?" | `behave-research-frontier` (is it a known frontier?) → `behave-research-methodology` (how to pursue it) | Hypothesis written with predicted numbers before any code |

**Fallback rule**: if no row matches, you are probably doing research —
load `behave-research-methodology` before touching code, and write the
hypothesis (with predicted numbers) first.

**Multi-shape tasks**: decompose and route each part. A typical feature task
becomes: classify (`behave-change-control`) → implement → validate
(`behave-validation-and-qa`) → document (`behave-docs-and-writing`).

## 3. The standard operating loop

Every change-making session follows this loop. Gates are not optional for
"small" changes.

1. **Baseline** (§1 above).
2. **Classify** the change using `behave-change-control` §1. The class
   determines the gate; ambiguity about class = escalate (§5).
3. **Load** the routed skills (§2).
4. **Work.** Respect the parallelization-compatibility checklist
   (`behave-change-control` §3) in every edit to `crates/` — it applies to
   ALL changes, not just campaign work.
5. **Gate**:
   ```bash
   cargo test --workspace
   cargo test -p behave-run --test parity -- --nocapture 2>&1 | grep 'parity:'
   ```
   plus the class-specific gate from `behave-change-control`.
6. **Record**: update REVIEW.org (findings row; divergence-ledger entry if
   applicable) per `behave-docs-and-writing`. A change that isn't recorded in
   the docs of record is not done.
7. **Land**: commit per the conventions in `behave-docs-and-writing`. Do not
   push or open PRs unless the task said to.

## 4. Multi-agent orchestration patterns (proven on this project)

These patterns built the parity suite, the special-model ports, and this skill
library. Each carries the failure mode observed alongside it. Use them when a
task fans out beyond one context window; skip them for single-file work.

### 4.1 Discovery fan-out (before authoring or big changes)
Spawn parallel **read-only** investigators with disjoint lenses (e.g.
build-system / git-history / code-state / domain-knowledge). Rules for every
sub-agent prompt:
- Ground truth only: verify every claim against the repo; cite file:line or
  commit hash; label anything unverifiable UNVERIFIED.
- Read-only: no file mutation, no mutating git commands.
- Return a structured report, not prose narrative.

### 4.2 Author → adversarial review → fix
For multi-artifact production: parallel authors (one artifact each, shared
context pack) → barrier → parallel reviewers **assigned to refute** (distinct
lenses: factual / consistency / usability) → one fixer applying
blocking+important findings.
**Observed failure mode**: a reviewer's "correction" was itself wrong (it
replaced a runtime-measured count with a static grep count). Rule: **the fixer
must re-verify each finding empirically before applying it.** A reviewer
finding is a hypothesis, not a verdict.

### 4.3 Executable evidence beats review consensus
A favorable multi-agent code review coexisted with 15 `todo!()` panics that a
single test run exposed (`behave-failure-archaeology` has the entry). When a
review and an execution disagree, execution wins. Prefer building one
executable check over commissioning a second review.

### 4.4 Sub-agent hygiene (bake into every spawned prompt)
- Declare the writable scope explicitly; everything else read-only.
- Forbid mutating git commands outright.
- Warn about side-effectful "verification": an agent regenerating docs dirtied
  ~400 tracked files under `docs/` (Doxygen output is tracked). Builds/tests
  writing to gitignored `build/` and `target/` are fine.
- Include the repo-specific traps that waste agent time (they live in
  `behave-debugging-playbook`; the most common: some C++ sources contain
  non-ASCII bytes — plain grep silently returns nothing; use `grep -a`).
- Context packs are maps, not sources of truth: instruct agents to re-verify
  anything they repeat from the pack.

### 4.5 Sequencing rule
Author/act in parallel only when artifacts are independent. Anything with
shared state (like the parity suite's ordered sections) must be sequenced or
given the full prefix context.

## 5. Escalation rules — stop and ask the owner

These map one-to-one to `behave-change-control` classes and the open decisions
in `behave-external-positioning`; escalation is a lookup, not a judgment call.
Stop work and ask before:

- Fixing a **preserved C++ quirk** (ledger §2.1 of `behave-change-control`) —
  even an "obvious" one.
- Changing any **default** or anything in the **behavior-change** class.
- Editing a **golden value** without a derivable source (C++ run, paper
  derivation, or BehavePlus reference — see `behave-validation-and-qa`).
- **Licensing** decisions or publishing crates.
- Any **upstream contact** (issues/PRs to the parent C++ repo).
- Declaring a **campaign phase complete** (`behave-parallelization-campaign`
  gates are proposals until the owner accepts the measurements).
- Anything **destructive or outward-facing** (force-push, deleting branches,
  publishing artifacts).

When escalating: state the change class, the evidence gathered, and the
specific decision needed — not an open-ended "what should I do?".

## 6. Context-budget discipline

Skills are swap-in memory, not a syllabus:
- Load at most 2–3 siblings at once; follow cross-references lazily.
- To check ONE fact, prefer running the owning skill's provenance
  re-verification one-liner over loading the whole skill.
- One-home-per-fact means you never need two skills open to trust one fact;
  if two skills state the same fact differently, that is a bug — fix the
  non-HOME copy to a cross-reference and note it.
- When spawning sub-agents, send them a distilled context pack plus pointers
  to the owning skills — not full skill texts.

## 7. When NOT to use this skill

- Do not use it as a substitute for the sibling that owns a fact — it
  intentionally contains no project facts. Commands shown here (bootstrap,
  gates) are the property of `behave-build-and-env`,
  `behave-diagnostics-and-tooling`, and `behave-validation-and-qa`; if they
  disagree with those skills, the owning skill wins and this file needs a fix.
- Do not load it for a pure domain question — go straight to
  `fire-behavior-reference`.
- Do not treat the routing table as permission: routing says which skill
  governs, and that skill's gates still apply.

## Provenance and maintenance

Based on: the 16-skill library authored 2026-07-06; the working patterns of
the sessions that produced the parity suite, the special-model/EXRATE ports,
and the library itself (multi-agent discovery, author/review/fix, empirical
gate discipline); owner directives of 2026-07-06 (canonical Rust,
parallelization compatibility, Opus-level agent audience).

This skill has fan-out coupling to ALL siblings — it MUST be updated whenever
a skill is added, renamed, or removed.

Re-verification one-liners:
```bash
# All routed skills still exist (expect 17 dirs incl. this one):
ls .claude/skills/ | wc -l && ls .claude/skills/

# Every skill referenced (inline-backticked) in this file exists
# (crate names like behave-run in code blocks are intentionally not matched):
grep -oE '`(behave-[a-z-]+|fire-behavior-reference)`' .claude/skills/behave-orchestrator/SKILL.md \
  | tr -d '`' | sort -u | while read s; do [ -d ".claude/skills/$s" ] || echo "MISSING: $s"; done

# Diagnostics scripts referenced here still exist:
ls .claude/skills/behave-diagnostics-and-tooling/scripts/

# Bootstrap gate commands still valid:
cargo test -p behave-run --test parity -- --nocapture 2>&1 | grep 'parity:'

# Expected branch still current (update §1 if the project moves on):
git branch --show-current
```
