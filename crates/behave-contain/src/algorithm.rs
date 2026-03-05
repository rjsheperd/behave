//! Core containment algorithm (single-flank simulation).
//!
//! C++ source: Contain.h / Contain.cpp (namespace Sem)
//!
//! Implements the Fried & Fried wildfire containment model.
//! The fire is assumed to grow as an ellipse under uniform conditions.
//! Containment forces attack one flank; the other is a mirror image.

use std::f64::consts::PI;

use crate::force::ContainForce;
use crate::resource::ContainFlank;

/// Attack tactic for containment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainTactic {
    HeadAttack = 0,
    RearAttack = 1,
}

/// Status of the containment simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainStatus {
    Unreported = 0,
    Reported = 1,
    Attacked = 2,
    Contained = 3,
    Overrun = 4,
    Exhausted = 5,
    Overflow = 6,
    SizeLimitExceeded = 7,
    TimeLimitExceeded = 8,
}

/// Single-flank containment algorithm.
///
/// Contains the core simulation methods needed to make a single simulation
/// pass for a single flank. The fire grows as an ellipse with constant
/// length-to-width ratio.
#[derive(Debug, Clone)]
pub struct Contain {
    // Input variables
    pub(crate) report_size: f64,    // Fire size at time of report (ac)
    pub(crate) report_rate: f64,    // Fire spread rate at time of report (ch/h)
    pub(crate) lw_ratio: f64,       // Fire length-to-width ratio
    pub(crate) attack_dist: f64,    // Parallel attack distance from fire (ch)
    pub(crate) attack_time: f64,    // Initial attack time (min since report)
    pub(crate) dist_step: f64,      // Simulation fire head distance step (ch)
    pub(crate) flank: ContainFlank, // Apply resources from this flank
    pub(crate) tactic: ContainTactic,
    force: ContainForce,

    // Diurnal spread rates (hourly, 24 values, ch/h)
    diurnal_spread_rate: [f64; 24],
    start_time: i32, // Fire start time (minutes since midnight)

    // Calculated intermediates
    eps: f64,          // Fire eccentricity
    eps2: f64,         // Fire eccentricity squared
    a: f64,            // sqrt((1-eps)/(1+eps))
    pub(crate) report_head: f64,   // Fire head position at report time (ch)
    report_time: f64,  // Elapsed time from fire start to report (min)
    pub(crate) current_time: f64,  // Total current time
    back_rate: f64,    // Fire backing spread rate (ch/h)
    report_back: f64,  // Fire back position at report time (ch)
    pub(crate) attack_head: f64,   // Fire head position at first attack (ch)
    pub(crate) attack_back: f64,   // Fire back position at first attack (ch)
    rkpr: [f64; 3],    // Runge-Kutta production rates
    pub(crate) exhausted: f64,     // Time after report when all forces exhausted
    time: f64,         // Simulation time (minutes since report)
    pub(crate) step: i32,          // Simulation step
    current_time_at_fire_head: f64,
    time_increment: f64,

    // Output variables
    pub(crate) u: f64,   // Angle to point of active line building
    u0: f64,             // Previous angle
    pub(crate) h: f64,   // Distance towards head at each time step
    h0: f64,             // Previous distance
    pub(crate) x: f64,   // Current attack point x-coordinate (ch)
    pub(crate) y: f64,   // Current attack point y-coordinate (ch)
    pub(crate) status: ContainStatus,
}

