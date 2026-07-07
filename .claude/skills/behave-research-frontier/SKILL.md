---
name: behave-research-frontier
description: Catalog of open research problems where Behave can advance the state of the art — GPU fire behavior simulation, cloud-native Rust distribution, WASM parity, differential testing, upstream contributions, and ensemble uncertainty. Each frontier defines why SOTA falls short, this project's asset, the first three concrete repository steps, and a falsifiable "result when..." milestone. Everything is labeled candidate/open.
---

# Research Frontiers: Advancing State of the Art

This skill catalogs open research problems where the Behave Rust port is positioned to advance beyond current published tools. Each frontier is grounded in a specific project asset (parity-validated kernels, pure-compute profile, clean architecture) and includes the first three actionable steps *within this repository* plus a falsifiable milestone.

**When to use this skill**: You are planning a multi-phase research campaign, designing a proof-of-concept, or assessing whether a proposed direction is in scope for the hardest technical challenges Behave is built to solve.

**When NOT to use this skill**: For incremental engineering work (bugfixes, API design, testing strategies) → use `behave-change-control`; for execution of the parallelization phases → use `behave-parallelization-campaign`; for proof-of-concept design → use `artifact-design`.

---

## (1) FLAGSHIP — GPU Landscape Fire Behavior Beating FlamMap/ElmFire

**Candidate status**: Owner-confirmed hardest problem (2026-07-06). Already on the parallelization roadmap (Phase 3, REVIEW.org).

### Why Current SOTA Falls Short

- **FlamMap** (USDA Forest Service): CPU-bound desktop batch tool; landscape raster evaluation is serial-per-cell or coarse fork-join; single-machine throughput caps out ~10^6–10^7 evaluations/sec on modern CPUs.
- **ElmFire** (Stratton et al., Fortran/MPI): cluster-parallel spread simulation with fancy numerics, but still CPU-first; per-cell cost ~1200 raw flops makes it ideal for GPU batching, yet the tools do not ship GPU implementations as of 2026-07.
- **Neither tool** publishes validated f32 precision budgets for GPU deployment, creating an integration risk for consumers.

### This Project's Specific Asset

1. **Parity-validated pure-Rust kernels**: The surface/crown/spot/ignite/mortality calculation stack is bit-exact to C++ reference via the parity suite (171 passing checks, crates/behave-run/tests/parity.rs:1–1383).
2. **Compute-bound profile**: surface kernel ≈ 1200 flops per call; transcendental-heavy (44 pow/exp/log/sqrt/trig sites); zero data dependency loops (all bounds fixed at compile time). The ~5000:1 flops-to-bytes ratio is textbook GPU workload (REVIEW.org "Readiness facts").
3. **WGSL path already designed**: Phase 3 architecture (REVIEW.org lines 359–381) specifies fuel-model-kind dispatch, pre-grouped SoA storage buffers, and per-dispatch kernel pipelines (surface → crown → spot chained). Eliminates warp divergence instead of paying for it.
4. **Licensing/autonomy**: USDA public-domain code, no external dependencies, pure Rust — zero friction to re-license, fork, or integrate with WebGPU runtime.

### First Three Concrete Steps (In This Repo)

**Step 1 — Phase 0 Pure-Kernel Refactor** (prerequisite for all GPU work).
- **What**: Separate calculation math from the mutable-state API.
  - Define `fn surface_kernel(inputs: &SurfaceInputs, tables: &FuelModels) -> SurfaceOutputs` — a pure function.
  - Move `FuelbedIntermediates` from a `Surface` field to stack-local scratch.
  - Make `FuelModels` `&'static` (was cloned per instance) → shared GPU upload.
  - Keep the existing stateful `Surface` wrapper for backward-compatibility tests.
- **Repo path**: All of `crates/behave-surface/src/fire.rs`, `facade.rs`, `inputs.rs`, `fuelbed.rs`.
- **Gate**: All 279 unit tests + 171 parity checks remain passing.
- **Effort**: Largest single chunk in the roadmap, but mechanical (pure refactoring, no math changes).

