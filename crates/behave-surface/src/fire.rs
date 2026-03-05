//! Surface fire spread rate, flame length, and related outputs.
//!
//! This is the core Rothermel surface fire spread calculator. It orchestrates
//! fuelbed intermediates, reaction intensity, wind/slope factors, fire ellipse,
//! fireline intensity, flame length, and scorch height calculations.
//!
//! C++ source: surfaceFire.h / surfaceFire.cpp

use std::f64::consts::PI;

use firelab_base::{
    FireSize, FractionUnits, FuelLifeState, LengthUnits, SlopeUnits, SpeedUnits,
    TemperatureUnits, UnitConversion,
    WindAdjustmentFactorCalculationMethod, WindAndSpreadOrientationMode,
    WindHeightInputMode,
};

use crate::fuel_models::FuelModels;
use crate::fuelbed::FuelbedIntermediates;
use crate::inputs::{SurfaceFireSpreadDirectionMode, SurfaceInputs};
use crate::reaction::ReactionIntensity;
use crate::wind::{WindAdjustmentFactor, WindAdjustmentFactorShelterMethod};

/// Core Rothermel surface fire spread calculator.
///
/// C++ class: `SurfaceFire`
#[derive(Debug, Clone)]
pub struct SurfaceFire {
    // Sub-calculators (owned)
    fuelbed: FuelbedIntermediates,
    reaction: ReactionIntensity,

    // Flags
    is_wind_limit_enabled: bool,
    is_wind_limit_exceeded: bool,

    // Wind/slope intermediate factors
    phi_s: f64,   // Slope factor, Rothermel 1972, equation 51
    phi_w: f64,   // Wind factor, Rothermel 1972, equation 47
    wind_b: f64,  // Rothermel 1972, equation 49
    wind_c: f64,  // Rothermel 1972, equation 48
    wind_e: f64,  // Rothermel 1972, equation 50

    // Spread rates (ft/min, base units)
    direction_of_interest: f64,
    direction_of_max_spread: f64,
    no_wind_no_slope_spread_rate: f64,
    forward_spread_rate: f64,
    backing_spread_rate: f64,
    flanking_spread_rate: f64,
    spread_rate_in_direction_of_interest: f64,

    // Fire properties
    heat_per_unit_area: f64,          // Btu/ft²
    fire_length_to_width_ratio: f64,
    residence_time: f64,              // min
    reaction_intensity: f64,          // Btu/ft²/min

    // Fireline intensities (Btu/ft/s, base units)
    forward_fireline_intensity: f64,
    backing_fireline_intensity: f64,
    flanking_fireline_intensity: f64,
    direction_of_interest_fireline_intensity: f64,

    // Flame lengths (ft)
    max_flame_length: f64,
    forward_flame_length: f64,
    backing_flame_length: f64,
    flanking_flame_length: f64,
    direction_of_interest_flame_length: f64,

    // Other
    heat_source: f64,
    scorch_height: f64,
    effective_wind_speed: f64,
    wind_speed_limit: f64,
    midflame_wind_speed: f64,
    wind_adjustment_factor: f64,
    wind_adjustment_factor_shelter_method: WindAdjustmentFactorShelterMethod,
    canopy_crown_fraction: f64,
}

impl Default for SurfaceFire {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceFire {
    pub fn new() -> Self {
        Self {
            fuelbed: FuelbedIntermediates::new(),
            reaction: ReactionIntensity::new(),
            is_wind_limit_enabled: false,
            is_wind_limit_exceeded: false,
            phi_s: 0.0,
            phi_w: 0.0,
            wind_b: 0.0,
            wind_c: 0.0,
            wind_e: 0.0,
            direction_of_interest: 0.0,
            direction_of_max_spread: 0.0,
            no_wind_no_slope_spread_rate: 0.0,
            forward_spread_rate: 0.0,
            backing_spread_rate: 0.0,
            flanking_spread_rate: 0.0,
            spread_rate_in_direction_of_interest: 0.0,
            heat_per_unit_area: 0.0,
            fire_length_to_width_ratio: 1.0,
            residence_time: 0.0,
            reaction_intensity: 0.0,
            forward_fireline_intensity: 0.0,
            backing_fireline_intensity: 0.0,
            flanking_fireline_intensity: 0.0,
            direction_of_interest_fireline_intensity: 0.0,
            max_flame_length: 0.0,
            forward_flame_length: 0.0,
            backing_flame_length: 0.0,
            flanking_flame_length: 0.0,
            direction_of_interest_flame_length: 0.0,
            heat_source: 0.0,
            scorch_height: 0.0,
            effective_wind_speed: 0.0,
            wind_speed_limit: 0.0,
            midflame_wind_speed: 0.0,
            wind_adjustment_factor: 0.0,
            wind_adjustment_factor_shelter_method: WindAdjustmentFactorShelterMethod::Unsheltered,
            canopy_crown_fraction: 0.0,
        }
    }

