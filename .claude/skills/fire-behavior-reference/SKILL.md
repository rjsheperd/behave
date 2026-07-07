---
name: fire-behavior-reference
description: Comprehensive domain-theory pack for fire behavior modeling — the field's math as implemented here, equations by model/paper/code-citation, base units, fuel systems, moisture scenarios, orientation conventions, wind machinery, special fuels, magic constants, and external ecosystem.
---

# Fire Behavior Reference

Reference pack for fire behavior modeling in Behave (BehaveCore). Audience: agents with zero fire-science background. This is the math AS IMPLEMENTED in the Rust port, grounded in the C++ reference and verified against the parity test suite.

**When NOT to use this skill**: for Rust/C++ architecture and crate structure, use `behave-architecture-contract`; for test validation and evidence standards, use `behave-validation-and-qa`; for numerical algorithms and debugging domain traps, use `behave-debugging-playbook` or `behave-numerics-proof-toolkit`; for parallelization/GPU modeling decisions, use `behave-parallelization-campaign`.

---

## 1. Model Zoo: Equations Cited in Code

| Model / Paper | Computes | Implemented In | Key Equations |
|---|---|---|---|
| **Rothermel 1972** | Surface fire spread rate (ROS) | `behave-surface/src/reaction.rs` | Eq. 27: Reaction intensity = (gamma_max × (1+beta/beta_op) × eta_M × eta_S × h_s × rho_b × epsilon) / (rho_e × c) |
| **Rothermel 1972** | Wind and slope factors | `behave-surface/src/fire.rs` (facade) | Eq. 47–51: wind factor B, slope factor C, effective ROS |
| **Byram 1959** | Fireline intensity → flame length | `behave-surface/src/fuelbed.rs` | Flame_length = 0.45 × FLI^0.46 (FLI in Btu/ft/s, output in feet) |
| **Rothermel 1991** | Crown fire L/W and area | `behave-crown/src/fire.rs` | Eq. 10/11/13: L/W = 1 + 0.125×U (where U in mi/hr); area, perimeter from ellipse geometry |
| **Scott & Reinhardt 2001** | Crown fire critical open wind speed | `behave-crown/src/fire.rs` | Eq. 20: Critical wind threshold with precomputed FM10 constants (prop_flux=0.048317, wind_b=1.4308, wind_k=0.001610) |
| **Anderson 1983** | Surface L/W (wind-driven ellipse) | `behave-surface/src/fire.rs` | L/W = 0.936·e^(0.1147U) + 0.461·e^(-0.0692U) − 0.397 (capped at 8.0; U in mi/hr) |
| **Albini & Baughman 1979** | Wind adjustment factor (unsheltered) | `behave-surface/src/wind.rs:128–129` | WAF = 1.83 / ln((20 + 0.36·h) / (0.13·h)), where h = fuelbed depth (feet) |
| **Finney (RMRS-RP-4, Eq. 45)** | WAF (sheltered, no crown ratio) | `behave-surface/src/wind.rs:81` | crown_fraction = (canopy_cover × π) / 12; WAF = 0.555 / (sqrt(crown_frac × h) × ln(...)) |
| **Finney (1998, 2004)** | Sheltered WAF with crown ratio | `behave-surface/src/wind.rs:62` | crown_fraction = crown_ratio × canopy_cover / 3 (conical crown assumption) |
| **Scorch height (empirical)** | Height above ground cambium killed | `behave-surface/src/fire.rs:357–361` | H = (63 / (140 − T)) × FLI^(7/6) / √(FLI + U³); T in °F, FLI in Btu/ft/s, U in mi/hr |
| **Fried & Fried 1996** | Fireline containment | `behave-contain/src/algorithm.rs` | Perimeter growth vs. attackable edge geometry |
| **Finney EXRATE** | Spot fire enumeration (deterministic) | `behave-surface/src/exrate.rs` | Factorial sampling of input matrix; NOT probabilistic despite name |
| **Latham lightning ignition** | Lightning strike ignition probability | `behave-ignite/src/lightning.rs` | Charge-based probability; implemented in ignite module |
| **FOFEM mortality (Forest-Vegetation-Simulator)** | Species mortality from scorch height | `behave-mortality/src/species.rs` | Logistic regression by species, DBH, scorch height; precomputed tables |

---

## 2. Quantity Table: Base Units, Display Conventions, Typical Magnitudes

