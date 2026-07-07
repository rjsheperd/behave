---
name: behave-parallelization-campaign
description: Executable decision-gated campaign for parallel/GPU execution of fire kernels. Five-phase roadmap (pure-kernel refactor, rayon batch, f32 study, SoA+SIMD, WebGPU) with gates, branch instructions, and change-control promotion protocol.
---

# Behave Parallelization Campaign

## Objective

Execute the hardest live problem (owner-confirmed 2026-07-06): landscape-scale batch evaluation of fire behavior kernels via rayon parallelization, SIMD vectorization, and GPU (WebGPU) dispatch. This campaign transforms the current mutable-API library into a pure-kernel design compatible with all backends while maintaining bit-identical fidelity to the f64 reference implementation.

**When NOT to use this skill**: for Rust/C++ architecture facts, use `behave-architecture-contract`; for per-phase validation gates, use `behave-change-control`; for evidence standards and test structure, use `behave-validation-and-qa`; for domain fire-behavior knowledge, use `fire-behavior-reference`.

## Success Criteria

- All five phases land via `behave-change-control` gates (golden parity suite 171/171 runtime checks passing at each phase).
- Scaling measurements (throughput cells/sec, speedup vs. serial) reported at every gate.
- f32 precision study completed with written error budget per output before GPU work begins.
- GPU implementation (WebGPU/wgpu) demonstrates falsifiable milestone: competitive cells/sec vs. CPU baseline.
- Zero interior mutability, zero panics in library code, no new heap fields in input structs.

## Target Workload

Raster-scale fire behavior: millions of independent cells, each cell = (fuel model, moistures ×7, wind speed/direction, slope/aspect, canopy parameters). One calculation per cell; results are independent. Embarrassingly parallel.

Kernel statistics:
- **Flops per cell**: ~1,200 (transcendental-heavy).
- **Compute-to-bandwidth ratio**: ~5000:1 (strongly compute-bound).
- **Loop structure**: only fixed-bound loops in surface (5 size classes × 2 life states), spot (6-iteration refinement), crown (composition); **not** contain (variable-length simulation).
- **Branching**: main divergence is fuel-model dispatch (standard vs. chaparral/palmetto/aspen vs. two-fuel) at facade level; cheap rejoining.

---

## Phase 0: Pure-Kernel Refactor (Prerequisite for Everything)

**Effort**: largest single chunk; mechanical.

**Goal**: split each module into `XInputs` struct (Copy, Pod), pure kernel function, and static tables, keeping the existing mutable wrapper API green.

### Phase 0 Deliverables

1. **Convert `SurfaceInputs` to Copy+Pod**:
   - Replace `current_moisture_scenario_name: String` (line 52 of `behave-surface/src/inputs.rs`) with `current_moisture_scenario_index: u8` or an enum.
   - Verify all moisture scenarios are covered by the new enum; map the C++ string names to indices (see `behave-surface/src/moisture.rs:MoistureScenarios`).
   - Size-check: target <1 KB total struct size (currently ~400 B of f64 + metadata).

2. **Extract pure kernel functions** (one per module):
   - `pub fn surface_kernel(inputs: &SurfaceInputs, tables: &Tables) -> SurfaceOutputs`
   - `pub fn crown_kernel(inputs: &CrownInputs, surface_out: &SurfaceOutputs, tables: &Tables) -> CrownOutputs`
   - `pub fn spot_kernel(inputs: &SpotInputs, tables: &Tables) -> SpotOutputs`
   - `pub fn mortality_kernel(inputs: &MortalityInputs, tables: &Tables) -> MortalityOutputs`
   - Similar for ignite, contain, weather modules.
   - Move internal scratch (e.g., `FuelbedIntermediates`) to stack-local stack frames inside the kernel.

3. **Make tables static/shared**:
   - `behave-surface/src/fuel_models.rs`: convert `FuelModels` table from heap-cloned to `&'static` via `LazyLock<FuelModels>`.
   - `behave-mortality/src/species.rs`: convert species master table to `&'static`.
   - Remove the stub duplicate `behave-surface/src/fire_size.rs` (see findings table in REVIEW.org P2).
   - Update all kernel calls to pass `&Tables` (containing references to static tables).

4. **Preserve legacy mutable API as thin wrappers**:
   ```rust
   impl Surface {
       pub fn do_surface_run(&mut self) {
           let out = surface_kernel(&self.inputs, &TABLES);
           self.outputs = out;  // store for getters
       }
   }
   ```
   - All setters/getters remain unchanged.
   - All 279 unit tests must pass without modification.

5. **Phase 0 Milestone**: land the **parity suite first** (P0 in REVIEW.org findings).
   - File: `crates/behave-run/tests/parity.rs` (~1400 loc, 171 checks executed at runtime (142 static call sites; four sites loop — two-fuel coverages, speed units, VPD, slope distances), mirrors `testBehave.cpp`).
   - Command: `cargo test -p behave-run --test parity 2>&1 | tee parity-baseline.log`
   - Expected output: `test result: ok. 1 passed` (171 internal checks passing; see `parity: 171 checks executed` with `-- --nocapture`).

### Phase 0 Gate Criteria

**Run these in order:**

```bash
# 1. Build and test baseline (from repo root)
cargo build --workspace 2>&1 | tee phase0-build.log
cargo test --workspace 2>&1 | tee phase0-tests.log

# Extract summary
grep "test result:" phase0-tests.log | awk '{print $NF}' | sort | uniq -c
```

