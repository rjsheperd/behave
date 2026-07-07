//! Fuelbed intermediate calculations for the Rothermel spread model.
//!
//! Computes weighted fuel properties, packing ratio, characteristic SAV ratio,
//! propagating flux, heat sink, and other intermediates needed before reaction
//! intensity and spread rate calculations.
//!
//! C++ source: surfaceFuelbedIntermediates.h / surfaceFuelbedIntermediates.cpp

use firelab_base::{
    BasalAreaUnits, FractionUnits, FuelConstants, FuelLifeState, HeatOfCombustionUnits,
    LengthUnits, LoadingUnits, MoistureInputMode, SurfaceAreaToVolumeUnits, UnitConversion,
};

use crate::chaparral::{ChaparralFuel, ChaparralFuelLoadInputMode, CHAPARRAL_NUM_FUEL_CLASSES};
use crate::fuel_models::FuelModels;
use crate::inputs::SurfaceInputs;
use crate::palmetto_gallberry::PalmettoGallberry;
use crate::western_aspen::WesternAspen;

const MAX_LIFE: usize = FuelConstants::MAX_LIFE_STATES;
const MAX_P: usize = FuelConstants::MAX_PARTICLES;
const MAX_SAVR: usize = FuelConstants::MAX_SAVR_SIZE_CLASSES;

/// Intermediate fuelbed properties derived from fuel model inputs.
///
/// C++ class: `SurfaceFuelbedIntermediates`
#[derive(Debug, Clone)]
pub struct FuelbedIntermediates {
    // Per-particle arrays (indexed by size class 0..MAX_P)
    load_dead: [f64; MAX_P],
    load_live: [f64; MAX_P],
    savr_dead: [f64; MAX_P],
    savr_live: [f64; MAX_P],
    moisture_dead: [f64; MAX_P],
    moisture_live: [f64; MAX_P],
    surface_area_dead: [f64; MAX_P],
    surface_area_live: [f64; MAX_P],
    heat_of_combustion_dead: [f64; MAX_P],
    heat_of_combustion_live: [f64; MAX_P],
    silica_effective_dead: [f64; MAX_P],
    silica_effective_live: [f64; MAX_P],
    fuel_density_dead: [f64; MAX_P],
    fuel_density_live: [f64; MAX_P],
    fraction_of_total_surface_area_dead: [f64; MAX_P],
    fraction_of_total_surface_area_live: [f64; MAX_P],
    size_sorted_fraction_dead: [f64; MAX_SAVR],
    size_sorted_fraction_live: [f64; MAX_SAVR],

    // Per-life-state arrays (indexed by FuelLifeState: Dead=0, Live=1)
    number_of_size_classes: [i32; MAX_LIFE],
    weighted_moisture: [f64; MAX_LIFE],
    total_surface_area: [f64; MAX_LIFE],
    weighted_heat: [f64; MAX_LIFE],
    weighted_silica: [f64; MAX_LIFE],
    weighted_fuel_load: [f64; MAX_LIFE],
    moisture_of_extinction: [f64; MAX_LIFE],
    fraction_of_total_surface_area: [f64; MAX_LIFE],
    total_load_for_life_state: [f64; MAX_LIFE],

    // Special fuel model helpers (C++ members chaparralFuel_,
    // palmettoGallberry_, westernAspen_)
    chaparral_fuel: ChaparralFuel,
    palmetto_gallberry: PalmettoGallberry,
    western_aspen: WesternAspen,

    // Scalar intermediates
    fuel_model_number: i32,
    depth: f64,
    sigma: f64,
    bulk_density: f64,
    packing_ratio: f64,
    relative_packing_ratio: f64,
    heat_sink: f64,
    propagating_flux: f64,
    total_silica_content: f64,
}

impl FuelbedIntermediates {
    pub fn new() -> Self {
        let mut s = Self {
            load_dead: [0.0; MAX_P],
            load_live: [0.0; MAX_P],
            savr_dead: [0.0; MAX_P],
            savr_live: [0.0; MAX_P],
            moisture_dead: [0.0; MAX_P],
            moisture_live: [0.0; MAX_P],
            surface_area_dead: [0.0; MAX_P],
            surface_area_live: [0.0; MAX_P],
            heat_of_combustion_dead: [0.0; MAX_P],
            heat_of_combustion_live: [0.0; MAX_P],
            silica_effective_dead: [0.0; MAX_P],
            silica_effective_live: [0.0; MAX_P],
            fuel_density_dead: [0.0; MAX_P],
            fuel_density_live: [0.0; MAX_P],
            fraction_of_total_surface_area_dead: [0.0; MAX_P],
            fraction_of_total_surface_area_live: [0.0; MAX_P],
            size_sorted_fraction_dead: [0.0; MAX_SAVR],
            size_sorted_fraction_live: [0.0; MAX_SAVR],

            number_of_size_classes: [0; MAX_LIFE],
            weighted_moisture: [0.0; MAX_LIFE],
            total_surface_area: [0.0; MAX_LIFE],
            weighted_heat: [0.0; MAX_LIFE],
            weighted_silica: [0.0; MAX_LIFE],
            weighted_fuel_load: [0.0; MAX_LIFE],
            moisture_of_extinction: [0.0; MAX_LIFE],
            fraction_of_total_surface_area: [0.0; MAX_LIFE],
            total_load_for_life_state: [0.0; MAX_LIFE],

            chaparral_fuel: ChaparralFuel::new(),
            palmetto_gallberry: PalmettoGallberry::new(),
            western_aspen: WesternAspen::new(),

            fuel_model_number: 0,
            depth: 0.0,
            sigma: 0.0,
            bulk_density: 0.0,
            packing_ratio: 0.0,
            relative_packing_ratio: 0.0,
            heat_sink: 0.0,
            propagating_flux: 0.0,
            total_silica_content: 0.0555,
        };
        s.initialize_members();
        s
    }

