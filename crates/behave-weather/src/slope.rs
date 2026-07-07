//! Slope calculation tool.
//!
//! C++ source: slopeTool.h / slopeTool.cpp
//!
//! Calculates slope from map measurements and horizontal distances
//! at various angles from upslope direction.

use std::f64::consts::PI;

use firelab_base::{LengthUnits, SlopeUnits, UnitConversion};

/// Representative fraction scale indices for map scales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepresentativeFraction {
    F1980 = 0,
    F3960 = 1,
    F7920 = 2,
    F10000 = 3,
    F15840 = 4,
    F21120 = 5,
    F24000 = 6,
    F31680 = 7,
    F50000 = 8,
    F62500 = 9,
    F63360 = 10,
    F100000 = 11,
    F126720 = 12,
    F250000 = 13,
    F253440 = 14,
    F506880 = 15,
    F1000000 = 16,
    F1013760 = 17,
}

/// Horizontal distance direction indices (angles from upslope).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalDistanceIndex {
    Upslope0 = 0,
    Degrees15 = 1,
    Degrees30 = 2,
    Degrees45 = 3,
    Degrees60 = 4,
    Degrees75 = 5,
    CrossSlope90 = 6,
}

/// Representative fraction table record.
struct RepFracRecord {
    fraction: i32,
    inches_per_mile: f64,
    miles_per_inch: f64,
    cm_per_km: f64,
    km_per_cm: f64,
}

const REP_FRAC_TABLE: &[RepFracRecord] = &[
    RepFracRecord { fraction: 1980, inches_per_mile: 32.0, miles_per_inch: 0.03125, cm_per_km: 50.5051, km_per_cm: 0.0198 },
    RepFracRecord { fraction: 3960, inches_per_mile: 16.0, miles_per_inch: 0.0625, cm_per_km: 25.2525, km_per_cm: 0.0396 },
    RepFracRecord { fraction: 7920, inches_per_mile: 8.0, miles_per_inch: 0.125, cm_per_km: 12.6263, km_per_cm: 0.0792 },
    RepFracRecord { fraction: 10000, inches_per_mile: 6.336, miles_per_inch: 0.15783, cm_per_km: 10.0, km_per_cm: 0.1 },
    RepFracRecord { fraction: 15840, inches_per_mile: 4.0, miles_per_inch: 0.25, cm_per_km: 6.3131, km_per_cm: 0.1584 },
    RepFracRecord { fraction: 21120, inches_per_mile: 3.0, miles_per_inch: 0.33330, cm_per_km: 4.7348, km_per_cm: 0.2112 },
    RepFracRecord { fraction: 24000, inches_per_mile: 2.64, miles_per_inch: 0.37879, cm_per_km: 4.1667, km_per_cm: 0.24 },
    RepFracRecord { fraction: 31680, inches_per_mile: 2.0, miles_per_inch: 0.5, cm_per_km: 3.1566, km_per_cm: 0.3168 },
    RepFracRecord { fraction: 50000, inches_per_mile: 1.2672, miles_per_inch: 0.78914, cm_per_km: 2.0, km_per_cm: 0.5 },
    RepFracRecord { fraction: 62500, inches_per_mile: 1.0138, miles_per_inch: 0.98643, cm_per_km: 1.6, km_per_cm: 0.625 },
    RepFracRecord { fraction: 63360, inches_per_mile: 1.0, miles_per_inch: 1.0, cm_per_km: 1.5783, km_per_cm: 0.6336 },
    RepFracRecord { fraction: 100000, inches_per_mile: 0.6336, miles_per_inch: 1.57828, cm_per_km: 1.0, km_per_cm: 1.0 },
    RepFracRecord { fraction: 126720, inches_per_mile: 0.5, miles_per_inch: 2.0, cm_per_km: 0.7891, km_per_cm: 1.2672 },
    RepFracRecord { fraction: 250000, inches_per_mile: 0.2534, miles_per_inch: 3.94571, cm_per_km: 0.4, km_per_cm: 2.50 },
    RepFracRecord { fraction: 253440, inches_per_mile: 0.25, miles_per_inch: 4.0, cm_per_km: 0.3946, km_per_cm: 2.5344 },
    RepFracRecord { fraction: 506880, inches_per_mile: 0.125, miles_per_inch: 8.0, cm_per_km: 0.1973, km_per_cm: 5.0688 },
    RepFracRecord { fraction: 1000000, inches_per_mile: 0.0634, miles_per_inch: 15.78283, cm_per_km: 0.1, km_per_cm: 10.0 },
    RepFracRecord { fraction: 1013760, inches_per_mile: 0.0625, miles_per_inch: 16.0, cm_per_km: 0.0986, km_per_cm: 10.1376 },
];

/// Slope calculation tool.
///
/// Provides two main calculations:
/// 1. Horizontal distances at various angles from upslope given a map distance and slope
/// 2. Slope from map measurements (scale, distance, contour interval, number of contours)
#[derive(Debug, Clone)]
pub struct SlopeTool {
    // calculateHorizontalDistance outputs
    max_slope_degrees: f64, // base units (degrees)
    horizontal_distances: [f64; 7], // base units (ft)