**Step 2 — Single-Thread Criterion Baseline** (perf floor, CPU-side reference).
- **What**: Use `cargo bench` with criterion to measure surface/crown/spot kernels (scalar f64, no SIMD/threads) on a golden scenario (fuel model GS124, 5 standard size classes).
  - Run `criterion::black_box()` over 10,000 iterations of a single-cell surface call.
  - Record cycles/call, MHz, flops/sec on a fixed machine (e.g., "Apple M1 base clock 3.2 GHz").
  - Establish target: e.g., "if GPU achieves > 10^8 cells/sec, that is ≈ 100× speedup over scalar CPU."
- **Repo path**: New `crates/behave-run/benches/criterion_baseline.rs` + `[dev-dependencies] criterion = "0.5"` in `crates/behave-run/Cargo.toml`.
- **Gate**: Baseline published (e.g., "GS124 surface: 48.3 µs/call on M1 base, 2.17 MHz equivalent flop rate").

**Step 3 — wgpu Surface-Kernel Prototype vs f32 Bit-Check** (MVP GPU path + precision study).
- **What**: Implement the WGSL compute kernel for surface fire only (no crown/spot chaining yet).
  - Create `crates/behave-gpu/` crate (new) with wgpu boilerplate.
  - Port `surface_kernel()` to WGSL (f32 throughout); dispatch over a small batch (e.g., 256 cells).
  - Run the same golden scenario (GS124) on both CPU (f64, f32-cast) and GPU (f32 native).
  - For each output (ROS, FLI, flame length, etc.), compute relative error: `|(gpu - cpu_f32) / cpu_f32|`.
  - Tabulate: which outputs stay <= 1%, 2%, 5%, and which need mitigation.
- **Repo path**: 
  - `crates/behave-gpu/Cargo.toml` (depends: wgpu, bytemuck, pollster; dev-dep: criterion).
  - `crates/behave-gpu/src/lib.rs` (wgpu boilerplate, buffer setup).
  - `crates/behave-gpu/src/compute/surface.wgsl` (WGSL kernel).
  - `crates/behave-gpu/tests/f32_parity.rs` (batch against CPU f32 cast for 171 parity check points).
- **Gate**: All 171 parity checks run GPU-side; tabulated error budget published (e.g., "ROS: 0.8% median error", "scorch height: 3.2% median, mitigate via compensated summation").

### Falsifiable "Result When..." Milestone

**Milestone**: GPU surface kernel (f32, wgpu, single dispatch) processes **N-million-cell landscape** at **>= 10^8 surface evaluations per second** on a **consumer GPU** (e.g., Apple M1 GPU, NVIDIA RTX 4060, AMD Radeon 7700) **with all outputs within the Phase-2 f32 error budget** (defined per-output during Step 3, e.g., "ROS, FLI, flame length ≤ 2%, scorch ≤ 5%").

- **How to test**: 
  - Generate a randomized 1M-cell landscape (fuel models uniformly FM124, then mixed).
  - Time the full GPU dispatch (setup + kernel + readback).
  - Assert: `cells / elapsed_sec >= 1e8` on a specified GPU model.
  - Compare outputs to CPU f64 reference; assert all divergences within declared budget.
- **Why this is falsifiable**: GPU throughput and output error are directly measurable; the threshold (10^8 cells/sec) is a concrete number tied to the criterion baseline (Step 2) and the consumer-GPU constraint is public.
- **Why this matters**: FlamMap's published landscape throughput is O(10^6–10^7) cells/sec. 10× speedup at 10^8 is a meaningful advance for real-world use (e.g., overnight 1M-cell burn probability simulation → < 10 min).
- **When to call it done**: GitHub PR to `behave-gpu` shows the criterion report + GPU benchmark side-by-side, with error budget table and landscape throughput measurement on two GPU models. Accepted into `main` branch.

---

