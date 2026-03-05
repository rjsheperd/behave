//! Surface fire facade — high-level API for surface fire calculations.
//!
//! The `Surface` struct orchestrates `SurfaceInputs`, `SurfaceFire`, `FireSize`,
//! `FuelModels`, `MoistureScenarios`, and `TwoFuelModels` into a single coherent
//! API with unit-converting getters and setters.
//!
//! C++ source: surface.h / surface.cpp

use firelab_base::{
    AreaUnits, DensityUnits, FireSize, FirelineIntensityUnits, FractionUnits,
    FuelLifeState, HeatPerUnitAreaUnits, HeatSinkUnits,
    HeatSourceAndReactionIntensityUnits, LengthUnits, SlopeUnits,
    SpeedUnits, SurfaceAreaToVolumeUnits, TemperatureUnits, TimeUnits, UnitConversion,
    WindAdjustmentFactorCalculationMethod, WindAndSpreadOrientationMode, WindHeightInputMode,
};

use crate::fire::SurfaceFire;
use crate::fuel_models::FuelModels;
use crate::inputs::{SurfaceFireSpreadDirectionMode, SurfaceInputs};
use crate::moisture::MoistureScenarios;
use crate::two_fuel_models::TwoFuelModels;

/// High-level surface fire facade.
///
/// Owns all components needed to run a surface fire calculation and provides
/// the main public API with unit-converting getters and setters.
///
/// C++ class: `Surface`
#[derive(Debug, Clone)]
pub struct Surface {
    fuel_models: FuelModels,
    inputs: SurfaceInputs,
    fire: SurfaceFire,
    size: FireSize,
    moisture_scenarios: MoistureScenarios,
    two_fuel_models: TwoFuelModels,
}

impl Surface {
    pub fn new(fuel_models: FuelModels) -> Self {
        Self {
            fuel_models,
            inputs: SurfaceInputs::new(),
            fire: SurfaceFire::new(),
            size: FireSize::new(),
            moisture_scenarios: MoistureScenarios::new(),
            two_fuel_models: TwoFuelModels::new(),
        }
    }

    /// Run surface fire calculation in direction of maximum spread.
    pub fn do_surface_run_in_direction_of_max_spread(&mut self) {
        self.inputs.update_moistures_based_on_input_mode(
            Some(&self.moisture_scenarios),
        );

        let has_doi = false;
        let doi = 0.0;
        let dir_mode = SurfaceFireSpreadDirectionMode::FromIgnitionPoint;

        if self.inputs.is_using_two_fuel_models() {
            let method = self.inputs.two_fuel_models_method();
            let first_fm = self.inputs.first_fuel_model_number();
            let first_cov = self.inputs.first_fuel_model_coverage();
            let second_fm = self.inputs.second_fuel_model_number();
            self.two_fuel_models.calculate_weighted_spread_rate(
                method,
                first_fm,
                first_cov,
                second_fm,
                has_doi,
                doi,
                dir_mode,
                &mut self.fire,
                &self.fuel_models,
                &self.inputs,
                &mut self.size,
            );
        } else {
            let fm_number = self.inputs.fuel_model_number();
            let is_special = self.inputs.is_using_palmetto_gallberry()
                || self.inputs.is_using_western_aspen()
                || self.inputs.is_using_chaparral();

            if !is_special
                && (self.fuel_models.is_all_fuel_load_zero(fm_number)
                    || !self.fuel_models.is_fuel_model_defined(fm_number))
            {
                // No fuel — zero spread
                self.fire = SurfaceFire::new();
            } else {
                self.fire.calculate_forward_spread_rate(
                    fm_number,
                    has_doi,
                    doi,
                    dir_mode,
                    &self.fuel_models,
                    &self.inputs,
                    &mut self.size,
                );
            }
        }
    }