Verified against `crates/firelab-base/src/units.rs` and parity test golden values.

| Quantity | Symbol | Base Unit | Display Convention | Typical Range (from parity suite) |
|---|---|---|---|---|
| **Spread Rate** | ROS | ft/min | chains/hour (1 chain = 66 ft; ch/hr = ROS×10/11) | 2–20 ch/hr (low to moderate fire) |
| **Fireline Intensity** | FLI, I | Btu/ft/s | kW/m or Btu/ft/min (×60) | 50–1000 Btu/ft/s (landscape-scale) |
| **Reaction Intensity** | RI | Btu/ft²/min | (no standard display alternate) | 1000–5000 Btu/ft²/min |
| **Flame Length** | FL | feet | feet | 1–20 ft (from 0.45×FLI^0.46) |
| **Heat of Combustion** | HOC | Btu/lb | (no alternate) | 8000–9000 Btu/lb (dead); 8000+ (live) |
| **Heat Sink** | hs | Btu/ft³ | (no alternate) | ~40 Btu/ft³ (typical) |
| **Propagating Flux** | prop_flux | Btu/ft² | (no alternate) | ~0.3–0.5 Btu/ft² |
| **Fuel Loading** | load | lb/ft² | tons/acre (÷0.045914) or tonnes/ha | 0.05–1.5 lb/ft² |
| **SAVR (Surface Area-to-Volume Ratio)** | σ | ft²/ft³ | m²/m³ (÷0.3048) | 1500–3000 ft²/ft³ |
| **Moisture (size class)** | M | fraction 0–1 | % (×100) | 3–30% dead; 30–150% live |
| **Moisture of Extinction** | MOE | fraction 0–1 | % (×100) | 20–40% (varies by fuel type) |
| **Slope** | S | degrees | % slope (tan(deg)×100) | 0–50% |
| **Canopy Height** | h_c | feet | feet | 40–100 ft (forest canopy) |
| **Crown Ratio** | CR | fraction 0–1 | % (×100) | 0.2–0.8 (live crown extent) |
| **Scorch Height** | H_s | feet | feet | 20–60 ft |
| **L/W Ratio** | λ | dimensionless | (no alternate) | 1–8 (elliptical fire perimeter) |
| **Wind Speed (20-ft or 10-m)** | U_20, U_10m | ft/min (base) | mi/hr or m/s | 5–30 mi/hr (standard input) |
| **Midflame Wind Speed** | U_mf | ft/min (base) | mi/hr (÷88) | 1–15 mi/hr (after WAF) |

**Verification:** Values pulled from `crates/behave-run/tests/parity.rs` test outputs (e.g., spread rate 19.677584 ch/hr; flame length from parity suite; heat source 5177.248579 Btu/ft²/min).

---

## 3. Fuel Model System

Verified against `crates/behave-surface/src/fuel_models.rs`.

### Standard (FM1–FM13)
Original 13 models, reserved (non-customizable):
- **FM1:** Short grass — 1.0 ft depth, 0.034 lb/ft² dead load, high SAVR (3500 ft²/ft³)
- **FM2:** Timber grass & understory — mixed dead/live loads
- **FM3:** Tall grass — 2.5 ft depth, large fuel bed
- **FM4–FM5:** Chaparral/Brush — mixed woody loads
- **FM6–FM7:** Hardwood slash, dormant brush
- **FM8–FM9:** Needle litter (short/long)
- **FM10:** Timber litter & understory — complex mixed load
- **FM11–FM13:** Logging slash (light/medium/heavy)

### Non-Burnable (NB91–NB99)
Reserved slots: NB1 (urban), NB2 (snow/ice), NB3 (agricultural), NB4–NB5 (future), NB8 (open water), NB9 (bare ground).
All have zero fuel loads; used for land-class coding.

