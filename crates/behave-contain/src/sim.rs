//! Multi-flank containment simulation.
//!
//! C++ source: ContainSim.h / ContainSim.cpp (namespace Sem)
//!
//! ContainSim controls multiple simulation passes to achieve desired
//! precision (step count range), retries after overrun, and accumulates
//! perimeter coordinates at each simulation step.

use std::f64::consts::PI;

use crate::algorithm::{Contain, ContainStatus, ContainTactic};
use crate::force::ContainForce;
use crate::resource::ContainFlank;

/// Runs the containment algorithm on both flanks simultaneously.
/// Currently only the left flank is simulated; the right flank is
/// a mirror image.
#[derive(Debug)]
pub struct ContainSim {
    // Output statistics
    final_cost: f64,
    final_line: f64,
    final_perim: f64,
    final_size: f64,
    final_sweep: f64,
    final_time: f64,
    x_max: f64,
    x_min: f64,
    y_max: f64,

    // Simulation arrays
    arr_u: Vec<f64>,
    arr_h: Vec<f64>,
    arr_x: Vec<f64>,
    arr_y: Vec<f64>,
    arr_a: Vec<f64>,
    arr_p: Vec<f64>,

    // Left flank Contain object
    left: Contain,

    // Force reference
    force: ContainForce,

    // Simulation parameters
    min_steps: i32,
    max_steps: i32,
    size: i32,
    pass: i32,
    used: i32,
    retry: bool,
    max_fire_size: i32,
    max_fire_time: i32,
}

impl ContainSim {
    pub fn new(
        report_size: f64,
        report_rate: f64,
        diurnal_ros: &[f64; 24],
        fire_start_minutes: i32,
        lw_ratio: f64,
        force: ContainForce,
        tactic: ContainTactic,
        attack_dist: f64,
        retry: bool,
        min_steps: i32,
        mut max_steps: i32,
        max_fire_size: i32,
        max_fire_time: i32,
    ) -> Self {
        if max_steps < 10 {
            max_steps = 10;
        }

        // Distance step is approximately 1 minute regardless of steps
        let dist_step = report_rate / 60.0;

        // Attack at first resource arrival
        let attack_time = force.first_arrival(ContainFlank::LeftFlank);

        // Create left flank
        let left = Contain::new(
            report_size,
            report_rate,
            diurnal_ros,
            fire_start_minutes,
            lw_ratio,
            dist_step,
            ContainFlank::LeftFlank,
            force.clone(),
            attack_time,
            tactic,
            attack_dist,
        );

        let size = max_steps + 1;
        let sz = size as usize;

        Self {
            final_cost: 0.0,
            final_line: 0.0,
            final_perim: 0.0,
            final_size: 0.0,
            final_sweep: 0.0,
            final_time: 0.0,
            x_max: 0.0,
            x_min: 0.0,
            y_max: 0.0,
            arr_u: vec![0.0; sz],
            arr_h: vec![0.0; sz],
            arr_x: vec![0.0; sz],
            arr_y: vec![0.0; sz],
            arr_a: vec![0.0; sz],
            arr_p: vec![0.0; sz],
            left,
            force,
            min_steps,
            max_steps,
            size,
            pass: 0,
            used: 0,
            retry,
            max_fire_size,
            max_fire_time,
        }
    }

