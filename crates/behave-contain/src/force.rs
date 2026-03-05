//! Containment force (collection of resources).
//!
//! C++ source: ContainForce.h / ContainForce.cpp (namespace Sem)

use crate::resource::{ContainFlank, ContainResource};

/// An ordered collection of suppression resources.
///
/// Provides aggregate production rate calculation and resource scheduling
/// queries. Production rates returned are for a single flank (half the
/// total rate entered for each resource).
#[derive(Debug, Clone)]
pub struct ContainForce {
    resources: Vec<ContainResource>,
}

impl ContainForce {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    pub fn add_resource(&mut self, resource: ContainResource) {
        self.resources.push(resource);
    }

    pub fn add_resource_params(
        &mut self,
        arrival: f64,
        production: f64,
        duration: f64,
        flank: ContainFlank,
        desc: &str,
        base_cost: f64,
        hour_cost: f64,
    ) {
        self.resources.push(ContainResource::new(
            arrival, production, duration, flank, desc, base_cost, hour_cost,
        ));
    }

    pub fn resources(&self) -> &[ContainResource] {
        &self.resources
    }

    pub fn num_resources(&self) -> usize {
        self.resources.len()
    }

    /// Time when all the containment resources assigned to the specified
    /// flank will be exhausted (minutes since fire report).
    pub fn exhausted(&self, flank: ContainFlank) -> f64 {
        let mut at = 0.0_f64;
        for r in &self.resources {
            if r.flank() == flank || r.flank() == ContainFlank::BothFlanks {
                let done = r.arrival() + r.duration();
                if done > at {
                    at = done;
                }
            }
        }
        at
    }

    /// Time of first resource arrival on the specified flank
    /// (minutes since fire report).
    pub fn first_arrival(&self, flank: ContainFlank) -> f64 {
        let mut at = 99999999.0_f64;
        for r in &self.resources {
            if (r.flank() == flank || r.flank() == ContainFlank::BothFlanks)
                && r.arrival() < at
            {
                at = r.arrival();
            }
        }
        at
    }

    /// Time of next productivity increase (next resource arrival)
    /// after `after` and before `until` on the specified flank.
    /// Returns 0.0 if no more productivity boosts found.
    pub fn next_arrival(&self, after: f64, until: f64, flank: ContainFlank) -> f64 {
        let prod_rate = self.production_rate(after, flank);
        let it = after as i64;
        let mut t = it as f64 + 1.0;
        while t < until {
            if (self.production_rate(t, flank) - prod_rate).abs() > 0.001 {
                return t;
            }
            t += 1.0;
        }
        0.0
    }

    /// Aggregate fireline production rate along one fire flank at the
    /// specified time. This is HALF the total production rate for both flanks.
    pub fn production_rate(&self, min_since_report: f64, flank: ContainFlank) -> f64 {
        let mut fpm = 0.0;
        for r in &self.resources {
            if (r.flank() == flank || r.flank() == ContainFlank::BothFlanks)
                && r.arrival() <= (min_since_report + 0.001)
                && (r.arrival() + r.duration()) >= min_since_report
            {
                fpm += 0.5 * r.production();
            }
        }
        fpm
    }

    /// Cost of the specified resource given the final time.
    pub fn resource_cost(&self, index: usize, final_time: f64) -> f64 {
        if index < self.resources.len() {
            let r = &self.resources[index];
            if final_time <= r.arrival() {
                return 0.0;
            }
            let mut minutes = final_time - r.arrival();
            if minutes > r.duration() {
                minutes = r.duration();
            }
            r.base_cost() + (r.hour_cost() * minutes / 60.0)
        } else {
            0.0
        }
    }

    /// Arrival time of the resource at the given index.
    pub fn resource_arrival(&self, index: usize) -> f64 {
        self.resources.get(index).map_or(0.0, |r| r.arrival())
    }
}

impl Default for ContainForce {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_production_rate() {
        let mut force = ContainForce::new();
        // Resource: arrival=2 min, production=20 ch/h, duration=480 min, LeftFlank
        force.add_resource(ContainResource::new(
            2.0, 20.0, 480.0, ContainFlank::LeftFlank, "test", 0.0, 0.0,
        ));
        // Before arrival
        assert_eq!(force.production_rate(1.0, ContainFlank::LeftFlank), 0.0);
        // At arrival: 0.5 * 20 = 10
        assert!((force.production_rate(2.0, ContainFlank::LeftFlank) - 10.0).abs() < 0.001);
        // After exhausted
        assert_eq!(force.production_rate(483.0, ContainFlank::LeftFlank), 0.0);
        // Right flank gets nothing from a LeftFlank resource
        assert_eq!(force.production_rate(10.0, ContainFlank::RightFlank), 0.0);
    }

    #[test]
    fn force_exhausted_and_first_arrival() {
        let mut force = ContainForce::new();
        force.add_resource(ContainResource::new(
            2.0, 20.0, 480.0, ContainFlank::LeftFlank, "a", 0.0, 0.0,
        ));
        force.add_resource(ContainResource::new(
            10.0, 15.0, 300.0, ContainFlank::LeftFlank, "b", 0.0, 0.0,
        ));
        assert!((force.first_arrival(ContainFlank::LeftFlank) - 2.0).abs() < 1e-10);
        assert!((force.exhausted(ContainFlank::LeftFlank) - 482.0).abs() < 1e-10);
    }
}
