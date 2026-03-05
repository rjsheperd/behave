//! Crown fire spread rate, transition, and activity calculations.
//!
//! The Crown struct owns two Surface instances internally (composition pattern
//! matching the C++ design): `surface_fuel` for surface fire calculations and
//! `crown_fuel` for crown fuel model 10 calculations.
//!
//! Implements both Rothermel (1991) and Scott & Reinhardt (2001) methods.
//!
//! C++ source: crown.h / crown.cpp

use firelab_base::{
    AreaUnits, DensityUnits, FireSize, FireType, FirelineIntensityUnits, FractionUnits,
    HeatPerUnitAreaUnits, HeatSinkUnits, HeatSourceAndReactionIntensityUnits, LengthUnits,
    SlopeUnits, SpeedUnits, TimeUnits, UnitConversion, WindAdjustmentFactorCalculationMethod,
    WindAndSpreadOrientationMode, WindHeightInputMode,
};

use behave_surface::facade::Surface;
use behave_surface::fuel_models::FuelModels;
use behave_surface::wind::WindSpeedUtility;

use crate::inputs::CrownInputs;

/// Internal crown-fire model selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrownModelType {
    Rothermel,
    ScottAndReinhardt,
}

/// Crown fire behavior calculator.
///
/// Owns two `Surface` instances (surface fuel and crown fuel) plus a `CrownInputs`
/// and a `FireSize` for crown fire dimensions.
///
/// C++ class: `Crown`
#[derive(Debug, Clone)]
pub struct Crown {
    crown_inputs: CrownInputs,
    surface_fuel: Surface,
    crown_fuel: Surface,
    crown_fire_size: FireSize,

    // Fire type classification
    fire_type: FireType,

    // Surface fire outputs (stored from surface run)
    surface_fire_heat_per_unit_area: f64, // Btu/ft²
    surface_fireline_intensity: f64,      // Btu/ft/s
    surface_fire_spread_rate: f64,        // ft/min
    surface_fire_flame_length: f64,       // ft
    surface_fire_critical_spread_rate: f64, // ft/min (S&R)

    // Crown fire outputs
    crown_fuel_load: f64,                           // lb/ft²
    canopy_heat_per_unit_area: f64,                 // Btu/ft²
    crown_fire_heat_per_unit_area: f64,             // Btu/ft²
    crown_fireline_intensity: f64,                  // Btu/ft/s
    crown_flame_length: f64,                        // ft
    crown_fire_spread_rate: f64,                    // ft/min
    crown_critical_surface_fireline_intensity: f64, // Btu/ft/s
    crown_critical_fire_spread_rate: f64,           // ft/min
    crown_critical_surface_flame_length: f64,       // ft
    crown_fire_active_ratio: f64,
    crown_fire_transition_ratio: f64,
    crown_fire_length_to_width_ratio: f64,
    crown_fire_active_wind_speed: f64, // ft/min
    crown_fraction_burned: f64,
    crowning_surface_fire_ros: f64, // ft/min
    wind_speed_at_twenty_feet: f64, // ft/min

    // Final fire behavior (depends on fire type and model)
    final_spread_rate: f64,           // ft/min
    final_heat_per_unit_area: f64,    // Btu/ft²
    final_fireline_intensity: f64,    // Btu/ft/s
    final_flame_length: f64,          // ft

    // Passive crown fire (S&R)
    passive_crown_fire_spread_rate: f64,        // ft/min
    passive_crown_fire_heat_per_unit_area: f64, // Btu/ft²
    passive_crown_fireline_intensity: f64,      // Btu/ft/s
    passive_crown_flame_length: f64,            // ft

    // Fire type flags
    is_surface_fire: bool,
    is_passive_crown_fire: bool,
    is_active_crown_fire: bool,
    is_crown_fire: bool,
}

impl Crown {
    pub fn new(fuel_models: FuelModels) -> Self {
        let fm2 = fuel_models.clone();
        Self {
            crown_inputs: CrownInputs::new(),
            surface_fuel: Surface::new(fuel_models),
            crown_fuel: Surface::new(fm2),
            crown_fire_size: FireSize::new(),

            fire_type: FireType::Surface,
            surface_fire_heat_per_unit_area: 0.0,
            surface_fireline_intensity: 0.0,
            surface_fire_spread_rate: 0.0,
            surface_fire_flame_length: 0.0,
            surface_fire_critical_spread_rate: 0.0,

            crown_fuel_load: 0.0,
            canopy_heat_per_unit_area: 0.0,
            crown_fire_heat_per_unit_area: 0.0,
            crown_fireline_intensity: 0.0,
            crown_flame_length: 0.0,
            crown_fire_spread_rate: 0.0,
            crown_critical_surface_fireline_intensity: 0.0,
            crown_critical_fire_spread_rate: 0.0,
            crown_critical_surface_flame_length: 0.0,
            crown_fire_active_ratio: 0.0,
            crown_fire_transition_ratio: 0.0,
            crown_fire_length_to_width_ratio: 1.0,
            crown_fire_active_wind_speed: 0.0,
            crown_fraction_burned: 0.0,
            crowning_surface_fire_ros: 0.0,
            wind_speed_at_twenty_feet: 0.0,

            final_spread_rate: 0.0,
            final_heat_per_unit_area: 0.0,
            final_fireline_intensity: 0.0,
            final_flame_length: 0.0,

            passive_crown_fire_spread_rate: 0.0,
            passive_crown_fire_heat_per_unit_area: 0.0,
            passive_crown_fireline_intensity: 0.0,
            passive_crown_flame_length: 0.0,

            is_surface_fire: false,
            is_passive_crown_fire: false,
            is_active_crown_fire: false,
            is_crown_fire: false,
        }
    }

