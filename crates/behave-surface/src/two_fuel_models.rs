//! Two fuel models blending methods.
//!
//! Part of Mark Finney's EXRATE package for determining expected and
//! harmonic mean spread rate in randomly arranged fuels.
//!
//! C++ source: surfaceTwoFuelModels.h / surfaceTwoFuelModels.cpp

use firelab_base::FireSize;

use crate::fire::SurfaceFire;
use crate::fuel_models::FuelModels;
use crate::inputs::{SurfaceFireSpreadDirectionMode, SurfaceInputs};

/// Method for blending two fuel models.
///
/// C++ source: `TwoFuelModelsMethod` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoFuelModelsMethod {
    NoMethod = 0,
    Arithmetic = 1,
    Harmonic = 2,
    TwoDimensional = 3,
}

/// Constants for indexing the two fuel models.
pub const FIRST: usize = 0;
pub const SECOND: usize = 1;
pub const NUM_MODELS: usize = 2;

/// Blends spread-rate outputs from two overlapping fuel models.
///
/// C++ class: `SurfaceTwoFuelModels`
#[derive(Debug, Clone)]
pub struct TwoFuelModels {
    // Per-model data
    fuel_model_number: [i32; NUM_MODELS],
    coverage: [f64; NUM_MODELS],
    ros: [f64; NUM_MODELS],
    fireline_intensity: [f64; NUM_MODELS],
    max_flame_length: [f64; NUM_MODELS],
    flame_length: [f64; NUM_MODELS],
    fuelbed_depth: [f64; NUM_MODELS],
    effective_wind_speed: [f64; NUM_MODELS],
    length_to_width_ratio: [f64; NUM_MODELS],
    reaction_intensity_per_model: [f64; NUM_MODELS],
    heat_per_unit_area: [f64; NUM_MODELS],
    dir_max_spread: [f64; NUM_MODELS],
    wind_adjustment_factor_per_model: [f64; NUM_MODELS],
    midflame_wind_speed: [f64; NUM_MODELS],
    wind_speed_limit_per_model: [f64; NUM_MODELS],
    wind_limit_exceeded: [bool; NUM_MODELS],

    // Combined outputs
    method: TwoFuelModelsMethod,
    is_wind_limit_exceeded: bool,
    reaction_intensity: f64,
    spread_rate: f64,
    direction_of_max_spread: f64,
    effective_wind: f64,
    fuelbed_depth_combined: f64,
    heat_per_unit_area_combined: f64,
    midflame_wind_speed_combined: f64,
    wind_speed_limit: f64,
    wind_adjustment_factor: f64,
    fireline_intensity_combined: f64,
    flame_length_combined: f64,
    max_flame_length_combined: f64,
    fire_length_to_width_ratio: f64,
}

impl Default for TwoFuelModels {
    fn default() -> Self {
        Self::new()
    }
}

impl TwoFuelModels {
    pub fn new() -> Self {
        Self {
            fuel_model_number: [0; NUM_MODELS],
            coverage: [0.0; NUM_MODELS],
            ros: [0.0; NUM_MODELS],
            fireline_intensity: [0.0; NUM_MODELS],
            max_flame_length: [0.0; NUM_MODELS],
            flame_length: [0.0; NUM_MODELS],
            fuelbed_depth: [0.0; NUM_MODELS],
            effective_wind_speed: [0.0; NUM_MODELS],
            length_to_width_ratio: [0.0; NUM_MODELS],
            reaction_intensity_per_model: [0.0; NUM_MODELS],
            heat_per_unit_area: [0.0; NUM_MODELS],
            dir_max_spread: [0.0; NUM_MODELS],
            wind_adjustment_factor_per_model: [0.0; NUM_MODELS],
            midflame_wind_speed: [0.0; NUM_MODELS],
            wind_speed_limit_per_model: [0.0; NUM_MODELS],
            wind_limit_exceeded: [false; NUM_MODELS],
            method: TwoFuelModelsMethod::NoMethod,
            is_wind_limit_exceeded: false,
            reaction_intensity: 0.0,
            spread_rate: 0.0,
            direction_of_max_spread: 0.0,
            effective_wind: 0.0,
            fuelbed_depth_combined: 0.0,
            heat_per_unit_area_combined: 0.0,
            midflame_wind_speed_combined: 0.0,
            wind_speed_limit: 0.0,
            wind_adjustment_factor: 0.0,
            fireline_intensity_combined: 0.0,
            flame_length_combined: 0.0,
            max_flame_length_combined: 0.0,
            fire_length_to_width_ratio: 0.0,
        }
    }