    /// Reset all member variables to initial state.
    fn initialize_members(&mut self) {
        self.is_wind_limit_enabled = false;
        self.is_wind_limit_exceeded = false;
        self.phi_s = 0.0;
        self.phi_w = 0.0;
        self.wind_b = 0.0;
        self.wind_c = 0.0;
        self.wind_e = 0.0;
        self.direction_of_interest = 0.0;
        self.direction_of_max_spread = 0.0;
        self.no_wind_no_slope_spread_rate = 0.0;
        self.forward_spread_rate = 0.0;
        self.backing_spread_rate = 0.0;
        self.flanking_spread_rate = 0.0;
        self.spread_rate_in_direction_of_interest = 0.0;
        self.heat_per_unit_area = 0.0;
        self.fire_length_to_width_ratio = 1.0;
        self.residence_time = 0.0;
        self.reaction_intensity = 0.0;
        self.forward_fireline_intensity = 0.0;
        self.backing_fireline_intensity = 0.0;
        self.flanking_fireline_intensity = 0.0;
        self.direction_of_interest_fireline_intensity = 0.0;
        self.max_flame_length = 0.0;
        self.forward_flame_length = 0.0;
        self.backing_flame_length = 0.0;
        self.flanking_flame_length = 0.0;
        self.direction_of_interest_flame_length = 0.0;
        self.heat_source = 0.0;
        self.scorch_height = 0.0;
        self.effective_wind_speed = 0.0;
        self.wind_speed_limit = 0.0;
        self.midflame_wind_speed = 0.0;
        self.wind_adjustment_factor = 0.0;
        self.wind_adjustment_factor_shelter_method = WindAdjustmentFactorShelterMethod::Unsheltered;
        self.canopy_crown_fraction = 0.0;
    }

    /// Main entry point: calculate forward spread rate and all derived outputs.
    ///
    /// Returns the spread rate in ft/min — either in direction of max spread
    /// or in direction of interest if `has_direction_of_interest` is true.
    ///
    /// C++ method: `calculateForwardSpreadRate`
    pub fn calculate_forward_spread_rate(
        &mut self,
        fuel_model_number: i32,
        has_direction_of_interest: bool,
        direction_of_interest: f64,
        direction_mode: SurfaceFireSpreadDirectionMode,
        fuel_models: &FuelModels,
        inputs: &SurfaceInputs,
        size: &mut FireSize,
    ) -> f64 {
        // Reset member variables
        self.initialize_members();

        // Calculate fuelbed intermediates
        self.fuelbed
            .calculate_fuelbed_intermediates(fuel_model_number, fuel_models, inputs);

        // Get needed fuelbed intermediates
        let propagating_flux = self.fuelbed.propagating_flux();
        let heat_sink = self.fuelbed.heat_sink();
        self.reaction_intensity = self.reaction.calculate_reaction_intensity(&self.fuelbed);

        // Calculate wind and slope factors
        self.calculate_midflame_wind_speed(inputs);
        self.calculate_wind_factor();
        self.calculate_slope_factor(inputs);

        // No-wind no-slope spread rate
        self.no_wind_no_slope_spread_rate =
            Self::calculate_no_wind_no_slope_spread_rate_static(
                self.reaction_intensity,
                propagating_flux,
                heat_sink,
            );
        self.forward_spread_rate = self.no_wind_no_slope_spread_rate;

        // Wind speed limit (slope factor may be capped)
        self.calculate_wind_speed_limit();

        // Slope and wind adjusted spread rate
        self.forward_spread_rate =
            self.no_wind_no_slope_spread_rate * (1.0 + self.phi_w + self.phi_s);

        // Direction of maximum spread
        self.calculate_direction_of_max_spread(inputs);
        self.calculate_effective_wind_speed();

        // Apply wind speed limit if enabled
        if self.is_wind_limit_enabled && self.effective_wind_speed > self.wind_speed_limit {
            self.apply_wind_speed_limit();
        }

        // Convert effective wind speed to ft/min for display
        self.effective_wind_speed =
            SpeedUnits::FeetPerMinute.from_base(self.effective_wind_speed);

        // Residence time
        self.calculate_residence_time();

        // Fire ellipse dimensions
        size.calculate_fire_basic_dimensions(
            false,
            self.effective_wind_speed,
            SpeedUnits::FeetPerMinute,
            self.forward_spread_rate,
            SpeedUnits::FeetPerMinute,
        );

        self.fire_length_to_width_ratio = size.fire_length_to_width_ratio();
        self.backing_spread_rate = size.backing_spread_rate(SpeedUnits::FeetPerMinute);
        self.flanking_spread_rate = size.flanking_spread_rate(SpeedUnits::FeetPerMinute);

        // Direction of interest spread rate
        if has_direction_of_interest {
            self.spread_rate_in_direction_of_interest =
                self.calculate_spread_rate_at_vector(direction_of_interest, direction_mode, size);
        } else {
            self.spread_rate_in_direction_of_interest = self.forward_spread_rate;
        }

        // Heat per unit area, fireline intensities, flame lengths
        self.calculate_heat_per_unit_area();
        self.calculate_fireline_intensities();
        self.calculate_flame_lengths();

        // Western Aspen mortality (if applicable)
        // Note: western aspen module not yet ported, so this is a stub
        // if inputs.is_using_western_aspen() {
        //     self.fuelbed.calculate_western_aspen_mortality(self.forward_flame_length);
        // }

        self.max_flame_length = self.forward_flame_length; // Used by SAFETY module
        self.calculate_heat_source();

        if has_direction_of_interest {
            self.spread_rate_in_direction_of_interest
        } else {
            self.forward_spread_rate
        }
    }