**Expected observations**:
- All 9 crates compile cleanly (no warnings promoted to errors).
- Unit test counts per crate (firelab-base 43, behave-surface 132, behave-crown 20, behave-spot 12, behave-ignite 7, behave-contain 15, behave-mortality 38, behave-weather 13, behave-run 2 = 282 total; parity 1 additional = 283 total tests passing).
- Zero unsafe blocks, zero panics in library code (verify with `cargo build --lib 2>&1 | grep -i panic`).

```bash
# 2. Parity suite baseline
cargo test -p behave-run --test parity 2>&1 | tee phase0-parity.log

# Extract assertion count
grep "parity checks passed" phase0-parity.log || echo "Not yet instrumented; count .check() calls"
grep "\.check(" crates/behave-run/tests/parity.rs | wc -l
```

**Expected observations**:
- Parity test passes (exit code 0).
- Runtime checks are 171 (142 static call sites; loops expand them). Verify: `cargo test -p behave-run --test parity -- --nocapture 2>&1 | grep parity:`
- No ignored tests.

```bash
# 3. Grep for forbidden constructs (will fail if any found)
cargo build --lib 2>&1
grep -r "interior_mutability\|Rc<\|RefCell\|Mutex\|lazy_static\|static mut" \
  crates/*/src --include="*.rs" | grep -v "test\|comment\|PRESERVED" || echo "No forbidden constructs found"
```

**Expected observations**:
- Either no matches or matches only in test code (OK) or commented-out code (OK).

```bash
# 4. Input struct Copy check (Phase 0 exit gate)
cargo build --lib 2>&1
for crate in firelab-base behave-surface behave-crown behave-spot behave-ignite behave-mortality behave-contain behave-weather; do
  echo "=== $crate ===" 
  grep -A 3 "pub struct.*Inputs" crates/$crate/src/*.rs | grep -c "String" || echo "0"
done
```

**Expected observations (after refactor)**:
- Zero String fields in any Inputs struct (all String fields replaced with enums or indices).

### Phase 0 Gate Resolution

**If all observations match**: `PASS` → proceed to Phase 1.

**If tests fail**:
1. Run `cargo test --lib 2>&1 | grep FAILED -A 5` to see which test(s) broke.
2. Identify the breaking change (did we drop a setter? change a constant? mis-index an enum?).
3. Fix the kernel or wrapper, re-run parity suite.
4. If parity diverges but unit tests pass: **escalate to `behave-change-control`** (see promotion protocol below).

**If struct still contains String fields after refactor**:
- Identify which field and why (e.g., custom fuel name not yet mapped to enum).
- Add the missing enum variant or accept the f64-count ceiling (seek design review).

---

## Phase 1: Rayon Batch API (Immediate Win)

**Effort**: ~100 loc; ~2–3 days to benchmark.

**Goal**: ship a parallel facade that runs cells independently; validate bit-identical results.

### Phase 1 Deliverables

1. **New `behave-batch` crate**:
   ```toml
   # crates/behave-batch/Cargo.toml
   [package]
   name = "behave-batch"
   version = "0.1.0"
   edition = "2021"
   
   [dependencies]
   firelab-base.workspace = true
   behave-surface.workspace = true
   rayon = "1.7"
   ```

2. **Batch facade**:
   ```rust
   // crates/behave-batch/src/lib.rs
   use rayon::prelude::*;
   use behave_surface::{SurfaceInputs, SurfaceOutputs, surface_kernel, Tables};
   
   pub fn run_surface_batch(
       cells: &[SurfaceInputs],
       tables: &Tables,
   ) -> Vec<SurfaceOutputs> {
       cells.par_iter()
           .map(|inputs| surface_kernel(inputs, tables))
           .collect()
   }
   
   // Optional: serial fallback for debugging
   pub fn run_surface_batch_serial(
       cells: &[SurfaceInputs],
       tables: &Tables,
   ) -> Vec<SurfaceOutputs> {
       cells.iter()
           .map(|inputs| surface_kernel(inputs, tables))
           .collect()
   }
   ```

3. **Benchmark harness** (use `criterion`):
   - Create `crates/behave-batch/benches/batch_scaling.rs`.
   - Generate 1000, 10000, 100000 random cells (same distribution across phases for comparability).
   - Measure: serial baseline, rayon 1–N threads (N = physical cores).
   - Record throughput (cells/sec) to CSV for plotting.

### Phase 1 Gate Criteria

**Run these in order:**

```bash
# 1. Build and run parity suite (must remain green)
# (run from repo root)
cargo test -p behave-run --test parity 2>&1 | tee phase1-parity.log
```

**Expected observations**:
- Parity test passes: exit code 0.
- No new divergences.

```bash
# 2. Add behave-batch to workspace, build
cargo build -p behave-batch 2>&1 | tee phase1-build.log
```

**Expected observations**:
- Clean build (no warnings).
- Check for warnings about `unsafe` or unsoundness: `grep -i "unsafe\|unsound" phase1-build.log || echo "clean"`

```bash
# 3. Run bit-identity assertion test
# (Add to crates/behave-batch/tests/identity.rs)
cargo test -p behave-batch --test identity 2>&1 | tee phase1-identity.log
```