## (2) Canonical Open-Source Rust Fire-Behavior Stack (crates.io)

**Candidate status**: Currently private workspace on GitHub; is a natural follow-on to parity validation + GPU work.

### Why Current SOTA Falls Short

- **Behave C++** lives at `firelab/behave` (public domain) but has no Rust consumer path; projects that want to use Behave today must:
  - Bind C++ via WASM/Emscripten (failed attempt on branch `rj-idl-bindings`, abandoned 2025).
  - Link C++ directly (requires CMake, C++14 toolchain, conflicts with Rust vendoring).
- **No semver policy, no changelog, no tags** in the C++ repo (REVIEW.org line 400 archaeology).
- **Rust ecosystem needs a "batteries-included" fire-sim crate** (like `ndarray` for numerics, `rayon` for parallelism). Behave's 9-crate workspace is ideal but is not discoverable.

### This Project's Specific Asset

1. **Parity-validated, parallelization-ready architecture**: The 9-crate DAG (firelab-base → behave-surface → behave-crown → behave-spot/mortality/contain/weather/ignite/run) is clean, testable, and designed for Phase 0–3 refactoring without API breakage.
2. **Production-quality codebase** at 19k LOC with 279 unit tests + 171 parity checks; zero unsafe, zero panics in library code.
3. **Clear versioning path**: semver 0.1.0 → 0.2.0 (Phase 0 public kernels) → 0.3.0 (Phase 1 rayon batch) → 1.0 (GPU-ready stable API).

### First Three Concrete Steps (In This Repo)

**Step 1 — Add License Metadata to All Crates**.
- **What**: Each of the 9 crates is currently missing `license` and `repository` fields in `Cargo.toml`.
  - Root `Cargo.toml` (or `crates/*/Cargo.toml`) and package section must have:
    ```toml
    [package]
    license = "Unlicense"  # USDA public domain = Unlicense SPDX
    repository = "https://github.com/firelab/behave"
    ```
  - Create `LICENSE` file at repo root (Unlicense text, 40 lines).
  - Update `README.md` with "Public Domain (Unlicense)" notice.
- **Repo path**: All `crates/*/Cargo.toml` + root `LICENSE` file.
- **Gate**: `cargo publish --dry-run` succeeds for all 9 crates (lints for metadata, does not actually upload).

**Step 2 — Rust-Specific CI (GitHub Actions)**.
- **What**: Add `.github/workflows/rust.yml` that runs on every PR to `rj-rust-port` (and `master` once merged):
  - `cargo build --workspace --all-targets`.
  - `cargo test --workspace` (currently 0/279 due to build config issue — fix and verify).
  - `cargo clippy --workspace -- -D warnings`.
  - Publish coverage report (e.g., `tarpaulin` or `llvm-cov`).
- **Repo path**: `.github/workflows/rust.yml` (new file).
- **Gate**: All 279 unit tests + 171 parity checks pass; clippy passes with zero warnings.
- **Evidence**: Green checkmark on CI dashboard; build log downloadable.

**Step 3 — Versioning & Changelog Policy**.
- **What**: Define semantic versioning for the Rust workspace:
  - 0.1.0 (current parity port): API identical to C++, kernel functions private.
  - 0.2.0 (Phase 0 completed): public `x_kernel()` functions, `Copy` inputs, `&'static` tables.
  - 0.3.0 (Phase 1 completed): `behave-batch` crate with rayon.
  - 1.0.0 (Phase 3 completed): GPU-ready stable API, MSRV defined.
  - Document in `VERSIONING.org` or `CONTRIBUTING.md`: breaking changes require major-version bump; minor = new features; patch = bugfixes.
- **Repo path**: Root `VERSIONING.org` (org-mode) + update `CONTRIBUTING.md`.
- **Gate**: Policy document merged; next release tagged `v0.1.0` and pushed to GitHub Releases.

### Falsifiable "Result When..." Milestone