impl Contain {
    pub fn new(
        report_size: f64,
        report_rate: f64,
        diurnal_ros: &[f64; 24],
        fire_start_minutes: i32,
        lw_ratio: f64,
        dist_step: f64,
        flank: ContainFlank,
        force: ContainForce,
        attack_time: f64,
        tactic: ContainTactic,
        attack_dist: f64,
    ) -> Self {
        let mut c = Self {
            report_size,
            report_rate,
            lw_ratio: if lw_ratio < 1.0 { 1.0 } else { lw_ratio },
            attack_dist,
            attack_time,
            dist_step: 0.01,
            flank,
            tactic,
            force,
            diurnal_spread_rate: [0.0; 24],
            start_time: fire_start_minutes,
            eps: 1.0,
            eps2: 1.0,
            a: 1.0,
            report_head: 0.0,
            report_time: 0.0,
            current_time: 0.0,
            back_rate: 0.0,
            report_back: 0.0,
            attack_head: 0.0,
            attack_back: 0.0,
            rkpr: [0.0; 3],
            exhausted: 0.0,
            time: 0.0,
            step: 0,
            current_time_at_fire_head: 0.0,
            time_increment: 0.0,
            u: 0.0,
            u0: 0.0,
            h: 0.0,
            h0: 0.0,
            x: 0.0,
            y: 0.0,
            status: ContainStatus::Unreported,
        };

        // Set report params
        c.set_report(report_size, report_rate, lw_ratio, dist_step);
        // Set attack params
        c.set_attack(flank, attack_time, tactic, attack_dist);
        // Set diurnal spread rates
        c.diurnal_spread_rate.copy_from_slice(diurnal_ros);
        // Initialize intermediates
        c.reset();
        c
    }

    fn set_report(&mut self, report_size: f64, report_rate: f64, lw_ratio: f64, dist_step: f64) {
        self.report_size = report_size;
        self.report_rate = report_rate;
        self.dist_step = dist_step;
        self.lw_ratio = if lw_ratio < 1.0 { 1.0 } else { lw_ratio };

        // Initialize diurnal spread rates to report rate
        for i in 0..24 {
            self.diurnal_spread_rate[i] = report_rate;
        }
    }

    fn set_attack(
        &mut self,
        flank: ContainFlank,
        attack_time: f64,
        tactic: ContainTactic,
        attack_dist: f64,
    ) {
        self.flank = flank;
        self.attack_time = attack_time;
        self.tactic = tactic;
        self.attack_dist = attack_dist;
        self.exhausted = self.force.exhausted(self.flank);
    }

    /// Initialize the Contain state from current parameter values.
    pub(crate) fn reset(&mut self) {
        self.current_time_at_fire_head = 0.0;
        self.time_increment = 0.0;
        self.current_time = self.attack_time;

        // Eccentricity
        let r = 1.0 / self.lw_ratio;
        self.eps2 = 1.0 - (r * r);
        self.eps = if self.eps2 > 0.00001 {
            self.eps2.sqrt()
        } else {
            0.0
        };
        self.a = ((1.0 - self.eps) / (1.0 + self.eps)).sqrt();

        // Fire head position at time of report (ch)
        let ch2 = 10.0 * self.report_size;
        self.report_head =
            (1.0 + self.eps) * (ch2 / (PI * (1.0 - self.eps2).sqrt())).sqrt();

        // Elapsed time from fire start to time of report (min)
        if self.report_rate > 0.0001 {
            self.report_time = 60.0 * self.report_head / self.report_rate;
        }

        // Fire backing spread rate (ch/h)
        self.back_rate = self.report_rate * (1.0 - self.eps) / (1.0 + self.eps);

        // Fire tail position at time of report (ch)
        self.report_back = self.back_rate * self.report_time / 60.0;

        // Recalculate report_back matching C++ (MAF 9/2010)
        self.report_back = self.report_head * (1.0 - self.eps) / (1.0 + self.eps);

        // Fire head position at first attack
        self.attack_head = self.head_position(self.attack_time);

        // Fire back position at first attack (MAF 6/2010)
        self.attack_back = self.attack_head * (1.0 - self.eps) / (1.0 + self.eps);

        // Initial angle and position depend on tactic
        if self.tactic == ContainTactic::RearAttack {
            self.u = PI;
            self.u0 = PI;
            self.x = -self.attack_back - self.attack_dist;
        } else {
            self.u = 0.0;
            self.u0 = 0.0;
            self.x = self.attack_head + self.attack_dist;
        }
        self.h = self.attack_head;
        self.h0 = self.attack_head;
        self.y = 0.0;

        // Initialization
        self.step = 0;
        self.time = 0.0;
        self.rkpr = [0.0; 3];
        self.status = ContainStatus::Reported;
    }

