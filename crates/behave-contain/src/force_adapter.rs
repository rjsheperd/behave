//! Adapter between the public API force representation and the internal Sem force.
//!
//! C++ source: ContainForceAdapter.h / ContainForceAdapter.cpp
//!
//! In Rust, ContainForce already uses Vec, so this adapter simply wraps
//! ContainForce and provides add/remove convenience methods.

use crate::force::ContainForce;
use crate::resource::{ContainFlank, ContainResource};

/// Bridges the public-facing resource API and the internal `ContainForce`.
#[derive(Debug, Clone)]
pub struct ContainForceAdapter {
    resources: Vec<ContainResource>,
}

impl ContainForceAdapter {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    pub fn add_resource(&mut self, resource: ContainResource) {
        self.resources.push(resource);
    }

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

    pub fn remove_resource_at(&mut self, index: usize) -> bool {
        if index < self.resources.len() {
            self.resources.remove(index);
            true
        } else {
            false
        }
    }

    pub fn remove_all_resources(&mut self) {
        self.resources.clear();
    }

    pub fn resources(&self) -> &[ContainResource] {
        &self.resources
    }

    /// Build a `ContainForce` from the current resource list (copies resources
    /// into the Sem-style ContainForce used by the simulation).
    pub fn build_force(&self) -> ContainForce {
        let mut force = ContainForce::new();
        for r in &self.resources {
            force.add_resource(r.clone());
        }
        force
    }
}

impl Default for ContainForceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_add_remove() {
        let mut adapter = ContainForceAdapter::new();
        adapter.add_resource(ContainResource::new(
            2.0, 20.0, 480.0, ContainFlank::LeftFlank, "test", 0.0, 0.0,
        ));
        assert_eq!(adapter.resources().len(), 1);
        adapter.remove_resource_at(0);
        assert_eq!(adapter.resources().len(), 0);
    }

    #[test]
    fn adapter_build_force() {
        let mut adapter = ContainForceAdapter::new();
        adapter.add_resource(ContainResource::new(
            2.0, 20.0, 480.0, ContainFlank::LeftFlank, "a", 0.0, 0.0,
        ));
        let force = adapter.build_force();
        assert_eq!(force.num_resources(), 1);
    }
}