**Milestone**: Behave crates are published to **crates.io** and discoverable via `cargo search behave` with:
- All 9 crates passing `cargo publish --dry-run` (metadata + license checks).
- **>= 1 external consumer** (person/org not firelab) successfully runs `cargo add behave-run` and integrates into their project, with evidence in an open GitHub issue or discussion (e.g., "I used behave-run to build X").
- **Rust CI passing** on `master` branch (all tests green, clippy zero warnings).
- **Semantic versioning policy documented** and applied to the first two releases (0.1.0, 0.2.0 after Phase 0 complete).

---

## (3) In-Browser Parity (WASM32 + WebGPU) — Redemption of the Failed Emscripten Attempt

**Candidate status**: "The redemption arc" — the failed `rj-idl-bindings` branch (emscripten/WebIDL, abandoned 2025) showed that direct C++ FFI to WASM is costly and unmaintainable. A pure-Rust approach via wasm32-wasi or browser targets is viable *after* Phase 0 (pure kernels).

### Why Current SOTA Falls Short

- **BehavePlus web app** currently runs C++ via an old Emscripten build; maintenance is manual, updates are infrequent, and bundle size is large (~10 MB gzipped WASM).
- **No published fire-sim WASM library** with parity guarantees. FlamMap is desktop-only.
- **Previous attempt** (`rj-idl-bindings`, 2024) tried WebIDL + Emscripten; took 3 months, produced bindings that diverged from the Rust port, and was abandoned. Root cause: maintaining two code paths (C++ + Rust + Emscripten glue) exceeded team capacity.

### This Project's Specific Asset

1. **Pure Rust + no C++ FFI**: Once Phase 0 kernels exist, a WASM build is just `wasm32-wasi` target + `wasm-bindgen` for browser JS interop. No Emscripten, no C++ build inside WASM.
2. **Parity suite is portable**: The 171 checks in `crates/behave-run/tests/parity.rs` compile to WASM (all integer/f64 math, no OS calls). WASM output bit-equals CPU output = "parity-by-construction."
3. **Small payload**: Rust WASM is 2–5 MB; the GPU work (Phase 3) can optionally fall back to CPU-side WASM for platforms without WebGPU.

### First Three Concrete Steps (In This Repo)

**Step 1 — WASM32-WASI Target & Parity Suite Port**.
- **What**: Add wasm32-wasi as a first-class CI target:
  - `rustup target add wasm32-wasi`.
  - Update `crates/behave-run/Cargo.toml` to include `[target.wasm32-wasi]` profile (opt-level=3 for perf).
  - Create `crates/behave-run/tests/wasm_parity.rs` that re-runs a subset of the 171 parity checks in WASM context (or prove that the existing parity tests compile + run under wasm32-wasi).
- **Repo path**: `.github/workflows/rust.yml` + new step `cargo test --target wasm32-wasi -p behave-run`.
- **Gate**: At least 50+ parity checks pass under wasm32-wasi; build artifact < 5 MB.

**Step 2 — Browser Harness (JS Interop Layer)**.
- **What**: Create a thin WASM-to-JS bridge:
  - New crate `crates/behave-wasm/` with `[lib] crate-type = ["cdylib"]`.
  - Export a single function `run_surface_batch(inputs_json: &str) -> String` (serde JSON serialization).
  - Use `wasm-bindgen` for JS method binding.
  - Build: `wasm-pack build --target web crates/behave-wasm`.
  - Write `crates/behave-wasm/www/index.html` (demo page) that:
    - Loads the WASM module.
    - Submits a fuel model (FM124) + wind/slope inputs via form.
    - Calls the WASM function.
    - Displays outputs (ROS, FLI, flame length).
    - Compares to C++ reference value from REVIEW.org golden data.
- **Repo path**: 
  - `crates/behave-wasm/Cargo.toml` (cdylib, wasm-bindgen, serde_json deps).
  - `crates/behave-wasm/src/lib.rs` (one public function).
  - `crates/behave-wasm/www/` (HTML/JS demo).