    /// Determines the next value of the angle (u) and fire head position (h)
    /// using 4th-order Runge-Kutta integration.
    pub(crate) fn calc_u(&mut self) {
        self.u0 = self.u;
        self.h0 = self.h;
        self.status = ContainStatus::Attacked;

        let old_dist_step = self.dist_step;

        // Force timestep to recognize 1-minute resource arrivals (MAF 6/2010)
        loop {
            self.time_increment = 0.0;
            self.rkpr[0] = if self.step != 0 {
                self.rkpr[2]
            } else {
                self.production_ratio(self.attack_head)
            };
            self.time_increment /= 2.0;
            self.rkpr[1] = self.production_ratio(self.h0 + 0.5 * self.dist_step);
            self.rkpr[2] = self.production_ratio(self.h0 + self.dist_step);
            if self.time_increment > 1.0 {
                self.dist_step /= 2.0;
                continue;
            }
            break;
        }

        // First RK constant
        let mut deriv = 0.0;
        if !self.calc_uh(self.rkpr[0], self.h0, self.u0, &mut deriv) {
            self.dist_step = old_dist_step;
            return;
        }
        let rk0 = self.dist_step * deriv;

        // Second RK constant
        if !self.calc_uh(
            self.rkpr[1],
            self.h0 + 0.5 * self.dist_step,
            self.u0 + 0.5 * rk0,
            &mut deriv,
        ) {
            self.dist_step = old_dist_step;
            return;
        }
        let rk1 = self.dist_step * deriv;

        // Third RK constant
        if !self.calc_uh(
            self.rkpr[1],
            self.h0 + 0.5 * self.dist_step,
            self.u0 + 0.5 * rk1,
            &mut deriv,
        ) {
            self.dist_step = old_dist_step;
            return;
        }
        let rk2 = self.dist_step * deriv;

        // Fourth RK constant
        if !self.calc_uh(
            self.rkpr[2],
            self.h0 + self.dist_step,
            self.u0 + rk2,
            &mut deriv,
        ) {
            self.dist_step = old_dist_step;
            return;
        }
        let rk3 = self.dist_step * deriv;

        // 4th order Runge-Kutta approximation of u
        self.u = self.u0 + (rk0 + rk3 + 2.0 * (rk1 + rk2)) / 6.0;

        // Calculate next free-burning fire head position (MAF 6/2010)
        if self.step == 0 {
            self.h = self.attack_head;
        }
        self.h += self.dist_step;

        self.dist_step = old_dist_step;
    }

    /// Determines du/dh for a particular u, h, and production ratio p.
    /// Returns false if forces are overrun (negative radical).
    fn calc_uh(&mut self, p: f64, h: f64, u: f64, d: &mut f64) -> bool {
        let cos_u = u.cos();
        let sin_u = u.sin();
        *d = 0.0;

        // Check radical sign
        let x = 1.0 - self.eps * cos_u;
        let mut uh_radical = (p * p * x / (1.0 + self.eps * cos_u)) - self.a * self.a;

        if uh_radical <= 1.0e-10 {
            uh_radical = 0.0; // MAF 6/2010: don't overrun, just zero it
        }

        // Change in fire perimeter distance at point of attack
        let dh = if self.attack_dist > 0.001 {
            x * (h
                + (1.0 - self.eps)
                    * (self.attack_dist * (1.0 - self.eps2).sqrt()
                        / (1.0 - self.eps2 * cos_u * cos_u).powf(1.5)))
        } else {
            x * h
        };

        // Change in angle of attack point
        let du = if self.tactic == ContainTactic::RearAttack {
            self.eps * sin_u - (1.0 + self.eps) * uh_radical.sqrt()
        } else {
            self.eps * sin_u + (1.0 + self.eps) * uh_radical.sqrt()
        };

        let uh = du / dh;
        *d = uh;
        true
    }