    /// Calculate no-wind no-slope spread rate.
    ///
    /// C++ method: `calculateNoWindNoSlopeSpreadRate`
    pub fn calculate_no_wind_no_slope_spread_rate_static(
        reaction_intensity: f64,
        propagating_flux: f64,
        heat_sink: f64,
    ) -> f64 {
        if heat_sink < 1e-07 {
            0.0
        } else {
            reaction_intensity * propagating_flux / heat_sink
        }
    }

    /// Calculate spread rate at an arbitrary vector direction.
    ///
    /// Uses Catchpole et al. (1982) elliptical perimeter equation.
    ///
    /// C++ method: `calculateSpreadRateAtVector`
    pub fn calculate_spread_rate_at_vector(
        &self,
        direction_of_interest: f64,
        direction_mode: SurfaceFireSpreadDirectionMode,
        size: &FireSize,
    ) -> f64 {
        // Normalize direction to [0, 360)
        let mut dir = direction_of_interest;
        while dir < 0.0 {
            dir += 360.0;
        }
        while dir >= 360.0 {
            dir -= 360.0;
        }

        let mut ros_vector = self.forward_spread_rate;
        let eccentricity = size.eccentricity();

        if self.forward_spread_rate > 0.0 {
            // Beta: angle between direction of max spread and direction of interest
            let mut beta = (self.direction_of_max_spread - dir).abs();
            if beta > 180.0 {
                beta = 360.0 - beta;
            }

            let radians = beta * PI / 180.0;
            let cos_beta = radians.cos();
            let sin_beta = radians.sin();

            // Catchpole et al. (1982) elliptical perimeter equation
            let l = self.forward_spread_rate + self.backing_spread_rate;
            let f = l / 2.0;
            let g = self.forward_spread_rate - f;
            let h = size.flanking_spread_rate(SpeedUnits::FeetPerMinute);

            ros_vector = (g * cos_beta)
                + (f * f * cos_beta * cos_beta + h * h * sin_beta * sin_beta).sqrt();

            if direction_mode == SurfaceFireSpreadDirectionMode::FromIgnitionPoint {
                ros_vector = self.forward_spread_rate * (1.0 - eccentricity)
                    / (1.0 - eccentricity * cos_beta);
            }
        }
        ros_vector
    }

    /// Calculate scorch height for given fireline intensity, wind speed, and
    /// air temperature.
    ///
    /// Inputs: fireline_intensity in Btu/ft/s, midflame_wind_speed in mi/hr,
    /// air_temperature in degrees Fahrenheit.
    pub fn calculate_scorch_height_static(
        fireline_intensity: f64,
        midflame_wind_speed: f64,
        air_temperature: f64,
    ) -> f64 {
        if fireline_intensity < 1e-07 {
            0.0
        } else {
            (63.0 / (140.0 - air_temperature))
                * fireline_intensity.powf(1.166667)
                / (fireline_intensity
                    + midflame_wind_speed * midflame_wind_speed * midflame_wind_speed)
                    .sqrt()
        }
    }

    // --- Public getters ---

    pub fn spread_rate(&self) -> f64 {
        self.forward_spread_rate
    }

    pub fn spread_rate_in_direction_of_interest(&self) -> f64 {
        self.spread_rate_in_direction_of_interest
    }

    pub fn direction_of_max_spread(&self) -> f64 {
        self.direction_of_max_spread
    }

    pub fn effective_wind_speed(&self) -> f64 {
        self.effective_wind_speed
    }