### Scott & Burgan 40-Model Series
**Ranges (fuel_models.rs lines 458–800+):**
- **GR (Grass) 101–111:** GR1–GR9 (dynamic); V-Hb/V-Ha (international); depth 0.35–5.0 ft, mostly herbaceous load
- **GS (Grass-Shrub) 121–124:** GS1–GS4 (dynamic); depth 0.9–2.1 ft; live woody + herbaceous mix
- **SH (Shrub) 141–159:** SH1–SH9 (static/dynamic); SCAL17–SCAL18 (chamise, ceanothus, manzanita); V-MH/V-MMb/V-MAb/V-MMa/V-MAa (international); depth 0.5–6.0 ft
- **TU (Timber-Understory) 161–172:** TU1–TU5, M-EUCd/M-H/M-F/M-CAD/M-ESC/M-PIN/M-EUC (international); depth 0.1–1.3 ft
- **TL (Timber-Litter) 181–204:** TL1–TL9, F-* (international fuelbed depth 0.2–0.4 ft; litter-dominated)
- **SB (Slash-Blowdown) 201–204:** Heavy slash; 3–5 ft depth

**Dynamic vs. Static:** Dynamic models apply live herbaceous load transfer (see section 4). Static models have fixed loads.

### Custom Models
Slots 14–89, 96–97, 112–120, 131–139, 173–179, 205–255 available. `set_custom_fuel_model()` accepts all properties with unit conversion.

---

## 4. Moisture System

Verified against `crates/behave-surface/src/moisture.rs`.

### Size-Class Semantics
- **1-hour:** <1/4" diameter; responds to humidity within ~1 hour (e.g., surface litter)
- **10-hour:** 1/4"–1" diameter; ~10-hour equilibration time (small twigs)
- **100-hour:** 1"–3" diameter; ~100-hour response (branches)
- **Live herbaceous:** Curing grass, herbs; moisture >100% when green, <30% when cured
- **Live woody:** Foliage, bark-attached wood; slower drying; >100% green, <50% dormant

### Built-in Scenarios (16 total, DxLy grid)
16 scenarios in `moisture.rs:203–273`; case-insensitive lookup via `index_by_name()`.

**Dead fuel levels:**
- D1: [0.03, 0.04, 0.05] (1-hr, 10-hr, 100-hr) — very low
- D2: [0.06, 0.07, 0.08] — low
- D3: [0.09, 0.10, 0.11] — moderate
- D4: [0.12, 0.13, 0.14] — high

**Live fuel levels (herbaceous, woody):**
- L1: [0.30, 0.60] — fully cured
- L2: [0.60, 0.90] — 2/3 cured
- L3: [0.90, 1.20] — 1/3 cured
- L4: [1.20, 1.50] — fully green

Example: `D1L1` = 3%, 4%, 5% dead + 30%, 60% live.

**Note (preserved C++ bug):** D4 scenario descriptions say "D3L1"–"D3L4" (copy-paste in C++). Rust preserves text for parity; actual scenario names are `D4L1`–`D4L4`.

### Moisture Input Modes
- **Individual:** Explicit 5 values (1-hr, 10-hr, 100-hr, live herbaceous, live woody)
- **By scenario:** Lookup `D2L3` etc. (16 presets)
- **By condition:** Fuel moisture tool (fine dead, canopy, slope, aspect, time-of-day interpolation — not yet fully ported to Rust)

---

## 5. Orientation Conventions and Gotchas

Verified against `crates/behave-surface/src/fire.rs` and `crates/behave-crown/src/fire.rs`.

### Wind Direction
- **Definition:** Direction wind **blows FROM** (meteorological convention)
- **0°:** North
- **90°:** East wind (blows from east toward west)
- **180°:** South wind
- **270°:** West wind

### Slope/Aspect
- **Slope:** Degrees from horizontal (base unit); %slope = tan(deg)×100
- **Aspect:** Direction of steepest upslope (0° = north; 90° = east-facing)

### Orientation Modes
1. **RelativeToNorth:** Wind direction and aspect measured from north; fire spreads in direction of vector sum
2. **RelativeToUpslope:** Wind direction measured from upslope (converted internally: `wind_dir_north = wind_dir_upslope + aspect`)

**CRITICAL GOTCHA:** Wind direction input CHANGES MEANING based on mode. Internal calculations always use north-relative; RelativeToUpslope mode subtracts aspect from input before computing wind factor (see `fire.rs` mode dispatch, ~line 200).

### Direction of Max Spread
- Computed via elliptical fire geometry
- **Heading:** atan2(x, y) components from wind + slope vector sum
- **0° = north; 90° = east**

### L/W (Length-to-Width) Ratio
- Characterizes elliptical fire perimeter in direction of max spread
- **L:** Long axis (direction of max ROS)
- **W:** Short axis (perpendicular)
- Surface fire (Anderson 1983): L/W = 0.936·e^(0.1147U) + 0.461·e^(-0.0692U) − 0.397 (capped ≤8)
- Crown fire (Rothermel 1991): L/W = 1 + 0.125·U_20 (linear with 20-ft wind speed)

