//! Crown fire input parameters.
//!
//! C++ source: crownInputs.h / crownInputs.cpp

use firelab_base::{DensityUnits, FractionUnits, LengthUnits, UnitConversion};

/// User-facing inputs for crown fire calculations.
///
/// Stores canopy base height (ft), canopy bulk density (lb/ft³),
/// foliar moisture (fraction 0-1), and optional user-provided flame
/// length and fireline intensity — all in base units.
///
/// C++ class: `CrownInputs`
#[derive(Debug, Clone)]
pub struct CrownInputs {
    canopy_base_height: f64,      // ft
    canopy_bulk_density: f64,     // lb/ft³
    canopy_user_provided_flame_length: f64,        // ft
    canopy_user_provided_fireline_intensity: f64,   // Btu/ft/s
    moisture_foliar: f64,         // fraction 0-1
}

impl CrownInputs {
    pub fn new() -> Self {
        Self {
            canopy_base_height: 0.0,
            canopy_bulk_density: 0.0,
            canopy_user_provided_flame_length: 0.0,
            canopy_user_provided_fireline_intensity: 0.0,
            moisture_foliar: 0.0,
        }
    }

    pub fn initialize_members(&mut self) {
        self.canopy_base_height = 0.0;
        self.canopy_bulk_density = 0.0;
        self.canopy_user_provided_flame_length = 0.0;
        self.canopy_user_provided_fireline_intensity = 0.0;
        self.moisture_foliar = 0.0;
    }

    // --- Setters ---

    pub fn set_canopy_base_height(&mut self, height: f64, units: LengthUnits) {
        self.canopy_base_height = units.to_base(height);
    }

    pub fn set_canopy_bulk_density(&mut self, density: f64, units: DensityUnits) {
        self.canopy_bulk_density = units.to_base(density);
    }

    pub fn set_canopy_flame_length(&mut self, flame_length: f64) {
        self.canopy_user_provided_flame_length = flame_length;
    }

    pub fn set_canopy_fireline_intensity(&mut self, fireline_intensity: f64) {
        self.canopy_user_provided_fireline_intensity = fireline_intensity;
    }

    pub fn set_moisture_foliar(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_foliar = units.to_base(moisture);
    }

    /// Bulk update setter matching C++ `updateCrownInputs`.
    pub fn update_crown_inputs(
        &mut self,
        canopy_base_height: f64,
        height_units: LengthUnits,
        canopy_bulk_density: f64,
        density_units: DensityUnits,
        moisture_foliar: f64,
        moisture_units: FractionUnits,
    ) {
        self.set_canopy_base_height(canopy_base_height, height_units);
        self.set_canopy_bulk_density(canopy_bulk_density, density_units);
        self.set_moisture_foliar(moisture_foliar, moisture_units);
    }

    // --- Getters ---

    pub fn canopy_base_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.canopy_base_height)
    }

    pub fn canopy_bulk_density(&self, units: DensityUnits) -> f64 {
        units.from_base(self.canopy_bulk_density)
    }

    pub fn canopy_flame_length(&self) -> f64 {
        self.canopy_user_provided_flame_length
    }

    pub fn canopy_fireline_intensity(&self) -> f64 {
        self.canopy_user_provided_fireline_intensity
    }

    pub fn moisture_foliar(&self, units: FractionUnits) -> f64 {
        units.from_base(self.moisture_foliar)
    }
}

impl Default for CrownInputs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crown_inputs_default_values() {
        let ci = CrownInputs::new();
        assert_eq!(ci.canopy_base_height(LengthUnits::Feet), 0.0);
        assert_eq!(ci.canopy_bulk_density(DensityUnits::PoundsPerCubicFoot), 0.0);
        assert_eq!(ci.moisture_foliar(FractionUnits::Fraction), 0.0);
    }

    #[test]
    fn crown_inputs_set_and_get() {
        let mut ci = CrownInputs::new();
        ci.set_canopy_base_height(6.0, LengthUnits::Feet);
        ci.set_canopy_bulk_density(0.03, DensityUnits::PoundsPerCubicFoot);
        ci.set_moisture_foliar(120.0, FractionUnits::Percent);

        assert!((ci.canopy_base_height(LengthUnits::Feet) - 6.0).abs() < 1e-10);
        assert!((ci.canopy_bulk_density(DensityUnits::PoundsPerCubicFoot) - 0.03).abs() < 1e-10);
        assert!((ci.moisture_foliar(FractionUnits::Percent) - 120.0).abs() < 0.01);
    }

    #[test]
    fn crown_inputs_update_bulk() {
        let mut ci = CrownInputs::new();
        ci.update_crown_inputs(
            6.0, LengthUnits::Feet,
            0.03, DensityUnits::PoundsPerCubicFoot,
            1.2, FractionUnits::Fraction,
        );
        assert!((ci.canopy_base_height(LengthUnits::Feet) - 6.0).abs() < 1e-10);
        assert!((ci.canopy_bulk_density(DensityUnits::PoundsPerCubicFoot) - 0.03).abs() < 1e-10);
        assert!((ci.moisture_foliar(FractionUnits::Fraction) - 1.2).abs() < 1e-10);
    }
}