    // calculateSlopeFromMapMeasurements outputs
    slope_from_map: f64,              // degrees (base)
    slope_horizontal_distance: f64,   // ft (base)
    slope_elevation_change: f64,      // ft (base)
}

impl SlopeTool {
    pub fn new() -> Self {
        Self {
            max_slope_degrees: -1.0,
            horizontal_distances: [0.0; 7],
            slope_from_map: -1.0,
            slope_horizontal_distance: -1.0,
            slope_elevation_change: -1.0,
        }
    }

    /// Calculate horizontal distances at 7 angles (0°, 15°, 30°, 45°, 60°, 75°, 90°)
    /// from the upslope direction.
    pub fn calculate_horizontal_distance(
        &mut self,
        map_distance: f64,
        distance_units: LengthUnits,
        max_slope_steepness: f64,
        slope_units: SlopeUnits,
    ) {
        self.max_slope_degrees = slope_units.to_base(max_slope_steepness);

        // C++ quirk preserved (slopeTool.cpp:140): the map distance is
        // converted to inches AS IF it were already in base units (feet);
        // `distance_units` is computed into groundDistanceInFeet in the C++
        // but never used. BehavePlus 6's published outputs bake this in.
        let _ = distance_units;
        let ground_distance_inches = LengthUnits::Inches.from_base(map_distance);

        let slope_rad = self.max_slope_degrees * PI / 180.0;

        for i in 0..7 {
            let direction = 15.0 * i as f64;
            let dir_rad = direction * PI / 180.0;
            let a = ground_distance_inches * dir_rad.cos();
            let b = ground_distance_inches * dir_rad.sin();
            let c = a * slope_rad.cos();
            let d = (c * c + b * b).sqrt();
            self.horizontal_distances[i] = LengthUnits::Inches.to_base(d);
        }
    }

    /// Calculate slope from map measurements.
    pub fn calculate_slope_from_map_measurements(
        &mut self,
        map_representative_fraction: i32,
        map_distance: f64,
        distance_units: LengthUnits,
        contour_interval: f64,
        number_of_contours: f64,
        contour_units: LengthUnits,
    ) {
        let distance_ft = distance_units.to_base(map_distance);
        let distance_inches = LengthUnits::Inches.from_base(distance_ft);

        self.slope_elevation_change =
            contour_units.to_base(contour_interval * number_of_contours);
        self.slope_horizontal_distance = LengthUnits::Inches
            .to_base(map_representative_fraction as f64 * distance_inches);

        if self.slope_horizontal_distance < 0.01 {
            self.slope_from_map = 0.0;
        } else {
            let slope_ratio = self.slope_elevation_change / self.slope_horizontal_distance;
            self.slope_from_map = slope_ratio.atan() * 180.0 / PI;
        }
    }

    // --- calculateHorizontalDistance getters ---

    pub fn get_number_of_horizontal_distances(&self) -> usize {
        7
    }

    pub fn get_horizontal_distance_max_slope(&self, units: SlopeUnits) -> f64 {
        units.from_base(self.max_slope_degrees)
    }

    pub fn get_horizontal_distance(
        &self,
        index: HorizontalDistanceIndex,
        units: LengthUnits,
    ) -> f64 {
        self.get_horizontal_distance_at_index(index as usize, units)
    }

    pub fn get_horizontal_distance_at_index(&self, index: usize, units: LengthUnits) -> f64 {
        if index < 7 {
            units.from_base(self.horizontal_distances[index])
        } else {
            -1.0
        }
    }

    // --- calculateSlopeFromMapMeasurements getters ---

    pub fn get_slope_from_map_measurements(&self, units: SlopeUnits) -> f64 {
        units.from_base(self.slope_from_map)
    }

    pub fn get_slope_from_map_measurements_in_percent(&self) -> f64 {
        SlopeUnits::Percent.from_base(self.slope_from_map)
    }

    pub fn get_slope_from_map_measurements_in_degrees(&self) -> f64 {
        SlopeUnits::Degrees.from_base(self.slope_from_map)
    }

    pub fn get_slope_horizontal_distance(&self, units: LengthUnits) -> f64 {
        units.from_base(self.slope_horizontal_distance)
    }

    pub fn get_slope_elevation_change(&self, units: LengthUnits) -> f64 {
        units.from_base(self.slope_elevation_change)
    }

    // --- Representative fraction table getters ---

    pub fn get_number_of_representative_fractions(&self) -> usize {
        REP_FRAC_TABLE.len()
    }

    pub fn get_representative_fraction_at_index(&self, index: usize) -> Option<i32> {
        REP_FRAC_TABLE.get(index).map(|r| r.fraction)
    }

    pub fn get_inches_per_mile_at_index(&self, index: usize) -> Option<f64> {
        REP_FRAC_TABLE.get(index).map(|r| r.inches_per_mile)
    }

    pub fn get_miles_per_inch_at_index(&self, index: usize) -> Option<f64> {
        REP_FRAC_TABLE.get(index).map(|r| r.miles_per_inch)
    }