    /// Run the simulation to completion.
    pub fn run(&mut self) {
        let mut rerun = true;
        let mut maxsteps_exceeded = false;
        self.pass = 0;

        while rerun {
            // Initialize first step
            let i_left_start = 0;
            self.arr_u[i_left_start] = self.left.u;
            self.arr_h[i_left_start] = self.left.h;
            self.arr_x[i_left_start] = self.left.x;
            self.arr_y[i_left_start] = self.left.y;

            let mut _elapsed = self.left.attack_time;

            // Main simulation loop
            self.final_sweep = 0.0;
            self.final_line = 0.0;
            self.final_perim = 0.0;
            let mut total_area = 0.0_f64;
            let mut suma = 0.0_f64;
            let mut sumb = 0.0_f64;
            let mut i_left = 0_usize;

            while self.left.status != ContainStatus::Overrun
                && self.left.status != ContainStatus::Contained
                && self.left.step < self.max_steps
                && total_area < self.max_fire_size as f64
                && self.left.current_time < self.max_fire_time as f64
                && self.left.current_time < self.left.exhausted
            {
                self.left.do_step();

                i_left += 1;
                if i_left >= self.arr_u.len() {
                    break;
                }

                self.arr_u[i_left] = self.left.u;
                self.arr_h[i_left] = self.left.h;
                self.arr_x[i_left] = self.left.x;
                self.arr_y[i_left] = self.left.y;
                _elapsed = self.left.current_time;

                // Update extent
                if self.arr_x[i_left] < self.x_min {
                    self.x_min = self.arr_x[i_left];
                }
                if self.arr_x[i_left] > self.x_max {
                    self.x_max = self.arr_x[i_left];
                }
                if self.arr_y[i_left] > self.y_max {
                    self.y_max = self.arr_y[i_left];
                }

                // Line constructed and area swept
                let dy = (self.arr_y[i_left - 1] - self.arr_y[i_left]).abs();
                let dx = (self.arr_x[i_left - 1] - self.arr_x[i_left]).abs();
                self.arr_p[i_left - 1] = (dy * dy + dx * dx).sqrt();
                self.final_line += 2.0 * self.arr_p[i_left - 1];

                // Accumulate area (shoelace / trapezoidal rule)
                suma += self.arr_y[i_left - 1] * self.arr_x[i_left];
                sumb += self.arr_x[i_left - 1] * self.arr_y[i_left];

                // Trapezoidal rule for area
                let mut sum_dt = 0.0;
                for j in 1..=i_left {
                    sum_dt += (self.arr_x[j] - self.arr_x[j - 1])
                        * (self.arr_y[j] + self.arr_y[j - 1]);
                }
                if sum_dt < 0.0 {
                    sum_dt = -sum_dt;
                }
                let mut area = sum_dt * 0.5;

                // Add uncontained portion of the fire ellipse (DT 1/2013)
                let uc_area = Self::uncontained_area(
                    self.arr_h[i_left],
                    self.left.fire_lw_ratio_at_report(),
                    self.arr_x[i_left],
                    self.arr_y[i_left],
                    self.left.tactic(),
                );
                area += uc_area;

                // Accumulate area for BOTH flanks (ac)
                self.arr_a[i_left - 1] = 0.2 * area;
                total_area = self.arr_a[i_left - 1];
            }

            // BEHAVEPLUS FIX: Adjust last x for contained head attacks
            if self.left.status == ContainStatus::Contained
                && self.left.tactic == ContainTactic::HeadAttack
            {
                let step_idx = self.left.step as usize;
                if step_idx < self.arr_x.len() {
                    self.arr_x[step_idx] -= 2.0 * self.left.attack_dist;
                }
            }

            let step_idx = self.left.step as usize;
            if step_idx < self.arr_y.len() {
                suma += self.arr_y[step_idx] * self.arr_x[0];
                sumb += self.arr_x[step_idx] * self.arr_y[0];
            }
            self.final_sweep = if suma > sumb {
                0.5 * (suma - sumb)
            } else {
                0.5 * (sumb - suma)
            };
            self.final_sweep *= 0.20;

            // Recalculate with trapezoidal rule
            let step_idx = self.left.step as usize;
            let mut sum_dt = 0.0;
            for j in 1..=step_idx.min(i_left) {
                sum_dt += (self.arr_x[j] - self.arr_x[j - 1])
                    * (self.arr_y[j] + self.arr_y[j - 1]);
            }
            if sum_dt < 0.0 {
                sum_dt = -sum_dt;
            }
            let mut area = sum_dt * 0.5;

            // Add uncontained area
            let si = step_idx.min(self.arr_h.len() - 1);
            let uc_area = Self::uncontained_area(
                self.arr_h[si],
                self.left.fire_lw_ratio_at_report(),
                self.arr_x[si],
                self.arr_y[si],
                self.left.tactic(),
            );
            area += uc_area;
            self.final_sweep = 0.2 * area;

            // Determine next action based on status
            if self.left.status == ContainStatus::Overrun {
                // Case 1: No retry
                if !self.retry {
                    rerun = false;
                }
                // Case 2: Try later attack time
                else if {
                    let at = self.force.next_arrival(
                        self.left.attack_time,
                        self.left.exhausted,
                        ContainFlank::LeftFlank,
                    );
                    at > 0.01
                } {
                    let at = self.force.next_arrival(
                        self.left.attack_time,
                        self.left.exhausted,
                        ContainFlank::LeftFlank,
                    );
                    self.pass += 1;
                    self.left.attack_time = at;
                    self.left.reset();
                    rerun = true;
                }
                // Case 3: All exhausted
                else {
                    rerun = false;
                    self.left.status = ContainStatus::Exhausted;
                }
            } else if self.left.current_time >= self.left.exhausted {
                // Resources exhausted (DT 7/8/10)
                rerun = false;
                self.left.status = ContainStatus::Exhausted;
            } else if i_left >= self.max_steps as usize {
                // Case 4: Max steps exceeded — increase dist step
                let factor = 2.0;
                self.left.dist_step *= factor;
                self.pass += 1;

                if !maxsteps_exceeded {
                    self.left.reset();
                    rerun = true;
                } else {
                    rerun = false;
                }
                maxsteps_exceeded = true;
            } else if self.left.status == ContainStatus::Contained {
                // Case 5: Too few steps — decrease dist step
                if i_left < self.min_steps as usize && !maxsteps_exceeded {
                    let factor = 0.5;
                    self.left.dist_step *= factor;
                    self.pass += 1;
                    self.left.reset();
                    rerun = true;
                }
                // Case 6: Contained within step range
                else {
                    rerun = false;
                }
            } else if total_area >= self.max_fire_size as f64 {
                // Size limit exceeded
                rerun = false;
                self.left.status = ContainStatus::SizeLimitExceeded;
            } else if self.left.current_time > (self.max_fire_time - 1) as f64 {
                // Time limit exceeded (MAF 6/2010)
                self.left.current_time = self.max_fire_time as f64;
                self.left.status = ContainStatus::TimeLimitExceeded;
                rerun = false;
            } else {
                // Unknown condition — rerun
                rerun = true;
            }
        }

        // Post-simulation time limit check (MAF 6/2010)
        if self.left.current_time > (self.max_fire_time - 1) as f64 {
            self.left.current_time = self.max_fire_time as f64;
            self.left.status = ContainStatus::TimeLimitExceeded;
        }

        // Final statistics
        self.final_stats();
    }

