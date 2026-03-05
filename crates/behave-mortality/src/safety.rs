//! Safety zone calculations.
//!
//! C++ source: safety.h / safety.cpp
//!
//! Calculates safety zone size, separation distance, and radius
//! based on flame height and number of personnel/equipment.

use std::f64::consts::PI;

use firelab_base::{AreaUnits, LengthUnits, UnitConversion};

/// Safety zone calculator.
///
/// Determines the minimum safety zone area and radius to protect
/// a specified number of personnel and equipment from radiant heat injury.
#[derive(Debug, Clone)]
pub struct Safety {
    // Inputs (base units: ft, ft²)
    flame_height: f64,        // ft
    number_of_personnel: f64,
    area_per_person: f64,     // ft²
    number_of_equipment: f64,
    area_per_equipment: f64,  // ft²

    // Outputs (base units: ft, ft²)
    separation_distance: f64, // ft
    safety_zone_radius: f64,  // ft
    safety_zone_area: f64,    // ft²
}

impl Safety {
    pub fn new() -> Self {
        Self {
            flame_height: 0.0,
            number_of_personnel: 0.0,
            area_per_person: 0.0,
            number_of_equipment: 0.0,
            area_per_equipment: 0.0,
            separation_distance: 0.0,
            safety_zone_radius: 0.0,
            safety_zone_area: 0.0,
        }
    }

    pub fn set_flame_height(&mut self, flame_height: f64, units: LengthUnits) {
        self.flame_height = units.to_base(flame_height);
    }

    pub fn set_number_of_personnel(&mut self, n: f64) {
        self.number_of_personnel = n;
    }

    pub fn set_area_per_person(&mut self, area: f64, units: AreaUnits) {
        self.area_per_person = units.to_base(area);
    }

    pub fn set_number_of_equipment(&mut self, n: f64) {
        self.number_of_equipment = n;
    }

    pub fn set_area_per_equipment(&mut self, area: f64, units: AreaUnits) {
        self.area_per_equipment = units.to_base(area);
    }

    pub fn update_safety_inputs(
        &mut self,
        flame_height: f64,
        length_units: LengthUnits,
        number_of_personnel: i32,
        number_of_equipment: i32,
        area_per_person: f64,
        area_per_equipment: f64,
        area_units: AreaUnits,
    ) {
        self.set_flame_height(flame_height, length_units);
        self.set_number_of_personnel(number_of_personnel as f64);
        self.set_number_of_equipment(number_of_equipment as f64);
        self.set_area_per_person(area_per_person, area_units);
        self.set_area_per_equipment(area_per_equipment, area_units);
    }

    pub fn calculate_safety_zone(&mut self) {
        // Separation distance = 4 * flame height
        self.separation_distance = 4.0 * self.flame_height;

        // Core radius from personnel + equipment area
        let mut core_radius = (self.area_per_person * self.number_of_personnel
            + self.number_of_equipment * self.area_per_equipment)
            / PI;
        if core_radius > 1.0e-07 {
            core_radius = core_radius.sqrt();
        }

        // Safety zone = separation distance + core
        self.safety_zone_radius = self.separation_distance + core_radius;
        self.safety_zone_area = PI * self.safety_zone_radius * self.safety_zone_radius;
    }

    pub fn get_separation_distance(&self, units: LengthUnits) -> f64 {
        units.from_base(self.separation_distance)
    }

    pub fn get_safety_zone_radius(&self, units: LengthUnits) -> f64 {
        units.from_base(self.safety_zone_radius)
    }

    pub fn get_safety_zone_area(&self, units: AreaUnits) -> f64 {
        units.from_base(self.safety_zone_area)
    }
}

impl Default for Safety {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_zone_test() {
        let mut safety = Safety::new();
        safety.update_safety_inputs(
            5.0,
            LengthUnits::Feet,
            6,
            1,
            50.0,
            300.0,
            AreaUnits::SquareFeet,
        );
        safety.calculate_safety_zone();

        let sep = safety.get_separation_distance(LengthUnits::Feet);
        assert!(
            (sep - 20.0).abs() < 1e-6,
            "separation distance: expected 20.0, got {sep}"
        );

        let area = safety.get_safety_zone_area(AreaUnits::Acres);
        assert!(
            (area - 0.082490356).abs() < 1e-6,
            "safety zone area: expected 0.082490356, got {area}"
        );

        let radius = safety.get_safety_zone_radius(LengthUnits::Feet);
        assert!(
            (radius - 33.819766).abs() < 1e-4,
            "safety zone radius: expected 33.819766, got {radius}"
        );
    }
}