    /// Determines the x,y coordinates for the current angle and head position.
    pub(crate) fn calc_coordinates(&mut self) {
        self.y = self.u.sin() * self.h * self.a;
        self.x = (self.u.cos() + self.eps) * self.h / (1.0 + self.eps);
        if self.attack_dist > 0.001 {
            let psi_val = self.contain_psi(self.u, self.eps2);
            self.y += self.attack_dist * psi_val.sin();
            self.x += self.attack_dist * psi_val.cos();
        }
    }

    /// Psi function for parallel attack coordinate translation.
    fn contain_psi(&self, mut u: f64, eps2: f64) -> f64 {
        let ro = u - PI / 2.0;
        if ro.abs() < 0.00001 {
            if ro > 0.0 {
                u = PI / 2.0 + 0.00001;
            } else {
                u = PI / 2.0 - 0.00001;
            }
        }
        let mut psi_val = (u.sin() / u.cos() / (1.0 - eps2).sqrt()).atan();
        if psi_val < 0.0 {
            psi_val += PI;
        }
        psi_val
    }

    /// Fire head position at the specified time since report (ch from origin).
    /// Uses hourly diurnal spread rates with hour-by-hour integration.
    pub(crate) fn head_position(&self, minutes_since_report: f64) -> f64 {
        let mut add_head_dist = self.report_head;

        // Deal with partial hour in first hour since discovery (DT 4/17/12)
        let time_remaining_hr =
            60.0 - (self.start_time as f64 - 60.0 * (self.start_time as f64 / 60.0).floor());
        let time_remaining_hr = if minutes_since_report < time_remaining_hr {
            minutes_since_report
        } else {
            time_remaining_hr
        };
        add_head_dist += self.get_diurnal_spread_rate(0.0) * time_remaining_hr / 60.0;
        let minutes_since_report = minutes_since_report - time_remaining_hr;
        let mut minutes_cumulative = time_remaining_hr;

        let num_hours = (minutes_since_report / 60.0) as i64 + 1;
        let minutes_remainder = minutes_since_report - ((num_hours - 1) as f64 * 60.0);

        for i in 0..num_hours {
            let minutes_increment = if i < num_hours - 1 {
                60.0
            } else {
                minutes_remainder
            };
            add_head_dist +=
                self.get_diurnal_spread_rate(minutes_cumulative) * minutes_increment / 60.0;
            minutes_cumulative += minutes_increment;
        }

        add_head_dist
    }

    /// Aggregate fireline production rate at the specified fire head position.
    fn production_rate_at(&self, _fire_head_position: f64) -> f64 {
        let minutes_since_report = self.current_time_at_fire_head + self.attack_time;
        self.force.production_rate(minutes_since_report, self.flank)
    }

    /// Ratio of aggregate production rate to fire head spread rate.
    fn production_ratio(&mut self, _fire_head_position: f64) -> f64 {
        let minutes_since_report =
            self.current_time_at_fire_head + self.attack_time + self.time_increment;
        let prod = self.force.production_rate(minutes_since_report, self.flank);
        let mut fire = self.get_diurnal_spread_rate(minutes_since_report);
        if fire < 0.0001 {
            fire = 0.0001;
        }

        // MAF 6/2010: minutes = ch / (ch/hr) * 60
        self.time_increment = self.dist_step / fire * 60.0;

        prod / fire
    }

    /// Get diurnal spread rate for the specified time since report.
    fn get_diurnal_spread_rate(&self, minutes_since_report: f64) -> f64 {
        let mut current_time = minutes_since_report + self.start_time as f64;
        while current_time >= 1440.0 {
            current_time -= 1440.0;
        }
        let mut current_hour = (current_time / 60.0) as usize;
        if current_hour > 23 {
            current_hour = 23;
        }
        self.diurnal_spread_rate[current_hour]
    }