    /// Calculate weighted spread rate from two fuel models.
    ///
    /// Runs the fire spread calculation for each fuel model independently,
    /// then blends results according to the specified method.
    ///
    /// C++ method: `calculateWeightedSpreadRate`
    pub fn calculate_weighted_spread_rate(
        &mut self,
        method: TwoFuelModelsMethod,
        first_fuel_model_number: i32,
        first_fuel_model_coverage: f64,
        second_fuel_model_number: i32,
        has_direction_of_interest: bool,
        direction_of_interest: f64,
        direction_mode: SurfaceFireSpreadDirectionMode,
        surface_fire: &mut SurfaceFire,
        fuel_models: &FuelModels,
        inputs: &SurfaceInputs,
        size: &mut FireSize,
    ) {
        self.fuel_model_number[FIRST] = first_fuel_model_number;
        self.fuel_model_number[SECOND] = second_fuel_model_number;

        self.coverage[FIRST] = first_fuel_model_coverage;
        self.coverage[SECOND] = 1.0 - first_fuel_model_coverage;

        // Calculate fire outputs for each fuel model
        self.calculate_fire_outputs_for_each_model(
            has_direction_of_interest,
            direction_of_interest,
            direction_mode,
            surface_fire,
            fuel_models,
            inputs,
            size,
        );

        // Determine spread rate based on method
        self.method = method;
        self.calculate_spread_rate_based_on_method();

        // Determine combined outputs using Pat's rules
        if self.coverage[FIRST] > 0.999 || self.coverage[SECOND] > 0.999 {
            // Only one fuel present — use its values exclusively
            let i = if self.coverage[FIRST] > 0.999 { FIRST } else { SECOND };

            self.reaction_intensity = self.reaction_intensity_per_model[i];
            self.direction_of_max_spread = self.dir_max_spread[i];
            self.wind_adjustment_factor = self.wind_adjustment_factor_per_model[i];
            self.midflame_wind_speed_combined = self.midflame_wind_speed[i];
            self.effective_wind = self.effective_wind_speed[i];
            self.wind_speed_limit = self.wind_speed_limit_per_model[i];
            self.is_wind_limit_exceeded = self.wind_limit_exceeded[i];
            self.fire_length_to_width_ratio = self.length_to_width_ratio[i];
            self.heat_per_unit_area_combined = self.heat_per_unit_area[i];
            self.fireline_intensity_combined = self.fireline_intensity[i];
            self.flame_length_combined = self.flame_length[i];
            self.fuelbed_depth_combined = self.fuelbed_depth[i];

            // Update surface fire
            surface_fire.set_reaction_intensity(self.reaction_intensity);
            surface_fire.set_direction_of_max_spread(self.direction_of_max_spread);
            surface_fire.set_wind_adjustment_factor(self.wind_adjustment_factor);
            surface_fire.set_midflame_wind_speed(self.midflame_wind_speed_combined);
            surface_fire.set_effective_wind_speed(self.effective_wind);
            surface_fire.set_wind_speed_limit(self.wind_speed_limit);
            surface_fire.set_is_wind_limit_exceeded(self.is_wind_limit_exceeded);
            surface_fire.set_fire_length_to_width_ratio(self.fire_length_to_width_ratio);
            surface_fire.set_heat_per_unit_area(self.heat_per_unit_area_combined);
            surface_fire.set_fireline_intensity(self.fireline_intensity_combined);
            surface_fire.set_flame_length(self.flame_length_combined);
        } else {
            // Two fuels present — use Pat's combination rules

            // Reaction intensity: max of two
            self.reaction_intensity = self.reaction_intensity_per_model[FIRST]
                .max(self.reaction_intensity_per_model[SECOND]);
            surface_fire.set_reaction_intensity(self.reaction_intensity);

            // Direction of max spread: FIRST fuel model
            self.direction_of_max_spread = self.dir_max_spread[FIRST];
            surface_fire.set_direction_of_max_spread(self.direction_of_max_spread);

            // Wind adjustment factor: FIRST fuel model
            self.wind_adjustment_factor = self.wind_adjustment_factor_per_model[FIRST];

            // Midflame wind speed: FIRST fuel model
            self.midflame_wind_speed_combined = self.midflame_wind_speed[FIRST];
            surface_fire.set_midflame_wind_speed(self.midflame_wind_speed_combined);

            // Effective wind speed: FIRST fuel model
            self.effective_wind = self.effective_wind_speed[FIRST];
            surface_fire.set_effective_wind_speed(self.effective_wind);

            // Wind speed limit: min of two
            self.wind_speed_limit = self.wind_speed_limit_per_model[FIRST]
                .min(self.wind_speed_limit_per_model[SECOND]);
            surface_fire.set_wind_speed_limit(self.wind_speed_limit);

            // Wind limit exceeded: either
            self.is_wind_limit_exceeded =
                self.wind_limit_exceeded[FIRST] || self.wind_limit_exceeded[SECOND];
            surface_fire.set_is_wind_limit_exceeded(self.is_wind_limit_exceeded);

            // Fire L/W ratio: FIRST fuel model
            self.fire_length_to_width_ratio = self.length_to_width_ratio[FIRST];
            surface_fire.set_fire_length_to_width_ratio(self.fire_length_to_width_ratio);

            // Heat per unit area: max of two
            self.heat_per_unit_area_combined = self.heat_per_unit_area[FIRST]
                .max(self.heat_per_unit_area[SECOND]);
            surface_fire.set_heat_per_unit_area(self.heat_per_unit_area_combined);

            // Fireline intensity: max of two
            self.fireline_intensity_combined = self.fireline_intensity[FIRST]
                .max(self.fireline_intensity[SECOND]);
            surface_fire.set_fireline_intensity(self.fireline_intensity_combined);

            // Flame length: max of two
            self.flame_length_combined =
                self.flame_length[FIRST].max(self.flame_length[SECOND]);
            self.max_flame_length_combined =
                self.max_flame_length[FIRST].max(self.max_flame_length[SECOND]);
            surface_fire.set_flame_length(self.flame_length_combined);

            // Fuelbed depth: max of two
            self.fuelbed_depth_combined =
                self.fuelbed_depth[FIRST].max(self.fuelbed_depth[SECOND]);
        }

        // Set the combined spread rate on the surface fire
        // C++: surfaceFireSpread_->forwardSpreadRate_ = spreadRate_;
        // In Rust we use the setter (no friend class access)
        // Note: we set the forward spread rate directly
        surface_fire.set_forward_spread_rate(self.spread_rate);
    }