**Expected test**:
```rust
#[test]
fn serial_and_parallel_are_identical() {
    let cells = vec![/* 100 random cells */];
    let tables = Tables::default();
    
    let serial_out = run_surface_batch_serial(&cells, &tables);
    let parallel_out = run_surface_batch(&cells, &tables);
    
    for (i, (s, p)) in serial_out.iter().zip(parallel_out.iter()).enumerate() {
        assert_eq!(s.spread_rate, p.spread_rate, "cell {i} diverged");
        // check all fields bit-equal (or use a DeepPartialEq macro)
    }
}
```

**Expected observations**:
- Test passes (bit-identical results, not just within tolerance).

```bash
# 4. Criterion benchmark (if already in behave-diagnostics-and-tooling)
# Otherwise, create a minimal bench:
cargo build --release -p behave-batch 2>&1
time cargo bench -p behave-batch --bench batch_scaling 2>&1 | tee phase1-bench.log
```

**Expected output** (sample):
```
Cells: 1000
  Serial:   12.3 ms (81,300 cells/sec)
  Rayon-2:   6.5 ms (153,800 cells/sec), speedup 1.9x
  Rayon-4:   3.4 ms (294,100 cells/sec), speedup 3.6x
  Rayon-8:   1.8 ms (555,500 cells/sec), speedup 6.8x
```

**Scaling rule of thumb**: near-linear (6–8× on 8 cores) for this compute-bound kernel with zero allocation per cell.

```bash
# 5. Check for regressions vs baseline
# Compare phase1-bench.log against phase0 (if phase 0 is parameterized);
# otherwise, this is the new baseline.
```

### Phase 1 Gate Resolution

**If all observations match**: `PASS` → create commit via `behave-change-control` gate.