    /// Perform one containment simulation step.
    pub(crate) fn do_step(&mut self) -> ContainStatus {
        // Determine next angle and fire head position
        self.calc_u();

        // Increment step counter
        self.step += 1;

        // Determine elapsed time since fire was reported
        self.time = self.time_since_report(self.h);

        // If forces were overrun, return
        if self.status == ContainStatus::Overrun {
            return self.status;
        }

        // If the forces contain the fire, interpolate final u and h
        if self.tactic == ContainTactic::HeadAttack && self.u >= PI {
            self.status = ContainStatus::Contained;
            self.h = self.h0 - self.dist_step * self.u0 / (self.u0 + self.u.abs());
            self.u = PI;
        } else if self.tactic == ContainTactic::RearAttack && self.u <= 0.0 {
            self.status = ContainStatus::Contained;
            self.h = self.h0 + self.dist_step * self.u0 / (self.u0 + self.u.abs());
            self.u = 0.0;
        }

        // Determine x and y coordinates
        self.calc_coordinates();

        // MAF 6/2010
        self.current_time_at_fire_head += self.time_increment;
        self.current_time = self.current_time_at_fire_head + self.attack_time;

        self.status
    }

    /// Time since fire report at which the free-burning fire head reaches
    /// the specified distance from fire origin (min).
    fn time_since_report(&self, head_pos: f64) -> f64 {
        if self.report_rate > 0.00001 {
            60.0 * (head_pos - self.report_head) / self.report_rate
        } else {
            0.0
        }
    }

    // --- Public accessors ---

    pub fn status(&self) -> ContainStatus {
        self.status
    }

    pub fn tactic(&self) -> ContainTactic {
        self.tactic
    }

    pub fn attack_distance(&self) -> f64 {
        self.attack_dist
    }

    pub fn attack_time(&self) -> f64 {
        self.attack_time
    }

    pub fn distance_step(&self) -> f64 {
        self.dist_step
    }

    pub fn fire_back_at_attack(&self) -> f64 {
        self.attack_back
    }

    pub fn fire_back_at_report(&self) -> f64 {
        self.report_back
    }

    pub fn fire_head_at_attack(&self) -> f64 {
        self.attack_head
    }

    pub fn fire_head_at_report(&self) -> f64 {
        self.report_head
    }

    pub fn fire_lw_ratio_at_report(&self) -> f64 {
        self.lw_ratio
    }

    pub fn fire_report_time(&self) -> f64 {
        self.report_time
    }

    pub fn fire_size_at_report(&self) -> f64 {
        self.report_size
    }

    pub fn fire_spread_rate_at_back(&self) -> f64 {
        self.back_rate
    }

    pub fn fire_spread_rate_at_report(&self) -> f64 {
        self.report_rate
    }

    pub fn force(&self) -> &ContainForce {
        &self.force
    }

    pub fn attack_point_x(&self) -> f64 {
        self.x
    }

    pub fn attack_point_y(&self) -> f64 {
        self.y
    }

    pub fn simulation_step(&self) -> i32 {
        self.step
    }

    pub fn simulation_time(&self) -> f64 {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::ContainResource;

    fn make_test_contain() -> Contain {
        let mut force = ContainForce::new();
        force.add_resource(ContainResource::new(
            2.0, 20.0, 480.0,
            ContainFlank::LeftFlank, "test", 0.0, 0.0,
        ));
        let diurnal = [5.0; 24];
        let dist_step = 5.0 / 60.0; // report_rate / 60
        Contain::new(
            1.0, 5.0, &diurnal, 0, 3.0,
            dist_step, ContainFlank::LeftFlank, force,
            2.0, ContainTactic::HeadAttack, 0.0,
        )
    }

    #[test]
    fn contain_initial_state() {
        let c = make_test_contain();
        assert_eq!(c.status(), ContainStatus::Reported);
        assert!(c.report_head > 0.0);
        assert!(c.attack_head > c.report_head);
        assert_eq!(c.tactic(), ContainTactic::HeadAttack);
    }

    #[test]
    fn contain_step_progresses() {
        let mut c = make_test_contain();
        let status = c.do_step();
        assert!(
            status == ContainStatus::Attacked
                || status == ContainStatus::Contained
                || status == ContainStatus::Overrun
        );
        assert!(c.step >= 1);
    }
}