    pub fn get_cm_per_km_at_index(&self, index: usize) -> Option<f64> {
        REP_FRAC_TABLE.get(index).map(|r| r.cm_per_km)
    }

    pub fn get_km_per_cm_at_index(&self, index: usize) -> Option<f64> {
        REP_FRAC_TABLE.get(index).map(|r| r.km_per_cm)
    }

    pub fn get_representative_fraction(&self, rf: RepresentativeFraction) -> i32 {
        REP_FRAC_TABLE[rf as usize].fraction
    }

    pub fn get_inches_per_mile(&self, rf: RepresentativeFraction) -> f64 {
        REP_FRAC_TABLE[rf as usize].inches_per_mile
    }

    pub fn get_miles_per_inch(&self, rf: RepresentativeFraction) -> f64 {
        REP_FRAC_TABLE[rf as usize].miles_per_inch
    }

    pub fn get_cm_per_km(&self, rf: RepresentativeFraction) -> f64 {
        REP_FRAC_TABLE[rf as usize].cm_per_km
    }

    pub fn get_km_per_cm(&self, rf: RepresentativeFraction) -> f64 {
        REP_FRAC_TABLE[rf as usize].km_per_cm
    }
}

impl Default for SlopeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario 1 — Imperial: 1:1980 scale, 3.6 in map distance, 50 ft contour, 4.1 contours.
    #[test]
    fn slope_from_map_imperial() {
        let mut tool = SlopeTool::new();
        tool.calculate_slope_from_map_measurements(
            1980,
            3.6,
            LengthUnits::Inches,
            50.0,
            4.1,
            LengthUnits::Feet,
        );

        let slope_deg = tool.get_slope_from_map_measurements_in_degrees();
        assert!(
            (slope_deg - 19.0).abs() < 1.0,
            "expected ~19 degrees, got {slope_deg}"
        );

        let slope_pct = tool.get_slope_from_map_measurements_in_percent();
        assert!(
            (slope_pct - 35.0).abs() < 1.0,
            "expected ~35 percent, got {slope_pct}"
        );

        let elev_change = tool.get_slope_elevation_change(LengthUnits::Feet);
        assert!(
            (elev_change - 205.0).abs() < 1.0,
            "expected ~205 ft, got {elev_change}"
        );

        let horiz_dist = tool.get_slope_horizontal_distance(LengthUnits::Feet);
        assert!(
            (horiz_dist - 594.0).abs() < 1.0,
            "expected ~594 ft, got {horiz_dist}"
        );
    }

    /// Scenario 2 — Metric: 1:3960 scale, 3.0 cm map distance, 15 m contour, 5.5 contours.
    #[test]
    fn slope_from_map_metric() {
        let mut tool = SlopeTool::new();
        tool.calculate_slope_from_map_measurements(
            3960,
            3.0,
            LengthUnits::Centimeters,
            15.0,
            5.5,
            LengthUnits::Meters,
        );

        let slope_deg = tool.get_slope_from_map_measurements_in_degrees();
        assert!(
            (slope_deg - 35.0).abs() < 1.0,
            "expected ~35 degrees, got {slope_deg}"
        );

        let slope_pct = tool.get_slope_from_map_measurements_in_percent();
        assert!(
            (slope_pct - 69.0).abs() < 1.0,
            "expected ~69 percent, got {slope_pct}"
        );

        let elev_change = tool.get_slope_elevation_change(LengthUnits::Meters);
        assert!(
            (elev_change - 82.0).abs() < 1.0,
            "expected ~82 m, got {elev_change}"
        );

        let horiz_dist = tool.get_slope_horizontal_distance(LengthUnits::Meters);
        assert!(
            (horiz_dist - 119.0).abs() < 1.0,
            "expected ~119 m, got {horiz_dist}"
        );
    }

    /// Horizontal distance calculation — 3.0 in map distance, 30% max slope.
    #[test]
    fn horizontal_distances() {
        let mut tool = SlopeTool::new();
        tool.calculate_horizontal_distance(
            3.0,
            LengthUnits::Inches,
            30.0,
            SlopeUnits::Percent,
        );

        // Expected values in FEET, per testBehave.cpp (the C++ treats the
        // 3.0 in map distance as feet when converting to inches — see the
        // quirk note in calculate_horizontal_distance).
        let expected = [2.9, 2.9, 2.9, 2.9, 3.0, 3.0, 3.0];
        for i in 0..7 {
            let dist = tool.get_horizontal_distance_at_index(i, LengthUnits::Feet);
            assert!(
                (dist - expected[i]).abs() < 0.1,
                "index {i}: expected ~{}, got {dist}",
                expected[i]
            );
        }
    }

    #[test]
    fn representative_fraction_table() {
        let tool = SlopeTool::new();
        assert_eq!(tool.get_number_of_representative_fractions(), 18);
        assert_eq!(tool.get_representative_fraction(RepresentativeFraction::F1980), 1980);
        assert_eq!(tool.get_representative_fraction(RepresentativeFraction::F1013760), 1013760);
    }
}
