//! Surface fire input parameters.
//!
//! C++ source: surfaceInputs.h / surfaceInputs.cpp / surfaceInputEnums.h

use firelab_base::{
    BasalAreaUnits, FractionUnits, LengthUnits, LoadingUnits, MoistureClassInput,
    MoistureInputMode, SlopeUnits, SpeedUnits, TemperatureUnits, TimeUnits, UnitConversion,
    WindAdjustmentFactorCalculationMethod, WindAndSpreadOrientationMode, WindHeightInputMode,
};

use crate::chaparral::{ChaparralFuelLoadInputMode, ChaparralFuelType};
use crate::moisture::MoistureScenarios;
use crate::two_fuel_models::TwoFuelModelsMethod;
use crate::western_aspen::AspenFireSeverity;

/// Surface-only enum: direction convention for fire spread output.
///
/// C++ source: `SurfaceFireSpreadDirectionMode` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFireSpreadDirectionMode {
    FromIgnitionPoint = 0,
    FromPerimeter = 1,
}

/// Number of moisture size classes (7: 1h, 10h, 100h, live herb, live woody, dead agg, live agg).
const NUM_MOISTURE_CLASSES: usize = 7;

/// All user-facing inputs for a surface fire calculation.
///
/// C++ class: `SurfaceInputs`
#[derive(Debug, Clone)]
pub struct SurfaceInputs {
    // Core
    fuel_model_number: i32,
    air_temperature: f64,

    // Weather/Terrain (all stored in base units)
    wind_speed: f64,
    wind_direction: f64,
    slope: f64,
    aspect: f64,

    // Moisture (base units = fraction 0-1)
    moisture_input_mode: MoistureInputMode,
    moisture_one_hour: f64,
    moisture_ten_hour: f64,
    moisture_hundred_hour: f64,
    moisture_live_herbaceous: f64,
    moisture_live_woody: f64,
    moisture_dead_aggregate: f64,
    moisture_live_aggregate: f64,
    current_moisture_scenario_name: String,
    current_moisture_scenario_index: i32,
    moisture_values_by_size_class: [f64; NUM_MOISTURE_CLASSES],

    // Two fuel models
    is_using_two_fuel_models: bool,
    second_fuel_model_number: i32,
    first_fuel_model_coverage: f64,

    // Palmetto-Gallberry
    is_using_palmetto_gallberry: bool,
    age_of_rough: f64,
    height_of_understory: f64,
    palmetto_coverage: f64,
    overstory_basal_area: f64,

    // Western Aspen
    is_using_western_aspen: bool,
    aspen_fuel_model_number: i32,
    aspen_curing_level: f64,
    dbh: f64,
    aspen_fire_severity: AspenFireSeverity,

    // Chaparral
    is_using_chaparral: bool,
    chaparral_fuel_load_input_mode: ChaparralFuelLoadInputMode,
    chaparral_fuel_type: ChaparralFuelType,
    chaparral_fuel_bed_depth: f64,
    chaparral_fuel_dead_load_fraction: f64,
    chaparral_total_fuel_load: f64,

    // Size module
    elapsed_time: f64,

    // Scorch height
    is_calculating_scorch_height: bool,

    // Wind adjustment factor parameters
    canopy_cover: f64,
    canopy_height: f64,
    crown_ratio: f64,
    user_provided_wind_adjustment_factor: f64,

    // Input modes
    two_fuel_models_method: TwoFuelModelsMethod,
    wind_height_input_mode: WindHeightInputMode,
    wind_and_spread_orientation_mode: WindAndSpreadOrientationMode,
    wind_adjustment_factor_calculation_method: WindAdjustmentFactorCalculationMethod,
    surface_fire_spread_direction_mode: SurfaceFireSpreadDirectionMode,
}

impl SurfaceInputs {
    pub fn new() -> Self {
        let mut s = Self {
            fuel_model_number: 0,
            air_temperature: -500.0, // sentinel: below absolute zero, indicates not set
            wind_speed: 0.0,
            wind_direction: 0.0,
            slope: 0.0,
            aspect: 0.0,

            moisture_input_mode: MoistureInputMode::BySizeClass,
            moisture_one_hour: 0.0,
            moisture_ten_hour: 0.0,
            moisture_hundred_hour: 0.0,
            moisture_live_herbaceous: 0.0,
            moisture_live_woody: 0.0,
            moisture_dead_aggregate: -1.0,
            moisture_live_aggregate: -1.0,
            current_moisture_scenario_name: String::new(),
            current_moisture_scenario_index: -1,
            moisture_values_by_size_class: [-1.0; NUM_MOISTURE_CLASSES],

            is_using_two_fuel_models: false,
            second_fuel_model_number: 0,
            first_fuel_model_coverage: 0.0,

            is_using_palmetto_gallberry: false,
            age_of_rough: 0.0,
            height_of_understory: 0.0,
            palmetto_coverage: 0.0,
            overstory_basal_area: 0.0,

            is_using_western_aspen: false,
            aspen_fuel_model_number: -1,
            aspen_curing_level: 0.0,
            dbh: 0.0,
            aspen_fire_severity: AspenFireSeverity::Low,

            is_using_chaparral: false,
            chaparral_fuel_load_input_mode: ChaparralFuelLoadInputMode::DirectFuelLoad,
            chaparral_fuel_type: ChaparralFuelType::NotSet,
            chaparral_fuel_bed_depth: 0.0,
            chaparral_fuel_dead_load_fraction: 0.0,
            chaparral_total_fuel_load: 0.0,

            elapsed_time: TimeUnits::Hours.to_base(1.0), // 1 hour in base units (minutes)

            is_calculating_scorch_height: false,

            canopy_cover: 0.0,
            canopy_height: 0.0,
            crown_ratio: 0.0,
            user_provided_wind_adjustment_factor: -1.0,

            two_fuel_models_method: TwoFuelModelsMethod::NoMethod,
            wind_height_input_mode: WindHeightInputMode::DirectMidflame,
            wind_and_spread_orientation_mode: WindAndSpreadOrientationMode::RelativeToUpslope,
            wind_adjustment_factor_calculation_method:
                WindAdjustmentFactorCalculationMethod::UseCrownRatio,
            surface_fire_spread_direction_mode: SurfaceFireSpreadDirectionMode::FromIgnitionPoint,
        };
        s.update_moistures_based_on_input_mode(None);
        s
    }