    /// Main calculation entry point.
    ///
    /// C++ method: `calculateFuelbedIntermediates`
    pub fn calculate_fuelbed_intermediates(
        &mut self,
        fuel_model_number: i32,
        fuel_models: &FuelModels,
        surface_inputs: &SurfaceInputs,
    ) {
        self.initialize_members();
        self.fuel_model_number = fuel_model_number;

        self.set_fuelbed_depth(fuel_models, surface_inputs);
        self.set_fuel_load(fuel_models, surface_inputs);
        self.count_size_classes();
        self.set_moisture_content(surface_inputs);
        self.set_savr(fuel_models, surface_inputs);

        let is_dynamic = fuel_models.is_dynamic(fuel_model_number);
        if is_dynamic {
            self.dynamic_load_transfer();
        }

        self.set_heat_of_combustion(fuel_models, surface_inputs);
        self.calculate_fraction_of_total_surface_area_for_life_states(surface_inputs);
        self.set_dead_fuel_moisture_of_extinction(fuel_models, surface_inputs);
        self.calculate_live_moisture_of_extinction();
        self.calculate_characteristic_savr(surface_inputs);

        // Final calculations
        let total_load = self.total_load_for_life_state[DEAD] + self.total_load_for_life_state[LIVE];
        self.bulk_density = total_load / self.depth;

        for i in 0..MAX_P {
            if self.fuel_density_dead[i] > 0.0 {
                self.packing_ratio += self.load_dead[i] / (self.depth * self.fuel_density_dead[i]);
            }
            if self.fuel_density_live[i] > 0.0 {
                self.packing_ratio += self.load_live[i] / (self.depth * self.fuel_density_live[i]);
            }
        }

        let optimum_packing_ratio = 3.348 / self.sigma.powf(0.8189);
        self.relative_packing_ratio = self.packing_ratio / optimum_packing_ratio;

        self.calculate_heat_sink();
        self.calculate_propagating_flux();
    }

    // -----------------------------------------------------------------------
    // Getters
    // -----------------------------------------------------------------------

    pub fn fuelbed_depth(&self) -> f64 {
        self.depth
    }

    pub fn bulk_density(&self) -> f64 {
        self.bulk_density
    }

    pub fn packing_ratio(&self) -> f64 {
        self.packing_ratio
    }

    pub fn propagating_flux(&self) -> f64 {
        self.propagating_flux
    }

    pub fn relative_packing_ratio(&self) -> f64 {
        self.relative_packing_ratio
    }

    pub fn sigma(&self) -> f64 {
        self.sigma
    }

    pub fn heat_sink(&self) -> f64 {
        self.heat_sink
    }

