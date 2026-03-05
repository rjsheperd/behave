//! Containment resource (individual suppression unit).
//!
//! C++ source: ContainResource.h / ContainResource.cpp (namespace Sem)

/// Which flank(s) a resource is assigned to.
///
/// C++ source: `ContainFlank` in ContainResource.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainFlank {
    LeftFlank = 0,
    RightFlank = 1,
    BothFlanks = 2,
    NeitherFlank = 3,
}

/// A single fire containment resource unit that can be dispatched to a fire.
///
/// Examples include engine crews, line crews, bulldozers, helicopters, etc.
/// If assigned to LeftFlank or RightFlank, full production rate is applied
/// to that flank. If assigned to BothFlanks, half the production rate is
/// applied to each flank. If NeitherFlank, the resource is inactive.
///
/// All times are in minutes since fire report.
/// Production rate is in ch/h (total for both flanks).
#[derive(Debug, Clone)]
pub struct ContainResource {
    arrival: f64,        // minutes since fire report
    production: f64,     // ch/h (total for both flanks)
    duration: f64,       // minutes
    base_cost: f64,
    hour_cost: f64,
    flank: ContainFlank,
    description: String,
}

impl ContainResource {
    pub fn new(
        arrival: f64,
        production: f64,
        duration: f64,
        flank: ContainFlank,
        description: &str,
        base_cost: f64,
        hour_cost: f64,
    ) -> Self {
        Self {
            arrival,
            production,
            duration,
            base_cost,
            hour_cost,
            flank,
            description: description.to_string(),
        }
    }

    pub fn arrival(&self) -> f64 {
        self.arrival
    }

    pub fn production(&self) -> f64 {
        self.production
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }

    pub fn flank(&self) -> ContainFlank {
        self.flank
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn base_cost(&self) -> f64 {
        self.base_cost
    }

    pub fn hour_cost(&self) -> f64 {
        self.hour_cost
    }
}

impl Default for ContainResource {
    fn default() -> Self {
        Self {
            arrival: 0.0,
            production: 0.0,
            duration: 0.0,
            base_cost: 0.0,
            hour_cost: 0.0,
            flank: ContainFlank::LeftFlank,
            description: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_new_and_getters() {
        let r = ContainResource::new(
            2.0, 20.0, 480.0,
            ContainFlank::LeftFlank,
            "Engine 1",
            100.0, 50.0,
        );
        assert_eq!(r.arrival(), 2.0);
        assert_eq!(r.production(), 20.0);
        assert_eq!(r.duration(), 480.0);
        assert_eq!(r.flank(), ContainFlank::LeftFlank);
        assert_eq!(r.description(), "Engine 1");
        assert_eq!(r.base_cost(), 100.0);
        assert_eq!(r.hour_cost(), 50.0);
    }

    #[test]
    fn resource_default() {
        let r = ContainResource::default();
        assert_eq!(r.arrival(), 0.0);
        assert_eq!(r.production(), 0.0);
        assert_eq!(r.flank(), ContainFlank::LeftFlank);
    }
}