    // -----------------------------------------------------------------------
    // Main surface inputs bulk setter
    // -----------------------------------------------------------------------

    /// Set all primary surface fire inputs at once.
    ///
    /// C++ method: `updateSurfaceInputs`
    #[allow(clippy::too_many_arguments)]
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
        canopy_cover_units: FractionUnits,
        canopy_height: f64,
        canopy_height_units: LengthUnits,
        crown_ratio: f64,
        crown_ratio_units: FractionUnits,
    ) {
        self.set_slope(slope, slope_units);
        self.aspect = aspect;

        self.set_fuel_model_number(fuel_model_number);

        self.set_moisture_one_hour(moisture_one_hour, moisture_units);
        self.set_moisture_ten_hour(moisture_ten_hour, moisture_units);
        self.set_moisture_hundred_hour(moisture_hundred_hour, moisture_units);
        self.set_moisture_live_herbaceous(moisture_live_herbaceous, moisture_units);
        self.set_moisture_live_woody(moisture_live_woody, moisture_units);

        self.set_wind_speed(wind_speed, wind_speed_units, wind_height_input_mode);
        self.set_wind_height_input_mode(wind_height_input_mode);

        // Normalize wind direction to [0, 360)
        let mut wd = wind_direction;
        if wd < 0.0 {
            wd += 360.0;
        }
        while wd >= 360.0 {
            wd -= 360.0;
        }

        self.set_wind_direction(wd);
        self.set_wind_and_spread_orientation_mode(wind_and_spread_orientation_mode);
        self.is_using_two_fuel_models = false;
        self.set_two_fuel_models_method(TwoFuelModelsMethod::NoMethod);

        self.set_canopy_cover(canopy_cover, canopy_cover_units);
        self.set_canopy_height(canopy_height, canopy_height_units);
        self.set_crown_ratio(crown_ratio, crown_ratio_units);
    }

    /// Set all inputs for a two-fuel-models calculation.
    ///
    /// C++ method: `updateSurfaceInputsForTwoFuelModels`
    #[allow(clippy::too_many_arguments)]
    pub fn update_surface_inputs_for_two_fuel_models(
        &mut self,
        first_fuel_model_number: i32,
        second_fuel_model_number: i32,
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
        first_fuel_model_coverage: f64,
        first_fuel_model_coverage_units: FractionUnits,
        two_fuel_models_method: TwoFuelModelsMethod,
        slope: f64,
        slope_units: SlopeUnits,
        aspect: f64,
        canopy_cover: f64,
        canopy_cover_units: FractionUnits,
        canopy_height: f64,
        canopy_height_units: LengthUnits,
        crown_ratio: f64,
        crown_ratio_units: FractionUnits,
    ) {
        self.update_surface_inputs(
            first_fuel_model_number,
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
            canopy_cover_units,
            canopy_height,
            canopy_height_units,
            crown_ratio,
            crown_ratio_units,
        );
        self.set_second_fuel_model_number(second_fuel_model_number);
        self.set_two_fuel_models_first_fuel_model_coverage(
            first_fuel_model_coverage,
            first_fuel_model_coverage_units,
        );
        self.is_using_two_fuel_models = true;
        self.set_two_fuel_models_method(two_fuel_models_method);
    }

    /// Set all inputs for a Palmetto-Gallberry calculation.
    ///
    /// C++ method: `updateSurfaceInputsForPalmettoGallbery` (note: C++ typo preserved)
    #[allow(clippy::too_many_arguments)]
    pub fn update_surface_inputs_for_palmetto_gallberry(
        &mut self,
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
        age_of_rough: f64,
        height_of_understory: f64,
        palmetto_coverage: f64,
        overstory_basal_area: f64,
        basal_area_units: BasalAreaUnits,
        slope: f64,
        slope_units: SlopeUnits,
        aspect: f64,
        canopy_cover: f64,
        canopy_cover_units: FractionUnits,
        canopy_height: f64,
        canopy_height_units: LengthUnits,
        crown_ratio: f64,
        crown_ratio_units: FractionUnits,
    ) {
        self.update_surface_inputs(
            0,
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
            canopy_cover_units,
            canopy_height,
            canopy_height_units,
            crown_ratio,
            crown_ratio_units,
        );

        self.set_palmetto_gallberry_age_of_rough(age_of_rough);
        self.set_palmetto_gallberry_height_of_understory(height_of_understory, canopy_height_units);
        self.set_palmetto_gallberry_palmetto_coverage(palmetto_coverage, canopy_cover_units);
        self.set_palmetto_gallberry_overstory_basal_area(overstory_basal_area, basal_area_units);
    }

    /// Set all inputs for a Western Aspen calculation.
    ///
    /// C++ method: `updateSurfaceInputsForWesternAspen`
    #[allow(clippy::too_many_arguments)]
    pub fn update_surface_inputs_for_western_aspen(
        &mut self,
        aspen_fuel_model_number: i32,
        aspen_curing_level: f64,
        curing_level_units: FractionUnits,
        aspen_fire_severity: AspenFireSeverity,
        dbh: f64,
        dbh_units: LengthUnits,
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
        canopy_cover_units: FractionUnits,
        canopy_height: f64,
        canopy_height_units: LengthUnits,
        crown_ratio: f64,
        crown_ratio_units: FractionUnits,
    ) {
        self.update_surface_inputs(
            0,
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
            canopy_cover_units,
            canopy_height,
            canopy_height_units,
            crown_ratio,
            crown_ratio_units,
        );

        self.set_aspen_fuel_model_number(aspen_fuel_model_number);
        self.set_aspen_curing_level(aspen_curing_level, curing_level_units);
        self.set_aspen_fire_severity(aspen_fire_severity);
        self.set_aspen_dbh(dbh, dbh_units);
    }

    // -----------------------------------------------------------------------
    // Individual setters
    // -----------------------------------------------------------------------

    pub fn set_fuel_model_number(&mut self, fuel_model_number: i32) {
        self.fuel_model_number = fuel_model_number;
    }

    pub fn set_moisture_one_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_one_hour = units.to_base(moisture);
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_moisture_ten_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_ten_hour = units.to_base(moisture);
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_moisture_hundred_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_hundred_hour = units.to_base(moisture);
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_moisture_live_herbaceous(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_live_herbaceous = units.to_base(moisture);
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_moisture_live_woody(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_live_woody = units.to_base(moisture);
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_moisture_dead_aggregate(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_dead_aggregate = units.to_base(moisture);
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_moisture_live_aggregate(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_live_aggregate = units.to_base(moisture);
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_moisture_scenarios_and_update(
        &mut self,
        scenarios: &MoistureScenarios,
    ) {
        // In C++ this stores a raw pointer. In Rust we don't store the reference;
        // instead the caller passes scenarios when needed for moisture updates.
        // This method exists for API parity but is a no-op for storage.
        // The scenarios reference is passed to update_moistures_based_on_input_mode.
        let _ = scenarios;
    }

    pub fn set_current_moisture_scenario_by_name(
        &mut self,
        name: &str,
        scenarios: &MoistureScenarios,
    ) -> bool {
        let is_defined = scenarios.is_defined_by_name(name);
        self.current_moisture_scenario_name = String::new();
        if is_defined {
            self.current_moisture_scenario_name = name.to_string();
            self.current_moisture_scenario_index = scenarios.index_by_name(name);
            self.update_moistures_based_on_input_mode(Some(scenarios));
        }
        is_defined
    }

    pub fn set_current_moisture_scenario_by_index(
        &mut self,
        index: i32,
        scenarios: &MoistureScenarios,
    ) -> bool {
        let is_defined = scenarios.is_defined_by_index(index);
        self.current_moisture_scenario_index = -1;
        if is_defined {
            self.current_moisture_scenario_index = index;
            self.current_moisture_scenario_name = scenarios.name_by_index(index);
            self.update_moistures_based_on_input_mode(Some(scenarios));
        }
        is_defined
    }

    pub fn set_moisture_input_mode(&mut self, mode: MoistureInputMode) {
        self.moisture_input_mode = mode;
        self.update_moistures_based_on_input_mode(None);
    }

    pub fn set_slope(&mut self, slope: f64, units: SlopeUnits) {
        self.slope = units.to_base(slope);
    }

    pub fn set_aspect(&mut self, aspect: f64) {
        self.aspect = aspect;
    }

    pub fn set_wind_speed(
        &mut self,
        wind_speed: f64,
        units: SpeedUnits,
        wind_height_input_mode: WindHeightInputMode,
    ) {
        self.wind_height_input_mode = wind_height_input_mode;
        self.wind_speed = units.to_base(wind_speed);
    }

    pub fn set_wind_direction(&mut self, wind_direction: f64) {
        self.wind_direction = wind_direction;
    }

    pub fn set_wind_and_spread_orientation_mode(&mut self, mode: WindAndSpreadOrientationMode) {
        self.wind_and_spread_orientation_mode = mode;
    }

    pub fn set_wind_height_input_mode(&mut self, mode: WindHeightInputMode) {
        self.wind_height_input_mode = mode;
    }

    pub fn set_canopy_cover(&mut self, cover: f64, units: FractionUnits) {
        self.canopy_cover = units.to_base(cover);
    }

    pub fn set_canopy_height(&mut self, height: f64, units: LengthUnits) {
        self.canopy_height = units.to_base(height);
    }

    pub fn set_crown_ratio(&mut self, ratio: f64, units: FractionUnits) {
        self.crown_ratio = units.to_base(ratio);
    }

    pub fn set_user_provided_wind_adjustment_factor(&mut self, waf: f64) {
        self.user_provided_wind_adjustment_factor = waf;
    }

    pub fn set_wind_adjustment_factor_calculation_method(
        &mut self,
        method: WindAdjustmentFactorCalculationMethod,
    ) {
        self.wind_adjustment_factor_calculation_method = method;
    }

    pub fn set_elapsed_time(&mut self, elapsed_time: f64, units: TimeUnits) {
        self.elapsed_time = units.to_base(elapsed_time);
    }

    pub fn set_air_temperature(&mut self, temperature: f64, units: TemperatureUnits) {
        self.air_temperature = units.to_base(temperature);
    }

    pub fn set_is_calculating_scorch_height(&mut self, is_calculating: bool) {
        self.is_calculating_scorch_height = is_calculating;
    }

    // Two fuel models setters

    pub fn set_first_fuel_model_number(&mut self, number: i32) {
        self.fuel_model_number = number;
    }

    pub fn set_second_fuel_model_number(&mut self, number: i32) {
        self.second_fuel_model_number = number;
    }

    pub fn set_two_fuel_models_method(&mut self, method: TwoFuelModelsMethod) {
        self.two_fuel_models_method = method;
    }

    pub fn set_two_fuel_models_first_fuel_model_coverage(
        &mut self,
        coverage: f64,
        units: FractionUnits,
    ) {
        self.first_fuel_model_coverage = units.to_base(coverage);
    }

    // Palmetto-Gallberry setters

    pub fn set_palmetto_gallberry_age_of_rough(&mut self, age: f64) {
        self.age_of_rough = age;
    }

    pub fn set_palmetto_gallberry_height_of_understory(&mut self, height: f64, units: LengthUnits) {
        self.height_of_understory = units.to_base(height);
    }

    pub fn set_palmetto_gallberry_palmetto_coverage(&mut self, coverage: f64, units: FractionUnits) {
        self.palmetto_coverage = units.to_base(coverage);
    }

    pub fn set_palmetto_gallberry_overstory_basal_area(
        &mut self,
        basal_area: f64,
        units: BasalAreaUnits,
    ) {
        self.overstory_basal_area = units.to_base(basal_area);
    }

    pub fn set_is_using_palmetto_gallberry(&mut self, is_using: bool) {
        self.is_using_palmetto_gallberry = is_using;
        if is_using {
            // Special fuel types are mutually exclusive
            self.is_using_chaparral = false;
            self.is_using_western_aspen = false;
        }
    }

    // Western Aspen setters

    pub fn set_aspen_fuel_model_number(&mut self, number: i32) {
        self.aspen_fuel_model_number = number;
    }

    pub fn set_aspen_curing_level(&mut self, level: f64, units: FractionUnits) {
        self.aspen_curing_level = units.to_base(level);
    }

    pub fn set_aspen_dbh(&mut self, dbh: f64, units: LengthUnits) {
        self.dbh = units.to_base(dbh);
    }

    pub fn set_aspen_fire_severity(&mut self, severity: AspenFireSeverity) {
        self.aspen_fire_severity = severity;
    }

    pub fn set_is_using_western_aspen(&mut self, is_using: bool) {
        self.is_using_western_aspen = is_using;
        if is_using {
            self.is_using_chaparral = false;
            self.is_using_palmetto_gallberry = false;
        }
    }

    // Chaparral setters

    pub fn set_chaparral_fuel_load_input_mode(&mut self, mode: ChaparralFuelLoadInputMode) {
        self.chaparral_fuel_load_input_mode = mode;
    }

    pub fn set_chaparral_fuel_type(&mut self, fuel_type: ChaparralFuelType) {
        self.chaparral_fuel_type = fuel_type;
    }

    pub fn set_chaparral_fuel_bed_depth(&mut self, depth: f64, units: LengthUnits) {
        self.chaparral_fuel_bed_depth = units.to_base(depth);
    }

    pub fn set_chaparral_fuel_dead_load_fraction(&mut self, fraction: f64) {
        self.chaparral_fuel_dead_load_fraction = fraction;
    }

    pub fn set_chaparral_total_fuel_load(&mut self, load: f64, units: LoadingUnits) {
        self.chaparral_total_fuel_load = units.to_base(load);
    }

    pub fn set_is_using_chaparral(&mut self, is_using: bool) {
        self.is_using_chaparral = is_using;
        if is_using {
            self.is_using_palmetto_gallberry = false;
            self.is_using_western_aspen = false;
        }
    }

    pub fn set_surface_fire_spread_direction_mode(&mut self, mode: SurfaceFireSpreadDirectionMode) {
        self.surface_fire_spread_direction_mode = mode;
    }

    // -----------------------------------------------------------------------
    // Getters
    // -----------------------------------------------------------------------

    pub fn fuel_model_number(&self) -> i32 {
        self.fuel_model_number
    }

    /// Get moisture for 1-hour dead fuel. Reads from the resolved moisture vector.
    pub fn moisture_one_hour(&self, units: FractionUnits) -> f64 {
        units.from_base(self.moisture_values_by_size_class[MoistureClassInput::OneHour as usize])
    }

    pub fn moisture_ten_hour(&self, units: FractionUnits) -> f64 {
        units.from_base(self.moisture_values_by_size_class[MoistureClassInput::TenHour as usize])
    }

    pub fn moisture_hundred_hour(&self, units: FractionUnits) -> f64 {
        units
            .from_base(self.moisture_values_by_size_class[MoistureClassInput::HundredHour as usize])
    }

    pub fn moisture_dead_aggregate_value(&self, units: FractionUnits) -> f64 {
        units.from_base(
            self.moisture_values_by_size_class[MoistureClassInput::DeadAggregate as usize],
        )
    }

    pub fn moisture_live_herbaceous(&self, units: FractionUnits) -> f64 {
        units.from_base(
            self.moisture_values_by_size_class[MoistureClassInput::LiveHerbaceous as usize],
        )
    }

    pub fn moisture_live_woody(&self, units: FractionUnits) -> f64 {
        units.from_base(self.moisture_values_by_size_class[MoistureClassInput::LiveWoody as usize])
    }

    pub fn moisture_live_aggregate_value(&self, units: FractionUnits) -> f64 {
        units.from_base(
            self.moisture_values_by_size_class[MoistureClassInput::LiveAggregate as usize],
        )
    }

    pub fn wind_speed(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.wind_speed)
    }

    pub fn wind_direction(&self) -> f64 {
        self.wind_direction
    }

    pub fn slope(&self, units: SlopeUnits) -> f64 {
        units.from_base(self.slope)
    }

    pub fn aspect(&self) -> f64 {
        self.aspect
    }

    pub fn canopy_cover(&self, units: FractionUnits) -> f64 {
        units.from_base(self.canopy_cover)
    }

    /// Get canopy height.
    ///
    /// NOTE: C++ `getCanopyHeight()` accepts a units parameter but ignores it,
    /// returning the raw base-unit value. We preserve this bug for numerical parity.
    pub fn canopy_height(&self, _units: LengthUnits) -> f64 {
        self.canopy_height
    }

    pub fn crown_ratio(&self, units: FractionUnits) -> f64 {
        units.from_base(self.crown_ratio)
    }

    pub fn wind_and_spread_orientation_mode(&self) -> WindAndSpreadOrientationMode {
        self.wind_and_spread_orientation_mode
    }

    pub fn wind_height_input_mode(&self) -> WindHeightInputMode {
        self.wind_height_input_mode
    }

    pub fn user_provided_wind_adjustment_factor(&self) -> f64 {
        self.user_provided_wind_adjustment_factor
    }

    pub fn wind_adjustment_factor_calculation_method(&self) -> WindAdjustmentFactorCalculationMethod {
        self.wind_adjustment_factor_calculation_method
    }

    pub fn elapsed_time(&self, units: TimeUnits) -> f64 {
        units.from_base(self.elapsed_time)
    }

    pub fn air_temperature(&self, units: TemperatureUnits) -> f64 {
        units.from_base(self.air_temperature)
    }

    pub fn is_calculating_scorch_height(&self) -> bool {
        self.is_calculating_scorch_height
    }

    // Two fuel models getters

    pub fn is_using_two_fuel_models(&self) -> bool {
        self.is_using_two_fuel_models
    }

    pub fn two_fuel_models_method(&self) -> TwoFuelModelsMethod {
        self.two_fuel_models_method
    }

    pub fn first_fuel_model_number(&self) -> i32 {
        self.fuel_model_number
    }

    pub fn second_fuel_model_number(&self) -> i32 {
        self.second_fuel_model_number
    }

    pub fn first_fuel_model_coverage(&self) -> f64 {
        self.first_fuel_model_coverage
    }

    // Palmetto-Gallberry getters

    pub fn is_using_palmetto_gallberry(&self) -> bool {
        self.is_using_palmetto_gallberry
    }

    pub fn palmetto_gallberry_age_of_rough(&self) -> f64 {
        self.age_of_rough
    }

    pub fn palmetto_gallberry_height_of_understory(&self, units: LengthUnits) -> f64 {
        units.from_base(self.height_of_understory)
    }

    pub fn palmetto_gallberry_palmetto_coverage(&self, units: FractionUnits) -> f64 {
        units.from_base(self.palmetto_coverage)
    }

    pub fn palmetto_gallberry_overstory_basal_area(&self, units: BasalAreaUnits) -> f64 {
        units.from_base(self.overstory_basal_area)
    }

    // Western Aspen getters

    pub fn is_using_western_aspen(&self) -> bool {
        self.is_using_western_aspen
    }

    pub fn aspen_fuel_model_number(&self) -> i32 {
        self.aspen_fuel_model_number
    }

    pub fn aspen_curing_level(&self, units: FractionUnits) -> f64 {
        units.from_base(self.aspen_curing_level)
    }

    pub fn aspen_dbh(&self, units: LengthUnits) -> f64 {
        units.from_base(self.dbh)
    }

    pub fn aspen_fire_severity(&self) -> AspenFireSeverity {
        self.aspen_fire_severity
    }

    // Chaparral getters

    pub fn chaparral_fuel_load_input_mode(&self) -> ChaparralFuelLoadInputMode {
        self.chaparral_fuel_load_input_mode
    }

    pub fn chaparral_fuel_type(&self) -> ChaparralFuelType {
        self.chaparral_fuel_type
    }

    pub fn chaparral_fuel_bed_depth(&self, units: LengthUnits) -> f64 {
        units.from_base(self.chaparral_fuel_bed_depth)
    }

    pub fn chaparral_fuel_dead_load_fraction(&self) -> f64 {
        self.chaparral_fuel_dead_load_fraction
    }

    pub fn chaparral_total_fuel_load(&self, units: LoadingUnits) -> f64 {
        units.from_base(self.chaparral_total_fuel_load)
    }

    pub fn is_using_chaparral(&self) -> bool {
        self.is_using_chaparral
    }

    pub fn surface_fire_spread_direction_mode(&self) -> SurfaceFireSpreadDirectionMode {
        self.surface_fire_spread_direction_mode
    }

    // Moisture scenario getters (delegate to scenarios reference)

    pub fn moisture_input_mode(&self) -> MoistureInputMode {
        self.moisture_input_mode
    }

    pub fn current_moisture_scenario_name(&self) -> &str {
        &self.current_moisture_scenario_name
    }

    pub fn current_moisture_scenario_index(&self) -> i32 {
        self.current_moisture_scenario_index
    }

    /// Check whether a specific moisture class input is needed given the current input mode.
    ///
    /// C++ method: `isMoistureClassInputNeeded`
    pub fn is_moisture_class_input_needed(&self, moisture_class: MoistureClassInput) -> bool {
        match moisture_class {
            MoistureClassInput::OneHour
            | MoistureClassInput::TenHour
            | MoistureClassInput::HundredHour => matches!(
                self.moisture_input_mode,
                MoistureInputMode::BySizeClass | MoistureInputMode::LiveAggregateAndDeadSizeClass
            ),
            MoistureClassInput::DeadAggregate => matches!(
                self.moisture_input_mode,
                MoistureInputMode::AllAggregate
                    | MoistureInputMode::DeadAggregateAndLiveSizeClass
            ),
            MoistureClassInput::LiveHerbaceous | MoistureClassInput::LiveWoody => matches!(
                self.moisture_input_mode,
                MoistureInputMode::BySizeClass
                    | MoistureInputMode::DeadAggregateAndLiveSizeClass
            ),
            MoistureClassInput::LiveAggregate => matches!(
                self.moisture_input_mode,
                MoistureInputMode::AllAggregate
                    | MoistureInputMode::LiveAggregateAndDeadSizeClass
            ),
        }
    }

    /// Get the raw moisture values vector (for internal use by fire spread calculations).
    pub fn moisture_by_size_class(&self, class: MoistureClassInput) -> f64 {
        self.moisture_values_by_size_class[class as usize]
    }

    // -----------------------------------------------------------------------
    // Internal: moisture resolution
    // -----------------------------------------------------------------------

    /// Resolve the moisture_values_by_size_class array based on the current input mode.
    ///
    /// C++ method: `updateMoisturesBasedOnInputMode`
    pub fn update_moistures_based_on_input_mode(
        &mut self,
        scenarios: Option<&MoistureScenarios>,
    ) {
        match self.moisture_input_mode {
            MoistureInputMode::BySizeClass => {
                self.moisture_values_by_size_class[MoistureClassInput::OneHour as usize] =
                    self.moisture_one_hour;
                self.moisture_values_by_size_class[MoistureClassInput::TenHour as usize] =
                    self.moisture_ten_hour;
                self.moisture_values_by_size_class[MoistureClassInput::HundredHour as usize] =
                    self.moisture_hundred_hour;
                self.moisture_values_by_size_class[MoistureClassInput::LiveHerbaceous as usize] =
                    self.moisture_live_herbaceous;
                self.moisture_values_by_size_class[MoistureClassInput::LiveWoody as usize] =
                    self.moisture_live_woody;
                self.moisture_values_by_size_class[MoistureClassInput::DeadAggregate as usize] =
                    -1.0;
                self.moisture_values_by_size_class[MoistureClassInput::LiveAggregate as usize] =
                    -1.0;
            }
            MoistureInputMode::AllAggregate => {
                self.moisture_values_by_size_class[MoistureClassInput::OneHour as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::TenHour as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::HundredHour as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::LiveHerbaceous as usize] =
                    self.moisture_live_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::LiveWoody as usize] =
                    self.moisture_live_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::DeadAggregate as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::LiveAggregate as usize] =
                    self.moisture_live_aggregate;
            }
            MoistureInputMode::DeadAggregateAndLiveSizeClass => {
                self.moisture_values_by_size_class[MoistureClassInput::OneHour as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::TenHour as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::HundredHour as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::LiveHerbaceous as usize] =
                    self.moisture_live_herbaceous;
                self.moisture_values_by_size_class[MoistureClassInput::LiveWoody as usize] =
                    self.moisture_live_woody;
                self.moisture_values_by_size_class[MoistureClassInput::DeadAggregate as usize] =
                    self.moisture_dead_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::LiveAggregate as usize] =
                    -1.0;
            }
            MoistureInputMode::LiveAggregateAndDeadSizeClass => {
                self.moisture_values_by_size_class[MoistureClassInput::OneHour as usize] =
                    self.moisture_one_hour;
                self.moisture_values_by_size_class[MoistureClassInput::TenHour as usize] =
                    self.moisture_ten_hour;
                self.moisture_values_by_size_class[MoistureClassInput::HundredHour as usize] =
                    self.moisture_hundred_hour;
                self.moisture_values_by_size_class[MoistureClassInput::LiveHerbaceous as usize] =
                    self.moisture_live_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::LiveWoody as usize] =
                    self.moisture_live_aggregate;
                self.moisture_values_by_size_class[MoistureClassInput::DeadAggregate as usize] =
                    -1.0;
                self.moisture_values_by_size_class[MoistureClassInput::LiveAggregate as usize] =
                    self.moisture_live_aggregate;
            }
            MoistureInputMode::MoistureScenario => {
                if let Some(sc) = scenarios {
                    let idx = self.current_moisture_scenario_index;
                    self.moisture_values_by_size_class[MoistureClassInput::OneHour as usize] =
                        sc.one_hour_by_index(idx, FractionUnits::Fraction);
                    self.moisture_values_by_size_class[MoistureClassInput::TenHour as usize] =
                        sc.ten_hour_by_index(idx, FractionUnits::Fraction);
                    self.moisture_values_by_size_class[MoistureClassInput::HundredHour as usize] =
                        sc.hundred_hour_by_index(idx, FractionUnits::Fraction);
                    self.moisture_values_by_size_class
                        [MoistureClassInput::LiveHerbaceous as usize] =
                        sc.live_herbaceous_by_index(idx, FractionUnits::Fraction);
                    self.moisture_values_by_size_class[MoistureClassInput::LiveWoody as usize] =
                        sc.live_woody_by_index(idx, FractionUnits::Fraction);
                    self.moisture_values_by_size_class
                        [MoistureClassInput::DeadAggregate as usize] = -1.0;
                    self.moisture_values_by_size_class
                        [MoistureClassInput::LiveAggregate as usize] = -1.0;
                } else {
                    // No scenarios available — all -1.0 (matching C++ nullptr case)
                    for v in self.moisture_values_by_size_class.iter_mut() {
                        *v = -1.0;
                    }
                }
            }
        }
    }
}