    pub fn weighted_moisture_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.weighted_moisture[life_state as usize]
    }

    pub fn moisture_of_extinction_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.moisture_of_extinction[life_state as usize]
    }

    pub fn weighted_heat_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.weighted_heat[life_state as usize]
    }

    pub fn weighted_silica_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.weighted_silica[life_state as usize]
    }

    pub fn weighted_fuel_load_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.weighted_fuel_load[life_state as usize]
    }

    pub fn fraction_of_total_surface_area_by_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.fraction_of_total_surface_area[life_state as usize]
    }

    pub fn total_live_fuel_load(&self, units: LoadingUnits) -> f64 {
        let mut total = 0.0;
        for i in 0..MAX_P {
            total += self.load_live[i];
        }
        units.from_base(total)
    }

    pub fn total_dead_fuel_load(&self, units: LoadingUnits) -> f64 {
        let mut total = 0.0;
        for i in 0..MAX_P {
            total += self.load_dead[i];
        }
        units.from_base(total)
    }

    /// Dead herbaceous fuel load (dynamic load transfer slot = index 3).
    pub fn total_dead_herbaceous_fuel_load(&self, units: LoadingUnits) -> f64 {
        units.from_base(self.load_dead[3])
    }

    // -----------------------------------------------------------------------
    // Internal calculation methods
    // -----------------------------------------------------------------------

    fn initialize_members(&mut self) {
        const NUM_LIVE_SIZE_CLASSES: usize = 2;

        self.depth = 0.0;
        self.relative_packing_ratio = 0.0;
        // C++ initializeMembers resets palmettoGallberry_ and westernAspen_
        // but deliberately not chaparralFuel_.
        self.palmetto_gallberry = PalmettoGallberry::new();
        self.western_aspen = WesternAspen::new();

        self.fuel_model_number = 0;
        self.sigma = 0.0;
        self.bulk_density = 0.0;
        self.packing_ratio = 0.0;
        self.heat_sink = 0.0;
        self.total_silica_content = 0.0555;
        self.propagating_flux = 0.0;

        self.size_sorted_fraction_dead = [0.0; MAX_SAVR];
        self.size_sorted_fraction_live = [0.0; MAX_SAVR];

        for i in 0..MAX_P {
            self.fraction_of_total_surface_area_dead[i] = 0.0;
            self.fraction_of_total_surface_area_live[i] = 0.0;
            self.surface_area_dead[i] = 0.0;
            self.surface_area_live[i] = 0.0;
            self.moisture_dead[i] = 0.0;
            self.moisture_live[i] = 0.0;
            self.load_dead[i] = 0.0;
            self.load_live[i] = 0.0;
            self.savr_dead[i] = 0.0;
            self.savr_live[i] = 0.0;
            self.heat_of_combustion_dead[i] = 0.0;
            self.heat_of_combustion_live[i] = 0.0;
            self.silica_effective_dead[i] = 0.01;
            self.silica_effective_live[i] = if i < NUM_LIVE_SIZE_CLASSES { 0.01 } else { 0.0 };
            self.fuel_density_dead[i] = 32.0; // Albini 1976, p. 91
            self.fuel_density_live[i] = 32.0;
        }

        for i in 0..MAX_LIFE {
            self.number_of_size_classes[i] = 0;
            self.total_load_for_life_state[i] = 0.0;
            self.fraction_of_total_surface_area[i] = 0.0;
            self.moisture_of_extinction[i] = 0.0;
            self.total_surface_area[i] = 0.0;
            self.weighted_moisture[i] = 0.0;
            self.weighted_silica[i] = 0.0;
        }
    }

    fn set_fuelbed_depth(&mut self, fuel_models: &FuelModels, surface_inputs: &SurfaceInputs) {
        if surface_inputs.is_using_palmetto_gallberry() {
            let height_of_understory =
                surface_inputs.palmetto_gallberry_height_of_understory(LengthUnits::Feet);
            self.depth = self.palmetto_gallberry.calculate_fuel_bed_depth(height_of_understory);
        } else if surface_inputs.is_using_western_aspen() {
            self.depth =
                WesternAspen::get_aspen_fuel_bed_depth(surface_inputs.aspen_fuel_model_number());
        } else if surface_inputs.is_using_chaparral() {
            self.depth = surface_inputs.chaparral_fuel_bed_depth(LengthUnits::Feet);
            self.chaparral_fuel.set_depth(self.depth);
        } else {
            self.depth = fuel_models.fuelbed_depth(self.fuel_model_number, LengthUnits::Feet);
        }
    }

    fn set_fuel_load(&mut self, fuel_models: &FuelModels, surface_inputs: &SurfaceInputs) {
        if surface_inputs.is_using_palmetto_gallberry() {
            let age = surface_inputs.palmetto_gallberry_age_of_rough();
            let height =
                surface_inputs.palmetto_gallberry_height_of_understory(LengthUnits::Feet);
            let coverage =
                surface_inputs.palmetto_gallberry_palmetto_coverage(FractionUnits::Fraction);
            let basal_area = surface_inputs
                .palmetto_gallberry_overstory_basal_area(BasalAreaUnits::SquareFeetPerAcre);

            let pg = &mut self.palmetto_gallberry;
            self.load_dead[0] = pg.calculate_dead_fine_fuel_load(age, height);
            self.load_dead[1] = pg.calculate_dead_medium_fuel_load(age, coverage);
            self.load_dead[2] = pg.calculate_dead_foliage_fuel_load(age, coverage);
            self.load_dead[3] = pg.calculate_litter_load(age, basal_area);
            self.load_dead[4] = 0.0;

            self.load_live[0] = pg.calculate_live_fine_fuel_load(age, height);
            self.load_live[1] = pg.calculate_live_medium_fuel_load(age, height);
            self.load_live[2] = pg.calculate_live_foliage_load(age, coverage, height);
            self.load_live[3] = 0.0;
            self.load_live[4] = 0.0;

            for i in 0..MAX_P {
                self.silica_effective_live[i] = 0.015;
            }
        } else if surface_inputs.is_using_western_aspen() {
            let model = surface_inputs.aspen_fuel_model_number();
            let curing = surface_inputs.aspen_curing_level(FractionUnits::Fraction);

            let wa = &mut self.western_aspen;
            self.load_dead[0] = wa.calculate_load_dead_one_hour(model, curing);
            self.load_dead[1] = wa.calculate_load_dead_ten_hour(model);
            self.load_dead[2] = 0.0;
            self.load_dead[3] = 0.0;
            self.load_dead[4] = 0.0;

            self.load_live[0] = wa.calculate_load_live_herbaceous(model, curing);
            self.load_live[1] = wa.calculate_load_live_woody(model, curing);
            self.load_live[2] = 0.0;
            self.load_live[3] = 0.0;
            self.load_live[4] = 0.0;
        } else if surface_inputs.is_using_chaparral() {
            let input_mode = surface_inputs.chaparral_fuel_load_input_mode();

            self.chaparral_fuel
                .set_dead_fuel_fraction(surface_inputs.chaparral_fuel_dead_load_fraction());

            match input_mode {
                ChaparralFuelLoadInputMode::DirectFuelLoad => {
                    self.chaparral_fuel.set_total_fuel_load(
                        surface_inputs.chaparral_total_fuel_load(LoadingUnits::PoundsPerSquareFoot),
                    );
                    self.chaparral_fuel.update_fuel_load_from_depth_and_dead_fuel_fraction();
                }
                ChaparralFuelLoadInputMode::FuelLoadFromDepthAndChaparralType => {
                    self.chaparral_fuel
                        .set_chaparral_fuel_type(surface_inputs.chaparral_fuel_type());
                    self.chaparral_fuel.set_depth(self.depth);
                    self.chaparral_fuel.update_fuel_load_from_depth_and_fuel_type();
                }
            }
            for i in 0..MAX_P {
                self.load_dead[i] = self.chaparral_fuel.get_load(FuelLifeState::Dead, i);
                self.load_live[i] = self.chaparral_fuel.get_load(FuelLifeState::Live, i);
            }
        } else {
            let n = self.fuel_model_number;
            let u = LoadingUnits::PoundsPerSquareFoot;
            self.load_dead[0] = fuel_models.fuel_load_one_hour(n, u);
            self.load_dead[1] = fuel_models.fuel_load_ten_hour(n, u);
            self.load_dead[2] = fuel_models.fuel_load_hundred_hour(n, u);
            self.load_dead[3] = 0.0;
            self.load_dead[4] = 0.0;

            self.load_live[0] = fuel_models.fuel_load_live_herbaceous(n, u);
            self.load_live[1] = fuel_models.fuel_load_live_woody(n, u);
            self.load_live[2] = 0.0;
            self.load_live[3] = 0.0;
            self.load_live[4] = 0.0;
        }
    }

    fn count_size_classes(&mut self) {
        // Count non-zero dead fuel loads (up to MaxDeadSizeClasses=4)
        for i in 0..FuelConstants::MAX_DEAD_SIZE_CLASSES {
            if self.load_dead[i] != 0.0 {
                self.number_of_size_classes[DEAD] += 1;
            }
        }
        // Count non-zero live fuel loads (up to MaxLiveSizeClasses=5)
        for i in 0..FuelConstants::MAX_LIVE_SIZE_CLASSES {
            if self.load_live[i] != 0.0 {
                self.number_of_size_classes[LIVE] += 1;
            }
        }
        // Boost to max if any are present (C++ behavior)
        if self.number_of_size_classes[LIVE] > 0 {
            self.number_of_size_classes[LIVE] = FuelConstants::MAX_LIVE_SIZE_CLASSES as i32;
        }
        if self.number_of_size_classes[DEAD] > 0 {
            self.number_of_size_classes[DEAD] = FuelConstants::MAX_DEAD_SIZE_CLASSES as i32;
        }
    }

    fn set_moisture_content(&mut self, surface_inputs: &SurfaceInputs) {
        if surface_inputs.is_using_chaparral() {
            let f = FractionUnits::Fraction;
            self.moisture_dead[0] = surface_inputs.moisture_one_hour(f);
            self.moisture_dead[1] = surface_inputs.moisture_ten_hour(f);
            self.moisture_dead[2] = surface_inputs.moisture_ten_hour(f);
            self.moisture_dead[3] = surface_inputs.moisture_hundred_hour(f);
            self.moisture_dead[4] = 0.0;

            self.moisture_live[0] = surface_inputs.moisture_live_herbaceous(f);
            self.moisture_live[1] = surface_inputs.moisture_live_woody(f);
            self.moisture_live[2] = surface_inputs.moisture_live_woody(f);
            self.moisture_live[3] = surface_inputs.moisture_live_woody(f);
            self.moisture_live[4] = surface_inputs.moisture_live_woody(f);
        } else if surface_inputs.is_using_palmetto_gallberry() {
            let f = FractionUnits::Fraction;
            self.moisture_dead[0] = surface_inputs.moisture_one_hour(f);
            self.moisture_dead[1] = surface_inputs.moisture_ten_hour(f);
            self.moisture_dead[2] = surface_inputs.moisture_one_hour(f);
            self.moisture_dead[3] = surface_inputs.moisture_hundred_hour(f);
            self.moisture_dead[4] = 0.0;

            self.moisture_live[0] = surface_inputs.moisture_live_woody(f);
            self.moisture_live[1] = surface_inputs.moisture_live_woody(f);
            self.moisture_live[2] = surface_inputs.moisture_live_herbaceous(f);
            self.moisture_live[3] = 0.0;
            self.moisture_live[4] = 0.0;
        } else {
            // Standard fuel models
            for i in 0..MAX_P {
                self.moisture_dead[i] = 0.0;
                self.moisture_live[i] = 0.0;
            }
            self.moisture_dead[0] =
                surface_inputs.moisture_one_hour(FractionUnits::Fraction);
            self.moisture_dead[1] =
                surface_inputs.moisture_ten_hour(FractionUnits::Fraction);
            self.moisture_dead[2] =
                surface_inputs.moisture_hundred_hour(FractionUnits::Fraction);
            self.moisture_dead[3] =
                surface_inputs.moisture_one_hour(FractionUnits::Fraction);

            self.moisture_live[0] =
                surface_inputs.moisture_live_herbaceous(FractionUnits::Fraction);
            self.moisture_live[1] =
                surface_inputs.moisture_live_woody(FractionUnits::Fraction);
        }
    }

    fn set_savr(&mut self, fuel_models: &FuelModels, surface_inputs: &SurfaceInputs) {
        if surface_inputs.is_using_palmetto_gallberry() {
            // Special values for Palmetto-Gallberry
            self.savr_dead[0] = 350.0;
            self.savr_dead[1] = 140.0;
            self.savr_dead[2] = 2000.0;
            self.savr_dead[3] = 2000.0; // C++ TODO: find appropriate savr for litter
            self.savr_dead[4] = 0.0;

            self.savr_live[0] = 350.0;
            self.savr_live[1] = 140.0;
            self.savr_live[2] = 2000.0;
            self.savr_live[3] = 0.0;
            self.savr_live[4] = 0.0;
        } else if surface_inputs.is_using_western_aspen() {
            let model = surface_inputs.aspen_fuel_model_number();
            let curing = surface_inputs.aspen_curing_level(FractionUnits::Fraction);

            let wa = &mut self.western_aspen;
            self.savr_dead[0] = wa.calculate_savr_dead_one_hour(model, curing);
            self.savr_dead[1] = wa.calculate_savr_dead_ten_hour();
            self.savr_dead[2] = 0.0;
            self.savr_dead[3] = 0.0;
            self.savr_dead[4] = 0.0;

            self.savr_live[0] = wa.calculate_savr_live_herbaceous();
            self.savr_live[1] = wa.calculate_savr_live_woody(model, curing);
            self.savr_live[2] = 0.0;
            self.savr_live[3] = 0.0;
            self.savr_live[4] = 0.0;
        } else if surface_inputs.is_using_chaparral() {
            for i in 0..CHAPARRAL_NUM_FUEL_CLASSES {
                self.savr_dead[i] = self.chaparral_fuel.get_savr(FuelLifeState::Dead, i);
                self.savr_live[i] = self.chaparral_fuel.get_savr(FuelLifeState::Live, i);
            }
        } else {
            let n = self.fuel_model_number;
            let u = SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet;
            self.savr_dead[0] = fuel_models.savr_one_hour(n, u);
            self.savr_dead[1] = 109.0; // Standard 10-hour SAVR
            self.savr_dead[2] = 30.0; // Standard 100-hour SAVR
            self.savr_dead[3] = fuel_models.savr_live_herbaceous(n, u);
            self.savr_dead[4] = 0.0;

            self.savr_live[0] = fuel_models.savr_live_herbaceous(n, u);
            self.savr_live[1] = fuel_models.savr_live_woody(n, u);
            self.savr_live[2] = 0.0;
            self.savr_live[3] = 0.0;
            self.savr_live[4] = 0.0;
        }
    }

    fn dynamic_load_transfer(&mut self) {
        if self.moisture_live[0] < 0.30 {
            self.load_dead[3] = self.load_live[0];
            self.load_live[0] = 0.0;
        } else if self.moisture_live[0] <= 1.20 {
            // BehavePlus-consistent formula
            self.load_dead[3] = self.load_live[0] * (1.333 - 1.11 * self.moisture_live[0]);
            self.load_live[0] -= self.load_dead[3];
        }
    }

    fn set_heat_of_combustion(&mut self, fuel_models: &FuelModels, surface_inputs: &SurfaceInputs) {
        const NUM_LIVE_HC_CLASSES: usize = 3;

        let mut hc_dead = 0.0;
        let mut hc_live = 0.0;

        if surface_inputs.is_using_palmetto_gallberry() {
            hc_dead = self.palmetto_gallberry.get_heat_of_combustion_dead();
            hc_live = self.palmetto_gallberry.get_heat_of_combustion_live();
        } else if surface_inputs.is_using_western_aspen() {
            hc_dead = WesternAspen::get_aspen_heat_of_combustion_dead();
            hc_live = WesternAspen::get_aspen_heat_of_combustion_live();
        } else if surface_inputs.is_using_chaparral() {
            for i in 0..MAX_P {
                self.heat_of_combustion_dead[i] =
                    self.chaparral_fuel.get_heat_of_combustion(FuelLifeState::Dead, i);
                self.heat_of_combustion_live[i] =
                    self.chaparral_fuel.get_heat_of_combustion(FuelLifeState::Live, i);
            }
        } else {
            let n = self.fuel_model_number;
            let u = HeatOfCombustionUnits::BtusPerPound;
            hc_dead = fuel_models.heat_of_combustion_dead(n, u);
            hc_live = fuel_models.heat_of_combustion_live(n, u);
        }

        if !surface_inputs.is_using_chaparral() {
            for i in 0..MAX_P {
                self.heat_of_combustion_dead[i] = hc_dead;
                self.heat_of_combustion_live[i] =
                    if i < NUM_LIVE_HC_CLASSES { hc_live } else { 0.0 };
            }
        }
    }

    fn set_dead_fuel_moisture_of_extinction(
        &mut self,
        fuel_models: &FuelModels,
        surface_inputs: &SurfaceInputs,
    ) {
        if surface_inputs.is_using_palmetto_gallberry() {
            self.moisture_of_extinction[DEAD] =
                self.palmetto_gallberry.get_moisture_of_extinction_dead();
        } else if surface_inputs.is_using_western_aspen() {
            self.moisture_of_extinction[DEAD] =
                WesternAspen::get_aspen_moisture_of_extinction_dead();
        } else if surface_inputs.is_using_chaparral() {
            self.moisture_of_extinction[DEAD] =
                self.chaparral_fuel.get_dead_moisture_of_extinction();
        } else {
            self.moisture_of_extinction[DEAD] = fuel_models
                .moisture_of_extinction_dead(self.fuel_model_number, FractionUnits::Fraction);
        }
    }

    fn calculate_fraction_of_total_surface_area_for_life_states(
        &mut self,
        surface_inputs: &SurfaceInputs,
    ) {
        for life_state in 0..MAX_LIFE {
            if self.number_of_size_classes[life_state] != 0 {
                self.calculate_total_surface_area_for_life_state(life_state, surface_inputs);
                self.calculate_fraction_of_total_surface_area_for_size_classes(life_state);
            }

            let mut summed = [0.0f64; MAX_SAVR];

            if life_state == DEAD {
                Self::sum_fraction_by_size_class(
                    &self.fraction_of_total_surface_area_dead,
                    &self.savr_dead,
                    &mut summed,
                );
                Self::assign_fraction_by_size_class(
                    &self.savr_dead,
                    &summed,
                    &mut self.size_sorted_fraction_dead,
                );
            }
            if life_state == LIVE {
                Self::sum_fraction_by_size_class(
                    &self.fraction_of_total_surface_area_live,
                    &self.savr_live,
                    &mut summed,
                );
                Self::assign_fraction_by_size_class(
                    &self.savr_live,
                    &summed,
                    &mut self.size_sorted_fraction_live,
                );
            }
        }

        let total = self.total_surface_area[DEAD] + self.total_surface_area[LIVE];
        if total > 0.0 {
            self.fraction_of_total_surface_area[DEAD] = self.total_surface_area[DEAD] / total;
            self.fraction_of_total_surface_area[LIVE] =
                1.0 - self.fraction_of_total_surface_area[DEAD];
        }
    }

    fn calculate_total_surface_area_for_life_state(
        &mut self,
        life_state: usize,
        surface_inputs: &SurfaceInputs,
    ) {
        // Reset total surface area for this life state
        self.total_surface_area[life_state] = 0.0;

        if surface_inputs.is_using_palmetto_gallberry() {
            for i in 0..MAX_P {
                self.fuel_density_dead[i] = 30.0;
                self.fuel_density_live[i] = 46.0;
            }
        } else if surface_inputs.is_using_chaparral() {
            for i in 0..MAX_P {
                self.fuel_density_dead[i] =
                    self.chaparral_fuel.get_density(FuelLifeState::Dead, i);
                self.fuel_density_live[i] =
                    self.chaparral_fuel.get_density(FuelLifeState::Live, i);
            }
        }
        // else: default density 32.0 already set in initialize_members

        let n = self.number_of_size_classes[life_state] as usize;
        for i in 0..n {
            if life_state == DEAD {
                if self.fuel_density_dead[i] > 0.0 {
                    self.surface_area_dead[i] =
                        self.load_dead[i] * self.savr_dead[i] / self.fuel_density_dead[i];
                }
                self.total_surface_area[life_state] += self.surface_area_dead[i];
            }
            if life_state == LIVE {
                if self.fuel_density_live[i] > 0.0 {
                    self.surface_area_live[i] =
                        self.load_live[i] * self.savr_live[i] / self.fuel_density_live[i];
                }
                self.total_surface_area[life_state] += self.surface_area_live[i];
            }
        }
    }

    fn calculate_fraction_of_total_surface_area_for_size_classes(&mut self, life_state: usize) {
        let n = self.number_of_size_classes[life_state] as usize;
        for i in 0..n {
            if life_state == DEAD {
                if self.total_surface_area[DEAD] > 1.0e-7 {
                    self.fraction_of_total_surface_area_dead[i] =
                        self.surface_area_dead[i] / self.total_surface_area[DEAD];
                } else {
                    self.fraction_of_total_surface_area_dead[i] = 0.0;
                }
            }
            if life_state == LIVE {
                if self.total_surface_area[LIVE] > 1.0e-7 {
                    self.fraction_of_total_surface_area_live[i] =
                        self.surface_area_live[i] / self.total_surface_area[LIVE];
                } else {
                    self.fraction_of_total_surface_area_live[i] = 0.0;
                }
            }
        }
    }

    fn sum_fraction_by_size_class(
        fraction: &[f64; MAX_P],
        savr: &[f64; MAX_P],
        summed: &mut [f64; MAX_SAVR],
    ) {
        *summed = [0.0; MAX_SAVR];
        for i in 0..MAX_P {
            if savr[i] >= 1200.0 {
                summed[0] += fraction[i];
            } else if savr[i] >= 192.0 {
                summed[1] += fraction[i];
            } else if savr[i] >= 96.0 {
                summed[2] += fraction[i];
            } else if savr[i] >= 48.0 {
                summed[3] += fraction[i];
            } else if savr[i] >= 16.0 {
                summed[4] += fraction[i];
            }
        }
    }

    fn assign_fraction_by_size_class(
        savr: &[f64; MAX_P],
        summed: &[f64; MAX_SAVR],
        dest: &mut [f64; MAX_SAVR],
    ) {
        for i in 0..MAX_P {
            if savr[i] >= 1200.0 {
                dest[i] = summed[0];
            } else if savr[i] >= 192.0 {
                dest[i] = summed[1];
            } else if savr[i] >= 96.0 {
                dest[i] = summed[2];
            } else if savr[i] >= 48.0 {
                dest[i] = summed[3];
            } else if savr[i] >= 16.0 {
                dest[i] = summed[4];
            } else {
                dest[i] = 0.0;
            }
        }
    }

    fn calculate_characteristic_savr(&mut self, surface_inputs: &SurfaceInputs) {
        let mut wn_dead = [0.0f64; MAX_P];
        let mut wn_live = [0.0f64; MAX_P];
        let mut weighted_savr = [0.0f64; MAX_LIFE];

        self.sigma = 0.0;
        for i in 0..MAX_LIFE {
            self.total_load_for_life_state[i] = 0.0;
            self.weighted_heat[i] = 0.0;
            self.weighted_silica[i] = 0.0;
            self.weighted_moisture[i] = 0.0;
            weighted_savr[i] = 0.0;
            self.weighted_fuel_load[i] = 0.0;
        }

        if surface_inputs.is_using_palmetto_gallberry() {
            self.total_silica_content = 0.030;
        } else if surface_inputs.is_using_chaparral() || surface_inputs.is_using_western_aspen() {
            self.total_silica_content = 0.055;
        }

        if surface_inputs.is_using_chaparral() {
            for i in 0..CHAPARRAL_NUM_FUEL_CLASSES {
                self.silica_effective_dead[i] =
                    self.chaparral_fuel.get_effective_silica(FuelLifeState::Dead, i);
                self.silica_effective_live[i] =
                    self.chaparral_fuel.get_effective_silica(FuelLifeState::Live, i);
            }
        }

        let moisture_input_mode = surface_inputs.moisture_input_mode();
        let is_dead_aggregated = matches!(
            moisture_input_mode,
            MoistureInputMode::AllAggregate | MoistureInputMode::DeadAggregateAndLiveSizeClass
        );
        let is_live_aggregated = matches!(
            moisture_input_mode,
            MoistureInputMode::AllAggregate | MoistureInputMode::LiveAggregateAndDeadSizeClass
        );

        if is_dead_aggregated {
            self.weighted_moisture[DEAD] =
                surface_inputs.moisture_dead_aggregate_value(FractionUnits::Fraction);
        }
        if is_live_aggregated {
            self.weighted_moisture[LIVE] =
                surface_inputs.moisture_live_aggregate_value(FractionUnits::Fraction);
        }

        for i in 0..MAX_P {
            if self.savr_dead[i] > 1.0e-07 {
                wn_dead[i] = self.load_dead[i] * (1.0 - self.total_silica_content);
                self.weighted_heat[DEAD] +=
                    self.fraction_of_total_surface_area_dead[i] * self.heat_of_combustion_dead[i];
                self.weighted_silica[DEAD] +=
                    self.fraction_of_total_surface_area_dead[i] * self.silica_effective_dead[i];
                if !is_dead_aggregated {
                    self.weighted_moisture[DEAD] +=
                        self.fraction_of_total_surface_area_dead[i] * self.moisture_dead[i];
                }
                weighted_savr[DEAD] +=
                    self.fraction_of_total_surface_area_dead[i] * self.savr_dead[i];
                self.total_load_for_life_state[DEAD] += self.load_dead[i];
            }
            if self.savr_live[i] > 1.0e-07 {
                wn_live[i] = self.load_live[i] * (1.0 - self.total_silica_content);
                self.weighted_heat[LIVE] +=
                    self.fraction_of_total_surface_area_live[i] * self.heat_of_combustion_live[i];
                self.weighted_silica[LIVE] +=
                    self.fraction_of_total_surface_area_live[i] * self.silica_effective_live[i];
                if !is_live_aggregated {
                    self.weighted_moisture[LIVE] +=
                        self.fraction_of_total_surface_area_live[i] * self.moisture_live[i];
                }
                weighted_savr[LIVE] +=
                    self.fraction_of_total_surface_area_live[i] * self.savr_live[i];
                self.total_load_for_life_state[LIVE] += self.load_live[i];
            }

            self.weighted_fuel_load[DEAD] +=
                self.size_sorted_fraction_dead[i] * wn_dead[i];
            self.weighted_fuel_load[LIVE] +=
                self.size_sorted_fraction_live[i] * wn_live[i];
        }

        for life_state in 0..MAX_LIFE {
            self.sigma += self.fraction_of_total_surface_area[life_state] * weighted_savr[life_state];
        }
    }

    fn calculate_live_moisture_of_extinction(&mut self) {
        if self.number_of_size_classes[LIVE] == 0 {
            return;
        }

        let mut fine_dead = 0.0;
        let mut fine_live = 0.0;
        let mut weighted_moisture_fine_dead = 0.0;

        for i in 0..MAX_P {
            if self.savr_dead[i] > 1.0e-7 {
                let w = self.load_dead[i] * (-138.0 / self.savr_dead[i]).exp();
                fine_dead += w;
                weighted_moisture_fine_dead += w * self.moisture_dead[i];
            }
        }

        let fine_dead_moisture = if fine_dead > 1.0e-07 {
            weighted_moisture_fine_dead / fine_dead
        } else {
            0.0
        };

        let n_live = self.number_of_size_classes[LIVE] as usize;
        for i in 0..n_live {
            if self.savr_live[i] > 1.0e-07 {
                fine_live += self.load_live[i] * (-500.0 / self.savr_live[i]).exp();
            }
        }

        let fine_dead_over_fine_live = if fine_live > 1.0e-7 {
            fine_dead / fine_live
        } else {
            0.0
        };

        self.moisture_of_extinction[LIVE] = (2.9 * fine_dead_over_fine_live
            * (1.0 - fine_dead_moisture / self.moisture_of_extinction[DEAD]))
            - 0.226;

        if self.moisture_of_extinction[LIVE] < self.moisture_of_extinction[DEAD] {
            self.moisture_of_extinction[LIVE] = self.moisture_of_extinction[DEAD];
        }
    }

    fn calculate_heat_sink(&mut self) {
        self.heat_sink = 0.0;

        for i in 0..MAX_P {
            if self.savr_dead[i] > 1.0e-07 {
                let qig = 250.0 + 1116.0 * self.moisture_dead[i];
                self.heat_sink += self.fraction_of_total_surface_area[DEAD]
                    * self.fraction_of_total_surface_area_dead[i]
                    * qig
                    * (-138.0 / self.savr_dead[i]).exp();
            }
            if self.savr_live[i] > 1.0e-07 {
                let qig = 250.0 + 1116.0 * self.moisture_live[i];
                self.heat_sink += self.fraction_of_total_surface_area[LIVE]
                    * self.fraction_of_total_surface_area_live[i]
                    * qig
                    * (-138.0 / self.savr_live[i]).exp();
            }
        }
        self.heat_sink *= self.bulk_density;
    }

    fn calculate_propagating_flux(&mut self) {
        if self.sigma < 1.0e-07 {
            self.propagating_flux = 0.0;
        } else {
            self.propagating_flux = ((0.792 + 0.681 * self.sigma.sqrt())
                * (self.packing_ratio + 0.1))
                .exp()
                / (192.0 + 0.2595 * self.sigma);
        }
    }

    /// C++ method: `calculateWesternAspenMortality`
    pub fn calculate_western_aspen_mortality(
        &mut self,
        surface_inputs: &SurfaceInputs,
        flame_length: f64,
    ) {
        self.western_aspen.calculate_mortality(
            surface_inputs.aspen_fire_severity(),
            flame_length,
            surface_inputs.aspen_dbh(LengthUnits::Inches),
        );
    }

    /// C++ method: `getAspenMortality` (fraction)
    pub fn aspen_mortality(&self) -> f64 {
        self.western_aspen.get_aspen_mortality()
    }
}