    pub fn fireline_intensity(&self) -> f64 {
        self.forward_fireline_intensity
    }

    pub fn backing_fireline_intensity(&self) -> f64 {
        self.backing_fireline_intensity
    }

    pub fn flanking_fireline_intensity(&self) -> f64 {
        self.flanking_fireline_intensity
    }

    pub fn fireline_intensity_in_direction_of_interest(&self) -> f64 {
        self.direction_of_interest_fireline_intensity
    }

    pub fn flame_length(&self) -> f64 {
        self.forward_flame_length
    }

    pub fn backing_flame_length(&self) -> f64 {
        self.backing_flame_length
    }

    pub fn flanking_flame_length(&self) -> f64 {
        self.flanking_flame_length
    }

    pub fn flame_length_in_direction_of_interest(&self) -> f64 {
        self.direction_of_interest_flame_length
    }

    pub fn max_flame_length(&self) -> f64 {
        self.max_flame_length
    }

    pub fn fire_length_to_width_ratio(&self) -> f64 {
        self.fire_length_to_width_ratio
    }

    pub fn heat_per_unit_area(&self) -> f64 {
        self.heat_per_unit_area
    }

    pub fn residence_time(&self) -> f64 {
        self.residence_time
    }

    pub fn wind_speed_limit(&self) -> f64 {
        self.wind_speed_limit
    }

    pub fn midflame_wind_speed(&self) -> f64 {
        self.midflame_wind_speed
    }

    pub fn slope_factor(&self) -> f64 {
        self.phi_s
    }

    pub fn heat_sink(&self) -> f64 {
        self.fuelbed.heat_sink()
    }

    pub fn heat_source(&self) -> f64 {
        self.heat_source
    }

    pub fn bulk_density(&self) -> f64 {
        self.fuelbed.bulk_density()
    }

    pub fn reaction_intensity_value(&self) -> f64 {
        self.reaction_intensity
    }

    pub fn reaction_intensity_for_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.reaction.reaction_intensity_for_life_state(life_state)
    }