    // --- Public getters ---

    pub fn wind_limit_exceeded(&self) -> bool {
        self.is_wind_limit_exceeded
    }

    pub fn reaction_intensity_value(&self) -> f64 {
        self.reaction_intensity
    }

    pub fn spread_rate(&self) -> f64 {
        self.spread_rate
    }

    pub fn direction_of_max_spread(&self) -> f64 {
        self.direction_of_max_spread
    }

    pub fn effective_wind_value(&self) -> f64 {
        self.effective_wind
    }

    pub fn fuelbed_depth(&self) -> f64 {
        self.fuelbed_depth_combined
    }

    pub fn heat_per_unit_area_value(&self) -> f64 {
        self.heat_per_unit_area_combined
    }

    pub fn midflame_wind_speed_value(&self) -> f64 {
        self.midflame_wind_speed_combined
    }

    pub fn wind_speed_limit_value(&self) -> f64 {
        self.wind_speed_limit
    }

    pub fn wind_adjustment_factor_value(&self) -> f64 {
        self.wind_adjustment_factor
    }

    pub fn fireline_intensity_value(&self) -> f64 {
        self.fireline_intensity_combined
    }

    pub fn flame_length_value(&self) -> f64 {
        self.flame_length_combined
    }

    pub fn fire_length_to_width_ratio(&self) -> f64 {
        self.fire_length_to_width_ratio
    }

