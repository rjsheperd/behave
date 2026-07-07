//! EXRATE — expected spread rate in randomly arranged fuels.
//!
//! C++ source: randfuel.h/cpp, randthread.h/cpp (Mark Finney's EXRATE
//! package, BehavePlus renaming by Collin D. Bevins).
//!
//! Despite the name, the computation is fully deterministic: it enumerates
//! every factorial arrangement of the fuel types over a small sample block
//! (`samples` columns × `depths` rows), computes the fastest fire travel
//! path through each arrangement (heading plus elliptical flanking), and
//! returns the probability-weighted average of the block spread rates.
//!
//! Scope: only the no-lateral-extension path is ported. Behave's only caller
//! (`SurfaceTwoFuelModels`, C++ surfaceTwoFuelModels.cpp:335-337) uses
//! samples=2, depth=2, laterals=0, threads=1, so the `Extension` machinery
//! (newext.h/cpp) and its helpers (`spreadTime`, `fastFlankTime`,
//! `calcStartDelay`, lateral start delays) are unreachable and not ported.
//! `surface_fire_expected_spread_rate` panics if `laterals > 0` is requested.

/// Fuel type entry. C++ struct: `FuelType`
#[derive(Debug, Clone, Copy)]
struct FuelType {
    rel_ros: f64, // relative spread rate 0-1
    abs_ros: f64, // actual spread rate
    fract: f64,   // fraction of landscape occupied
}

/// Elliptical fire shape dimensions derived from the length-to-breadth ratio.
///
/// C++ method: `RandThread::calcEllipticalDimensions`
/// `a` is the (relative) lateral spread rate; `b + c` is the forward rate.
#[derive(Debug, Clone, Copy)]
struct Ellipse {
    a: f64,
    b: f64,
    c: f64,
}

fn calc_elliptical_dimensions(lb_ratio: f64) -> Ellipse {
    let root = (lb_ratio * lb_ratio - 1.0).sqrt();
    let hb_ratio = (lb_ratio + root) / (lb_ratio - root);
    let a = 0.5 * (1.0 + 1.0 / hb_ratio) / lb_ratio;
    let b = (1.0 + 1.0 / hb_ratio) / 2.0;
    let c = b - 1.0 / hb_ratio;
    Ellipse { a, b, c }
}

/// Total travel time for a point-source ignition spreading at a constant
/// angle β = atan2(overlap, separation) through `num_layers` adjacent cells.
///
/// C++ method: `RandThread::calcFlankingTime`
fn calc_flanking_time(
    e: &Ellipse,
    num_layers: usize,
    separation: f64,
    overlap: f64,
    lat_dist: &[f64],
    ros: &[f64],
) -> f64 {
    let beta = overlap.atan2(separation);
    let cos_b = beta.cos();
    let sin_b = beta.sin();
    let cos_b2 = cos_b * cos_b;
    let sin_b2 = sin_b * sin_b;

    // Angle from center of ellipse. NOTE: the C++ does not clamp cos_t into
    // [-1, 1] here (unlike its spreadTime sibling); preserved as-is.
    let cos_t = (e.a * cos_b * (e.a * e.a * cos_b2 + (e.b * e.b - e.c * e.c) * sin_b2).sqrt()
        - e.b * e.c * sin_b2)
        / (e.a * e.a * cos_b2 + e.b * e.b * sin_b2);
    let theta = cos_t.acos();

    let mut travel_time = 0.0;
    for i in 0..num_layers {
        let r = e.a * theta.sin() * ros[i];
        travel_time += lat_dist[i] / r;
    }
    travel_time
}

/// One entry in the fire-path frontier. C++ struct: `PathStruct`
#[derive(Debug, Clone, Copy, Default)]
struct Path {
    loc: isize,        // column in the block
    ignition_pt: i32,  // 0=center, -1=left, 1=right
    path_time: f64,    // time for fire to reach this cell
    rel_cell_size: f64, // cumulative straight-run distance for this point
}

fn add_new_path(
    new_path: &mut [Path],
    num_path2: &mut usize,
    samples: usize,
    time: f64,
    loc: isize,
    ignition_pt: i32,
    rel_cell_size: f64,
) {
    if loc < 0 || loc > samples as isize - 1 {
        return;
    }
    new_path[*num_path2] = Path { loc, ignition_pt, path_time: time, rel_cell_size };
    *num_path2 += 1;
}