- **Gate**: `wasm-pack build --target web` succeeds; demo page loads in Chromium/Firefox; form submission returns correct output.

**Step 3 — Parity Assertion in Browser** (closes the loop).
- **What**: Extend the browser harness to run the parity suite *inside* the JS test page:
  - Embed the 171 golden test vectors (fuel model, inputs, expected outputs) as JSON.
  - In JS, iterate over vectors, call WASM, assert outputs match (tolerance 1e-6).
  - Display: "✓ 171/171 parity checks passed in WASM" in the UI.
  - Wire into CI: a headless browser test (e.g., `wasm-pack test --headless --firefox`) that fails the build if any assertion fails.
- **Repo path**: 
  - `crates/behave-wasm/www/parity_test.js` (test harness).
  - `crates/behave-wasm/tests/wasm.rs` (headless browser runner via `wasm-pack test`).
- **Gate**: `cargo test -p behave-wasm --target wasm32-wasi` passes; CI green badge on the `rj-rust-port` PR.

### Falsifiable "Result When..." Milestone

**Milestone**: WASM + browser harness passes **all 171 parity checks** in a **headless browser environment** (Firefox/Chromium via wasm-pack), with:
- **Output binary identical to CPU Rust** (bit-exact f64, no rounding surprises).
- **Build artifact < 5 MB** (gzipped).
- **Demo page publicly viewable** (deployed to GitHub Pages or similar) showing a working fire behavior calculator fed by the WASM module.
- **Zero undefined behavior / panics** in WASM runtime (all assertions pass, no memory violations).

---

## (4) Randomized Differential-Testing Corpus vs C++ (10^5–10^6 Input Tuples)

**Candidate status**: Foundational QA for all downstream work. High confidence this catches remaining bugs.

### Why Current SOTA Falls Short

- **Parity suite covers 171 hand-curated test cases** from the C++ test file (`testBehave.cpp`). This is a small fraction of the input space.
- **No random mutation testing** between Rust and C++ — the two codebases can silently diverge on edge cases, especially in chaparral/palmetto/aspen special models or extreme moisture/wind values.
- **FOFEM (mortality comparison, `make test_mortality`) runs ~40 scenarios**; parity with FlamMap-scale randomization (millions of cells) is unverified.

### This Project's Specific Asset

1. **Both C++ and Rust codebases are deterministic and pure-functional** — same inputs → same outputs (modulo floating-point rounding).
2. **Parity suite already mirrors C++ test sequence** → infrastructure to replay both exists.
3. **C++ testBehave executable can be mined for input/output pairs** via instrumentation or (easier) by adding a CSV dump mode to the C++ client.

### First Three Concrete Steps (In This Repo)

**Step 1 — C++ CSV Dump Utility** (mine reference outputs).
- **What**: Create a new C++ CLI tool that generates a CSV of randomized inputs + reference outputs:
  - `src/behave/examples/dump_csv.cpp` (new file, ~150 lines).
  - Loop: for N = 10^5 or 10^6 iterations, sample random (fuel_model, moisture_class, wind_speed, slope, aspect, canopy, …) tuples.
  - Call the surface/crown/spot/mortality calculation APIs.
  - Write to CSV: `fuel_model, 1hr_moisture, 10hr_moisture, …, ros, fli, flame_length, …`.
  - Use the existing C++ test macro to ensure consistency with `testBehave.cpp` methods.
- **Repo path**: `src/behave/examples/dump_csv.cpp` + update root `CMakeLists.txt` to build it as `example_dump_csv`.
- **Gate**: Build succeeds; `./build/example_dump_csv 100000` produces `parity_corpus.csv` with 100k rows in < 10 sec.

