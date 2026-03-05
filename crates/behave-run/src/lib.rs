//! Top-level BehaveRun facade.
//!
//! C++ source: behaveRun.h / behaveRun.cpp
//!
//! Composes all BehavePlus modules into a single struct.

#![allow(dead_code)]

use firelab_base::*;
use behave_surface::facade::Surface;
use behave_surface::fuel_models::FuelModels;
use behave_surface::moisture::MoistureScenarios;
use behave_crown::fire::Crown;
use behave_spot::distance::Spot;
use behave_ignite::probability::Ignite;
use behave_contain::adapter::ContainAdapter;
use behave_mortality::safety::Safety;
use behave_mortality::calculator::MortalityCalculator;
use behave_mortality::species::SpeciesMasterTable;
use behave_weather::fine_dead_fuel::FineDeadFuelMoistureTool;
use behave_weather::slope::SlopeTool;
use behave_weather::vpd::VaporPressureDeficitCalculator;
use behave_weather::relative_humidity::RelativeHumidityTool;

/// Top-level facade composing all BehavePlus fire behavior modules.
///
/// All module instances are public fields, matching the C++ design where
/// callers access `behave_run.surface`, `behave_run.crown`, etc. directly.
pub struct BehaveRun {
    pub surface: Surface,
    pub crown: Crown,
    pub spot: Spot,
    pub ignite: Ignite,
    pub contain: ContainAdapter,
    pub safety: Safety,
    pub mortality: MortalityCalculator,
    pub fine_dead_fuel_moisture_tool: FineDeadFuelMoistureTool,
    pub slope_tool: SlopeTool,
    pub vpd_calculator: VaporPressureDeficitCalculator,
    pub rh_tool: RelativeHumidityTool,
}

impl BehaveRun {
    pub fn new(fuel_models: FuelModels, species_table: SpeciesMasterTable) -> Self {
        let surface = Surface::new(fuel_models.clone());
        let crown = Crown::new(fuel_models);
        Self {
            surface,
            crown,
            spot: Spot::new(),
            ignite: Ignite::new(),
            contain: ContainAdapter::new(),
            safety: Safety::new(),
            mortality: MortalityCalculator::new(species_table),
            fine_dead_fuel_moisture_tool: FineDeadFuelMoistureTool::new(),
            slope_tool: SlopeTool::new(),
            vpd_calculator: VaporPressureDeficitCalculator::new(),
            rh_tool: RelativeHumidityTool::new(),
        }
    }

    /// Re-create Surface and Crown with new fuel models.
    pub fn set_fuel_models(&mut self, fuel_models: FuelModels) {
        self.surface = Surface::new(fuel_models.clone());
        self.crown = Crown::new(fuel_models);
    }

    pub fn set_moisture_scenarios(&mut self, scenarios: MoistureScenarios) {
        self.surface.set_moisture_scenarios(scenarios.clone());
        self.crown.set_moisture_scenarios(scenarios);
    }

    // --- Fuel model delegation methods ---

    pub fn fuel_code(&self, model: i32) -> &str {
        self.surface.fuel_models().fuel_code(model)
    }

    pub fn fuel_name(&self, model: i32) -> &str {
        self.surface.fuel_models().fuel_name(model)
    }

    pub fn fuelbed_depth(&self, model: i32, units: LengthUnits) -> f64 {
        self.surface.fuel_models().fuelbed_depth(model, units)
    }

    pub fn fuel_moisture_of_extinction_dead(&self, model: i32, units: FractionUnits) -> f64 {
        self.surface.fuel_models().moisture_of_extinction_dead(model, units)
    }

    pub fn is_fuel_dynamic(&self, model: i32) -> bool {
        self.surface.fuel_models().is_dynamic(model)
    }

    pub fn is_fuel_model_defined(&self, model: i32) -> bool {
        self.surface.fuel_models().is_fuel_model_defined(model)
    }

    pub fn is_fuel_model_reserved(&self, model: i32) -> bool {
        self.surface.fuel_models().is_fuel_model_reserved(model)
    }

    pub fn is_all_fuel_load_zero(&self, model: i32) -> bool {
        self.surface.fuel_models().is_all_fuel_load_zero(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behave_run_construction() {
        let fuel_models = FuelModels::new();
        let species_table = SpeciesMasterTable::new();
        let run = BehaveRun::new(fuel_models, species_table);

        assert!(run.is_fuel_model_defined(1));
        assert!(!run.is_fuel_model_defined(200));
    }

    #[test]
    fn behave_run_fuel_model_queries() {
        let fuel_models = FuelModels::new();
        let species_table = SpeciesMasterTable::new();
        let run = BehaveRun::new(fuel_models, species_table);

        let depth = run.fuelbed_depth(1, LengthUnits::Feet);
        assert!(depth > 0.0, "FM1 depth should be positive, got {depth}");

        let code = run.fuel_code(1);
        assert!(!code.is_empty(), "FM1 code should not be empty");
    }
}