/// Computes the maximum spread rate (relative, 0-1) through each fuel
/// arrangement block and stores it in `max_ros_array`.
///
/// C++ method: `RandThread::calcSpreadPaths2`, ported for the
/// no-lateral-extension case (firstSample == 0, i.e. `Lateral == false`).
/// The C++ quirks are preserved deliberately:
/// - the "go right" bounds check compares a flat ros index against
///   `num_alloc - 1` (the *path array* capacity, samples^depths), not the
///   ros array length;
/// - `spread_rates[0]` is populated by the forward pass and reused by the
///   left/right flanking passes without being rewritten;
/// - the flanking layer count truncates `separation / cell_size`.
#[allow(clippy::needless_range_loop)]
fn calc_spread_paths2(
    samples: usize,
    depths: usize,
    lb_ratio: f64,
    cell_size: f64,
    ros_array: &[Vec<f64>],
    max_ros_array: &mut [f64],
    less_igns: usize,
) {
    let e = calc_elliptical_dimensions(lb_ratio);
    let num_alloc = (samples as f64).powi(depths as i32) as usize;
    let num_max = samples.max(depths);

    let mut sample_time = vec![0.0f64; samples];
    let mut lateral_distances = vec![0.0f64; num_max];
    let mut spread_rates = vec![0.0f64; num_max + 1];
    let mut first_path = vec![Path::default(); num_alloc];
    let mut new_path = vec![Path::default(); num_alloc];

    for i in 0..ros_array.len() {
        max_ros_array[i] = 0.0;
        for t in sample_time.iter_mut() {
            *t = 9e12;
        }
        for k in less_igns..(samples - less_igns) {
            let mut j = 0usize; // current row
            first_path[0] = Path {
                loc: k as isize,
                ignition_pt: 0, // centered (no lateral extensions)
                path_time: 0.0,
                rel_cell_size: 0.0,
            };
            let mut num_path1 = 1usize;
            let mut num_path2 = 0usize;
            let mut n = 0usize;

            while n < num_path1 {
                let cur = first_path[n];
                let parent_ros = ros_array[i][j * samples + cur.loc as usize];
                if parent_ros > 0.0 {
                    let mut separation = cell_size;
                    let mut overlap = cell_size;
                    if cur.ignition_pt == 0 {
                        overlap /= 2.0;
                    }
                    separation += cur.rel_cell_size;
                    let mut p = 0usize;
                    loop {
                        lateral_distances[p] = overlap;
                        spread_rates[p] = ros_array[i][p * samples + cur.loc as usize];
                        if separation > cell_size {
                            lateral_distances[p] = overlap / (j as f64 + 1.0);
                        }
                        p += 1;
                        if p > j {
                            break;
                        }
                    }
                    let mut delay = calc_flanking_time(
                        &e,
                        (separation / cell_size) as usize,
                        separation,
                        overlap,
                        &lateral_distances,
                        &spread_rates,
                    );
                    let parent_loc = cur.loc;
                    let parent_time = cur.path_time;
                    // go straight ahead
                    add_new_path(
                        &mut new_path,
                        &mut num_path2,
                        samples,
                        parent_time + cell_size / parent_ros,
                        parent_loc,
                        cur.ignition_pt,
                        separation,
                    );
                    if j < depths - 1 {
                        let straight_num = (cur.rel_cell_size / cell_size) as usize;
                        let mut straight_time = 0.0;
                        for p in 0..straight_num {
                            straight_time +=
                                cell_size / ros_array[i][(j - p - 1) * samples + cur.loc as usize];
                        }
                        delay += parent_time - straight_time;
                        separation = cell_size;

                        match cur.ignition_pt {
                            -1 => add_new_path(
                                &mut new_path, &mut num_path2, samples,
                                delay, parent_loc - 1, -1, 0.0,
                            ),
                            1 => add_new_path(
                                &mut new_path, &mut num_path2, samples,
                                delay, parent_loc + 1, 1, 0.0,
                            ),
                            _ => {
                                add_new_path(
                                    &mut new_path, &mut num_path2, samples,
                                    delay, parent_loc - 1, -1, 0.0,
                                );
                                add_new_path(
                                    &mut new_path, &mut num_path2, samples,
                                    delay, parent_loc + 1, 1, 0.0,
                                );
                            }
                        }

                        // go left
                        let old_overlap = overlap;
                        let old_separation = separation;
                        lateral_distances[0] = overlap;
                        for p in 1..samples {
                            let idx = (j * samples) as isize + parent_loc - p as isize;
                            if idx < 0 {
                                break;
                            }
                            spread_rates[p] = ros_array[i][idx as usize];
                        }
                        for p in 1..samples.saturating_sub(1) {
                            if cur.ignition_pt > 0 && cur.rel_cell_size == 0.0 {
                                break;
                            }
                            let idx = (j * samples) as isize + parent_loc - p as isize;
                            if idx < 0 {
                                break;
                            }
                            lateral_distances[p] = cell_size;
                            overlap += cell_size;
                            let d = calc_flanking_time(
                                &e, p + 1, separation, overlap,
                                &lateral_distances, &spread_rates,
                            );
                            add_new_path(
                                &mut new_path, &mut num_path2, samples,
                                d + parent_time, parent_loc - (p as isize + 1), -1, 0.0,
                            );
                        }

                        // go right
                        overlap = old_overlap;
                        separation = old_separation;
                        for p in 1..samples {
                            let idx = j * samples + (parent_loc + p as isize) as usize;
                            if idx > num_alloc - 1 {
                                break;
                            }
                            spread_rates[p] = ros_array[i][idx];
                        }
                        for p in 1..samples.saturating_sub(1) {
                            if cur.ignition_pt < 0 && cur.rel_cell_size == 0.0 {
                                break;
                            }
                            let idx = j * samples + (parent_loc + p as isize) as usize;
                            if idx > num_alloc - 1 {
                                break;
                            }
                            lateral_distances[p] = cell_size;
                            overlap += cell_size;
                            let d = calc_flanking_time(
                                &e, p + 1, separation, overlap,
                                &lateral_distances, &spread_rates,
                            );
                            add_new_path(
                                &mut new_path, &mut num_path2, samples,
                                d + parent_time, parent_loc + (p as isize + 1), 1, 0.0,
                            );
                        }
                    }
                } else {
                    // path dies in an unburnable cell
                    first_path[n].path_time = 0.0;
                }

                // if this is the last path in the current row
                if n == num_path1 - 1 {
                    if num_path2 > 0 {
                        num_path1 = num_path2;
                        num_path2 = 0;
                        std::mem::swap(&mut first_path, &mut new_path);
                        n = 0;
                        j += 1;
                        if j >= depths {
                            break;
                        }
                    } else {
                        break; // no paths left (C++ sets NumPath1 = 0)
                    }
                } else {
                    n += 1;
                }
            }

            // minimum exit time across all surviving final-row paths
            for path in first_path.iter().take(num_path1) {
                if path.path_time > 0.0 && path.path_time < sample_time[k] {
                    sample_time[k] = path.path_time;
                }
            }
        }
        // fastest ignition column determines the block's max spread rate
        for t in sample_time.iter() {
            if *t == 0.0 || *t == 9e12 {
                continue;
            }
            let ros = (depths as f64 * cell_size) / *t;
            if ros > max_ros_array[i] {
                max_ros_array[i] = ros;
            }
        }
    }
}