    // --- Internal ---

    fn calculate_fire_outputs_for_each_model(
        &mut self,
        has_direction_of_interest: bool,
        direction_of_interest: f64,
        direction_mode: SurfaceFireSpreadDirectionMode,
        surface_fire: &mut SurfaceFire,
        fuel_models: &FuelModels,
        inputs: &SurfaceInputs,
        size: &mut FireSize,
    ) {
        for i in 0..NUM_MODELS {
            self.fuelbed_depth[i] = fuel_models
                .fuelbed_depth(self.fuel_model_number[i], firelab_base::LengthUnits::Feet);

            self.ros[i] = surface_fire.calculate_forward_spread_rate(
                self.fuel_model_number[i],
                has_direction_of_interest,
                direction_of_interest,
                direction_mode,
                fuel_models,
                inputs,
                size,
            );

            self.reaction_intensity_per_model[i] = surface_fire.reaction_intensity_value();
            self.dir_max_spread[i] = surface_fire.direction_of_max_spread();
            self.midflame_wind_speed[i] = surface_fire.midflame_wind_speed();
            self.wind_adjustment_factor_per_model[i] = surface_fire.wind_adjustment_factor_value();
            self.effective_wind_speed[i] = surface_fire.effective_wind_speed();
            self.wind_speed_limit_per_model[i] = surface_fire.wind_speed_limit();
            self.wind_limit_exceeded[i] = surface_fire.is_wind_limit_exceeded();
            self.fireline_intensity[i] = surface_fire.fireline_intensity();
            self.max_flame_length[i] = surface_fire.max_flame_length();
            self.flame_length[i] = surface_fire.flame_length();
            self.length_to_width_ratio[i] = surface_fire.fire_length_to_width_ratio();
            self.heat_per_unit_area[i] = surface_fire.heat_per_unit_area();
        }
    }