    fn final_stats(&mut self) {
        self.final_time = self.left.current_time;
        self.final_perim = self.final_line;
        self.final_size = 0.0;

        if self.left.status == ContainStatus::Contained {
            self.final_size = self.final_sweep;
        } else {
            self.final_size = self.final_sweep;
            self.final_perim = 0.0;
            self.final_sweep = 0.0;
        }

        // Resources used
        for res in 0..self.force.num_resources() {
            if self.force.resource_arrival(res) < self.final_time {
                self.used += 1;
                self.final_cost += self.force.resource_cost(res, self.final_time);
            }
        }
    }

    /// Calculate the area of the uncontained portion of the fire ellipse.
    /// By DT 1/2013.
    fn uncontained_area(
        head: f64,
        mut lw_ratio: f64,
        x: f64,
        mut y: f64,
        tactic: ContainTactic,
    ) -> f64 {
        if lw_ratio < 1.0 {
            lw_ratio = 1.00000001;
        }

        let ecc = (1.0 - 1.0 / (lw_ratio * lw_ratio)).sqrt();
        let a = head / (1.0 + ecc);
        let b = a / lw_ratio;

        // Angle from center of ellipse
        let mut xcenter = x - a * ecc;

        if xcenter < -a {
            xcenter = -a;
            y = 0.0;
        }
        if xcenter > a {
            xcenter = a;
            y = 0.0;
        }

        let r = (xcenter * xcenter + y * y).sqrt();
        let theta = if xcenter >= 0.0 {
            if r > 0.0 {
                (xcenter / r).acos()
            } else {
                0.0
            }
        } else {
            let theta_a = (-xcenter / r).acos();
            PI - theta_a
        };

        let sintheta = (2.0 * theta).sin();
        let costheta = (2.0 * theta).cos();

        let arctan_of = ((b - a) * sintheta) / ((a + b) + (b - a) * costheta);
        let ellsector = (a * b / 2.0) * (theta - arctan_of.atan());

        let mut area = ellsector - (xcenter * y / 2.0);

        if tactic == ContainTactic::HeadAttack {
            area = PI * a * b / 2.0 - area;
        }

        if area < 0.0 {
            area = 0.0;
        }

        area
    }