**Eccentricity:** From L/W, e = sqrt(1 − (1/λ)²) where λ = L/W (used in perimeter/area formulas).

---

## 6. Wind Machinery

Verified against `crates/behave-surface/src/wind.rs`.

### Wind Speed Heights and Conversions

**Three standard heights:**
1. **20-foot:** Measurement height for most US data and BehavePlus I/O
2. **10-meter:** Metric standard; convert to 20-ft via `U_20 = U_10m / 1.15` (Lawson & Armitage 2008)
3. **Midflame:** Effective wind speed at flame base; `U_mf = U_20 × WAF` (wind adjustment factor)

**Conversion factor 1/1.15 (≈0.8696)** is hardcoded in `WindSpeedUtility::ten_meter_to_twenty_foot()`.

### Wind Adjustment Factor (WAF)

WAF converts 20-ft or 10-m wind to midflame. Two calculation paths:

#### Path A: With Crown Ratio (Albini & Baughman 1979)
```
crown_fraction = (crown_ratio × canopy_cover) / 3
  (3 = conical crown assumption)
shelter_method = Sheltered if:
  - canopy_cover > 0, AND
  - crown_fraction ≥ 0.05, AND
  - canopy_height ≥ 6 ft
  Otherwise: Unsheltered
```

#### Path B: Without Crown Ratio (Finney RMRS-RP-4, Eq. 45)
```
crown_fraction = (canopy_cover × π) / 12
  (π/12 = 0.2618... is Finney's variant vs. Albini's 1/3)
```

#### Log-Profile Computation
**Unsheltered** (wind.rs:128–129):
```
WAF = 1.83 / ln((20 + 0.36×h) / (0.13×h))
  where h = fuelbed_depth (feet)
  Typical: h=1 ft → WAF ≈ 0.362
```

**Sheltered** (wind.rs:133–135):
```
WAF = 0.555 / (sqrt(crown_fraction × canopy_height)
             × ln((20 + 0.36×h_c) / (0.13×h_c)))
  where h_c = canopy_height
  More complex; generally WAF < 0.4
```

**Threshold:** Unsheltered if canopy_height < 6 ft or crown_fraction < 5%, regardless of cover.

### Wind Limit (Speed Governing)
- **Default:** Wind speed is NOT capped (wind limit OFF by default since BHP1-1367, 2017)
- **When enabled:** max(U_mf) = 0.9 × RI (reaction intensity)
- **Effect:** Speeds approaching extinction moisture dampen spread artificially; most deployments disable this

---

## 7. Special Fuels: One-Pager Each

### Chaparral (Dynamic Model)
**Module:** `crates/behave-surface/src/chaparral.rs`

- **Triggered:** Fuel model 4 OR special chaparral type selector
- **Inputs beyond standard:**
  - Age (years since burn): regressed depth, live moisture, live HOC
  - Cover percent: scales all loads
- **Depth, MOE, HOC:** Empirical functions of age
- **Load transfer:** Lives become dead as "cured" fraction increases
- **Quirk:** Depth changes with age via exp() model; old chaparral deepens
- **Not yet fully ported:** Complete age/cover semantics in Rust; stub in place

### Palmetto-Gallberry (Regional specialty, SE US)
**Module:** `crates/behave-surface/src/palmetto_gallberry.rs`

- **Triggered:** Fuel model selector to palmetto-gallberry type
- **Region:** Southeast US (North Carolina, etc.)
- **Inputs:** Understory height, rough age (years since thinning), overstory basal area
- **Dead foliage load:** Regressed from age, cover: `L_d = 0.00221 × age^0.51263 × exp(0.02482 × cover%)`
- **Live foliage:** Function of basal area and age
- **Depth:** Always 2/3 × understory height (fixed)
- **HOC:** Dead=8000, Live=8300 Btu/lb (fixed); MOE=0.40 (40%)

### Western Aspen (North American specialty)
**Module:** `crates/behave-surface/src/western_aspen.rs`