/// Builds every factorial fuel arrangement of an `n_x` × `n_y` block.
/// Returns (probability blocks, relative-spread-rate blocks), each
/// `fuels.len()^(n_x*n_y)` rows of `n_x * n_y` cells (row-major).
///
/// C++ method: `RandFuel::calcCombinations`
fn calc_combinations(n_x: usize, n_y: usize, fuels: &[FuelType]) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let f = fuels.len();
    let cols = f.pow(n_x as u32);

    // Column combinations, flat layout: `cols` combinations × n_x cells.
    let types = cols * n_x;
    let mut comb = vec![0.0f64; types];
    let mut ros = vec![0.0f64; types];
    let mut terms = 1usize;
    for i in 0..n_x {
        let mut m = 0usize;
        for fuel in fuels {
            for _ in 0..terms {
                comb[m + i] = fuel.fract;
                ros[m + i] = fuel.rel_ros;
                m += n_x;
            }
        }
        if i < n_x - 1 {
            let block = f * terms * n_x;
            for rep in 1..f {
                comb.copy_within(0..block, rep * block);
                ros.copy_within(0..block, rep * block);
            }
        }
        terms *= f;
    }

    // Block combinations: n_t arrangements of n_y rows drawn from `cols`
    // column combinations.
    let n_t = cols.pow(n_y as u32);
    let mut ca = vec![vec![0.0f64; n_x * n_y]; n_t];
    let mut ra = vec![vec![0.0f64; n_x * n_y]; n_t];
    let mut terms = 1usize;
    for i in 0..n_y {
        let mut m = 0usize;
        for j in 0..cols {
            for _ in 0..terms {
                ca[m][i * n_x..(i + 1) * n_x].copy_from_slice(&comb[j * n_x..(j + 1) * n_x]);
                ra[m][i * n_x..(i + 1) * n_x].copy_from_slice(&ros[j * n_x..(j + 1) * n_x]);
                m += 1;
            }
        }
        let n_limit = cols * m;
        if i < n_y - 1 {
            let q = m;
            while m < n_limit {
                for p in 0..q {
                    ca[m] = ca[p].clone();
                    ra[m] = ra[p].clone();
                    m += 1;
                }
            }
        }
        terms *= cols;
    }

    (ca, ra)
}