**Step 2 — Rust Corpus Verifier**.
- **What**: New integration test that loads the CSV and runs the same inputs through Rust:
  - `crates/behave-run/tests/corpus_verify.rs` (new file).
  - Read `parity_corpus.csv` (assume it's checked into the repo or generated at test time).
  - For each row, construct Rust `SurfaceInputs`, run `surface_kernel()`, extract outputs.
  - Assert: Rust output matches C++ within tolerance (1e-6 relative or 1e-12 absolute, as appropriate).
  - Histogram divergences: bucketed by fuel model, moisture class, etc. — identify which input regions are divergent.
- **Repo path**: `crates/behave-run/tests/corpus_verify.rs` (new file); data/corpus_100k.csv (committed, ~50 MB compressed).
- **Gate**: Test passes with zero failures; divergence histogram is clean (e.g., "max error 2.3e-7 on FM123, nominal 5e-8").

**Step 3 — CI Integration & Corpus Growth**.
- **What**: Add a CI step that regenerates the corpus quarterly and runs corpus_verify in CI:
  - Extend `.github/workflows/rust.yml`: on quarterly schedule (or manual trigger), build the C++ dump tool, regenerate corpus (10^6 rows), commit to a `corpus/` branch, and run `cargo test -p behave-run corpus_verify` against it.
  - Optional: add a `corpus_divergence_report.md` artifact that summarizes any systematic divergences (e.g., "6 out of 1M inputs diverge in scorch calculation; all within 1e-5 relative error").
- **Repo path**: `.github/workflows/corpus_check.yml` (scheduled) + `corpus/README.org` (documents corpus generation process).
- **Gate**: CI completes successfully; corpus commit is pushed; corpus_divergence_report.md is generated (even if report is "zero divergences").

### Falsifiable "Result When..." Milestone

**Milestone**: A randomized corpus of **>= 10^5 input tuples** is generated from the C++ reference, verified to run identically in Rust, and checked into CI with **zero unexplained divergences**. Any divergence discovered must be:
- **Documented** with a root cause (e.g., "fuel model TL5 has a C++ quirk, Rust matches it intentionally for parity").
- **Either fixed upstream** (if a C++ bug is found and accepted by firelab) or **explicitly accepted and annotated** in the codebase.
- **Included in the tolerance budget** for GPU work (Phase 3).

---

## (5) Upstream Contribution: Found-C++-Bug List Filed (and Accepted)

**Candidate status**: Not a research frontier per se, but a path-clearing deliverable. The `behave-change-control` skill catalogs C++ quirks and bugs.

### Why Current SOTA Falls Short

- **C++ Behave has dormant infrastructure** (last active 2024, now 2026; no automated changelog, no issue templates).
- **Known bugs live as comments in Rust code** (see REVIEW.org divergence ledger) but are not filed upstream.
- **FlamMap/ElmFire users deserve corrected C++ sources** (at minimum, a published errata list).

### This Project's Specific Asset

1. **Rust port found bugs by accident** during parity testing — e.g., ignite.cpp field-write shadowing, chaparral index errors, slopeTool unit-handling quirk.
2. **All bugs are documented with code references and test cases** in REVIEW.org.
3. **Upstream (`firelab/behave`) is responsive to issues** (see Git archaeology: BHP1-#### PRs are merged regularly).

### First Three Concrete Steps (In This Repo)

**Step 1 — Bug Triage & Prioritization**.
- **What**: Audit `behave-change-control` skill and REVIEW.org:
  - Create `UPSTREAM_ISSUES.org` in the repo root.
  - For each divergence/quirk/bug, decide: (a) C++ bug requiring upstream fix, (b) intentional Rust correction (parity assertion diverges), (c) ambiguous (document and ask upstream).
  - Include: bug description, C++ file:line, reproduction steps, suggested fix, and a link to the Rust test that validates the correction.
- **Repo path**: `UPSTREAM_ISSUES.org` (new file, ~200 lines).
- **Gate**: Document is complete and reviewed by the project owner.

**Step 2 — File GitHub Issues Upstream**.
- **What**: For each "C++ bug" category, file an issue on https://github.com/firelab/behave:
  - Issue title: "Bug: ignite.cpp setMoistureHundredHour writes to wrong field" (or similar, specific).
  - Body: Include C++ code snippet, expected behavior, reproduction test (copy from Rust parity suite), and a link to the Rust port for reference.
  - Label: `type:bug` (if firelab uses labels).
  - Reference: link back to `UPSTREAM_ISSUES.org` in the SIG fork.
- **Repo path**: Issue tracker on firelab/behave (external).
- **Gate**: All category-A bugs filed; issues are open and acknowledged by firelab maintainers.

**Step 3 — Track Acceptance & Merge**.
- **What**: Maintain `UPSTREAM_ISSUES.org` with resolution status:
  - For each upstream issue, add a "Status" field: Proposed → Accepted → Merged (with upstream commit hash) → Closed.
  - Once upstream is fixed, update the Rust code comment: "Fixed upstream in commit abc1234; Rust now aligns with corrected C++ behavior."
  - If upstream rejects a bug report, document the rationale (e.g., "intentional quirk for backward compatibility").
- **Repo path**: `UPSTREAM_ISSUES.org` updated continuously.
- **Gate**: At least 1 upstream PR merged; status tracking is current.

### Falsifiable "Result When..." Milestone

**Milestone**: **>= 1 upstream GitHub issue is filed, reviewed, and accepted** (merged into `firelab/behave` with a commit reference). Examples of acceptance:
- "ignite.cpp field-write bug confirmed; fix merged in PR #NNN."
- "C++ slopeTool unit quirk is intentional for BehavePlus compatibility; documented in PR #NNN."
- "chaparral index error acknowledged; fix merged."

---

## (6) Uncertainty/Ensemble Runs at GPU Scale (Candidate — Speculative)

**Candidate status**: Post-Phase 3 speculative research; high risk/high reward. Only pursue after GPU baseline (frontier #1) is proven.

### Why Current SOTA Falls Short

- **FlamMap and ElmFire** do single deterministic runs (one wind scenario, one fuel moisture, one spread model).
- **Uncertainty quantification** in fire behavior (e.g., 10th/50th/90th percentile spread rates over wind/moisture distributions) is hand-rolled by external code (e.g., CFFDRS ensemble wrappers).
- **GPU batch evaluation** makes ensemble sampling *cheap* — a 1000-member ensemble is no more expensive than 1000 cells, yet these tools do not ship ensemble APIs.

### This Project's Specific Asset

1. **Post-Phase 3, the GPU kernel takes embarrassingly parallel inputs** → ensemble variance sampling is just another batch dispatch.
2. **Pure inputs (Copy + static tables)** make it trivial to run 10^5 independent scenarios with perturbed moisture/wind over probability distributions (e.g., sample from beta or log-normal).
3. **Output aggregation** (percentile maps, Bayesian posterior) is a separate CPU-side map-reduce (not a GPU bottleneck).

### First Three Concrete Steps (In This Repo)

**Step 1 — Ensemble Input Sampling Module**.
- **What**: New crate `crates/behave-ensemble/` with distribution samplers:
  - `EnsembleConfig { n_members: usize, moisture_distribution: Distribution, wind_distribution: Distribution, … }`.
  - Functions: `sample_ensemble(config) -> Vec<SurfaceInputs>` (generate N randomized input tuples from distributions).
  - Use `rand` crate for sampling (beta, normal, lognormal, uniform, etc.).
  - Serialize to JSON for GPU batch ingestion.
- **Repo path**: `crates/behave-ensemble/Cargo.toml` + `src/lib.rs`.
- **Gate**: Compiles; produces valid `SurfaceInputs` tuples matching distributions (checked via histogram of samples).

**Step 2 — Ensemble GPU Dispatch**.
- **What**: Extend `crates/behave-gpu/` to accept ensemble batches:
  - Function `run_ensemble(config: EnsembleConfig, device: &Device) -> EnsembleResults`.
  - Allocate GPU buffers for N members; dispatch surface kernel over all N in one pass.
  - Collect outputs into `Vec<SurfaceOutputs>`.
  - Time the dispatch: assert overhead is O(1) relative to N.
- **Repo path**: `crates/behave-gpu/src/ensemble.rs` (new module).
- **Gate**: Ensemble of 10,000 members runs at >= 80% of single-member throughput (i.e., GPU overhead is < 20%).

**Step 3 — Percentile Map & Benchmark**.
- **What**: New integration test that compares ensemble runtime to FlamMap-style baseline:
  - Benchmark: generate a 1M-cell landscape (mixed fuel models).
  - Ensemble: run 100 moisture/wind samples over the landscape (100M total evaluations).
  - Time GPU ensemble: target < 10 sec (on consumer GPU).
  - Time single FlamMap-equivalent run: estimate as (single-cell time) × 1M × 100 ≈ 100–1000 sec (naive serial, or FlamMap's actual hardware).
  - Compute percentile maps (10th, 50th, 90th) of ROS across ensemble.
  - Assert: GPU ensemble is >= 10× faster than single FlamMap run.
- **Repo path**: `crates/behave-gpu/benches/ensemble_vs_flammap.rs` + integration test.
- **Gate**: Benchmark completes; speedup table is published (e.g., "GPU 10,000-member ensemble: 6.3 sec; estimated single FlamMap run: 150 sec; speedup 24×").

### Falsifiable "Result When..." Milestone

**Milestone**: GPU ensemble module runs a **100-member uncertainty quantification** over a **1M-cell landscape** (100M total surface evaluations) **in < 60 seconds** on a consumer GPU, with **percentile maps (10th/50th/90th) of spread rate, flame length, fireline intensity** output as a GeoTIFF raster stack. Comparison to FlamMap:
- **FlamMap single run**: ~100–500 sec per landscape (reported in literature / measured on reference hardware).
- **GPU ensemble**: 100× scenarios (equivalent to 100 FlamMap runs) in < 60 sec.
- **Evidence**: Benchmark report with GPU device name, timing breakdown, and output raster sample.

---

## Provenance and Maintenance

This skill synthesizes research-frontier definitions from:
1. **REVIEW.org** (2026-07-06): parallelization roadmap, readiness matrix, precision strategy.
2. **RUST_PORT.org** (2026-03-05): original porting roadmap and reference test cases.
3. **Git archaeology** (750 commits since 2016, dormant 2016–2024, active 2025–2026): recurring failure patterns, upstream change control, consumer needs.
4. **Parity suite** (crates/behave-run/tests/parity.rs, 171 checks as of 2026-07-06): evidence of numerical fidelity.
5. **Workspace inventory** (crates/, 9 crates, 0 license metadata, no Rust CI as of 2026-07-06).

To re-verify this skill's factual claims, run:

| Fact Class                  | Re-Verification Command                                                            | Expected Output |
|-----------------------------|------------------------------------------------------------------------------------|---------|
| Parity check count          | `grep -c "assert" crates/behave-run/tests/parity.rs`                               | >= 171  |
| Crate count                 | `ls -1d crates/*/` \| wc -l`                                                        | 9       |
| Rust CI coverage            | `grep -q "cargo test" .github/workflows/ci.yml` && echo "covered" \|\| echo "none"` | none    |
| License metadata present    | `grep -r "license" crates/*/Cargo.toml` \| wc -l`                                   | 0 (gap) |
| Emscripten branch history   | `git log --all --oneline \| grep -i emscripten` \| wc -l`                           | >= 2    |
| Phase 0 docs in REVIEW      | `grep -c "Phase 0 — Pure-kernel" REVIEW.org`                                       | 1       |
| GPU milestone in REVIEW     | `grep "10\^8.*cells" REVIEW.org` \| wc -l`                                         | >= 1    |

**Last verified**: 2026-07-06 (against `rj-rust-port` branch, HEAD f11cbc5).
**Maintainer**: SIG. Update this skill when REVIEW.org is updated or Phase 0/1/2 are completed.

