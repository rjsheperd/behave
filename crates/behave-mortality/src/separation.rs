//! Safe separation distance calculator.
//!
//! C++ source: safeSeparationDistanceCalculator.h / safeSeparationDistanceCalculator.cpp
//!
//! Calculates safe separation distance using a lookup table based on
//! burning condition, slope class, and wind speed class.

use std::f64::consts::PI;

use firelab_base::{AreaUnits, LengthUnits, UnitConversion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurningCondition {
    Low = 0,
    Moderate = 1,
    Extreme = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlopeClass {
    Flat = 0,
    Moderate = 1,
    Steep = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedClass {
    Light = 0,
    Moderate = 1,
    High = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyCondition {
    Low = 0,
    Moderate = 1,
    Extreme = 2,
}

/// Delta lookup table indexed by (SpeedClass, BurningCondition, SlopeClass).
/// 3×3×3 = 27 entries.
const DELTA_TABLE: [[[f64; 3]; 3]; 3] = [
    // SpeedClass::Light
    [
        // BurningCondition::Low: [Flat, Moderate, Steep]
        [1.0, 1.0, 2.0],
        // BurningCondition::Moderate
        [1.0, 1.0, 2.0],
        // BurningCondition::Extreme
        [1.0, 2.0, 3.0],
    ],
    // SpeedClass::Moderate
    [
        // BurningCondition::Low
        [1.5, 3.0, 4.0],
        // BurningCondition::Moderate
        [2.0, 4.0, 5.0],
        // BurningCondition::Extreme
        [2.5, 5.0, 5.0],
    ],
    // SpeedClass::High
    [
        // BurningCondition::Low
        [3.0, 4.0, 6.0],
        // BurningCondition::Moderate
        [3.0, 5.0, 7.0],
        // BurningCondition::Extreme
        [4.0, 5.0, 10.0],
    ],
];

fn get_delta(speed: SpeedClass, burning: BurningCondition, slope: SlopeClass) -> f64 {
    DELTA_TABLE[speed as usize][burning as usize][slope as usize]
}

/// Safe separation distance calculator.
///
/// Uses a lookup table of delta multipliers based on wind speed class,
/// burning condition, and slope class. The safe separation distance is
/// 8 * vegetation height * delta.
#[derive(Debug, Clone)]
pub struct SafeSeparationDistanceCalculator {
    burning_condition: BurningCondition,
    slope_class: SlopeClass,
    speed_class: SpeedClass,
    vegetation_height: f64, // ft (base)
    safe_separation_distance: f64, // ft (base)
    safety_zone_size: f64,  // ft² (base)
}

impl SafeSeparationDistanceCalculator {
    pub fn new() -> Self {
        Self {
            burning_condition: BurningCondition::Low,
            slope_class: SlopeClass::Flat,
            speed_class: SpeedClass::Light,
            vegetation_height: 0.0,
            safe_separation_distance: 0.0,
            safety_zone_size: 0.0,
        }
    }

    pub fn calculate(&mut self) {
        let vegetation_height = LengthUnits::Feet.from_base(self.vegetation_height);
        let delta = get_delta(self.speed_class, self.burning_condition, self.slope_class);

        let safe_separation_distance = 8.0 * vegetation_height * delta;
        let safety_zone_size = PI * safe_separation_distance * safe_separation_distance;

        self.safe_separation_distance =
            LengthUnits::Feet.to_base(safe_separation_distance);
        self.safety_zone_size = AreaUnits::SquareFeet.to_base(safety_zone_size);
    }

    pub fn get_burning_condition(&self) -> BurningCondition {
        self.burning_condition
    }

    pub fn get_slope_class(&self) -> SlopeClass {
        self.slope_class
    }

    pub fn get_speed_class(&self) -> SpeedClass {
        self.speed_class
    }

    pub fn get_safety_condition(&self) -> SafetyCondition {
        match self.slope_class {
            SlopeClass::Flat => {
                if self.speed_class == SpeedClass::High {
                    SafetyCondition::Moderate
                } else {
                    SafetyCondition::Low
                }
            }
            SlopeClass::Moderate => match self.speed_class {
                SpeedClass::Light => SafetyCondition::Low,
                SpeedClass::Moderate => SafetyCondition::Moderate,
                SpeedClass::High => SafetyCondition::Extreme,
            },
            SlopeClass::Steep => match self.speed_class {
                SpeedClass::Light => {
                    if self.burning_condition == BurningCondition::Extreme {
                        SafetyCondition::Moderate
                    } else {
                        SafetyCondition::Low
                    }
                }
                SpeedClass::Moderate => SafetyCondition::Moderate,
                SpeedClass::High => SafetyCondition::Extreme,
            },
        }
    }

    pub fn get_vegetation_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.vegetation_height)
    }

    pub fn get_safe_separation_distance(&self, units: LengthUnits) -> f64 {
        units.from_base(self.safe_separation_distance)
    }

    pub fn get_safety_zone_size(&self, units: AreaUnits) -> f64 {
        units.from_base(self.safety_zone_size)
    }

    pub fn set_burning_condition(&mut self, condition: BurningCondition) {
        self.burning_condition = condition;
    }

    pub fn set_slope_class(&mut self, slope: SlopeClass) {
        self.slope_class = slope;
    }

    pub fn set_speed_class(&mut self, speed: SpeedClass) {
        self.speed_class = speed;
    }

    pub fn set_vegetation_height(&mut self, height: f64, units: LengthUnits) {
        self.vegetation_height = units.to_base(height);
    }
}

impl Default for SafeSeparationDistanceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_lookup_light_low_flat() {
        assert_eq!(get_delta(SpeedClass::Light, BurningCondition::Low, SlopeClass::Flat), 1.0);
    }

    #[test]
    fn delta_lookup_high_extreme_steep() {
        assert_eq!(get_delta(SpeedClass::High, BurningCondition::Extreme, SlopeClass::Steep), 10.0);
    }

    #[test]
    fn separation_distance_basic() {
        let mut calc = SafeSeparationDistanceCalculator::new();
        calc.set_vegetation_height(10.0, LengthUnits::Feet);
        calc.set_speed_class(SpeedClass::Light);
        calc.set_burning_condition(BurningCondition::Low);
        calc.set_slope_class(SlopeClass::Flat);
        calc.calculate();

        // 8 * 10 * 1.0 = 80 ft
        let dist = calc.get_safe_separation_distance(LengthUnits::Feet);
        assert!(
            (dist - 80.0).abs() < 1e-6,
            "expected 80.0 ft, got {dist}"
        );
    }

    #[test]
    fn safety_condition_classification() {
        let mut calc = SafeSeparationDistanceCalculator::new();
        calc.set_slope_class(SlopeClass::Flat);
        calc.set_speed_class(SpeedClass::Light);
        assert_eq!(calc.get_safety_condition(), SafetyCondition::Low);

        calc.set_speed_class(SpeedClass::High);
        assert_eq!(calc.get_safety_condition(), SafetyCondition::Moderate);

        calc.set_slope_class(SlopeClass::Steep);
        calc.set_speed_class(SpeedClass::High);
        assert_eq!(calc.get_safety_condition(), SafetyCondition::Extreme);
    }
}