/// Expected (probability-weighted mean) *relative* spread rate over all
/// factorial fuel arrangements, together with the maximum absolute spread
/// rate of the fuel set (multiply the two for the absolute expected rate).
///
/// C++ method: `RandFuel::computeSpread2`, no-extension branch.
fn compute_spread2(
    fuels: &mut [FuelType],
    samples: usize,
    depths: usize,
    lb_ratio: f64,
    less_igns: usize,
) -> (f64, f64) {
    if samples < 1 || samples > 50 {
        return (0.0, 0.0);
    }

    let mut max_ros: f64 = 0.0;
    for fuel in fuels.iter() {
        max_ros = max_ros.max(fuel.abs_ros);
    }
    for fuel in fuels.iter_mut() {
        fuel.rel_ros = fuel.abs_ros / max_ros;
    }

    const CELL_SIZE: f64 = 10.0; // C++: RandFuel cell size ("irrelevant, but he sets it anyway")

    let (comb_array, ros_array) = calc_combinations(samples, depths, fuels);
    let combs = comb_array.len();
    let mut max_ros_array = vec![0.0f64; combs];
    calc_spread_paths2(
        samples, depths, lb_ratio, CELL_SIZE,
        &ros_array, &mut max_ros_array, less_igns,
    );

    let mut average = 0.0;
    for i in 0..combs {
        let mut prob = 1.0;
        for cell in 0..depths * samples {
            prob *= comb_array[i][cell];
        }
        average += max_ros_array[i] * prob;
    }

    (average, max_ros)
}

/// Finney's two-dimensional expected surface fire spread rate for fuels
/// randomly arranged in proportions `cov`, with spread rates `ros`.
///
/// C++ method: `SurfaceTwoFuelModels::surfaceFireExpectedSpreadRate`
/// (which normalizes `cov` in place; here a local copy is normalized).
pub fn surface_fire_expected_spread_rate(
    ros: &[f64],
    cov: &[f64],
    lb_ratio: f64,
    samples: usize,
    depth: usize,
    laterals: usize,
) -> f64 {
    assert!(
        laterals == 0,
        "EXRATE lateral extensions (newext) are not ported; laterals must be 0"
    );

    let total_cov: f64 = cov.iter().sum();
    if total_cov <= 0.0 {
        return 0.0;
    }

    let mut fuels: Vec<FuelType> = ros
        .iter()
        .zip(cov.iter())
        .map(|(&r, &c)| FuelType { rel_ros: -1.0, abs_ros: r, fract: c / total_cov })
        .collect();

    let (expected_rel, max_ros) = compute_spread2(&mut fuels, samples, depth, lb_ratio, 0);
    expected_rel * max_ros
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uniform fuel: expected rate must equal that fuel's rate.
    #[test]
    fn uniform_fuel_is_identity() {
        let rate = surface_fire_expected_spread_rate(&[7.5, 20.0], &[0.0, 1.0], 2.0, 2, 2, 0);
        assert!((rate - 20.0).abs() < 1e-9, "got {rate}");
    }

    /// Expected rate is bounded by the harmonic mean below and the fastest
    /// fuel above, and is monotone in coverage. (It can exceed the
    /// *arithmetic* mean: fire exploits fast-fuel corridors, e.g. the
    /// BehavePlus golden case at 10% coverage gives 10.47 vs 10.18 ch/hr.)
    #[test]
    fn expected_rate_bounded_and_monotone() {
        let ros = [5.0, 20.0];
        let mut last = f64::INFINITY;
        for cov1 in [0.1, 0.3, 0.5, 0.7, 0.9] {
            let cov = [cov1, 1.0 - cov1];
            let expected = surface_fire_expected_spread_rate(&ros, &cov, 2.0, 2, 2, 0);
            let harmonic = 1.0 / (cov[0] / ros[0] + cov[1] / ros[1]);
            assert!(
                expected >= harmonic - 1e-9 && expected <= ros[1] + 1e-9,
                "cov {cov1}: expected {expected} not in [{harmonic}, {}]",
                ros[1]
            );
            // ros[0] < ros[1], so more of fuel 0 means slower spread
            assert!(expected < last, "not monotone at cov {cov1}");
            last = expected;
        }
    }

    /// A block containing an unburnable fuel still spreads around it.
    #[test]
    fn unburnable_fuel_slows_but_does_not_stop() {
        let rate = surface_fire_expected_spread_rate(&[0.0, 10.0], &[0.5, 0.5], 2.0, 2, 2, 0);
        assert!(rate > 0.0 && rate < 10.0, "got {rate}");
    }
}