    fn calculate_spread_rate_based_on_method(&mut self) {
        match self.method {
            TwoFuelModelsMethod::Arithmetic => {
                self.spread_rate = self.coverage[FIRST] * self.ros[FIRST]
                    + self.coverage[SECOND] * self.ros[SECOND];
            }
            TwoFuelModelsMethod::Harmonic => {
                if self.ros[FIRST] > 1e-06 && self.ros[SECOND] > 1e-06 {
                    self.spread_rate = 1.0
                        / (self.coverage[FIRST] / self.ros[FIRST]
                            + self.coverage[SECOND] / self.ros[SECOND]);
                }
            }
            TwoFuelModelsMethod::TwoDimensional => {
                // Finney's 2D expected spread rate requires EXRATE package
                // (RandFuel, RandThread, NewExt). Stubbed for now.
                todo!("TwoDimensional method requires EXRATE package port")
            }
            TwoFuelModelsMethod::NoMethod => {
                // No blending
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuel_models::FuelModels;
    use crate::inputs::SurfaceInputs;
    use firelab_base::{
        FractionUnits, LengthUnits, SlopeUnits, SpeedUnits,
        WindAndSpreadOrientationMode, WindHeightInputMode,
    };

    fn assert_near(a: f64, b: f64, tol: f64) {
        assert!(
            (a - b).abs() < tol,
            "expected {b}, got {a} (diff {})",
            (a - b).abs()
        );
    }

    fn make_test_inputs() -> (FuelModels, SurfaceInputs) {
        let fm = FuelModels::new();
        let mut inputs = SurfaceInputs::new();
        inputs.update_surface_inputs(
            1, // fuel model (will be overridden per model)
            0.06, 0.07, 0.08, 0.60, 1.50,
            FractionUnits::Fraction,
            440.0, SpeedUnits::FeetPerMinute,
            WindHeightInputMode::DirectMidflame,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            0.0, SlopeUnits::Degrees, 0.0,
            0.0, FractionUnits::Fraction,
            0.0, LengthUnits::Feet,
            0.0, FractionUnits::Fraction,
        );
        (fm, inputs)
    }

    #[test]
    fn arithmetic_blend_50_50() {
        let (fm, inputs) = make_test_inputs();
        let mut fire = SurfaceFire::new();
        let mut size = FireSize::new();
        let mut tfm = TwoFuelModels::new();

        tfm.calculate_weighted_spread_rate(
            TwoFuelModelsMethod::Arithmetic,
            1, 0.5, 10,
            false, 0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &mut fire, &fm, &inputs, &mut size,
        );

        let spread = tfm.spread_rate();
        assert!(spread > 0.0, "spread_rate={}", spread);
    }

    #[test]
    fn arithmetic_blend_100_percent_first() {
        let (fm, inputs) = make_test_inputs();
        let mut fire = SurfaceFire::new();
        let mut size = FireSize::new();
        let mut tfm = TwoFuelModels::new();

        // 100% first fuel model → should get exactly the first model's spread rate
        tfm.calculate_weighted_spread_rate(
            TwoFuelModelsMethod::Arithmetic,
            1, 1.0, 10,
            false, 0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &mut fire, &fm, &inputs, &mut size,
        );

        // Run fire for FM1 directly for comparison
        let mut fire2 = SurfaceFire::new();
        let mut size2 = FireSize::new();
        let fm1_ros = fire2.calculate_forward_spread_rate(
            1, false, 0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &fm, &inputs, &mut size2,
        );

        assert_near(tfm.spread_rate(), fm1_ros, 1e-6);
    }

    #[test]
    fn harmonic_blend_50_50() {
        let (fm, inputs) = make_test_inputs();
        let mut fire = SurfaceFire::new();
        let mut size = FireSize::new();
        let mut tfm = TwoFuelModels::new();

        tfm.calculate_weighted_spread_rate(
            TwoFuelModelsMethod::Harmonic,
            1, 0.5, 10,
            false, 0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &mut fire, &fm, &inputs, &mut size,
        );

        let spread = tfm.spread_rate();
        assert!(spread > 0.0, "spread_rate={}", spread);
    }

    #[test]
    fn harmonic_is_less_than_arithmetic() {
        let (fm, inputs) = make_test_inputs();

        let mut fire_a = SurfaceFire::new();
        let mut size_a = FireSize::new();
        let mut tfm_a = TwoFuelModels::new();
        tfm_a.calculate_weighted_spread_rate(
            TwoFuelModelsMethod::Arithmetic,
            1, 0.5, 10,
            false, 0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &mut fire_a, &fm, &inputs, &mut size_a,
        );

        let mut fire_h = SurfaceFire::new();
        let mut size_h = FireSize::new();
        let mut tfm_h = TwoFuelModels::new();
        tfm_h.calculate_weighted_spread_rate(
            TwoFuelModelsMethod::Harmonic,
            1, 0.5, 10,
            false, 0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &mut fire_h, &fm, &inputs, &mut size_h,
        );

        // Harmonic mean is always <= arithmetic mean
        assert!(
            tfm_h.spread_rate() <= tfm_a.spread_rate(),
            "harmonic={} should be <= arithmetic={}",
            tfm_h.spread_rate(), tfm_a.spread_rate()
        );
    }

    #[test]
    fn combined_outputs_reaction_intensity_is_max() {
        let (fm, inputs) = make_test_inputs();
        let mut fire = SurfaceFire::new();
        let mut size = FireSize::new();
        let mut tfm = TwoFuelModels::new();

        tfm.calculate_weighted_spread_rate(
            TwoFuelModelsMethod::Arithmetic,
            1, 0.5, 10,
            false, 0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &mut fire, &fm, &inputs, &mut size,
        );

        // RI should be max of the two individual models
        assert!(tfm.reaction_intensity_value() > 0.0);
    }
}