    pub fn moisture_of_extinction_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.fuelbed.moisture_of_extinction_by_life_state(life_state)
    }

    pub fn weighted_moisture_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.fuelbed.weighted_moisture_by_life_state(life_state)
    }

    pub fn wind_adjustment_factor_value(&self) -> f64 {
        self.wind_adjustment_factor
    }

    pub fn characteristic_savr(&self) -> f64 {
        self.fuelbed.sigma()
    }

    pub fn is_wind_limit_exceeded(&self) -> bool {
        self.is_wind_limit_exceeded
    }

    pub fn relative_packing_ratio(&self) -> f64 {
        self.fuelbed.relative_packing_ratio()
    }

    pub fn packing_ratio(&self) -> f64 {
        self.fuelbed.packing_ratio()
    }

    pub fn no_wind_no_slope_spread_rate(&self) -> f64 {
        self.no_wind_no_slope_spread_rate
    }

    pub fn scorch_height(&self) -> f64 {
        self.scorch_height
    }

    pub fn fuelbed(&self) -> &FuelbedIntermediates {
        &self.fuelbed
    }

    pub fn fuelbed_mut(&mut self) -> &mut FuelbedIntermediates {
        &mut self.fuelbed
    }

    // --- Setters (used by SurfaceTwoFuelModels, friend class in C++) ---

    pub fn set_direction_of_max_spread(&mut self, value: f64) {
        self.direction_of_max_spread = value;
    }

    pub fn set_effective_wind_speed(&mut self, value: f64) {
        self.effective_wind_speed = value;
    }

    pub fn set_fireline_intensity(&mut self, value: f64) {
        self.forward_fireline_intensity = value;
    }

    pub fn set_flame_length(&mut self, value: f64) {
        self.forward_flame_length = value;
    }

    pub fn set_fire_length_to_width_ratio(&mut self, value: f64) {
        self.fire_length_to_width_ratio = value;
    }

    pub fn set_residence_time(&mut self, value: f64) {
        self.residence_time = value;
    }

    pub fn set_wind_speed_limit(&mut self, value: f64) {
        self.wind_speed_limit = value;
    }

    pub fn set_reaction_intensity(&mut self, value: f64) {
        self.reaction_intensity = value;
    }

    pub fn set_heat_per_unit_area(&mut self, value: f64) {
        self.heat_per_unit_area = value;
    }

    pub fn set_is_wind_limit_enabled(&mut self, value: bool) {
        self.is_wind_limit_enabled = value;
    }

    pub fn set_is_wind_limit_exceeded(&mut self, value: bool) {
        self.is_wind_limit_exceeded = value;
    }

    pub fn set_wind_adjustment_factor(&mut self, value: f64) {
        self.wind_adjustment_factor = value;
    }

    pub fn set_midflame_wind_speed(&mut self, value: f64) {
        self.midflame_wind_speed = value;
    }

    /// Set forward spread rate directly (used by TwoFuelModels).
    /// C++ accesses `forwardSpreadRate_` directly via friend class.
    pub fn set_forward_spread_rate(&mut self, value: f64) {
        self.forward_spread_rate = value;
    }

    // --- Internal calculations ---

    /// Calculate midflame wind speed from input wind speed and height mode.
    fn calculate_midflame_wind_speed(&mut self, inputs: &SurfaceInputs) {
        let mut wind_speed = inputs.wind_speed(SpeedUnits::FeetPerMinute);
        let wind_height_input_mode = inputs.wind_height_input_mode();

        if wind_height_input_mode == WindHeightInputMode::DirectMidflame {
            self.midflame_wind_speed = wind_speed;
        } else if wind_height_input_mode == WindHeightInputMode::TwentyFoot
            || wind_height_input_mode == WindHeightInputMode::TenMeter
        {
            if wind_height_input_mode == WindHeightInputMode::TenMeter {
                wind_speed /= 1.15;
            }

            let waf_method = inputs.wind_adjustment_factor_calculation_method();
            if waf_method == WindAdjustmentFactorCalculationMethod::UserInput {
                self.wind_adjustment_factor = inputs.user_provided_wind_adjustment_factor();
            } else {
                self.calculate_wind_adjustment_factor(inputs);
            }
            self.midflame_wind_speed = self.wind_adjustment_factor * wind_speed;
        }
    }

    /// Calculate wind adjustment factor based on canopy and fuelbed properties.
    fn calculate_wind_adjustment_factor(&mut self, inputs: &SurfaceInputs) {
        let mut waf_calc = WindAdjustmentFactor::new();

        let canopy_cover = inputs.canopy_cover(FractionUnits::Fraction);
        let canopy_height = inputs.canopy_height(LengthUnits::Feet);
        let crown_ratio = inputs.crown_ratio(FractionUnits::Fraction);
        let fuelbed_depth = self.fuelbed.fuelbed_depth();

        let waf_method = inputs.wind_adjustment_factor_calculation_method();
        if waf_method == WindAdjustmentFactorCalculationMethod::UseCrownRatio {
            self.wind_adjustment_factor = waf_calc.calculate_with_crown_ratio(
                canopy_cover,
                canopy_height,
                crown_ratio,
                fuelbed_depth,
            );
        } else if waf_method == WindAdjustmentFactorCalculationMethod::DontUseCrownRatio {
            self.wind_adjustment_factor = waf_calc.calculate_without_crown_ratio(
                canopy_cover,
                canopy_height,
                fuelbed_depth,
            );
        }
        self.wind_adjustment_factor_shelter_method = waf_calc.shelter_method();
    }

    /// Calculate wind factor (phiW), Rothermel 1972 equations 47-50.
    fn calculate_wind_factor(&mut self) {
        let sigma = self.fuelbed.sigma();
        let relative_packing_ratio = self.fuelbed.relative_packing_ratio();

        self.wind_c = 7.47 * (-0.133 * sigma.powf(0.55)).exp();
        self.wind_b = 0.02526 * sigma.powf(0.54);
        self.wind_e = 0.715 * (-0.000359 * sigma).exp();

        if self.midflame_wind_speed < 1e-07 {
            self.phi_w = 0.0;
        } else {
            self.phi_w = self.midflame_wind_speed.powf(self.wind_b)
                * self.wind_c
                * relative_packing_ratio.powf(-self.wind_e);
        }
    }

    /// Calculate slope factor (phiS).
    fn calculate_slope_factor(&mut self, inputs: &SurfaceInputs) {
        let packing_ratio = self.fuelbed.packing_ratio();
        let slope_degrees = inputs.slope(SlopeUnits::Degrees);
        let slope_tan = (slope_degrees / 180.0 * PI).tan();
        self.phi_s = 5.275 * packing_ratio.powf(-0.3) * (slope_tan * slope_tan);
    }

    /// Calculate wind speed limit (0.9 * reaction_intensity).
    /// Also caps slope factor if it exceeds the limit.
    fn calculate_wind_speed_limit(&mut self) {
        self.wind_speed_limit = 0.9 * self.reaction_intensity;
        if self.phi_s > 0.0 && self.phi_s > self.wind_speed_limit {
            self.phi_s = self.wind_speed_limit;
        }
    }

    /// Calculate direction of maximum fire spread.
    ///
    /// Vector sum of slope and wind components, then convert to azimuth.
    fn calculate_direction_of_max_spread(&mut self, inputs: &SurfaceInputs) {
        let mut corrected_wind_direction = inputs.wind_direction();

        let orientation_mode = inputs.wind_and_spread_orientation_mode();
        if orientation_mode == WindAndSpreadOrientationMode::RelativeToNorth {
            let aspect = inputs.aspect();
            corrected_wind_direction -= aspect;
        }

        let wind_dir_radians = corrected_wind_direction * PI / 180.0;

        // Calculate wind and slope rate components
        let slope_rate = self.no_wind_no_slope_spread_rate * self.phi_s;
        let wind_rate = self.no_wind_no_slope_spread_rate * self.phi_w;

        let x = slope_rate + (wind_rate * wind_dir_radians.cos());
        let y = wind_rate * wind_dir_radians.sin();
        let rate_vector = (x * x + y * y).sqrt();

        // Apply combined wind/slope rate
        self.forward_spread_rate = self.no_wind_no_slope_spread_rate + rate_vector;

        // Calculate azimuth
        let mut azimuth = y.atan2(x) * 180.0 / PI;

        if azimuth < -1e-20 {
            azimuth += 360.0;
        }

        // Undocumented hack from BehavePlus code
        if azimuth.abs() < 0.5 {
            azimuth = 0.0;
        }

        // Convert to relative-to-north if necessary
        if orientation_mode == WindAndSpreadOrientationMode::RelativeToNorth {
            azimuth = Self::convert_direction_to_relative_to_north(azimuth, inputs.aspect());
            while azimuth >= 360.0 {
                azimuth -= 360.0;
            }
        }

        self.direction_of_max_spread = azimuth;
    }

    /// Calculate effective wind speed from combined wind+slope spread rate.
    fn calculate_effective_wind_speed(&mut self) {
        let phi_effective_wind =
            self.forward_spread_rate / self.no_wind_no_slope_spread_rate - 1.0;
        let relative_packing_ratio = self.fuelbed.relative_packing_ratio();
        self.effective_wind_speed = ((phi_effective_wind
            * relative_packing_ratio.powf(self.wind_e))
            / self.wind_c)
            .powf(1.0 / self.wind_b);
    }

    /// Apply wind speed limit by capping effective wind speed and
    /// recalculating spread rate.
    fn apply_wind_speed_limit(&mut self) {
        self.is_wind_limit_exceeded = true;
        self.effective_wind_speed = self.wind_speed_limit;

        let relative_packing_ratio = self.fuelbed.relative_packing_ratio();
        let phi_effective_wind = self.wind_c
            * self.wind_speed_limit.powf(self.wind_b)
            * relative_packing_ratio.powf(-self.wind_e);
        self.forward_spread_rate =
            self.no_wind_no_slope_spread_rate * (1.0 + phi_effective_wind);
    }

    /// Calculate residence time (min) = 384 / sigma.
    fn calculate_residence_time(&mut self) {
        let sigma = self.fuelbed.sigma();
        self.residence_time = if sigma < 1e-07 { 0.0 } else { 384.0 / sigma };
    }

    /// Calculate heat per unit area = reaction_intensity * residence_time.
    fn calculate_heat_per_unit_area(&mut self) {
        self.heat_per_unit_area = self.reaction_intensity * self.residence_time;
    }

    /// Calculate heat source = RI * propagating_flux * (1 + phiS + phiW).
    fn calculate_heat_source(&mut self) {
        let propagating_flux = self.fuelbed.propagating_flux();
        self.heat_source =
            self.reaction_intensity * propagating_flux * (1.0 + self.phi_s + self.phi_w);
    }

    /// Calculate fireline intensity = spread_rate * RI * (residence_time / 60).
    ///
    /// Result in Btu/ft/s (base units for fireline intensity).
    fn calculate_fireline_intensity_for_rate(&self, spread_rate_fpm: f64) -> f64 {
        spread_rate_fpm * self.reaction_intensity * (self.residence_time / 60.0)
    }

    /// Calculate fireline intensities for all directions.
    fn calculate_fireline_intensities(&mut self) {
        self.forward_fireline_intensity =
            self.calculate_fireline_intensity_for_rate(self.forward_spread_rate);
        self.backing_fireline_intensity =
            self.calculate_fireline_intensity_for_rate(self.backing_spread_rate);
        self.flanking_fireline_intensity =
            self.calculate_fireline_intensity_for_rate(self.flanking_spread_rate);
        self.direction_of_interest_fireline_intensity =
            self.calculate_fireline_intensity_for_rate(self.spread_rate_in_direction_of_interest);
    }

    /// Calculate flame length from fireline intensity using Byram 1959.
    ///
    /// flame_length = 0.45 * FLI^0.46 (with FLI in Btu/ft/s)
    fn calculate_flame_length_from_intensity(fireline_intensity: f64) -> f64 {
        if fireline_intensity < 1e-07 {
            0.0
        } else {
            0.45 * fireline_intensity.powf(0.46)
        }
    }

    /// Calculate flame lengths for all directions.
    fn calculate_flame_lengths(&mut self) {
        self.forward_flame_length =
            Self::calculate_flame_length_from_intensity(self.forward_fireline_intensity);
        self.backing_flame_length =
            Self::calculate_flame_length_from_intensity(self.backing_fireline_intensity);
        self.flanking_flame_length =
            Self::calculate_flame_length_from_intensity(self.flanking_fireline_intensity);
        self.direction_of_interest_flame_length =
            Self::calculate_flame_length_from_intensity(self.direction_of_interest_fireline_intensity);
    }

    /// Calculate scorch height from internal state.
    fn calculate_scorch_height(&mut self, inputs: &SurfaceInputs) {
        let air_temperature = inputs.air_temperature(TemperatureUnits::Fahrenheit);
        let wind_speed = inputs.wind_speed(SpeedUnits::MilesPerHour);
        self.scorch_height = if self.forward_fireline_intensity < 1e-07 {
            0.0
        } else {
            (63.0 / (140.0 - air_temperature))
                * self.forward_fireline_intensity.powf(1.166667)
                / (self.forward_fireline_intensity
                    + wind_speed * wind_speed * wind_speed)
                    .sqrt()
        };
    }

    /// Convert direction of spread from upslope-relative to north-relative.
    fn convert_direction_to_relative_to_north(
        direction_from_upslope: f64,
        aspect: f64,
    ) -> f64 {
        direction_from_upslope + aspect + 180.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuel_models::FuelModels;
    use crate::inputs::SurfaceInputs;
    use firelab_base::{
        FractionUnits, LengthUnits, SlopeUnits,
        WindAndSpreadOrientationMode, WindHeightInputMode,
    };

    fn assert_near(a: f64, b: f64, tol: f64) {
        assert!(
            (a - b).abs() < tol,
            "expected {b}, got {a} (diff {})",
            (a - b).abs()
        );
    }

    /// Helper to create standard test inputs and run the fire calculator.
    fn run_fire(
        fuel_model: i32,
        m1h: f64,
        m10h: f64,
        m100h: f64,
        m_herb: f64,
        m_woody: f64,
        wind_speed: f64,
        wind_speed_units: SpeedUnits,
        slope: f64,
        slope_units: SlopeUnits,
    ) -> (SurfaceFire, FireSize) {
        let fm = FuelModels::new();
        let mut inputs = SurfaceInputs::new();
        inputs.update_surface_inputs(
            fuel_model,
            m1h, m10h, m100h, m_herb, m_woody,
            FractionUnits::Fraction,
            wind_speed, wind_speed_units,
            WindHeightInputMode::DirectMidflame,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            slope, slope_units, 0.0,
            0.0, FractionUnits::Fraction,
            0.0, LengthUnits::Feet,
            0.0, FractionUnits::Fraction,
        );
        let mut fire = SurfaceFire::new();
        let mut size = FireSize::new();
        fire.calculate_forward_spread_rate(
            fuel_model,
            false,
            0.0,
            SurfaceFireSpreadDirectionMode::FromPerimeter,
            &fm,
            &inputs,
            &mut size,
        );
        (fire, size)
    }

    #[test]
    fn fm1_no_wind_no_slope_positive_spread() {
        let (fire, _size) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            0.0, SpeedUnits::FeetPerMinute,
            0.0, SlopeUnits::Degrees,
        );
        assert!(fire.spread_rate() > 0.0, "spread_rate={}", fire.spread_rate());
        assert!(fire.flame_length() > 0.0, "flame_length={}", fire.flame_length());
        assert!(fire.fireline_intensity() > 0.0);
        assert!(fire.heat_per_unit_area() > 0.0);
        assert!(fire.residence_time() > 0.0);
    }

    #[test]
    fn fm1_wind_increases_spread() {
        let (fire_no_wind, _) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            0.0, SpeedUnits::FeetPerMinute,
            0.0, SlopeUnits::Degrees,
        );
        let (fire_with_wind, _) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            440.0, SpeedUnits::FeetPerMinute, // ~5 mph midflame
            0.0, SlopeUnits::Degrees,
        );
        assert!(
            fire_with_wind.spread_rate() > fire_no_wind.spread_rate(),
            "wind should increase spread: wind={}, no_wind={}",
            fire_with_wind.spread_rate(),
            fire_no_wind.spread_rate()
        );
    }

    #[test]
    fn fm1_slope_increases_spread() {
        let (fire_flat, _) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            0.0, SpeedUnits::FeetPerMinute,
            0.0, SlopeUnits::Degrees,
        );
        let (fire_slope, _) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            0.0, SpeedUnits::FeetPerMinute,
            30.0, SlopeUnits::Degrees,
        );
        assert!(
            fire_slope.spread_rate() > fire_flat.spread_rate(),
            "slope should increase spread: slope={}, flat={}",
            fire_slope.spread_rate(),
            fire_flat.spread_rate()
        );
    }

    #[test]
    fn fm10_spread_rate_positive() {
        let (fire, _) = run_fire(
            10, 0.06, 0.07, 0.08, 0.60, 1.50,
            440.0, SpeedUnits::FeetPerMinute,
            0.0, SlopeUnits::Degrees,
        );
        assert!(fire.spread_rate() > 0.0);
        assert!(fire.flame_length() > 0.0);
    }

    #[test]
    fn no_wind_no_slope_spread_rate_static() {
        let result =
            SurfaceFire::calculate_no_wind_no_slope_spread_rate_static(100.0, 0.5, 200.0);
        assert_near(result, 100.0 * 0.5 / 200.0, 1e-10);
    }

    #[test]
    fn no_wind_no_slope_zero_heat_sink() {
        let result =
            SurfaceFire::calculate_no_wind_no_slope_spread_rate_static(100.0, 0.5, 0.0);
        assert_near(result, 0.0, 1e-10);
    }

    #[test]
    fn flame_length_from_intensity() {
        // Byram 1959: FL = 0.45 * FLI^0.46
        let fl = SurfaceFire::calculate_flame_length_from_intensity(100.0);
        let expected = 0.45 * 100.0_f64.powf(0.46);
        assert_near(fl, expected, 1e-6);
    }

    #[test]
    fn flame_length_zero_intensity() {
        let fl = SurfaceFire::calculate_flame_length_from_intensity(0.0);
        assert_near(fl, 0.0, 1e-10);
    }

    #[test]
    fn scorch_height_static() {
        // FLI=100, wind=5mph, temp=70F
        let sh = SurfaceFire::calculate_scorch_height_static(100.0, 5.0, 70.0);
        assert!(sh > 0.0, "scorch_height={}", sh);
    }

    #[test]
    fn scorch_height_zero_intensity() {
        let sh = SurfaceFire::calculate_scorch_height_static(0.0, 5.0, 70.0);
        assert_near(sh, 0.0, 1e-10);
    }

    #[test]
    fn residence_time_formula() {
        // sigma = 1500 → residence_time = 384/1500 = 0.256 min
        let (fire, _) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            0.0, SpeedUnits::FeetPerMinute,
            0.0, SlopeUnits::Degrees,
        );
        let sigma = fire.characteristic_savr();
        assert!(sigma > 0.0);
        assert_near(fire.residence_time(), 384.0 / sigma, 1e-10);
    }

    #[test]
    fn backing_flanking_rates_with_wind() {
        let (fire, size) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            440.0, SpeedUnits::FeetPerMinute,
            0.0, SlopeUnits::Degrees,
        );
        let backing = size.backing_spread_rate(SpeedUnits::FeetPerMinute);
        let flanking = size.flanking_spread_rate(SpeedUnits::FeetPerMinute);
        assert!(backing > 0.0, "backing={}", backing);
        assert!(flanking > 0.0, "flanking={}", flanking);
        assert!(
            fire.spread_rate() > backing,
            "forward={} should > backing={}",
            fire.spread_rate(), backing
        );
    }

    #[test]
    fn direction_of_max_spread_no_wind_is_upslope() {
        // With slope but no wind, direction of max spread should be upslope (0)
        let (fire, _) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            0.0, SpeedUnits::FeetPerMinute,
            30.0, SlopeUnits::Degrees,
        );
        assert_near(fire.direction_of_max_spread(), 0.0, 1.0);
    }

    #[test]
    fn convert_direction_to_north() {
        // Upslope direction 0 with aspect 180 → north-relative = 0+180+180 = 360 → 0
        let result = SurfaceFire::convert_direction_to_relative_to_north(0.0, 180.0);
        assert_near(result, 360.0, 1e-10);
    }

    #[test]
    fn fireline_intensities_consistent() {
        let (fire, _) = run_fire(
            1, 0.06, 0.07, 0.08, 0.60, 1.50,
            440.0, SpeedUnits::FeetPerMinute,
            0.0, SlopeUnits::Degrees,
        );
        // Forward FLI should be largest
        assert!(fire.fireline_intensity() >= fire.backing_fireline_intensity());
        assert!(fire.fireline_intensity() >= fire.flanking_fireline_intensity());
    }
}