    /// Run surface fire calculation in a specific direction of interest.
    pub fn do_surface_run_in_direction_of_interest(
        &mut self,
        direction_of_interest: f64,
        direction_mode: SurfaceFireSpreadDirectionMode,
    ) {
        self.inputs.update_moistures_based_on_input_mode(
            Some(&self.moisture_scenarios),
        );

        let has_doi = true;

        if self.inputs.is_using_two_fuel_models() {
            let method = self.inputs.two_fuel_models_method();
            let first_fm = self.inputs.first_fuel_model_number();
            let first_cov = self.inputs.first_fuel_model_coverage();
            let second_fm = self.inputs.second_fuel_model_number();
            self.two_fuel_models.calculate_weighted_spread_rate(
                method,
                first_fm,
                first_cov,
                second_fm,
                has_doi,
                direction_of_interest,
                direction_mode,
                &mut self.fire,
                &self.fuel_models,
                &self.inputs,
                &mut self.size,
            );
        } else {
            let fm_number = self.inputs.fuel_model_number();
            if self.fuel_models.is_all_fuel_load_zero(fm_number)
                || !self.fuel_models.is_fuel_model_defined(fm_number)
            {
                self.fire = SurfaceFire::new();
            } else {
                self.fire.calculate_forward_spread_rate(
                    fm_number,
                    has_doi,
                    direction_of_interest,
                    direction_mode,
                    &self.fuel_models,
                    &self.inputs,
                    &mut self.size,
                );
            }
        }
    }

    /// Calculate flame length from fireline intensity (static utility).
    pub fn calculate_flame_length(
        fireline_intensity: f64,
        fli_units: FirelineIntensityUnits,
        fl_units: LengthUnits,
    ) -> f64 {
        let fli_base = fli_units.to_base(fireline_intensity);
        let fl = if fli_base < 1e-07 {
            0.0
        } else {
            0.45 * fli_base.powf(0.46)
        };
        fl_units.from_base(fl)
    }

    // --- SurfaceFire output getters (with unit conversion) ---