- **Triggered:** Fuel model selector or detected aspen species
- **Region:** Mountain West aspen stands
- **Method:** 5 sub-models (by DBH class and curing state), interpolated by live herbaceous moisture
- **Inputs:** DBH (diameter at breast height), live herbaceous moisture (curing proxy)
- **Output:** Spread rate, then scorch height → mortality logistic by DBH + flame length / 1.8
- **Mortality curve:** S-curve; low DBH trees killed at lower flame lengths

---

## 8. Magic-Number Table

Critical constants used in calculations. Verify against code comments for citation.

| Constant | Value | Used In | Citation | Cited? |
|---|---|---|---|---|
| **Scorch formula coefficient** | 63 | `fire.rs:357` | Empirical (flame energy → cambium kill height) | Uncited in code |
| **Scorch denominator base** | 140 | `fire.rs:357` | Temperature scaling for scorch | Uncited in code |
| **Scorch exponent** | 7/6 = 1.1667 | `fire.rs:358` | (FLI^(7/6)) | Uncited in code |
| **Flame length coefficient** | 0.45 | (implicit in fuelbed.rs calc) | Byram 1959 | Not explicit; known by domain |
| **Flame length exponent** | 0.46 | (implicit in fuelbed.rs calc) | Byram 1959 | Not explicit; known by domain |
| **Optimum packing ratio** | 0.3348 | `reaction.rs:~80` | Albini 1976 p. 88 (γ_max = 133/σ^0.7913) | Cited as constant, not equation |
| **Albini density constant** | 0.8189 | `reaction.rs:~95` | Albini 1976 p. 91 (ρ = 32 lb/ft³) | Cited as constant |
| **Heat release (q_ig)** | 250 Btu/lb (pre-ignition) + 1116 × ρ_b | `reaction.rs` | Internal energy calc | Uncited |
| **Mineral damping (a, b)** | a=0.792, b=0.681 | `reaction.rs:~130` | Mineral silica; Rothermel table | Uncited in code |
| **WAF unsheltered coeff** | 1.83 | `wind.rs:128` | Albini & Baughman 1979 (unsheltered) | Cited in docstring |
| **WAF log-profile numerators** | 20, 0.36, 0.13 | `wind.rs:128–129` | Logarithmic wind profile | Uncited |
| **WAF sheltered coeff** | 0.555 | `wind.rs:133` | Finney variant | Uncited |
| **WAF Finney π/12** | 0.2618 | `wind.rs:81` | RMRS-RP-4 eq. 45 | Cited in docstring |
| **TL5 SAVR (likely typo)** | 160.0 | `fuel_models.rs:~809` | Original fuel model spec; suspected 1600 | Known quirk (preserved) |
| **Crown ratio divisor** | 3.0 | `wind.rs:62` | Conical crown assumption | Uncited |
| **Ten-meter conversion** | 1.15 | `wind.rs:153` | Lawson & Armitage 2008 | Cited in docstring |
| **Canopy height WAF threshold** | 6.0 ft | `wind.rs:110–112` | Albini & Baughman | Uncited |
| **Crown fraction WAF threshold** | 0.05 | `wind.rs:110–112` | Albini & Baughman | Uncited |

**Uncited load-bearing constants (high risk):** scorch 63/140, WAF 1.83/0.555, wind profile 20/0.36/0.13. If challenged, cite original papers or revert to C++ source.

---

## 9. Glossary