    // -----------------------------------------------------------------------
    // Main calculation methods
    // -----------------------------------------------------------------------

    /// Rothermel (1991) crown fire correlation.
    pub fn do_crown_run_rothermel(&mut self) {
        let canopy_height = self.surface_fuel.get_canopy_height(LengthUnits::Feet);
        let canopy_base_height = self.crown_inputs.canopy_base_height(LengthUnits::Feet);
        let crown_ratio = if canopy_height > 0.0 {
            (canopy_height - canopy_base_height) / canopy_height
        } else {
            0.0
        };

        // Step 1: Do surface run and store values
        if self.surface_fuel.get_wind_adjustment_factor_calculation_method()
            == WindAdjustmentFactorCalculationMethod::UseCrownRatio
        {
            self.surface_fuel
                .set_crown_ratio(crown_ratio, FractionUnits::Fraction);
        }
        self.surface_fuel.do_surface_run_in_direction_of_max_spread();
        self.surface_fire_heat_per_unit_area =
            self.surface_fuel.heat_per_unit_area(HeatPerUnitAreaUnits::BtusPerSquareFoot);
        self.surface_fireline_intensity =
            self.surface_fuel.fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond);
        self.surface_fire_spread_rate =
            self.surface_fuel.spread_rate(SpeedUnits::FeetPerMinute);
        self.surface_fire_flame_length =
            self.surface_fuel.flame_length_output(LengthUnits::Feet);

        // Step 2: Create crown fuel model (fuel model 10)
        self.crown_fuel = self.surface_fuel.clone();
        self.crown_fuel
            .set_wind_adjustment_factor_calculation_method(
                WindAdjustmentFactorCalculationMethod::UserInput,
            );
        self.crown_fuel.set_user_provided_wind_adjustment_factor(0.4);
        self.crown_fuel.set_fuel_model_number(10);
        self.crown_fuel.set_slope(0.0, SlopeUnits::Degrees);
        self.crown_fuel
            .set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
        self.crown_fuel.set_wind_direction(0.0);

        // Step 3: Determine crown fire behavior
        self.crown_fuel.do_surface_run_in_direction_of_max_spread();
        self.crown_fire_spread_rate =
            3.34 * self.crown_fuel.spread_rate(SpeedUnits::FeetPerMinute);

        // Step 4: Calculate remaining crown fire characteristics
        self.calculate_crown_fuel_load();
        self.calculate_canopy_heat_per_unit_area();
        self.calculate_crown_fire_heat_per_unit_area();
        self.calculate_crown_fireline_intensity();
        self.calculate_crown_flame_length();

        self.calculate_crown_critical_fire_spread_rate();
        self.calculate_crown_fire_active_ratio();

        self.calculate_crown_critical_surface_fire_intensity();
        self.calculate_crown_critical_surface_flame_length();
        self.calculate_crown_fire_transition_ratio();

        self.calculate_wind_speed_at_twenty_feet();

        // Calculate crown fire dimensions
        self.crown_fire_size.calculate_fire_basic_dimensions(
            true,
            self.wind_speed_at_twenty_feet,
            SpeedUnits::MilesPerHour,
            self.crown_fire_spread_rate,
            SpeedUnits::FeetPerMinute,
        );
        self.crown_fire_length_to_width_ratio =
            self.crown_fire_size.fire_length_to_width_ratio();