    pub fn spread_rate(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.fire.spread_rate())
    }

    pub fn spread_rate_in_direction_of_interest(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.fire.spread_rate_in_direction_of_interest())
    }

    pub fn backing_spread_rate(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.size.backing_spread_rate(SpeedUnits::FeetPerMinute))
    }

    pub fn flanking_spread_rate(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.size.flanking_spread_rate(SpeedUnits::FeetPerMinute))
    }

    pub fn spread_distance(
        &self,
        length_units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        let t = time_units.to_base(elapsed_time);
        length_units.from_base(self.fire.spread_rate() * t)
    }

    pub fn direction_of_max_spread(&self) -> f64 {
        self.fire.direction_of_max_spread()
    }

    pub fn flame_length_output(&self, units: LengthUnits) -> f64 {
        units.from_base(self.fire.flame_length())
    }

    pub fn backing_flame_length(&self, units: LengthUnits) -> f64 {
        units.from_base(self.fire.backing_flame_length())
    }

    pub fn flanking_flame_length(&self, units: LengthUnits) -> f64 {
        units.from_base(self.fire.flanking_flame_length())
    }

    pub fn flame_length_in_direction_of_interest(&self, units: LengthUnits) -> f64 {
        units.from_base(self.fire.flame_length_in_direction_of_interest())
    }

    pub fn fire_length_to_width_ratio(&self) -> f64 {
        self.fire.fire_length_to_width_ratio()
    }

    pub fn fire_eccentricity(&self) -> f64 {
        self.size.eccentricity()
    }

    pub fn heading_to_backing_ratio(&self) -> f64 {
        self.size.heading_to_backing_ratio()
    }

    pub fn fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        units.from_base(self.fire.fireline_intensity())
    }

    pub fn backing_fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        units.from_base(self.fire.backing_fireline_intensity())
    }

    pub fn flanking_fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        units.from_base(self.fire.flanking_fireline_intensity())
    }

    pub fn heat_per_unit_area(&self, units: HeatPerUnitAreaUnits) -> f64 {
        units.from_base(self.fire.heat_per_unit_area())
    }

    pub fn midflame_wind_speed(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.fire.midflame_wind_speed())
    }

    pub fn residence_time(&self, units: TimeUnits) -> f64 {
        units.from_base(self.fire.residence_time())
    }

    pub fn reaction_intensity(&self, units: HeatSourceAndReactionIntensityUnits) -> f64 {
        units.from_base(self.fire.reaction_intensity_value())
    }

    pub fn reaction_intensity_for_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.fire.reaction_intensity_for_life_state(life_state)
    }

    pub fn elliptical_a(
        &self,
        length_units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.size.elliptical_a(length_units, elapsed_time, time_units)
    }

    pub fn elliptical_b(
        &self,
        length_units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.size.elliptical_b(length_units, elapsed_time, time_units)
    }

    pub fn elliptical_c(
        &self,
        length_units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.size.elliptical_c(length_units, elapsed_time, time_units)
    }

    pub fn fire_length(
        &self,
        length_units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.size.fire_length(length_units, elapsed_time, time_units)
    }

    pub fn max_fire_width(
        &self,
        length_units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.size.max_fire_width(length_units, elapsed_time, time_units)
    }

    pub fn slope_factor(&self) -> f64 {
        self.fire.slope_factor()
    }

    pub fn bulk_density(&self, units: DensityUnits) -> f64 {
        units.from_base(self.fire.bulk_density())
    }

    pub fn heat_sink(&self, units: HeatSinkUnits) -> f64 {
        units.from_base(self.fire.heat_sink())
    }

    pub fn heat_source(&self, units: HeatSourceAndReactionIntensityUnits) -> f64 {
        units.from_base(self.fire.heat_source())
    }

    pub fn fire_perimeter(
        &self,
        length_units: LengthUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.size
            .fire_perimeter(false, length_units, elapsed_time, time_units)
    }

    pub fn fire_area(
        &self,
        area_units: AreaUnits,
        elapsed_time: f64,
        time_units: TimeUnits,
    ) -> f64 {
        self.size
            .fire_area(false, area_units, elapsed_time, time_units)
    }

    pub fn characteristic_moisture_by_life_state(
        &self,
        life_state: FuelLifeState,
        units: FractionUnits,
    ) -> f64 {
        units.from_base(self.fire.weighted_moisture_by_life_state(life_state))
    }

    pub fn live_fuel_moisture_of_extinction(&self, units: FractionUnits) -> f64 {
        units.from_base(
            self.fire
                .moisture_of_extinction_by_life_state(FuelLifeState::Live),
        )
    }

    pub fn characteristic_savr(&self, units: SurfaceAreaToVolumeUnits) -> f64 {
        units.from_base(self.fire.characteristic_savr())
    }

    pub fn relative_packing_ratio(&self) -> f64 {
        self.fire.relative_packing_ratio()
    }

    pub fn packing_ratio(&self) -> f64 {
        self.fire.packing_ratio()
    }

    // --- SurfaceInputs pass-through setters ---

    pub fn inputs(&self) -> &SurfaceInputs {
        &self.inputs
    }

    pub fn inputs_mut(&mut self) -> &mut SurfaceInputs {
        &mut self.inputs
    }

    pub fn fuel_models(&self) -> &FuelModels {
        &self.fuel_models
    }

    pub fn fuel_models_mut(&mut self) -> &mut FuelModels {
        &mut self.fuel_models
    }

    pub fn moisture_scenarios(&self) -> &MoistureScenarios {
        &self.moisture_scenarios
    }

    pub fn moisture_scenarios_mut(&mut self) -> &mut MoistureScenarios {
        &mut self.moisture_scenarios
    }

    pub fn fire(&self) -> &SurfaceFire {
        &self.fire
    }

    pub fn size(&self) -> &FireSize {
        &self.size
    }

    /// Bulk input setter matching C++ `updateSurfaceInputs`.
    pub fn update_surface_inputs(
        &mut self,
        fuel_model_number: i32,
        moisture_one_hour: f64,
        moisture_ten_hour: f64,
        moisture_hundred_hour: f64,
        moisture_live_herbaceous: f64,
        moisture_live_woody: f64,
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
        canopy_height_units: LengthUnits,
        crown_ratio: f64,
        crown_ratio_units: FractionUnits,
    ) {
        self.inputs.update_surface_inputs(
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
    }

    // --- Individual setters (pass-through to SurfaceInputs) ---

    pub fn set_fuel_model_number(&mut self, number: i32) {
        self.inputs.set_fuel_model_number(number);
    }

    pub fn set_wind_speed(
        &mut self,
        speed: f64,
        units: SpeedUnits,
        mode: WindHeightInputMode,
    ) {
        self.inputs.set_wind_speed(speed, units, mode);
    }

    pub fn set_wind_direction(&mut self, direction: f64) {
        self.inputs.set_wind_direction(direction);
    }

    pub fn set_wind_height_input_mode(&mut self, mode: WindHeightInputMode) {
        self.inputs.set_wind_height_input_mode(mode);
    }

    pub fn set_wind_and_spread_orientation_mode(&mut self, mode: WindAndSpreadOrientationMode) {
        self.inputs.set_wind_and_spread_orientation_mode(mode);
    }

    pub fn set_wind_adjustment_factor_calculation_method(
        &mut self,
        method: WindAdjustmentFactorCalculationMethod,
    ) {
        self.inputs
            .set_wind_adjustment_factor_calculation_method(method);
    }

    pub fn set_user_provided_wind_adjustment_factor(&mut self, waf: f64) {
        self.inputs.set_user_provided_wind_adjustment_factor(waf);
    }

    pub fn set_slope(&mut self, slope: f64, units: SlopeUnits) {
        self.inputs.set_slope(slope, units);
    }

    pub fn set_aspect(&mut self, aspect: f64) {
        self.inputs.set_aspect(aspect);
    }

    pub fn set_canopy_cover(&mut self, cover: f64, units: FractionUnits) {
        self.inputs.set_canopy_cover(cover, units);
    }

    pub fn set_canopy_height(&mut self, height: f64, units: LengthUnits) {
        self.inputs.set_canopy_height(height, units);
    }

    pub fn set_crown_ratio(&mut self, ratio: f64, units: FractionUnits) {
        self.inputs.set_crown_ratio(ratio, units);
    }

    pub fn set_moisture_one_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.inputs.set_moisture_one_hour(moisture, units);
    }

    pub fn set_moisture_ten_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.inputs.set_moisture_ten_hour(moisture, units);
    }

    pub fn set_moisture_hundred_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.inputs.set_moisture_hundred_hour(moisture, units);
    }

    pub fn set_moisture_live_herbaceous(&mut self, moisture: f64, units: FractionUnits) {
        self.inputs.set_moisture_live_herbaceous(moisture, units);
    }

    pub fn set_moisture_live_woody(&mut self, moisture: f64, units: FractionUnits) {
        self.inputs.set_moisture_live_woody(moisture, units);
    }

    pub fn set_moisture_dead_aggregate(&mut self, moisture: f64, units: FractionUnits) {
        self.inputs.set_moisture_dead_aggregate(moisture, units);
    }

    pub fn set_moisture_live_aggregate(&mut self, moisture: f64, units: FractionUnits) {
        self.inputs.set_moisture_live_aggregate(moisture, units);
    }

    pub fn set_moisture_input_mode(&mut self, mode: firelab_base::MoistureInputMode) {
        self.inputs.set_moisture_input_mode(mode);
    }

    pub fn set_moisture_scenarios(&mut self, scenarios: MoistureScenarios) {
        self.moisture_scenarios = scenarios;
    }

    pub fn set_current_moisture_scenario_by_name(&mut self, name: &str) -> bool {
        self.inputs.set_current_moisture_scenario_by_name(name, &self.moisture_scenarios)
    }

    pub fn set_current_moisture_scenario_by_index(&mut self, index: i32) -> bool {
        self.inputs.set_current_moisture_scenario_by_index(index, &self.moisture_scenarios)
    }

    pub fn set_air_temperature(&mut self, temperature: f64, units: TemperatureUnits) {
        self.inputs.set_air_temperature(temperature, units);
    }

    // --- Individual getters (pass-through to SurfaceInputs) ---

    pub fn get_fuel_model_number(&self) -> i32 {
        self.inputs.fuel_model_number()
    }

    pub fn get_wind_speed(&self, units: SpeedUnits, _mode: WindHeightInputMode) -> f64 {
        // C++ stores wind speed and converts based on mode. Our Rust SurfaceInputs
        // stores in base units (ft/min); the mode is used during set, not get.
        self.inputs.wind_speed(units)
    }

    pub fn get_wind_direction(&self) -> f64 {
        self.inputs.wind_direction()
    }

    pub fn get_wind_height_input_mode(&self) -> WindHeightInputMode {
        self.inputs.wind_height_input_mode()
    }

    pub fn get_wind_adjustment_factor_calculation_method(&self) -> WindAdjustmentFactorCalculationMethod {
        self.inputs.wind_adjustment_factor_calculation_method()
    }

    pub fn get_slope(&self, units: SlopeUnits) -> f64 {
        self.inputs.slope(units)
    }

    pub fn get_aspect(&self) -> f64 {
        self.inputs.aspect()
    }

    pub fn get_canopy_cover(&self, units: FractionUnits) -> f64 {
        self.inputs.canopy_cover(units)
    }

    pub fn get_canopy_height(&self, units: LengthUnits) -> f64 {
        self.inputs.canopy_height(units)
    }

    pub fn get_crown_ratio(&self, units: FractionUnits) -> f64 {
        self.inputs.crown_ratio(units)
    }

    pub fn get_moisture_one_hour(&self, units: FractionUnits) -> f64 {
        self.inputs.moisture_one_hour(units)
    }

    pub fn get_moisture_ten_hour(&self, units: FractionUnits) -> f64 {
        self.inputs.moisture_ten_hour(units)
    }

    pub fn get_moisture_hundred_hour(&self, units: FractionUnits) -> f64 {
        self.inputs.moisture_hundred_hour(units)
    }

    pub fn get_moisture_live_herbaceous(&self, units: FractionUnits) -> f64 {
        self.inputs.moisture_live_herbaceous(units)
    }

    pub fn get_moisture_live_woody(&self, units: FractionUnits) -> f64 {
        self.inputs.moisture_live_woody(units)
    }

    pub fn get_number_of_moisture_scenarios(&self) -> usize {
        self.moisture_scenarios.num_scenarios()
    }

    // --- Fuel model getters ---

    pub fn is_all_fuel_load_zero(&self, fuel_model_number: i32) -> bool {
        self.fuel_models.is_all_fuel_load_zero(fuel_model_number)
    }

    pub fn is_fuel_model_defined(&self, fuel_model_number: i32) -> bool {
        self.fuel_models.is_fuel_model_defined(fuel_model_number)
    }

    pub fn fuelbed_depth(&self, fuel_model_number: i32, units: LengthUnits) -> f64 {
        self.fuel_models.fuelbed_depth(fuel_model_number, units)
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

    /// Standard test scenario: GS4 (124), upslope mode, 5 mph 20-ft wind,
    /// 30% slope, 50% canopy cover, 30 ft canopy height, 50% crown ratio.
    /// Matches C++ setSurfaceInputsForGS4LowMoistureScenario.
    fn make_gs4_surface() -> Surface {
        let fm = FuelModels::new();
        let mut s = Surface::new(fm);
        s.update_surface_inputs(
            124, // GS4
            0.06, 0.07, 0.08, 0.60, 0.90,
            FractionUnits::Fraction,
            5.0, SpeedUnits::MilesPerHour,
            WindHeightInputMode::TwentyFoot,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            30.0, SlopeUnits::Percent,
            0.0,
            0.50, FractionUnits::Fraction,
            30.0, LengthUnits::Feet,
            0.50, FractionUnits::Fraction,
        );
        s
    }

    #[test]
    fn gs4_upslope_spread_rate() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        let ros = s.spread_rate(SpeedUnits::ChainsPerHour);
        // Expected: 8.876216 ch/hr (from RUST_PORT.org testSurfaceSingleFuelModel)
        assert_near(ros, 8.876216, 0.01);
    }

    #[test]
    fn gs4_fireline_intensity() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        let fli = s.fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond);
        // Should be positive
        assert!(fli > 0.0, "FLI={}", fli);
    }

    #[test]
    fn gs4_flame_length_positive() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        let fl = s.flame_length_output(LengthUnits::Feet);
        assert!(fl > 0.0, "flame_length={}", fl);
    }

    #[test]
    fn gs4_backing_and_flanking_rates() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        let backing = s.backing_spread_rate(SpeedUnits::ChainsPerHour);
        let flanking = s.flanking_spread_rate(SpeedUnits::ChainsPerHour);
        let forward = s.spread_rate(SpeedUnits::ChainsPerHour);
        // Expected from RUST_PORT.org:
        // backing=2.916147, flanking=5.087666
        assert_near(backing, 2.916147, 0.05);
        assert_near(flanking, 5.087666, 0.05);
        assert!(forward > backing);
        assert!(forward > flanking);
    }

    #[test]
    fn gs4_spread_rate_in_multiple_units() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        // Same 8.876216 ch/hr rate in different units (from RUST_PORT.org)
        assert_near(s.spread_rate(SpeedUnits::ChainsPerHour), 8.876216, 0.01);
        assert_near(s.spread_rate(SpeedUnits::FeetPerMinute), 9.763838, 0.01);
    }

    #[test]
    fn nonburnable_model_91_zero_spread() {
        let fm = FuelModels::new();
        let mut s = Surface::new(fm);
        s.update_surface_inputs(
            91, // NB1
            0.06, 0.07, 0.08, 0.60, 0.90,
            FractionUnits::Fraction,
            5.0, SpeedUnits::MilesPerHour,
            WindHeightInputMode::TwentyFoot,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            0.0, SlopeUnits::Degrees,
            0.0,
            0.0, FractionUnits::Fraction,
            0.0, LengthUnits::Feet,
            0.0, FractionUnits::Fraction,
        );
        s.do_surface_run_in_direction_of_max_spread();
        assert_near(s.spread_rate(SpeedUnits::ChainsPerHour), 0.0, 1e-10);
    }

    #[test]
    fn characteristic_savr_gs4() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        // Expected: 1631.128734 ft²/ft³
        let savr = s.characteristic_savr(SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet);
        assert_near(savr, 1631.128734, 1.0);
    }

    #[test]
    fn live_moisture_of_extinction_gs4() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        // Expected: 137.968551 %
        let moe = s.live_fuel_moisture_of_extinction(FractionUnits::Percent);
        assert_near(moe, 137.968551, 0.5);
    }

    #[test]
    fn characteristic_dead_moisture_gs4() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        // Expected: 6.005463 %
        let cdm = s.characteristic_moisture_by_life_state(
            FuelLifeState::Dead,
            FractionUnits::Percent,
        );
        assert_near(cdm, 6.005463, 0.1);
    }

    #[test]
    fn characteristic_live_moisture_gs4() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        // Expected: 85.874007 %
        let clm = s.characteristic_moisture_by_life_state(
            FuelLifeState::Live,
            FractionUnits::Percent,
        );
        assert_near(clm, 85.874007, 0.5);
    }

    #[test]
    fn heat_source_gs4_no_slope() {
        // C++ test: RelativeToNorth, wind dir 0, aspect 0, slope 0,
        // but still 5 mph 20-ft wind, 50% canopy, 30 ft height, 0.50 crown ratio
        let fm = FuelModels::new();
        let mut s = Surface::new(fm);
        s.update_surface_inputs(
            124,
            0.06, 0.07, 0.08, 0.60, 0.90,
            FractionUnits::Fraction,
            5.0, SpeedUnits::MilesPerHour,
            WindHeightInputMode::TwentyFoot,
            0.0,
            WindAndSpreadOrientationMode::RelativeToNorth,
            0.0, SlopeUnits::Degrees,
            0.0,
            0.50, FractionUnits::Fraction,
            30.0, LengthUnits::Feet,
            0.50, FractionUnits::Fraction,
        );
        s.do_surface_run_in_direction_of_max_spread();
        // Expected: 1164.267376 Btu/ft²/min
        let hs = s.heat_source(
            HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerMinute,
        );
        assert_near(hs, 1164.267376, 1.0);
    }

    #[test]
    fn residence_time_gs4() {
        let mut s = make_gs4_surface();
        s.do_surface_run_in_direction_of_max_spread();
        let rt = s.residence_time(TimeUnits::Minutes);
        assert!(rt > 0.0 && rt < 1.0, "residence_time={}", rt);
    }
}