**If bit-identity assertion fails**:
- Example: serial gives ROS=25.123, parallel gives ROS=25.124.
- Root cause: floating-point reordering (par_iter doesn't guarantee same order as fold).
- Fix: either (a) accept the tiny ULP differences as a phase 2 tolerance, or (b) ensure deterministic ordering by cell ID.
- Decision: if diff < 1e-10 relative, accept; otherwise re-examine fold strategy.

**If scaling is poor** (e.g., 2× on 8 cores):
- Suspect: table cloning or allocation per cell.
- Debug: run with `RAYON_NUM_THREADS=1` vs. `RAYON_NUM_THREADS=8` and profile with `perf stat` or `cargo flamegraph`.
- Common cause: if `Tables` is being cloned into the par_iter closure, move it outside.
- Fix and re-measure.

---

## Phase 2: f32 Sensitivity Study (Decision Gate for GPU Work)

**Effort**: ~5–10 days (benchmark suite + error analysis).

**Goal**: determine which outputs are f32-safe, which need f64 or mitigation.

### Phase 2 Deliverables

1. **Make kernels generic over float type**:
   - Add a trait `Float: Copy + std::ops::Add + … + exp() + sqrt()` (or use `num_traits`).
   - Instantiate both `f64` and `f32` versions of each kernel.
   - `f32` instances go into a new `behave-batch-f32` or feature-gated module.

   ```rust
   // behave-surface/src/lib.rs
   trait Float: Copy + /* ... */ {
       fn pow(self, n: Self) -> Self;
       fn exp(self) -> Self;
       // ... etc
   }
   
   pub fn surface_kernel_generic<T: Float>(inputs: &SurfaceInputs<T>, tables: &Tables) -> SurfaceOutputs<T> {
       // same math as before, but works for T = f32 or f64
   }
   ```

2. **Re-run parity corpus in f32**:
   - Feed the 171 golden test cases (from `parity.rs`) to the f32 kernel.
   - Store observed f32 values and compare against f64 baseline.
   - Compute per-output error distribution: mean, stddev, max relative error.

   **Risk spots to check first** (from REVIEW.org):
   - Scorch height (Byram): `fli^1.166667 / sqrt(fli + U^3)` — cancellation risk if FLI ≈ U³.
   - Wind factor φ_w: exponential terms can amplify small f32 errors into ROS divergence.
   - Reaction intensity α: weighted sum of 5 terms; accumulated rounding.

3. **Error budget**: document in a file `crates/behave-batch/ERROR_BUDGET_f32.md`:
   ```markdown
   # f32 Error Budget for GPU Deployment
   
   | Output            | max rel error | f32 safe? | Notes |
   |-------------------+---------------+-----------+-------|
   | Spread rate (ROS) | 0.8%          | YES       | 2% tolerance typical |
   | Flame length      | 1.2%          | YES       |       |
   | Reaction intensity| 0.5%          | YES       |       |
   | Wind factor φ_w   | 2.1%          | VERIFY    | exponential; downstream impact |
   | Scorch height     | 5.8%          | NO        | cancellation; use f64 or rearrange |
   | Crown ROS         | 1.5%          | YES       |       |
   | Spot distance     | 3.2%          | YES       | discretized distance output |
   ```

4. **Decision matrix**:
   - If output max error < 2% and 2% is acceptable downstream: **F32_SAFE**.
   - If output max error 2–10% and can be mitigated (e.g., algebraic rearrangement): **F32_RISKY_FIXABLE**.
   - If output max error > 10% or no fix found: **F32_NOT_SAFE** (keep f64 or CPU-side).

### Phase 2 Gate Criteria

**Run these in order:**

```bash
# 1. Build generic kernels (f32 + f64)
# (run from repo root)
cargo build --release --features "float-f32" 2>&1 | tee phase2-build.log
```

**Expected observations**:
- Compiles cleanly.
- No `unsafe` blocks introduced.

```bash
# 2. Run parity corpus in f32
# Add test: crates/behave-batch/tests/parity_f32.rs
# (Copy of parity.rs, but instantiate f32 kernels)
cargo test -p behave-batch --test parity_f32 2>&1 | tee phase2-parity-f32.log
```

**Expected observations**:
- Test runs (may fail tolerance checks, that's OK).
- Collect error deltas for each of 171 checks.

```bash
# 3. Analyze error distribution
python3 << 'EOF'
import json

# Parse phase2-parity-f32.log and compute stats
# (Example: grep lines like "scorch_height: f32=123.45 f64=123.46 err=0.08%")
errors = {}
with open('phase2-parity-f32.log') as f:
    for line in f:
        if 'err=' in line:
            # example: "spread_rate: err=0.8%"
            parts = line.split()
            output = parts[0].rstrip(':')
            err = float(parts[-1].rstrip('%')) / 100.0
            if output not in errors:
                errors[output] = []
            errors[output].append(err)

# Print summary
for output in sorted(errors.keys()):
    errs = errors[output]
    print(f"{output:20s}: max={max(errs)*100:6.2f}%, mean={sum(errs)/len(errs)*100:6.2f}%, n={len(errs)}")
EOF
```

**Expected observations** (sample):
```
spread_rate         : max=  0.85%, mean=  0.32%, n=171
flame_length        : max=  1.23%, mean=  0.41%, n=171
reaction_intensity  : max=  0.52%, mean=  0.18%, n=171
wind_factor_phi_w   : max=  2.18%, mean=  0.89%, n=171  ← risky
scorch_height       : max=  8.43%, mean=  4.21%, n=171  ← NOT SAFE
crown_ros           : max=  1.54%, mean=  0.63%, n=171
```

```bash
# 4. Write error budget (manual, based on above)
cat > crates/behave-batch/ERROR_BUDGET_f32.md << 'EOF'
# f32 Error Budget

[... table as shown above ...]

## Recommendations

- **GPU path**: use f32 for ROS, flame, reaction, crown (all <2%).
- **GPU mitigation needed**: wind factor (exponential amplification; rearrange formula or use compensated arithmetic).
- **F64 fallback**: scorch height (cancellation risk; CPU-side calculation acceptable).

## Sensitivity of downstream use

- FlamMap/ElmFire client tolerance: typically 2–5% for runtime efficiency.
- BehavePlus outputs: round to 2 significant figures (>>2% already).
- Decision: proceed with f32 GPU pipeline, f64 scorch height.
EOF
cat crates/behave-batch/ERROR_BUDGET_f32.md
```

### Phase 2 Gate Resolution

**If error budget written and decision made**: `PASS` → commit via `behave-change-control`.

**If scorch height or wind factor divergence is > 10%**:
- Option A: use f64 in GPU for those outputs (mixed precision kernel).
- Option B: rearrange formulas algebraically to reduce cancellation.
  - Scorch: instead of `fli^(7/6) / sqrt(fli + U^3)`, factor out common terms.
  - Wind: use `exp2()` or `logb()` to scale exponents before computing.
- Option C: accept and document the error; GPU results are not suitable for scorch height.
- **Escalate to `behave-change-control`** for decision.

**If convergence tests reveal compiler autovectorization is already helping**:
- Measure: release build `cargo build --release` with/without LTO.
- If f32 release binary is already 30–50% faster than f64, Phase 3 (explicit SIMD) may have diminishing returns.
- Proceed but revise Phase 3 target (focus on GPU, defer CPU SIMD).

---

## Phase 3: SoA + SIMD Vectorization (CPU Acceleration)

**Effort**: ~10–15 days.

**Goal**: 30–80× speedup over scalar single-thread via SIMD; compose with rayon for multi-threaded batches.

### Phase 3 Deliverables

1. **SoA batch layout**:
   ```rust
   // crates/behave-batch/src/soa.rs
   pub struct SurfaceBatch {
       // One Vec per input field
       fuel_model: Vec<i32>,
       wind_speed: Vec<f64>,
       slope: Vec<f64>,
       aspect: Vec<f64>,
       // ... etc for all ~30 input fields
       
       count: usize,  // number of cells
   }
   
   impl SurfaceBatch {
       pub fn from_cells(cells: &[SurfaceInputs]) -> Self { /* transpose AoS→SoA */ }
       pub fn into_cells(self) -> Vec<SurfaceOutputs> { /* transpose SoA→AoS */ }
   }
   ```

2. **Fuel-model grouping** (critical for efficiency):
   - Before vectorizing, bucket cells by fuel-model *kind* (standard FM1–89, chaparral, palmetto, aspen, two-fuel).
   - Why: each kind branches into a different code path; vectorizing without grouping requires expensive per-lane masking.
   - Alternative rejected: full lane masking = ~30% efficiency loss vs. grouping.

   ```rust
   pub fn group_cells_by_fuel_kind(batch: &SurfaceBatch) -> Vec<(FuelKind, Vec<usize>)> {
       // Return (fuel_kind, indices of cells with that kind)
       // Standard fuels dominate landscapes; chaparral/palmetto/aspen fall back to scalar.
   }
   ```

3. **Vectorized kernels** (using `std::simd` or `wide` crate):
   ```rust
   // Using std::simd (nightly) or wide::f64x8 (stable)
   use std::simd::f64x8;
   
   pub fn surface_kernel_simd_f64x8(
       batch: &SurfaceBatchView,  // SoA slice of 8 cells
       tables: &Tables,
   ) -> SurfaceBatchOutputView {
       // Vectorized arithmetic: wind_speeds = f64x8, slopes = f64x8, etc.
       // One iteration computes 8 cells in parallel
   }
   ```
   
   - **Transcendental functions**: `wide` crate provides vectorized `exp`, `pow`, `sqrt` with <1e-10 relative error.
   - Validate against parity suite: create assertion `assert!(rel_err(simd_out, scalar_out) <= 1e-10)` for each output.

4. **Composition with rayon**:
   ```rust
   pub fn run_surface_batch_simd(
       cells: &[SurfaceInputs],
       tables: &Tables,
   ) -> Vec<SurfaceOutputs> {
       let batch = SurfaceBatch::from_cells(cells);
       let grouped = group_cells_by_fuel_kind(&batch);
       
       grouped.par_iter()
           .flat_map(|(kind, indices)| {
               if *kind == FuelKind::Standard {
                   // vectorized path: chunks of 8
                   run_chunks_simd::<8>(&batch, indices, tables)
               } else {
                   // fallback scalar for special models
                   run_scalar(&batch, indices, tables)
               }
           })
           .collect()
   }
   ```

### Phase 3 Gate Criteria

**Run these in order:**

```bash
# 1. Build and test
# (run from repo root)
cargo +nightly build --release 2>&1 | tee phase3-build.log
# (or replace with stable-compatible wide crate build)
```

**Expected observations**:
- Clean compile (no SIMD-specific warnings).

```bash
# 2. Parity and correctness (SIMD must match scalar)
cargo test -p behave-batch --test parity_simd 2>&1 | tee phase3-parity-simd.log
```

**Expected test**:
```rust
#[test]
fn simd_matches_scalar_within_ulp() {
    let cells = vec![/* 171 parity-suite cells */];
    let tables = Tables::default();
    
    let scalar_out = run_surface_batch_serial(&cells, &tables);
    let simd_out = run_surface_batch_simd(&cells, &tables);
    
    for (i, (s, si)) in scalar_out.iter().zip(simd_out.iter()).enumerate() {
        // For fields where vectorized transcendentals differ, allow 1e-10 relative
        let rel_err = (s.spread_rate - si.spread_rate).abs() / s.spread_rate.max(1e-10);
        assert!(rel_err <= 1e-10, "cell {i} spread_rate diverged by {rel_err}");
    }
}
```

**Expected observations**:
- Test passes.
- Most outputs are bit-identical; transcendental outputs have bounded rel error ≤ 1e-10.

```bash
# 3. Benchmark: scalar vs. SIMD vs. rayon+SIMD
cargo build --release -p behave-batch 2>&1
cargo bench -p behave-batch --bench simd_scaling 2>&1 | tee phase3-bench.log
```

**Expected output** (sample):
```
Cells: 100,000
  Scalar serial:         128.3 ms (779.6 cells/sec)
  Scalar + rayon-8:       19.2 ms (5,208 cells/sec), speedup 6.7x
  SIMD (f64x8) serial:    34.1 ms (2,932 cells/sec), speedup 3.8x
  SIMD + rayon-8:          4.3 ms (23,256 cells/sec), speedup 29.8x
```

**Scaling rule of thumb**: SIMD alone ≈ 3–5×; SIMD + rayon ≈ 20–40×.

```bash
# 4. Divergence check: grouped vs. non-grouped
# Compare grouping overhead vs. per-lane masking approach
```

### Phase 3 Gate Resolution

**If all observations match**: `PASS` → commit via `behave-change-control`.

**If SIMD correctness fails** (rel_err > 1e-10):
- Example: `exp()` in vectorized form differs more than expected.
- Debug: use `wide` crate's higher-precision variants or compensated summation.
- Or: fall back to scalar transcendentals in SIMD kernel, vectorize only arithmetic (typically gets 70% of benefit).
- Fix and re-measure.

**If grouping overhead is significant** (>10% time):
- Example: sorting + grouping takes 50 ms on 100k cells; SIMD gain is only 100 ms.
- Alternative: amortize by running many batches in sequence (grouping is one-time).
- Or: cache grouped indices if cells are stable across runs.

**If scaling is sublinear** (e.g., 15× on 8 cores instead of 20–40×):
- Suspect: memory bandwidth or SIMD utilization.
- Debug with `perf stat -e cache-misses,cycle_activity.stalled_cycles` or `cargo flamegraph`.
- May indicate tables not fitting in L3 cache; move critical tables to TLS or use read-only buffer pinning.

---

## Phase 4: WebGPU via wgpu (GPU Dispatch)

**Effort**: ~20–30 days (shader development, cross-platform testing).

**Goal**: landscape-scale cells/sec throughput competitive with or exceeding CPU SIMD; portable across native (Metal/Vulkan/DX12) and browser (WASM + WebGPU).

### Phase 4 Deliverables

1. **Precision validation (MANDATORY before shader writing)**:
   - f32 sensitivity study (Phase 2) must be complete.
   - Error budget must document which outputs are GPU-safe.
   - Known risk: WGSL has no f64 — all GPU compute is f32.
   - Gate: CPU f32 baseline must show acceptable error before GPU work starts.

2. **New `behave-gpu` crate** (or feature in `behave-batch`):
   ```toml
   # crates/behave-gpu/Cargo.toml
   [package]
   name = "behave-gpu"
   version = "0.1.0"
   
   [dependencies]
   wgpu = "0.20"
   wgpu-core = "0.20"
   ```

3. **Kernel design** (WGSL compute shaders):
   - One shader per fuel-model *kind* (standard, chaparral, palmetto, aspen, two-fuel).
   - Input layout: SoA storage buffers (coalesced memory access).
   - Tables: read-only storage buffers uploaded once per batch.
   - Output: SoA storage buffers written back to CPU.

   ```wgsl
   // surface_standard.wgsl
   @group(0) @binding(0) var<storage, read> fuel_models: array<FuelModelData>;
   @group(0) @binding(1) var<storage, read> cells: CellBatch;  // SoA
   @group(0) @binding(2) var<storage, read_write> outputs: OutputBatch;
   
   @compute @workgroup_size(256)
   fn main(@builtin(global_invocation_id) idx: vec3u) {
       let cell_id = idx.x;
       if (cell_id >= cells.count) { return; }
       
       // Fetch SoA fields for this cell
       let wind_speed = cells.wind_speeds[cell_id];
       let slope = cells.slopes[cell_id];
       // ... etc
       
       // Compute surface fire (all f32)
       let ros = compute_spread_rate(wind_speed, slope, /* ... */);
       
       // Store output
       outputs.spread_rates[cell_id] = ros;
   }
   ```

4. **Kernel pipeline** (surface → crown → spot):
   - Surface kernel dispatches 100,000 cells, writes to intermediate buffer.
   - Crown kernel reads surface outputs, computes crown ROS, writes intermediate.
   - Spot kernel reads crown outputs, computes spotting distance.
   - All kernels grouped by fuel model kind; special models spill to CPU fallback.

5. **Target platforms**:
   - Native: Vulkan, Metal, DX12 (via `wgpu` auto-selection).
   - Browser: WASM target `wasm32-unknown-unknown`, WebGPU backend.
   - Build command: `cargo build --release --target wasm32-unknown-unknown`.

### Phase 4 Gate Criteria

**Run these in order:**

```bash
# 1. Phase 2 (f32 error budget) must be signed off
# Review: crates/behave-batch/ERROR_BUDGET_f32.md
# Decision: which outputs are GPU-safe, which need f64 fallback?
```

**Expected observations**:
- Error budget file exists and documents decision.
- Example: "ROS, flame, reaction: f32_safe. Scorch height: f64_fallback."

```bash
# 2. Build GPU crate (wgpu initialization)
# (run from repo root)
cargo build --release -p behave-gpu 2>&1 | tee phase4-build.log
```

**Expected observations**:
- Clean compile.
- No unsafe (wgpu is safe on the Rust side; WGSL is safe by design).

```bash
# 3. Integration test: GPU vs. CPU f32 baseline
# (Create: crates/behave-gpu/tests/gpu_correctness.rs)
cargo test -p behave-gpu --test gpu_correctness 2>&1 | tee phase4-gpu-correctness.log
```

**Expected test**:
```rust
#[test]
fn gpu_f32_matches_cpu_f32_within_phase2_budget() {
    let cells = vec![/* 100 random cells, standard fuel models only */];
    let tables = Tables::default();
    
    // CPU f32 baseline
    let cpu_out = run_surface_batch_simd_f32(&cells, &tables);
    
    // GPU f32
    let gpu_out = run_surface_batch_gpu(&cells, &tables);
    
    // Compare against Phase 2 error budget
    for (i, (cpu, gpu)) in cpu_out.iter().zip(gpu_out.iter()).enumerate() {
        // Scorch: not compared (f64 fallback)
        // ROS: allow 0.1% (within 2% budget)
        let ros_err = (cpu.spread_rate - gpu.spread_rate).abs() / cpu.spread_rate.max(1e-10);
        assert!(ros_err <= 0.001, "cell {i} ROS diverged by {ros_err*100}%");
    }
}
```

**Expected observations**:
- Test passes (GPU f32 matches CPU f32 within budget).
- If test fails with large divergence, likely causes:
  - Shader transcendental precision issue (use `wide`-style algorithms in WGSL).
  - Data layout mismatch (SoA indexing off-by-one).
  - Uninitialized output buffer.

```bash
# 4. Throughput benchmark
cargo build --release -p behave-gpu 2>&1
cargo bench -p behave-gpu --bench gpu_scaling 2>&1 | tee phase4-bench.log
```

**Expected output** (sample, assuming 8-core CPU, RTX 3070 GPU):
```
Cells: 1,000,000
  CPU (scalar):           1,280 ms (781 cells/sec)
  CPU (SIMD + rayon-8):     128 ms (7,812 cells/sec)
  GPU (RTX 3070):            9.6 ms (104,166 cells/sec)
  GPU speedup vs. SIMD:      13.3×
  GPU speedup vs. scalar:    133×
```

**Falsifiable milestone** (from `behave-research-frontier`):
- Target: GPU implementation beats FlamMap/ElmFire at landscape-scale batch evaluation.
- Metric: cells/sec for a 30 million cell landscape (480 km² @ 4 m/cell).
- FlamMap baseline: ~100k cells/sec (requires ~5 min per run on CPU).
- Target: >500k cells/sec (achieves <1 min per run on GPU).

```bash
# 5. Check WASM build
cargo build --release --target wasm32-unknown-unknown -p behave-gpu 2>&1 | tee phase4-wasm.log
```

**Expected observations**:
- Binary compiles (may have platform-specific warnings for WGPU; OK if no link errors).
- WASM file size reasonable (<10 MB uncompressed for the kernel crate).

### Phase 4 Gate Resolution

**If all observations match**: `PASS` → commit via `behave-change-control`.

**If GPU throughput is not competitive** (e.g., only 2× vs. CPU SIMD):
- Suspect: small batch size, GPU overhead, memory transfer overhead.
- Debug: measure kernel time vs. host↔device transfer time.
- Fix: if transfer-bound, batch amortization (run 10 batches in sequence, measured per 100k cells, not per batch).
- Or: GPU only viable for >1M cells per run; document threshold.

**If WASM build fails**:
- Likely: WGPU does not have WASM backend yet, or version mismatch.
- Alternative: use `wgpu-core` + manual WebGPU binding, or defer WASM to Phase 4.5.
- Escalate to `behave-change-control`.

**If GPU correctness diverges > Phase 2 budget**:
- Example: scorch height GPU is f32, Phase 2 said "not safe".
- Fix: implement f64 scorch height on CPU (async readback after GPU dispatch).
- Or: use mixed-precision kernel (f32 for ROS/flame, f64 for scorch via intermediate CPU call).
- Escalate for decision.

---

## Fenced Wrong Paths (Explicitly Rejected)

Each with reason — do not pursue:

1. **BLAS integration** (Openblas, BLAS.jl, CuBLAS):
   - Reason: Rothermel model is *not linear algebra*. No matrix products, no factorizations. Reshaping into GEMM form = unnecessary memory traffic.
   - Right abstraction: batched elementwise kernels (rayon/SIMD/WGSL).
   - Deferred to: future if fire-spread PDEs on rasters emerge as need.

2. **GPU-ing the containment simulator** (contain module):
   - Reason: variable-length retry loops, per-step control flow. Never a per-cell raster workload.
   - Decision: stay on CPU. Contain sim is a small part of typical workflows.

3. **Per-lane masking before grouping** (within SIMD phase):
   - Reason: fuel-model divergence → masking cost = ~30% efficiency loss vs. grouping.
   - Decision: group first, vectorize homogeneous fuel kinds.

4. **WASM-first (emscripten/WebIDL approach)**:
   - Reason: historically costly (rj-idl-bindings branch, abandoned 2024–2025). WASM should come AFTER kernels are pure.
   - Decision: build GPU pipeline first (Phase 4), then target wasm32 as deployment backend (Phase 4.5 or later).
   - Right order: pure kernels → tests → GPU → WASM.

5. **Fixing preserved C++ quirks while refactoring**:
   - Reason: change-control violation. Introduces risk that testing cannot isolate.
   - Examples: TL5 savrLiveWoody=160.0 (likely typo), pressure units inverted, canopy-height units ignored.
   - Decision: preserve quirks in Phase 0; document with code comments. Fix only via `behave-change-control` as separate PR.

6. **Introducing f32 into the reference path**:
   - Reason: f64 remains source of truth forever. f32 is a GPU deployment option, validated against f64.
   - Decision: f64 is the canonical library; Phase 2 validates f32 as a variant; Phase 4 uses f32 on GPU.

---

## Promotion Protocol (Change Control)

Every phase lands via a `behave-change-control` gate. Template:

### Phase N Promotion Checklist

**Before creating PR:**

```bash
# 1. Verify gate criteria (all commands from Phase N section pass)
# (run from repo root)
# [Run all Phase N gate commands — see Phase N section above]
# Expected: all observations match.

# 2. Update REVIEW.org roadmap section
# Open REVIEW.org, find "Parallelization Roadmap" section
# Add line under "Suggested sequence":
#   N. Phase N: [name] — [brief status, e.g. "DONE: commit abc123"]
# (Keep previous phases for reference.)

# 3. Create feature branch
git checkout -b rj-phase-N-[feature-name]

# 4. Stage all changes
git add crates/
git add .claude/skills/  # if skill updates

# 5. Commit with structured message
git commit -m "$(cat << 'EOF'
Phase N: [Feature Name] (gate pass)

- [Deliverable 1]: [short summary]
- [Deliverable 2]: [short summary]
- [Measurement]: [e.g. "6.8× speedup on 8 cores"]

Gate criteria: [yes/no for each]
- Parity 171/171: YES
- Scaling: 6.8× (expected: 6–8×) PASS
- Zero panics: YES

Associated PRs/issues: [reference behave-change-control gate]

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>
EOF
)"

# 6. Push and open PR
git push -u origin rj-phase-N-[feature-name]
# Create PR via GitHub:
#   Title: "Phase N: [Feature]"
#   Body: [link to REVIEW.org section, gate summary, performance table]

# 7. CI checks
# Wait for .github/workflows/ci.yml (currently C++ only)
# Post-land: request Rust CI activation if not already running.
```

**PR Template Body**:
```markdown
## Phase N: [Feature Name]

Closes #??? (if applicable)

### Summary

[One paragraph: what did we accomplish this phase?]

### Gate Criteria

| Criterion | Status | Details |
|-----------|--------|---------|
| Parity suite (171/171) | PASS | crates/behave-run/tests/parity.rs |
| [Criterion 2] | PASS | [details] |
| [Criterion 3] | PASS | [details] |

### Measurements

- **Scalar baseline**: 781 cells/sec
- **Phase N output**: 6,123 cells/sec (7.8× speedup)
- **Expected**: 6–8×; **Achieved**: 7.8× ✓

### Files Changed

- crates/behave-batch/ (new)
- crates/behave-run/tests/parity.rs (green)
- REVIEW.org (roadmap updated)

### Testing

Run before merge:
```bash
cargo test --workspace 2>&1
cargo test -p behave-run --test parity 2>&1
```

Expected: all pass.
```

**Post-PR Review**:
- `behave-change-control` gate owner or senior AI agent reviews:
  - Gate criteria matched?
  - Measurements realistic and reproducible?
  - No breaking changes to public API (or changes are deliberate and documented)?
  - REVIEW.org updated?
- Approval: merge or request changes.

---

## Progress Tracking Convention

After each phase lands, update `REVIEW.org` section **Suggested sequence**:

```org
** Suggested sequence

1. Parity test suite (P0) — golden values from =testBehave.cpp= *(DONE:
   =behave-run/tests/parity.rs=, 171 checks)* [commit abc123]
2. Phase 0 pure-kernel refactor — *IN_PROGRESS* (target: 2026-07-20)
   - [ ] SurfaceInputs String → enum
   - [ ] Extract kernel functions
   - [ ] Make tables static
   - [ ] All 279 unit tests green
   - [ ] Parity 171/171 green
3. Phase 1 =behave-batch= crate with rayon — *(QUEUED)*
   - [ ] ~100 loc rayon facade
   - [ ] Benchmark harness
   - [ ] Bit-identity assertion
   - [ ] Expected 6–8× on 8 cores
4. [... rest as in current REVIEW.org ...]
```

---

## Quick Reference: Command Checklist

### Gate Run (all phases)

```bash
# (run from repo root)

# Baseline tests
cargo test --workspace 2>&1 | grep "test result:" | tail -1
cargo test -p behave-run --test parity 2>&1 | tail -3

# Forbidden constructs check
grep -r "interior_mutability\|Rc<\|RefCell" crates/*/src --include="*.rs" || echo "clean"

# Build log scan
cargo build --lib 2>&1 | grep -i "warning\|error" || echo "clean"
```

### Phase 0 Exit (before proceeding)

```bash
# Verify SurfaceInputs is Copy + Pod
grep "pub struct SurfaceInputs" crates/behave-surface/src/inputs.rs -A 80 | grep -c "String"
# Expected: 0 (after refactor)

# Verify tables are static
grep "pub.*fuel_models:" crates/behave-surface/src/ -r | grep -i "static\|lazylock"
# Expected: at least one match with static/LazyLock
```

### Phase 1 Exit (before proceeding)

```bash
# Bit-identity test
cargo test -p behave-batch --test identity 2>&1 | grep "passed"
# Expected: 1 passed

# Scaling data
cargo bench -p behave-batch --bench batch_scaling 2>&1 | tee phase1-final-bench.log
grep -E "cells/sec|speedup" phase1-final-bench.log
```

### Phase 2 Exit (before proceeding)

```bash
# Error budget file
cat crates/behave-batch/ERROR_BUDGET_f32.md | head -20

# f32 parity divergence
cargo test -p behave-batch --test parity_f32 2>&1 | tail -5
```

### Phase 3 Exit (before proceeding)

```bash
# SIMD correctness
cargo test -p behave-batch --test parity_simd 2>&1 | grep "passed"

# Scaling: SIMD + rayon
cargo bench -p behave-batch 2>&1 | grep "SIMD.*rayon"
```

### Phase 4 Exit (delivery ready)

```bash
# GPU correctness
cargo test -p behave-gpu --test gpu_correctness 2>&1 | tail -5

# GPU throughput
cargo bench -p behave-gpu 2>&1 | grep "GPU.*cells/sec"

# WASM build
cargo build --release --target wasm32-unknown-unknown -p behave-gpu 2>&1 | tail -3
```

---

## Provenance and Maintenance

**This skill is based on:**
- REVIEW.org § "Parallelization Roadmap" (lines 262–421, as of 2026-07-06).
- REVIEW.org § "Parity suite results" (lines 180–221) for test counts and gate strategy.
- Context pack § "Readiness facts" and "Phase 0/1/2/3/4 descriptions."

**Re-verification commands** (re-run these if plan changes):
- Parity test count: `grep -c '\.check(' crates/behave-run/tests/parity.rs` (should be 128).
- REVIEW.org section title: `grep "Parallelization Roadmap" REVIEW.org` (should exist).
- Rust workspace structure: `ls crates/` (should list 9 crates).
- SurfaceInputs String field: `grep "current_moisture_scenario_name: String" crates/behave-surface/src/inputs.rs` (exists at line 52).

**Related skills:**
- `behave-change-control` — gates, divergence ledger, C++ quirk preservation (HOME for promotion protocol).
- `behave-validation-and-qa` — parity suite anatomy, test addition (HOME for evidence standards).
- `fire-behavior-reference` — domain theory (HOME for scorch/wind formula details).
- `behave-diagnostics-and-tooling` — benchmark methodology, criterion baseline (HOME for measurement).
- `behave-architecture-contract` — crate DAG, API-style rationale (cross-ref for Phase 0 struct design).

---

*Last updated: 2026-07-06. Campaign authored by Claude per task: "behave-parallelization-campaign skill creation."*