| Term | Definition | Typical Range | Unit |
|---|---|---|---|
| **ROS** | Rate of spread (fire perimeter advance in direction of max spread) | 2–20 | ch/hr |
| **FLI** | Fireline intensity (energy flux perpendicular to fire edge) | 50–1000 | Btu/ft/s |
| **HPUA** | Heat per unit area (total energy released by fuel particles) | ~500 | Btu/ft² |
| **SAVR** | Surface-area-to-volume ratio; particle size proxy | 500–3500 | ft²/ft³ |
| **MOE** | Moisture of extinction; max moisture at which fuel ignites | 20–40 | % |
| **WAF** | Wind adjustment factor; sheltering multiplier on wind speed | 0.2–0.5 | dimensionless |
| **L/W** | Length-to-width ratio of elliptical fire perimeter | 1–8 | dimensionless |
| **Eccentricity** | Shape parameter of fire ellipse (e = √(1 − (1/L/W)²)) | 0.8–0.97 | dimensionless |
| **Fuelbed** | Collection of fuel particles (dead 1-hr, 10-hr, 100-hr; live herbaceous, woody) | (structural) | (structural) |
| **Fireline** | Active flame edge; perimeter where fire energy is concentrated | (geometric) | (geometric) |
| **Chain** | Unit of length; 1 chain = 66 feet (surveyor's chain) | 1 ch = 66 ft | feet |

---

## 10. External Ecosystem

Brief one-line orientation to key consumers and peer systems.

| System | Role | Relationship to Behave |
|---|---|---|
| **BehavePlus (USFS desktop)** | User-facing fire behavior calculator; dominant field standard | Consumes BehaveCore C++; Rust port aims to replace C++ backend |
| **IFTDSS (Interagency Fuel Treatment Decision Support System)** | Strategic fuel treatment ROI optimizer | Calls BehaveCore for landscape-scale batch fire behavior |
| **FlamMap / ArcGIS Wildfire Analyst** | Landscape-scale fire simulation (gridded); probabilistic fire spread | Consumes BehaveCore spread model for each grid cell; independent momentum/wind field |
| **ElmFire (formerly FARSITE 5)** | Physics-based fire growth model; wind-field integration | Consumes Rothermel ROS; more detailed than Behave for tactical simulation |
| **FARSITE (Finney vector-based simulator)** | Elliptical perimeter advancement; retired but still used | Original integrator of Rothermel+Albini L/W; EXRATE (spot fire) born here |
| **FOFEM (First-Order Fire Effects Model)** | Post-fire severity (scorch height → mortality by species/size) | Consumes Behave scorch height; links to landscape damage assessment |

---

## Parallelization Roadmap (Canonical Context)

Rust port is becoming canonical implementation (owner decision 2026-07-06). Parallelization phases:

1. **Phase 0 (current):** Pure-kernel refactor. No interior mutability, global state, or panics in library code. Tables must be shareable.
2. **Phase 1 (candidate):** Rayon batch processing (multi-core CPU on same input matrix).
3. **Phase 2 (planned):** f32 study (scorch height cancellation risk: sqrt(FLI + U³)).
4. **Phase 3 (exploratory):** Structure-of-Arrays (SoA) + SIMD (within f32 constraints).
5. **Phase 4 (research frontier):** WebGPU (wgpu/WGSL); beat FlamMap/ElmFire at landscape scale.

**Constraint:** Every change must keep pure-kernel path viable (Copy inputs, static tables, no String/heap fields in input structs).

---

## Provenance and Maintenance

**Source:** Verified against:
- Rust port workspace (9 crates, ~21k lines)
- C++ reference (`src/behave/`, ~28k lines)
- Golden parity suite (`crates/behave-run/tests/parity.rs`, 171 runtime checks)
- Original papers cited in code (Rothermel 1972, Albini 1976, etc.)

**Re-verification commands (all read-only; run from repo root):**

```bash
# Fuel model counts and ranges
grep -c "set_record" crates/behave-surface/src/fuel_models.rs  # ~90 fuel models defined

# Moisture scenario count (should be 16)
grep -c "add_record" crates/behave-surface/src/moisture.rs  # 16 DxLy scenarios

# Scorch height formula (63/(140-T)*I^(7/6)/sqrt(...))
grep -A 5 "calculate_scorch_height_static" crates/behave-surface/src/fire.rs

# WAF constants (1.83 unsheltered, 0.555 sheltered)
grep -E "1\.83|0\.555" crates/behave-surface/src/wind.rs

# Base units (length=ft, speed=ft/min, area=ft2, etc.)
head -50 crates/firelab-base/src/units.rs

# Parity runtime check count (current: 171; static call sites 142)
grep -E "\.check\(|\.check_bool\(" crates/behave-run/tests/parity.rs | wc -l

# CI status (Rust: no CI coverage; C++ only on master)
cat .github/workflows/ci.yml | grep -A 10 "runs-on"
```

**Known gaps:**
- Parity check count: 171 at runtime (142 static call sites; four sites loop) — resolved 2026-07-06; verify with `cargo test -p behave-run --test parity -- --nocapture`
- C++ CI only; Rust workspace has zero CI coverage (known gap, issue #ABC [TBD])
- Cargo workspace missing license field (gap in metadata)
- Chaparral age/cover semantics not fully ported to Rust (stub in place)
- behave-run facade is skeleton (no full I/O plumbing)

**Last updated:** 2026-07-06 (verified against HEAD f11cbc5)