impl Default for SurfaceInputs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let inputs = SurfaceInputs::new();
        assert_eq!(inputs.fuel_model_number(), 0);
        assert_eq!(inputs.air_temperature(TemperatureUnits::Fahrenheit), -500.0);
        assert_eq!(inputs.wind_speed(SpeedUnits::FeetPerMinute), 0.0);
        assert_eq!(inputs.wind_direction(), 0.0);
        assert_eq!(inputs.slope(SlopeUnits::Degrees), 0.0);
        assert_eq!(inputs.aspect(), 0.0);
        assert!(!inputs.is_calculating_scorch_height());
        assert!(!inputs.is_using_two_fuel_models());
        assert!(!inputs.is_using_palmetto_gallberry());
        assert!(!inputs.is_using_western_aspen());
        assert!(!inputs.is_using_chaparral());
        assert_eq!(inputs.aspen_fuel_model_number(), -1);
        assert_eq!(
            inputs.wind_height_input_mode(),
            WindHeightInputMode::DirectMidflame
        );
        assert_eq!(
            inputs.wind_and_spread_orientation_mode(),
            WindAndSpreadOrientationMode::RelativeToUpslope
        );
        assert_eq!(
            inputs.two_fuel_models_method(),
            TwoFuelModelsMethod::NoMethod
        );
        assert_eq!(
            inputs.wind_adjustment_factor_calculation_method(),
            WindAdjustmentFactorCalculationMethod::UseCrownRatio
        );
        assert_eq!(inputs.user_provided_wind_adjustment_factor(), -1.0);
    }

    #[test]
    fn default_elapsed_time_is_one_hour() {
        let inputs = SurfaceInputs::new();
        let elapsed = inputs.elapsed_time(TimeUnits::Hours);
        assert!((elapsed - 1.0).abs() < 1e-10);

        let elapsed_min = inputs.elapsed_time(TimeUnits::Minutes);
        assert!((elapsed_min - 60.0).abs() < 1e-10);
    }

    #[test]
    fn default_moisture_values_are_zero_in_by_size_class_mode() {
        let inputs = SurfaceInputs::new();
        // In BySizeClass mode, moisture values come from the individual fields (all 0.0)
        assert_eq!(inputs.moisture_one_hour(FractionUnits::Fraction), 0.0);
        assert_eq!(inputs.moisture_ten_hour(FractionUnits::Fraction), 0.0);
        assert_eq!(inputs.moisture_hundred_hour(FractionUnits::Fraction), 0.0);
        assert_eq!(
            inputs.moisture_live_herbaceous(FractionUnits::Fraction),
            0.0
        );
        assert_eq!(inputs.moisture_live_woody(FractionUnits::Fraction), 0.0);
        // Aggregates are -1.0 in BySizeClass mode
        assert_eq!(
            inputs.moisture_dead_aggregate_value(FractionUnits::Fraction),
            -1.0
        );
        assert_eq!(
            inputs.moisture_live_aggregate_value(FractionUnits::Fraction),
            -1.0
        );
    }

    #[test]
    fn set_and_get_fuel_model() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_fuel_model_number(10);
        assert_eq!(inputs.fuel_model_number(), 10);
    }

    #[test]
    fn set_and_get_moisture_with_units() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_moisture_one_hour(6.0, FractionUnits::Percent);
        let m = inputs.moisture_one_hour(FractionUnits::Fraction);
        assert!((m - 0.06).abs() < 1e-10);

        let m_pct = inputs.moisture_one_hour(FractionUnits::Percent);
        assert!((m_pct - 6.0).abs() < 1e-10);
    }

    #[test]
    fn set_and_get_wind_speed_with_units() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_wind_speed(5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot);
        assert_eq!(inputs.wind_height_input_mode(), WindHeightInputMode::TwentyFoot);

        // 5 mph = 440 ft/min
        let ft_min = inputs.wind_speed(SpeedUnits::FeetPerMinute);
        assert!((ft_min - 440.0).abs() < 1e-6);
    }

    #[test]
    fn set_and_get_slope_degrees() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_slope(30.0, SlopeUnits::Degrees);
        let s = inputs.slope(SlopeUnits::Degrees);
        assert!((s - 30.0).abs() < 1e-10);
    }

    #[test]
    fn set_and_get_air_temperature() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_air_temperature(70.0, TemperatureUnits::Fahrenheit);
        let t = inputs.air_temperature(TemperatureUnits::Fahrenheit);
        assert!((t - 70.0).abs() < 1e-10);
    }

    #[test]
    fn canopy_height_bug_preserved() {
        // C++ getCanopyHeight ignores its units parameter and returns base value
        let mut inputs = SurfaceInputs::new();
        inputs.set_canopy_height(50.0, LengthUnits::Feet);
        // 50 ft stored as base (ft)
        let h_ft = inputs.canopy_height(LengthUnits::Feet);
        assert!((h_ft - 50.0).abs() < 1e-10);
        // Asking for meters should still return feet (the C++ bug)
        let h_m = inputs.canopy_height(LengthUnits::Meters);
        assert!((h_m - 50.0).abs() < 1e-10);
    }

    #[test]
    fn special_fuel_types_mutually_exclusive() {
        let mut inputs = SurfaceInputs::new();

        inputs.set_is_using_palmetto_gallberry(true);
        assert!(inputs.is_using_palmetto_gallberry());
        assert!(!inputs.is_using_chaparral());
        assert!(!inputs.is_using_western_aspen());

        inputs.set_is_using_chaparral(true);
        assert!(inputs.is_using_chaparral());
        assert!(!inputs.is_using_palmetto_gallberry());
        assert!(!inputs.is_using_western_aspen());

        inputs.set_is_using_western_aspen(true);
        assert!(inputs.is_using_western_aspen());
        assert!(!inputs.is_using_chaparral());
        assert!(!inputs.is_using_palmetto_gallberry());
    }

    #[test]
    fn moisture_input_mode_all_aggregate() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_moisture_input_mode(MoistureInputMode::AllAggregate);
        inputs.set_moisture_dead_aggregate(0.10, FractionUnits::Fraction);
        inputs.set_moisture_live_aggregate(1.50, FractionUnits::Fraction);

        // All dead size classes should equal dead aggregate
        let m1 = inputs.moisture_one_hour(FractionUnits::Fraction);
        let m10 = inputs.moisture_ten_hour(FractionUnits::Fraction);
        let m100 = inputs.moisture_hundred_hour(FractionUnits::Fraction);
        assert!((m1 - 0.10).abs() < 1e-10);
        assert!((m10 - 0.10).abs() < 1e-10);
        assert!((m100 - 0.10).abs() < 1e-10);

        // All live classes should equal live aggregate
        let herb = inputs.moisture_live_herbaceous(FractionUnits::Fraction);
        let woody = inputs.moisture_live_woody(FractionUnits::Fraction);
        assert!((herb - 1.50).abs() < 1e-10);
        assert!((woody - 1.50).abs() < 1e-10);
    }

    #[test]
    fn moisture_input_mode_dead_aggregate_live_size_class() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_moisture_input_mode(MoistureInputMode::DeadAggregateAndLiveSizeClass);
        inputs.set_moisture_dead_aggregate(0.08, FractionUnits::Fraction);
        inputs.set_moisture_live_herbaceous(1.00, FractionUnits::Fraction);
        inputs.set_moisture_live_woody(0.80, FractionUnits::Fraction);

        // Dead classes = dead aggregate
        assert!((inputs.moisture_one_hour(FractionUnits::Fraction) - 0.08).abs() < 1e-10);
        assert!((inputs.moisture_ten_hour(FractionUnits::Fraction) - 0.08).abs() < 1e-10);
        // Live classes = individual values
        assert!(
            (inputs.moisture_live_herbaceous(FractionUnits::Fraction) - 1.00).abs() < 1e-10
        );
        assert!((inputs.moisture_live_woody(FractionUnits::Fraction) - 0.80).abs() < 1e-10);
        // Live aggregate = -1.0
        assert_eq!(
            inputs.moisture_live_aggregate_value(FractionUnits::Fraction),
            -1.0
        );
    }

    #[test]
    fn moisture_input_mode_live_aggregate_dead_size_class() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_moisture_input_mode(MoistureInputMode::LiveAggregateAndDeadSizeClass);
        inputs.set_moisture_one_hour(0.05, FractionUnits::Fraction);
        inputs.set_moisture_ten_hour(0.07, FractionUnits::Fraction);
        inputs.set_moisture_hundred_hour(0.10, FractionUnits::Fraction);
        inputs.set_moisture_live_aggregate(1.20, FractionUnits::Fraction);

        assert!((inputs.moisture_one_hour(FractionUnits::Fraction) - 0.05).abs() < 1e-10);
        assert!((inputs.moisture_ten_hour(FractionUnits::Fraction) - 0.07).abs() < 1e-10);
        // Live herb & woody = live aggregate
        assert!(
            (inputs.moisture_live_herbaceous(FractionUnits::Fraction) - 1.20).abs() < 1e-10
        );
        assert!((inputs.moisture_live_woody(FractionUnits::Fraction) - 1.20).abs() < 1e-10);
    }

    #[test]
    fn moisture_scenario_mode_no_scenarios() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_moisture_input_mode(MoistureInputMode::MoistureScenario);
        // Without scenarios, all values should be -1.0
        // Need to re-trigger the update since set_moisture_input_mode calls with None
        assert_eq!(inputs.moisture_one_hour(FractionUnits::Fraction), -1.0);
        assert_eq!(inputs.moisture_ten_hour(FractionUnits::Fraction), -1.0);
    }

    #[test]
    fn moisture_scenario_mode_with_scenarios() {
        let mut inputs = SurfaceInputs::new();
        let scenarios = MoistureScenarios::new();
        inputs.set_moisture_input_mode(MoistureInputMode::MoistureScenario);

        // Set D1L1 scenario by name
        let ok = inputs.set_current_moisture_scenario_by_name("D1L1", &scenarios);
        assert!(ok);
        assert_eq!(inputs.current_moisture_scenario_name(), "D1L1");

        // D1L1: 1h=0.03, 10h=0.04, 100h=0.05, herb=0.30, woody=0.60
        let m1 = inputs.moisture_one_hour(FractionUnits::Fraction);
        assert!((m1 - 0.03).abs() < 1e-10);
        let m10 = inputs.moisture_ten_hour(FractionUnits::Fraction);
        assert!((m10 - 0.04).abs() < 1e-10);
        let herb = inputs.moisture_live_herbaceous(FractionUnits::Fraction);
        assert!((herb - 0.30).abs() < 1e-10);
    }

    #[test]
    fn moisture_scenario_by_index() {
        let mut inputs = SurfaceInputs::new();
        let scenarios = MoistureScenarios::new();
        inputs.set_moisture_input_mode(MoistureInputMode::MoistureScenario);

        // Index 0 = D1L1
        let ok = inputs.set_current_moisture_scenario_by_index(0, &scenarios);
        assert!(ok);
        assert_eq!(inputs.current_moisture_scenario_index(), 0);
        let m1 = inputs.moisture_one_hour(FractionUnits::Fraction);
        assert!((m1 - 0.03).abs() < 1e-10);
    }

    #[test]
    fn moisture_class_input_needed() {
        let mut inputs = SurfaceInputs::new();

        // BySizeClass mode
        assert!(inputs.is_moisture_class_input_needed(MoistureClassInput::OneHour));
        assert!(inputs.is_moisture_class_input_needed(MoistureClassInput::LiveHerbaceous));
        assert!(!inputs.is_moisture_class_input_needed(MoistureClassInput::DeadAggregate));
        assert!(!inputs.is_moisture_class_input_needed(MoistureClassInput::LiveAggregate));

        // AllAggregate mode
        inputs.set_moisture_input_mode(MoistureInputMode::AllAggregate);
        assert!(!inputs.is_moisture_class_input_needed(MoistureClassInput::OneHour));
        assert!(inputs.is_moisture_class_input_needed(MoistureClassInput::DeadAggregate));
        assert!(inputs.is_moisture_class_input_needed(MoistureClassInput::LiveAggregate));
        assert!(!inputs.is_moisture_class_input_needed(MoistureClassInput::LiveHerbaceous));
    }

    #[test]
    fn update_surface_inputs_normalizes_wind_direction() {
        let mut inputs = SurfaceInputs::new();
        inputs.update_surface_inputs(
            1,
            0.06,
            0.07,
            0.08,
            1.0,
            1.5,
            FractionUnits::Fraction,
            5.0,
            SpeedUnits::MilesPerHour,
            WindHeightInputMode::DirectMidflame,
            -10.0, // negative wind direction
            WindAndSpreadOrientationMode::RelativeToUpslope,
            30.0,
            SlopeUnits::Degrees,
            180.0,
            0.5,
            FractionUnits::Fraction,
            60.0,
            LengthUnits::Feet,
            0.3,
            FractionUnits::Fraction,
        );
        // -10 + 360 = 350
        assert!((inputs.wind_direction() - 350.0).abs() < 1e-10);
        assert_eq!(inputs.fuel_model_number(), 1);
        assert!(!inputs.is_using_two_fuel_models());
    }

    #[test]
    fn update_surface_inputs_for_two_fuel_models() {
        let mut inputs = SurfaceInputs::new();
        inputs.update_surface_inputs_for_two_fuel_models(
            1,
            2,
            0.06,
            0.07,
            0.08,
            1.0,
            1.5,
            FractionUnits::Fraction,
            5.0,
            SpeedUnits::MilesPerHour,
            WindHeightInputMode::DirectMidflame,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            0.75,
            FractionUnits::Fraction,
            TwoFuelModelsMethod::Arithmetic,
            30.0,
            SlopeUnits::Degrees,
            180.0,
            0.5,
            FractionUnits::Fraction,
            60.0,
            LengthUnits::Feet,
            0.3,
            FractionUnits::Fraction,
        );
        assert!(inputs.is_using_two_fuel_models());
        assert_eq!(inputs.first_fuel_model_number(), 1);
        assert_eq!(inputs.second_fuel_model_number(), 2);
        assert!((inputs.first_fuel_model_coverage() - 0.75).abs() < 1e-10);
        assert_eq!(inputs.two_fuel_models_method(), TwoFuelModelsMethod::Arithmetic);
    }

    #[test]
    fn palmetto_gallberry_setters_and_getters() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_palmetto_gallberry_age_of_rough(5.0);
        inputs.set_palmetto_gallberry_height_of_understory(3.0, LengthUnits::Feet);
        inputs.set_palmetto_gallberry_palmetto_coverage(0.6, FractionUnits::Fraction);
        inputs.set_palmetto_gallberry_overstory_basal_area(80.0, BasalAreaUnits::SquareFeetPerAcre);

        assert!((inputs.palmetto_gallberry_age_of_rough() - 5.0).abs() < 1e-10);
        assert!(
            (inputs.palmetto_gallberry_height_of_understory(LengthUnits::Feet) - 3.0).abs()
                < 1e-10
        );
        assert!(
            (inputs.palmetto_gallberry_palmetto_coverage(FractionUnits::Fraction) - 0.6).abs()
                < 1e-10
        );
    }

    #[test]
    fn chaparral_setters_and_getters() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_chaparral_fuel_type(ChaparralFuelType::Chamise);
        inputs.set_chaparral_fuel_bed_depth(6.0, LengthUnits::Feet);
        inputs.set_chaparral_fuel_dead_load_fraction(0.5);
        inputs.set_chaparral_total_fuel_load(3.0, LoadingUnits::TonsPerAcre);

        assert_eq!(inputs.chaparral_fuel_type(), ChaparralFuelType::Chamise);
        assert!(
            (inputs.chaparral_fuel_bed_depth(LengthUnits::Feet) - 6.0).abs() < 1e-10
        );
        assert!((inputs.chaparral_fuel_dead_load_fraction() - 0.5).abs() < 1e-10);
        let load = inputs.chaparral_total_fuel_load(LoadingUnits::TonsPerAcre);
        assert!((load - 3.0).abs() < 1e-6);
    }

    #[test]
    fn aspen_setters_and_getters() {
        let mut inputs = SurfaceInputs::new();
        inputs.set_aspen_fuel_model_number(1);
        inputs.set_aspen_curing_level(50.0, FractionUnits::Percent);
        inputs.set_aspen_dbh(10.0, LengthUnits::Inches);
        inputs.set_aspen_fire_severity(AspenFireSeverity::Moderate);

        assert_eq!(inputs.aspen_fuel_model_number(), 1);
        assert!((inputs.aspen_curing_level(FractionUnits::Percent) - 50.0).abs() < 1e-10);
        assert!((inputs.aspen_dbh(LengthUnits::Inches) - 10.0).abs() < 1e-6);
        assert_eq!(inputs.aspen_fire_severity(), AspenFireSeverity::Moderate);
    }
}