impl Default for FuelbedIntermediates {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience index constants
const DEAD: usize = FuelLifeState::Dead as usize;
const LIVE: usize = FuelLifeState::Live as usize;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuel_models::FuelModels;
    use crate::inputs::SurfaceInputs;
    use firelab_base::{
        FractionUnits, LengthUnits, SlopeUnits, SpeedUnits,
        WindAndSpreadOrientationMode, WindHeightInputMode,
    };

    /// Helper: create inputs for a standard fuel model run.
    fn make_inputs(
        fuel_model: i32,
        m1h: f64,
        m10h: f64,
        m100h: f64,
        m_herb: f64,
        m_woody: f64,
    ) -> SurfaceInputs {
        let mut inputs = SurfaceInputs::new();
        inputs.update_surface_inputs(
            fuel_model,
            m1h,
            m10h,
            m100h,
            m_herb,
            m_woody,
            FractionUnits::Fraction,
            0.0,
            SpeedUnits::FeetPerMinute,
            WindHeightInputMode::DirectMidflame,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            0.0,
            SlopeUnits::Degrees,
            0.0,
            0.0,
            FractionUnits::Fraction,
            0.0,
            LengthUnits::Feet,
            0.0,
            FractionUnits::Fraction,
        );
        inputs
    }

    #[test]
    fn fm1_depth() {
        let fm = FuelModels::new();
        let inputs = make_inputs(1, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(1, &fm, &inputs);
        // FM1 depth = 1.0 ft
        assert!((fb.fuelbed_depth() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn fm1_basic_intermediates() {
        let fm = FuelModels::new();
        let inputs = make_inputs(1, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(1, &fm, &inputs);

        // FM1: short grass, depth=1.0, load_1h=0.034 (0.74 tons/acre), no 10h/100h, herb=0, woody=0
        // sigma should be dominated by 1h SAVR of 3500
        assert!(fb.sigma() > 3000.0, "sigma={}", fb.sigma());
        assert!(fb.bulk_density() > 0.0);
        assert!(fb.packing_ratio() > 0.0);
        assert!(fb.relative_packing_ratio() > 0.0);
        assert!(fb.propagating_flux() > 0.0);
        assert!(fb.heat_sink() > 0.0);
    }

    #[test]
    fn fm10_has_all_three_dead_classes() {
        let fm = FuelModels::new();
        let inputs = make_inputs(10, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(10, &fm, &inputs);

        // FM10 has 1h, 10h, 100h, and live woody
        let dead_load = fb.total_dead_fuel_load(LoadingUnits::PoundsPerSquareFoot);
        let live_load = fb.total_live_fuel_load(LoadingUnits::PoundsPerSquareFoot);
        assert!(dead_load > 0.0, "dead_load={}", dead_load);
        assert!(live_load > 0.0, "live_load={}", live_load);
        assert!(fb.sigma() > 0.0);
    }

    #[test]
    fn fm4_chaparral_depth() {
        let fm = FuelModels::new();
        let inputs = make_inputs(4, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(4, &fm, &inputs);

        // FM4 depth = 6.0 ft
        assert!((fb.fuelbed_depth() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn dead_moisture_of_extinction_fm1() {
        let fm = FuelModels::new();
        let inputs = make_inputs(1, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(1, &fm, &inputs);

        // FM1 dead moisture of extinction = 0.12 (12%)
        let mox_dead = fb.moisture_of_extinction_by_life_state(FuelLifeState::Dead);
        assert!(
            (mox_dead - 0.12).abs() < 1e-10,
            "mox_dead={}",
            mox_dead
        );
    }

    #[test]
    fn weighted_moisture_fm1() {
        let fm = FuelModels::new();
        let inputs = make_inputs(1, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(1, &fm, &inputs);

        // FM1 has only 1h dead fuel (0.034 lb/ft²), so weighted dead moisture ~ 0.06
        let wm_dead = fb.weighted_moisture_by_life_state(FuelLifeState::Dead);
        assert!(
            (wm_dead - 0.06).abs() < 0.01,
            "weighted_dead_moisture={}",
            wm_dead
        );
    }

    #[test]
    fn dynamic_fuel_model_gr1() {
        let fm = FuelModels::new();
        // GR1 is dynamic — test that load transfer works
        // With low live herbaceous moisture (< 0.30), all herb load transfers to dead
        let inputs = make_inputs(101, 0.06, 0.07, 0.08, 0.20, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(101, &fm, &inputs);

        // After dynamic transfer with moisture_live[0]=0.20 < 0.30:
        // All live herb load should be transferred to dead[3]
        // The dead herbaceous load slot should have the transferred load
        let dead_herb = fb.total_dead_herbaceous_fuel_load(LoadingUnits::PoundsPerSquareFoot);
        assert!(
            dead_herb > 0.0,
            "Expected dynamic load transfer, got dead_herb={}",
            dead_herb
        );
    }

    #[test]
    fn dynamic_fuel_model_partial_transfer() {
        let fm = FuelModels::new();
        // GR1 with live herb moisture in the partial transfer range (0.30..1.20)
        let inputs = make_inputs(101, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(101, &fm, &inputs);

        let dead_herb = fb.total_dead_herbaceous_fuel_load(LoadingUnits::PoundsPerSquareFoot);
        let live_load = fb.total_live_fuel_load(LoadingUnits::PoundsPerSquareFoot);
        // Partial transfer: some herb moved to dead
        assert!(dead_herb > 0.0, "dead_herb={}", dead_herb);
        assert!(live_load > 0.0, "live_load should still have some remaining");
    }

    #[test]
    fn propagating_flux_nonzero_for_fm10() {
        let fm = FuelModels::new();
        let inputs = make_inputs(10, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(10, &fm, &inputs);

        assert!(fb.propagating_flux() > 0.0);
        assert!(fb.relative_packing_ratio() > 0.0);
    }

    #[test]
    fn live_moisture_of_extinction_for_fm10() {
        let fm = FuelModels::new();
        let inputs = make_inputs(10, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(10, &fm, &inputs);

        // FM10 has live fuels, so live moisture of extinction should be computed
        let mox_live = fb.moisture_of_extinction_by_life_state(FuelLifeState::Live);
        assert!(mox_live > 0.0, "mox_live={}", mox_live);
    }
}