    // --- Public accessors ---

    pub fn status(&self) -> ContainStatus {
        self.left.status()
    }

    pub fn tactic(&self) -> ContainTactic {
        self.left.tactic()
    }

    pub fn final_fire_cost(&self) -> f64 {
        self.final_cost
    }

    pub fn final_fire_line(&self) -> f64 {
        self.final_line
    }

    pub fn final_fire_perimeter(&self) -> f64 {
        self.final_perim
    }

    pub fn final_fire_size(&self) -> f64 {
        self.final_size
    }

    pub fn final_fire_sweep(&self) -> f64 {
        self.final_sweep
    }

    pub fn final_fire_time(&self) -> f64 {
        self.final_time
    }

    pub fn final_resources_used(&self) -> i32 {
        self.used
    }

    pub fn fire_head_at_report(&self) -> f64 {
        self.left.fire_head_at_report()
    }

    pub fn fire_back_at_report(&self) -> f64 {
        self.left.fire_back_at_report()
    }

    pub fn fire_head_at_attack(&self) -> f64 {
        self.left.fire_head_at_attack()
    }

    pub fn fire_back_at_attack(&self) -> f64 {
        self.left.fire_back_at_attack()
    }

    pub fn fire_lw_ratio_at_report(&self) -> f64 {
        self.left.fire_lw_ratio_at_report()
    }

    pub fn fire_size_at_report(&self) -> f64 {
        self.left.fire_size_at_report()
    }

    pub fn fire_spread_rate_at_report(&self) -> f64 {
        self.left.fire_spread_rate_at_report()
    }

    pub fn fire_spread_rate_at_back(&self) -> f64 {
        self.left.fire_spread_rate_at_back()
    }

    pub fn fire_report_time(&self) -> f64 {
        self.left.fire_report_time()
    }

    pub fn fire_perimeter_x(&self) -> &[f64] {
        &self.arr_x
    }

    pub fn fire_perimeter_y(&self) -> &[f64] {
        &self.arr_y
    }

    pub fn fire_points(&self) -> usize {
        self.size as usize
    }

    pub fn attack_distance(&self) -> f64 {
        self.left.attack_distance()
    }

    pub fn attack_time(&self) -> f64 {
        self.left.attack_time()
    }

    pub fn distance_step(&self) -> f64 {
        self.left.distance_step()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::ContainResource;

    #[test]
    fn sim_basic_containment() {
        let mut force = ContainForce::new();
        force.add_resource(ContainResource::new(
            2.0, 20.0, 480.0,
            ContainFlank::LeftFlank, "test", 0.0, 0.0,
        ));
        let diurnal = [5.0; 24];
        let mut sim = ContainSim::new(
            1.0, 5.0, &diurnal, 0, 3.0,
            force, ContainTactic::HeadAttack, 0.0,
            true, 250, 1000, 1000, 1080,
        );
        sim.run();
        assert_eq!(sim.status(), ContainStatus::Contained);
        assert!(sim.final_fire_line() > 0.0);
        assert!(sim.final_fire_time() > 0.0);
    }
}