        // Determine fire type
        self.calculate_fire_type_rothermel();
        self.assign_final_fire_behavior(CrownModelType::Rothermel);
    }

    /// Scott & Reinhardt (2001) linked models method.
    pub fn do_crown_run_scott_and_reinhardt(&mut self) {
        let canopy_height = self.surface_fuel.get_canopy_height(LengthUnits::Feet);
        let canopy_base_height = self.crown_inputs.canopy_base_height(LengthUnits::Feet);
        let crown_ratio = if canopy_height > 0.0 {
            (canopy_height - canopy_base_height) / canopy_height
        } else {
            0.0
        };

        // Step 1: Surface run
        if self.surface_fuel.get_wind_adjustment_factor_calculation_method()
            == WindAdjustmentFactorCalculationMethod::UseCrownRatio
        {
            self.surface_fuel
                .set_crown_ratio(crown_ratio, FractionUnits::Fraction);
        }
        let wind_speed = self
            .surface_fuel
            .get_wind_speed(SpeedUnits::FeetPerMinute, WindHeightInputMode::TwentyFoot);
        self.surface_fuel.set_wind_speed(
            wind_speed,
            SpeedUnits::FeetPerMinute,
            WindHeightInputMode::TwentyFoot,
        );
        self.surface_fuel.do_surface_run_in_direction_of_max_spread();
        self.surface_fire_spread_rate =
            self.surface_fuel.spread_rate(SpeedUnits::FeetPerMinute);
        self.surface_fire_heat_per_unit_area =
            self.surface_fuel.heat_per_unit_area(HeatPerUnitAreaUnits::BtusPerSquareFoot);
        self.surface_fireline_intensity =
            self.surface_fuel.fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond);
        self.surface_fire_flame_length =
            self.surface_fuel.flame_length_output(LengthUnits::Feet);

        // Step 2: Crown fuel model (fuel model 10)
        self.crown_fuel = self.surface_fuel.clone();
        self.crown_fuel.set_fuel_model_number(10);
        self.crown_fuel
            .set_wind_adjustment_factor_calculation_method(
                WindAdjustmentFactorCalculationMethod::UserInput,
            );
        self.crown_fuel.set_user_provided_wind_adjustment_factor(0.4);
        self.crown_fuel.set_slope(0.0, SlopeUnits::Degrees);
        self.crown_fuel
            .set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
        self.crown_fuel.set_wind_direction(0.0);
        self.crown_fuel.set_wind_speed(
            wind_speed,
            SpeedUnits::FeetPerMinute,
            WindHeightInputMode::TwentyFoot,
        );

        // Step 3: Crown fire spread rate
        self.crown_fuel.do_surface_run_in_direction_of_max_spread();
        self.crown_fire_spread_rate =
            3.34 * self.crown_fuel.spread_rate(SpeedUnits::FeetPerMinute);

        // Step 4: Remaining characteristics
        self.calculate_crown_fire_active_wind_speed();
        self.calculate_crown_fuel_load();
        self.calculate_canopy_heat_per_unit_area();
        self.calculate_crown_fire_heat_per_unit_area();
        self.calculate_crown_fireline_intensity();
        self.calculate_crown_flame_length();
        self.calculate_crown_critical_fire_spread_rate();
        self.calculate_crown_critical_surface_fire_intensity();
        self.calculate_crown_critical_surface_flame_length();
        self.calculate_crown_fire_active_ratio();
        self.calculate_crown_fire_transition_ratio();
        self.calculate_wind_speed_at_twenty_feet();

        // Crown fire dimensions
        self.crown_fire_size.calculate_fire_basic_dimensions(
            true,
            self.wind_speed_at_twenty_feet,
            SpeedUnits::MilesPerHour,
            self.crown_fire_spread_rate,
            SpeedUnits::FeetPerMinute,
        );
        self.crown_fire_length_to_width_ratio =
            self.crown_fire_size.fire_length_to_width_ratio();

        // Fire type classification (uses Rothermel logic first)
        self.calculate_fire_type_rothermel();
        self.calculate_fire_type_scott_and_reinhardt();

        // S&R critical surface fire spread rate
        self.calculate_surface_fire_critical_spread_rate_scott_and_reinhardt();

        // Crowning surface fire ROS
        self.calculate_crowning_surface_fire_ros();

        // Crown fraction burned
        self.calculate_crown_fraction_burned();

        // Passive crown fire spread rate, HPUA, fireline intensity
        self.passive_crown_fire_spread_rate = self.surface_fire_spread_rate
            + self.crown_fraction_burned
                * (self.crown_fire_spread_rate - self.surface_fire_spread_rate);
        self.passive_crown_fire_heat_per_unit_area = self.surface_fire_heat_per_unit_area
            + self.canopy_heat_per_unit_area * self.crown_fraction_burned;
        self.passive_crown_fireline_intensity = self.passive_crown_fire_heat_per_unit_area
            * self.passive_crown_fire_spread_rate
            / 60.0;

        // Passive flame length
        self.calculate_passive_crown_flame_length();

        // Final fire behavior
        self.assign_final_fire_behavior(CrownModelType::ScottAndReinhardt);
    }

    // -----------------------------------------------------------------------
    // Internal calculation methods
    // -----------------------------------------------------------------------

    fn calculate_crown_fuel_load(&mut self) {
        let cbd = self
            .crown_inputs
            .canopy_bulk_density(DensityUnits::PoundsPerCubicFoot);
        let cbh = self.crown_inputs.canopy_base_height(LengthUnits::Feet);
        let ch = self.surface_fuel.get_canopy_height(LengthUnits::Feet);
        self.crown_fuel_load = cbd * (ch - cbh);
    }

    fn calculate_canopy_heat_per_unit_area(&mut self) {
        const LOW_HEAT_OF_COMBUSTION: f64 = 8000.0; // Btu/lb
        self.canopy_heat_per_unit_area = self.crown_fuel_load * LOW_HEAT_OF_COMBUSTION;
    }

    fn calculate_crown_fire_heat_per_unit_area(&mut self) {
        self.crown_fire_heat_per_unit_area =
            self.surface_fire_heat_per_unit_area + self.canopy_heat_per_unit_area;
    }

    fn calculate_crown_fireline_intensity(&mut self) {
        self.crown_fireline_intensity =
            (self.crown_fire_spread_rate / 60.0) * self.crown_fire_heat_per_unit_area;
    }

    fn calculate_crown_flame_length(&mut self) {
        // Thomas (1963) flame length from fireline intensity (Btu/ft/s)
        if self.crown_fireline_intensity <= 0.0 {
            self.crown_flame_length = 0.0;
        } else {
            self.crown_flame_length =
                0.2 * self.crown_fireline_intensity.powf(2.0 / 3.0);
        }
    }

    fn calculate_passive_crown_flame_length(&mut self) {
        if self.passive_crown_fireline_intensity <= 0.0 {
            self.passive_crown_flame_length = 0.0;
        } else {
            self.passive_crown_flame_length =
                0.2 * self.passive_crown_fireline_intensity.powf(2.0 / 3.0);
        }
    }

    fn calculate_crown_critical_surface_fire_intensity(&mut self) {
        // Get foliar moisture in percent, constrain lower limit
        let mut moisture_foliar = self.crown_inputs.moisture_foliar(FractionUnits::Percent);
        if moisture_foliar < 30.0 {
            moisture_foliar = 30.0;
        }

        // Crown base height in meters, constrain lower limit
        let mut crown_base_height = self.crown_inputs.canopy_base_height(LengthUnits::Meters);
        if crown_base_height < 0.1 {
            crown_base_height = 0.1;
        }

        // Critical surface fireline intensity (kW/m)
        let i_critical_kw_m =
            (0.010 * crown_base_height * (460.0 + 25.9 * moisture_foliar)).powf(1.5);

        // Convert to base units (Btu/ft/s)
        self.crown_critical_surface_fireline_intensity =
            FirelineIntensityUnits::KilowattsPerMeter.to_base(i_critical_kw_m);
    }

    fn calculate_crown_critical_surface_flame_length(&mut self) {
        self.crown_critical_surface_flame_length = Surface::calculate_flame_length(
            self.crown_critical_surface_fireline_intensity,
            FirelineIntensityUnits::BtusPerFootPerSecond,
            LengthUnits::Feet,
        );
    }

    fn calculate_crown_critical_fire_spread_rate(&mut self) {
        // CBD in kg/m³
        let cbd_kg_m3 = self
            .crown_inputs
            .canopy_bulk_density(DensityUnits::KilogramsPerCubicMeter);

        let r_critical_m_min = if cbd_kg_m3 < 1e-07 {
            0.0
        } else {
            3.0 / cbd_kg_m3
        };

        // Convert m/min to ft/min (base units)
        self.crown_critical_fire_spread_rate =
            SpeedUnits::MetersPerMinute.to_base(r_critical_m_min);
    }

    fn calculate_crown_fire_active_ratio(&mut self) {
        self.crown_fire_active_ratio = if self.crown_critical_fire_spread_rate < 1e-07 {
            0.0
        } else {
            self.crown_fire_spread_rate / self.crown_critical_fire_spread_rate
        };
    }

    fn calculate_crown_fire_transition_ratio(&mut self) {
        self.crown_fire_transition_ratio =
            if self.crown_critical_surface_fireline_intensity < 1e-07 {
                0.0
            } else {
                self.surface_fireline_intensity
                    / self.crown_critical_surface_fireline_intensity
            };
    }

    fn calculate_wind_speed_at_twenty_feet(&mut self) {
        let mode = self.surface_fuel.get_wind_height_input_mode();
        if mode == WindHeightInputMode::TwentyFoot {
            self.wind_speed_at_twenty_feet = self
                .surface_fuel
                .get_wind_speed(SpeedUnits::FeetPerMinute, mode);
        } else if mode == WindHeightInputMode::TenMeter {
            let ws_10m = self
                .surface_fuel
                .get_wind_speed(SpeedUnits::FeetPerMinute, mode);
            self.wind_speed_at_twenty_feet =
                WindSpeedUtility::ten_meter_to_twenty_foot(ws_10m);
        }
    }

    fn calculate_crown_fire_active_wind_speed(&mut self) {
        // O'active: 20-ft wind speed at which crown canopy becomes fully available
        // Scott & Reinhardt (2001) equation 20
        let cbd = 16.0185
            * self
                .crown_inputs
                .canopy_bulk_density(DensityUnits::PoundsPerCubicFoot);
        let r_active = 3.28084 * (3.0 / cbd); // R'active, ft/min
        let r10 = r_active / 3.34;            // R10 from R'active = 3.34 * R10

        // Fuel model 10 constants (precomputed)
        let prop_flux: f64 = 0.048317062998571636;
        let reaction_intensity = self.crown_fuel.reaction_intensity(
            HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerMinute,
        );
        let heat_sink = self.crown_fuel.heat_sink(HeatSinkUnits::BtusPerCubicFoot);
        let ros0 = reaction_intensity * prop_flux / heat_sink;

        let wind_b: f64 = 1.4308256324729873;
        let wind_b_inv = 1.0 / wind_b;
        let wind_k: f64 = 0.0016102128596515481;
        let slope_factor = 0.0;

        let a = ((r10 / ros0) - 1.0 - slope_factor) / wind_k;
        let u_mid = a.powf(wind_b_inv); // midflame wind speed (ft/min)
        self.crown_fire_active_wind_speed = u_mid / 0.4; // 20-ft wind speed for waf=0.4
    }

    fn calculate_fire_type_rothermel(&mut self) {
        self.fire_type = FireType::Surface;
        self.is_active_crown_fire = false;
        self.is_passive_crown_fire = false;
        self.is_surface_fire = true;

        if self.crown_fire_transition_ratio < 1.0 {
            if self.crown_fire_active_ratio < 1.0 {
                self.fire_type = FireType::Surface;
            } else {
                self.fire_type = FireType::ConditionalCrownFire;
            }
        } else {
            if self.crown_fire_active_ratio < 1.0 {
                self.fire_type = FireType::Torching;
                self.is_passive_crown_fire = true;
                self.is_surface_fire = false;
            } else {
                self.fire_type = FireType::Crowning;
                self.is_active_crown_fire = true;
                self.is_surface_fire = false;
            }
        }
    }

    fn calculate_fire_type_scott_and_reinhardt(&mut self) {
        self.is_surface_fire =
            self.fire_type == FireType::Surface || self.fire_type == FireType::ConditionalCrownFire;
        self.is_passive_crown_fire = self.fire_type == FireType::Torching;
        self.is_active_crown_fire = self.fire_type == FireType::Crowning;
        self.is_crown_fire = self.is_active_crown_fire || self.is_passive_crown_fire;
    }

    fn calculate_surface_fire_critical_spread_rate_scott_and_reinhardt(&mut self) {
        self.surface_fire_critical_spread_rate =
            (60.0 * self.crown_critical_surface_fireline_intensity)
                / self.surface_fire_heat_per_unit_area;
    }

    fn calculate_crowning_surface_fire_ros(&mut self) {
        // Run surface fire at the crown fire active wind speed to find R'sa
        let surface_temp = self.surface_fuel.clone();
        self.surface_fuel.set_wind_speed(
            self.crown_fire_active_wind_speed,
            SpeedUnits::FeetPerMinute,
            WindHeightInputMode::TwentyFoot,
        );
        self.surface_fuel.do_surface_run_in_direction_of_max_spread();
        self.crowning_surface_fire_ros =
            self.surface_fuel.spread_rate(SpeedUnits::FeetPerMinute);
        self.surface_fuel = surface_temp; // Restore state
    }

    fn calculate_crown_fraction_burned(&mut self) {
        let numerator = self.surface_fire_spread_rate - self.surface_fire_critical_spread_rate;
        let denominator =
            self.crowning_surface_fire_ros - self.surface_fire_critical_spread_rate;

        self.crown_fraction_burned = if denominator > 1e-07 {
            numerator / denominator
        } else {
            0.0
        };
        self.crown_fraction_burned = self.crown_fraction_burned.clamp(0.0, 1.0);
    }

    fn assign_final_fire_behavior(&mut self, model_type: CrownModelType) {
        match model_type {
            CrownModelType::ScottAndReinhardt => {
                if self.is_surface_fire {
                    self.final_spread_rate = self.surface_fire_spread_rate;
                    self.final_heat_per_unit_area = self.surface_fire_heat_per_unit_area;
                    self.final_fireline_intensity = self.surface_fireline_intensity;
                    self.final_flame_length = self.surface_fire_flame_length;
                } else if self.is_passive_crown_fire {
                    self.final_spread_rate = self.passive_crown_fire_spread_rate;
                    self.final_heat_per_unit_area =
                        self.passive_crown_fire_heat_per_unit_area;
                    self.final_fireline_intensity = self.passive_crown_fireline_intensity;
                    self.final_flame_length = self.passive_crown_flame_length;
                } else if self.is_active_crown_fire {
                    self.final_spread_rate = self.crown_fire_spread_rate;
                    self.final_heat_per_unit_area = self.crown_fire_heat_per_unit_area;
                    self.final_fireline_intensity = self.crown_fireline_intensity;
                    self.final_flame_length = self.crown_flame_length;
                }
            }
            CrownModelType::Rothermel => {
                if self.is_surface_fire {
                    self.final_spread_rate = self.surface_fire_spread_rate;
                    self.final_heat_per_unit_area = self.surface_fire_heat_per_unit_area;
                    self.final_fireline_intensity = self.surface_fireline_intensity;
                    self.final_flame_length = self.surface_fire_flame_length;
                } else if self.fire_type == FireType::Torching {
                    self.final_spread_rate = self.surface_fire_spread_rate;
                    self.final_heat_per_unit_area = self.crown_fire_heat_per_unit_area;
                    self.final_fireline_intensity = self.crown_fireline_intensity;
                    self.final_flame_length = self.crown_flame_length;
                } else if self.fire_type == FireType::Crowning {
                    self.final_spread_rate = self.crown_fire_spread_rate;
                    self.final_heat_per_unit_area = self.crown_fire_heat_per_unit_area;
                    self.final_fireline_intensity = self.crown_fireline_intensity;
                    self.final_flame_length = self.crown_flame_length;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Crown module setters
    // -----------------------------------------------------------------------

    /// Bulk input setter matching C++ `updateCrownInputs`.
    pub fn update_crown_inputs(
        &mut self,
        fuel_model_number: i32,
        moisture_one_hour: f64,
        moisture_ten_hour: f64,
        moisture_hundred_hour: f64,
        moisture_live_herbaceous: f64,
        moisture_live_woody: f64,
        moisture_foliar: f64,
        moisture_units: FractionUnits,
        wind_speed: f64,
        wind_speed_units: SpeedUnits,
        wind_height_input_mode: WindHeightInputMode,
        wind_direction: f64,
        wind_and_spread_orientation_mode: WindAndSpreadOrientationMode,
        slope: f64,
        slope_units: SlopeUnits,
        aspect: f64,
        canopy_cover: f64,
        cover_units: FractionUnits,
        canopy_height: f64,
        canopy_base_height: f64,
        canopy_height_units: LengthUnits,
        crown_ratio: f64,
        crown_ratio_units: FractionUnits,
        canopy_bulk_density: f64,
        density_units: DensityUnits,
    ) {
        self.surface_fuel.update_surface_inputs(
            fuel_model_number,
            moisture_one_hour,
            moisture_ten_hour,
            moisture_hundred_hour,
            moisture_live_herbaceous,
            moisture_live_woody,
            moisture_units,
            wind_speed,
            wind_speed_units,
            wind_height_input_mode,
            wind_direction,
            wind_and_spread_orientation_mode,
            slope,
            slope_units,
            aspect,
            canopy_cover,
            cover_units,
            canopy_height,
            canopy_height_units,
            crown_ratio,
            crown_ratio_units,
        );
        self.crown_inputs.update_crown_inputs(
            canopy_base_height,
            canopy_height_units,
            canopy_bulk_density,
            density_units,
            moisture_foliar,
            moisture_units,
        );
    }

    pub fn set_canopy_base_height(&mut self, height: f64, units: LengthUnits) {
        self.crown_inputs.set_canopy_base_height(height, units);
    }

    pub fn set_canopy_bulk_density(&mut self, density: f64, units: DensityUnits) {
        self.crown_inputs.set_canopy_bulk_density(density, units);
    }

    pub fn set_moisture_foliar(&mut self, moisture: f64, units: FractionUnits) {
        self.crown_inputs.set_moisture_foliar(moisture, units);
    }

    // Pass-through surface setters

    pub fn set_canopy_cover(&mut self, cover: f64, units: FractionUnits) {
        self.surface_fuel.set_canopy_cover(cover, units);
    }

    pub fn set_canopy_height(&mut self, height: f64, units: LengthUnits) {
        self.surface_fuel.set_canopy_height(height, units);
    }

    pub fn set_crown_ratio(&mut self, ratio: f64, _units: FractionUnits) {
        // C++ always passes FractionUnits::Fraction here
        self.surface_fuel.set_crown_ratio(ratio, FractionUnits::Fraction);
    }

    pub fn set_fuel_model_number(&mut self, number: i32) {
        self.surface_fuel.set_fuel_model_number(number);
    }

    pub fn set_moisture_one_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.surface_fuel.set_moisture_one_hour(moisture, units);
    }

    pub fn set_moisture_ten_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.surface_fuel.set_moisture_ten_hour(moisture, units);
    }

    pub fn set_moisture_hundred_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.surface_fuel.set_moisture_hundred_hour(moisture, units);
    }

    pub fn set_moisture_dead_aggregate(&mut self, moisture: f64, units: FractionUnits) {
        self.surface_fuel.set_moisture_dead_aggregate(moisture, units);
        self.crown_fuel.set_moisture_dead_aggregate(moisture, units);
    }

    pub fn set_moisture_live_herbaceous(&mut self, moisture: f64, units: FractionUnits) {
        self.surface_fuel.set_moisture_live_herbaceous(moisture, units);
    }

    pub fn set_moisture_live_woody(&mut self, moisture: f64, units: FractionUnits) {
        self.surface_fuel.set_moisture_live_woody(moisture, units);
        self.crown_fuel.set_moisture_live_woody(moisture, units);
    }

    pub fn set_moisture_live_aggregate(&mut self, moisture: f64, units: FractionUnits) {
        self.surface_fuel.set_moisture_live_aggregate(moisture, units);
        self.crown_fuel.set_moisture_live_aggregate(moisture, units);
    }

    pub fn set_moisture_input_mode(&mut self, mode: firelab_base::MoistureInputMode) {
        self.surface_fuel.set_moisture_input_mode(mode);
        self.crown_fuel.set_moisture_input_mode(mode);
    }

    pub fn set_moisture_scenarios(&mut self, scenarios: behave_surface::moisture::MoistureScenarios) {
        self.surface_fuel.set_moisture_scenarios(scenarios);
    }

    pub fn set_current_moisture_scenario_by_name(&mut self, name: &str) -> bool {
        self.surface_fuel.set_current_moisture_scenario_by_name(name)
    }

    pub fn set_current_moisture_scenario_by_index(&mut self, index: i32) -> bool {
        self.surface_fuel.set_current_moisture_scenario_by_index(index)
    }

    pub fn set_slope(&mut self, slope: f64, units: SlopeUnits) {
        self.surface_fuel.set_slope(slope, units);
    }

    pub fn set_aspect(&mut self, aspect: f64) {
        self.surface_fuel.set_aspect(aspect);
    }

    pub fn set_wind_speed(&mut self, speed: f64, units: SpeedUnits, mode: WindHeightInputMode) {
        self.surface_fuel.set_wind_speed(speed, units, mode);
    }

    pub fn set_wind_direction(&mut self, direction: f64) {
        self.surface_fuel.set_wind_direction(direction);
    }

    pub fn set_wind_height_input_mode(&mut self, mode: WindHeightInputMode) {
        self.surface_fuel.set_wind_height_input_mode(mode);
        self.crown_fuel.set_wind_height_input_mode(mode);
    }

    pub fn set_wind_and_spread_orientation_mode(&mut self, mode: WindAndSpreadOrientationMode) {
        self.surface_fuel.set_wind_and_spread_orientation_mode(mode);
    }

    pub fn set_user_provided_wind_adjustment_factor(&mut self, waf: f64) {
        self.surface_fuel.set_user_provided_wind_adjustment_factor(waf);
    }

    pub fn set_wind_adjustment_factor_calculation_method(
        &mut self,
        method: WindAdjustmentFactorCalculationMethod,
    ) {
        self.surface_fuel
            .set_wind_adjustment_factor_calculation_method(method);
    }

    // -----------------------------------------------------------------------
    // Crown module getters
    // -----------------------------------------------------------------------

    pub fn get_canopy_base_height(&self, units: LengthUnits) -> f64 {
        self.crown_inputs.canopy_base_height(units)
    }

    pub fn get_canopy_bulk_density(&self, units: DensityUnits) -> f64 {
        self.crown_inputs.canopy_bulk_density(units)
    }

    pub fn get_moisture_foliar(&self, units: FractionUnits) -> f64 {
        self.crown_inputs.moisture_foliar(units)
    }

    pub fn get_crown_fire_spread_rate(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.crown_fire_spread_rate)
    }

    pub fn get_crown_fire_spread_distance(
        &self,
        units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        let t = time_units.to_base(elapsed_time);
        units.from_base(self.crown_fire_spread_rate * t)
    }

    pub fn get_surface_fire_spread_rate(&self, units: SpeedUnits) -> f64 {
        self.surface_fuel.spread_rate(units)
    }

    pub fn get_surface_fire_spread_distance(
        &self,
        units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.surface_fuel.spread_distance(units, elapsed_time, time_units)
    }

    pub fn get_crown_fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        units.from_base(self.crown_fireline_intensity)
    }

    pub fn get_crown_flame_length(&self, units: LengthUnits) -> f64 {
        units.from_base(self.crown_flame_length)
    }

    pub fn get_fire_type(&self) -> FireType {
        self.fire_type
    }

    pub fn get_final_spread_rate(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.final_spread_rate)
    }

    pub fn get_final_heat_per_unit_area(&self, units: HeatPerUnitAreaUnits) -> f64 {
        units.from_base(self.final_heat_per_unit_area)
    }

    pub fn get_final_fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        units.from_base(self.final_fireline_intensity)
    }

    pub fn get_final_flame_length(&self, units: LengthUnits) -> f64 {
        units.from_base(self.final_flame_length)
    }

    pub fn get_crown_fire_length_to_width_ratio(&self) -> f64 {
        self.crown_fire_length_to_width_ratio
    }

    pub fn get_crown_fire_area(
        &self,
        units: AreaUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.crown_fire_size
            .fire_area(true, units, elapsed_time, time_units)
    }

    pub fn get_crown_fire_perimeter(
        &self,
        units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.crown_fire_size
            .fire_perimeter(true, units, elapsed_time, time_units)
    }

    pub fn get_critical_open_wind_speed(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.crown_fire_active_wind_speed)
    }

    pub fn get_crown_fraction_burned(&self) -> f64 {
        self.crown_fraction_burned
    }

    // Pass-through surface getters

    pub fn get_fuel_model_number(&self) -> i32 {
        self.surface_fuel.get_fuel_model_number()
    }

    pub fn get_wind_speed(&self, units: SpeedUnits, mode: WindHeightInputMode) -> f64 {
        self.surface_fuel.get_wind_speed(units, mode)
    }

    pub fn get_wind_direction(&self) -> f64 {
        self.surface_fuel.get_wind_direction()
    }

    pub fn get_slope(&self, units: SlopeUnits) -> f64 {
        self.surface_fuel.get_slope(units)
    }

    pub fn get_aspect(&self) -> f64 {
        self.surface_fuel.get_aspect()
    }

    pub fn get_canopy_cover(&self, units: FractionUnits) -> f64 {
        self.surface_fuel.get_canopy_cover(units)
    }

    pub fn get_canopy_height(&self, units: LengthUnits) -> f64 {
        self.surface_fuel.get_canopy_height(units)
    }

    pub fn get_crown_ratio(&self, units: FractionUnits) -> f64 {
        self.surface_fuel.get_crown_ratio(units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_near(a: f64, b: f64, tol: f64) {
        assert!(
            (a - b).abs() < tol,
            "expected {b}, got {a} (diff {})",
            (a - b).abs()
        );
    }

    /// Standard crown test inputs (from setCrownInputsLowMoistureScenario in testBehave.cpp):
    /// FM 124 (GS4), 6/7/8/60/90% moisture, 120% foliar, 5 mph 20-ft wind,
    /// RelativeToNorth, 30% slope, 50% canopy cover, 30 ft canopy height,
    /// 6 ft canopy base height, 0.03 lb/ft³ CBD, 0.50 crown ratio.
    fn make_crown_standard() -> Crown {
        let fm = FuelModels::new();
        let mut c = Crown::new(fm);
        c.update_crown_inputs(
            124,                                          // fuel model GS4
            6.0, 7.0, 8.0, 60.0, 90.0,                  // moistures (%)
            120.0,                                        // foliar moisture (%)
            FractionUnits::Percent,
            5.0,                                          // wind speed
            SpeedUnits::MilesPerHour,
            WindHeightInputMode::TwentyFoot,
            0.0,                                          // wind direction
            WindAndSpreadOrientationMode::RelativeToNorth,
            30.0,                                         // slope (%)
            SlopeUnits::Percent,
            0.0,                                          // aspect
            50.0,                                         // canopy cover (%)
            FractionUnits::Percent,
            30.0,                                         // canopy height (ft)
            6.0,                                          // canopy base height (ft)
            LengthUnits::Feet,
            0.50,                                         // crown ratio
            FractionUnits::Fraction,
            0.03,                                         // canopy bulk density (lb/ft³)
            DensityUnits::PoundsPerCubicFoot,
        );
        c
    }

    // --- Rothermel tests ---

    #[test]
    fn rothermel_spread_rate() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_near(
            c.get_crown_fire_spread_rate(SpeedUnits::ChainsPerHour),
            10.259921,
            0.01,
        );
    }

    #[test]
    fn rothermel_length_to_width_ratio() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_near(c.get_crown_fire_length_to_width_ratio(), 1.625, 0.001);
    }

    #[test]
    fn rothermel_area() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_near(
            c.get_crown_fire_area(AreaUnits::Acres, 1.0, TimeUnits::Hours),
            5.087736,
            0.01,
        );
    }

    #[test]
    fn rothermel_perimeter() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_near(
            c.get_crown_fire_perimeter(LengthUnits::Chains, 1.0, TimeUnits::Hours),
            26.033937,
            0.1,
        );
    }

    #[test]
    fn rothermel_flame_length() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_near(
            c.get_crown_flame_length(LengthUnits::Feet),
            29.320557,
            0.01,
        );
    }

    #[test]
    fn rothermel_fireline_intensity() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_near(
            c.get_crown_fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond),
            1775.061222,
            1.0,
        );
    }

    #[test]
    fn rothermel_fire_type_surface() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.set_moisture_one_hour(20.0, FractionUnits::Percent);
        c.do_crown_run_rothermel();
        assert_eq!(c.get_fire_type(), FireType::Surface);
    }

    #[test]
    fn rothermel_fire_type_torching() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_eq!(c.get_fire_type(), FireType::Torching);
    }

    #[test]
    fn rothermel_fire_type_crowning() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.set_wind_speed(10.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_eq!(c.get_fire_type(), FireType::Crowning);
    }

    #[test]
    fn rothermel_fire_type_conditional() {
        let mut c = make_crown_standard();
        c.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        c.set_canopy_height(60.0, LengthUnits::Feet);
        c.set_canopy_base_height(30.0, LengthUnits::Feet);
        c.set_canopy_bulk_density(0.06, DensityUnits::PoundsPerCubicFoot);
        c.set_wind_speed(5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot);
        c.do_crown_run_rothermel();
        assert_eq!(c.get_fire_type(), FireType::ConditionalCrownFire);
    }

    // --- Scott & Reinhardt tests ---

    #[test]
    fn scott_reinhardt_scenario1_spread_rate() {
        let fm = FuelModels::new();
        let mut c = Crown::new(fm);
        c.set_wind_adjustment_factor_calculation_method(
            WindAdjustmentFactorCalculationMethod::UserInput,
        );
        c.set_user_provided_wind_adjustment_factor(0.4);
        c.update_crown_inputs(
            10,                                            // fuel model
            8.0, 9.0, 10.0, 0.0, 117.0,                   // moistures (%)
            100.0,                                         // foliar (%)
            FractionUnits::Percent,
            2187.226624,                                   // wind speed (ft/min)
            SpeedUnits::FeetPerMinute,
            WindHeightInputMode::TwentyFoot,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            20.0, SlopeUnits::Percent,
            0.0,
            50.0, FractionUnits::Percent,                  // canopy cover
            38.104626,                                     // canopy height (ft)
            2.952756,                                      // canopy base height (ft)
            LengthUnits::Feet,
            0.50, FractionUnits::Fraction,                 // crown ratio
            0.01311,                                       // CBD (lb/ft³)
            DensityUnits::PoundsPerCubicFoot,
        );
        c.do_crown_run_scott_and_reinhardt();

        assert_near(
            c.get_final_spread_rate(SpeedUnits::FeetPerMinute),
            65.221842,
            0.1,
        );
        assert_near(
            c.get_final_flame_length(LengthUnits::Feet),
            60.744542,
            0.5,
        );
        assert_near(
            c.get_final_fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond),
            5293.170672,
            5.0,
        );
        assert_near(
            c.get_critical_open_wind_speed(SpeedUnits::FeetPerMinute),
            1717.916785,
            1.0,
        );
        assert_eq!(c.get_fire_type(), FireType::Crowning);
    }

    #[test]
    fn scott_reinhardt_scenario2_torching() {
        let fm = FuelModels::new();
        let mut c = Crown::new(fm);
        c.set_moisture_input_mode(firelab_base::MoistureInputMode::BySizeClass);
        c.set_wind_adjustment_factor_calculation_method(
            WindAdjustmentFactorCalculationMethod::UserInput,
        );
        c.set_user_provided_wind_adjustment_factor(0.15);
        c.update_crown_inputs(
            5,                                             // fuel model
            5.0, 6.0, 8.0, 0.0, 117.0,                    // moistures (%)
            100.0,                                         // foliar (%)
            FractionUnits::Percent,
            24.854848,                                     // wind speed (mph)
            SpeedUnits::MilesPerHour,
            WindHeightInputMode::TwentyFoot,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            20.0, SlopeUnits::Percent,
            0.0,
            50.0, FractionUnits::Percent,
            71.631562,                                     // canopy height (ft)
            4.92126,                                       // canopy base height (ft)
            LengthUnits::Feet,
            0.50, FractionUnits::Fraction,
            0.003746,                                      // CBD (lb/ft³)
            DensityUnits::PoundsPerCubicFoot,
        );
        c.do_crown_run_scott_and_reinhardt();

        assert_near(
            c.get_final_spread_rate(SpeedUnits::FeetPerMinute),
            29.475388,
            0.1,
        );
        assert_near(
            c.get_final_flame_length(LengthUnits::Feet),
            12.759447,
            0.1,
        );
        assert_near(
            c.get_final_fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond),
            509.568753,
            1.0,
        );
        assert_near(
            c.get_critical_open_wind_speed(SpeedUnits::FeetPerMinute),
            3874.421988,
            5.0,
        );
        assert_eq!(c.get_fire_type(), FireType::Torching);
    }
}
