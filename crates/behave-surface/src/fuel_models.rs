//! Standard and custom fuel model definitions.
//!
//! Contains the 13 original fuel models (FM1–FM13), 9 non-burnable (NB1–NB9),
//! 40 Scott & Burgan models (GR, GS, SH, TU, TL, SB series), and additional
//! international models (V-*, SCAL*, M-*, F-*), plus support for user-defined
//! custom models.
//!
//! All values are stored internally in base units:
//! - Fuel bed depth: feet
//! - Moisture of extinction: fraction (0–1)
//! - Heat of combustion: Btu/lb
//! - Fuel loading: lb/ft²
//! - Surface-area-to-volume ratio: ft²/ft³
//!
//! C++ source: fuelModels.h / fuelModels.cpp

use firelab_base::{
    FractionUnits, HeatOfCombustionUnits, LengthUnits, LoadingUnits,
    SurfaceAreaToVolumeUnits, UnitConversion,
};

/// Maximum number of fuel model slots (indices 0..255).
const MAX_FUEL_MODELS: usize = 256;

/// Conversion factor: tons/acre → lb/ft² (2000 / 43560).
const F: f64 = 2000.0 / 43560.0;

/// A single fuel model record with all particle properties.
///
/// C++ struct: `FuelModelRecord` (nested in FuelModels)
#[derive(Debug, Clone)]
pub struct FuelModelRecord {
    pub number: i32,
    pub code: String,
    pub name: String,
    pub fuelbed_depth: f64,
    pub moisture_of_extinction_dead: f64,
    pub heat_of_combustion_dead: f64,
    pub heat_of_combustion_live: f64,
    pub fuel_load_one_hour: f64,
    pub fuel_load_ten_hour: f64,
    pub fuel_load_hundred_hour: f64,
    pub fuel_load_live_herbaceous: f64,
    pub fuel_load_live_woody: f64,
    pub savr_one_hour: f64,
    pub savr_live_herbaceous: f64,
    pub savr_live_woody: f64,
    pub is_dynamic: bool,
    pub is_reserved: bool,
    pub is_defined: bool,
}

impl Default for FuelModelRecord {
    fn default() -> Self {
        Self {
            number: 0,
            code: "NO_CODE".to_string(),
            name: "NO_NAME".to_string(),
            fuelbed_depth: 0.0,
            moisture_of_extinction_dead: 0.0,
            heat_of_combustion_dead: 0.0,
            heat_of_combustion_live: 0.0,
            fuel_load_one_hour: 0.0,
            fuel_load_ten_hour: 0.0,
            fuel_load_hundred_hour: 0.0,
            fuel_load_live_herbaceous: 0.0,
            fuel_load_live_woody: 0.0,
            savr_one_hour: 0.0,
            savr_live_herbaceous: 0.0,
            savr_live_woody: 0.0,
            is_dynamic: false,
            is_reserved: false,
            is_defined: false,
        }
    }
}

/// Catalog of all available fuel models (256 slots, indexed by model number).
///
/// C++ class: `FuelModels`
#[derive(Debug, Clone)]
pub struct FuelModels {
    records: Vec<FuelModelRecord>,
}

impl FuelModels {
    pub fn new() -> Self {
        let mut catalog = Self {
            records: (0..MAX_FUEL_MODELS).map(|_| FuelModelRecord::default()).collect(),
        };
        catalog.populate_fuel_models();
        catalog
    }

    // -----------------------------------------------------------------------
    // Getters (with unit conversion from base)
    // -----------------------------------------------------------------------

    pub fn fuel_code(&self, number: i32) -> &str {
        &self.records[number as usize].code
    }

    pub fn fuel_name(&self, number: i32) -> &str {
        &self.records[number as usize].name
    }

    pub fn fuelbed_depth(&self, number: i32, units: LengthUnits) -> f64 {
        units.from_base(self.records[number as usize].fuelbed_depth)
    }

    pub fn moisture_of_extinction_dead(&self, number: i32, units: FractionUnits) -> f64 {
        units.from_base(self.records[number as usize].moisture_of_extinction_dead)
    }

    pub fn heat_of_combustion_dead(&self, number: i32, units: HeatOfCombustionUnits) -> f64 {
        units.from_base(self.records[number as usize].heat_of_combustion_dead)
    }

    pub fn heat_of_combustion_live(&self, number: i32, units: HeatOfCombustionUnits) -> f64 {
        units.from_base(self.records[number as usize].heat_of_combustion_live)
    }

    pub fn fuel_load_one_hour(&self, number: i32, units: LoadingUnits) -> f64 {
        units.from_base(self.records[number as usize].fuel_load_one_hour)
    }

    pub fn fuel_load_ten_hour(&self, number: i32, units: LoadingUnits) -> f64 {
        units.from_base(self.records[number as usize].fuel_load_ten_hour)
    }

    pub fn fuel_load_hundred_hour(&self, number: i32, units: LoadingUnits) -> f64 {
        units.from_base(self.records[number as usize].fuel_load_hundred_hour)
    }

    pub fn fuel_load_live_herbaceous(&self, number: i32, units: LoadingUnits) -> f64 {
        units.from_base(self.records[number as usize].fuel_load_live_herbaceous)
    }

    pub fn fuel_load_live_woody(&self, number: i32, units: LoadingUnits) -> f64 {
        units.from_base(self.records[number as usize].fuel_load_live_woody)
    }

    pub fn savr_one_hour(&self, number: i32, units: SurfaceAreaToVolumeUnits) -> f64 {
        units.from_base(self.records[number as usize].savr_one_hour)
    }

    pub fn savr_live_herbaceous(&self, number: i32, units: SurfaceAreaToVolumeUnits) -> f64 {
        units.from_base(self.records[number as usize].savr_live_herbaceous)
    }

    pub fn savr_live_woody(&self, number: i32, units: SurfaceAreaToVolumeUnits) -> f64 {
        units.from_base(self.records[number as usize].savr_live_woody)
    }

    pub fn is_dynamic(&self, number: i32) -> bool {
        if number <= 0 || number > 256 {
            return false;
        }
        self.records[number as usize].is_dynamic
    }

    pub fn is_fuel_model_defined(&self, number: i32) -> bool {
        if number <= 0 || number > 256 {
            return false;
        }
        self.records[number as usize].is_defined
    }

    pub fn is_fuel_model_reserved(&self, number: i32) -> bool {
        if number <= 0 || number > 256 {
            return false;
        }
        self.records[number as usize].is_reserved
    }

    pub fn is_all_fuel_load_zero(&self, number: i32) -> bool {
        let r = &self.records[number as usize];
        r.fuel_load_one_hour == 0.0
            && r.fuel_load_ten_hour == 0.0
            && r.fuel_load_hundred_hour == 0.0
            && r.fuel_load_live_herbaceous == 0.0
            && r.fuel_load_live_woody == 0.0
    }

    /// Direct access to a fuel model record by number.
    pub fn record(&self, number: i32) -> &FuelModelRecord {
        &self.records[number as usize]
    }

    // -----------------------------------------------------------------------
    // Custom fuel model support
    // -----------------------------------------------------------------------

    /// Set a custom fuel model. Returns false if the slot is reserved.
    ///
    /// All input values are converted to base units before storage.
    pub fn set_custom_fuel_model(
        &mut self,
        number: i32,
        code: &str,
        name: &str,
        fuelbed_depth: f64,
        length_units: LengthUnits,
        moisture_of_extinction_dead: f64,
        moisture_units: FractionUnits,
        heat_of_combustion_dead: f64,
        heat_of_combustion_live: f64,
        heat_units: HeatOfCombustionUnits,
        fuel_load_one_hour: f64,
        fuel_load_ten_hour: f64,
        fuel_load_hundred_hour: f64,
        fuel_load_live_herbaceous: f64,
        fuel_load_live_woody: f64,
        loading_units: LoadingUnits,
        savr_one_hour: f64,
        savr_live_herbaceous: f64,
        savr_live_woody: f64,
        savr_units: SurfaceAreaToVolumeUnits,
        is_dynamic: bool,
    ) -> bool {
        if self.records[number as usize].is_reserved {
            return false;
        }

        let depth = length_units.to_base(fuelbed_depth);
        let moe = moisture_units.to_base(moisture_of_extinction_dead);
        let hoc_dead = heat_units.to_base(heat_of_combustion_dead);
        let hoc_live = heat_units.to_base(heat_of_combustion_live);

        let l1h = loading_units.to_base(fuel_load_one_hour);
        let l10h = loading_units.to_base(fuel_load_ten_hour);
        let l100h = loading_units.to_base(fuel_load_hundred_hour);
        let llh = loading_units.to_base(fuel_load_live_herbaceous);
        let llw = loading_units.to_base(fuel_load_live_woody);

        let s1h = savr_units.to_base(savr_one_hour);
        let slh = savr_units.to_base(savr_live_herbaceous);
        let slw = savr_units.to_base(savr_live_woody);

        // C++ truncates code to 3 chars
        let code_trunc: String = code.chars().take(3).collect();

        self.set_record(
            number, &code_trunc, name, depth, moe, hoc_dead, hoc_live,
            l1h, l10h, l100h, llh, llw, s1h, slh, slw,
            is_dynamic, false,
        );
        true
    }

    /// Clear a custom fuel model. Returns false if the slot is reserved.
    pub fn clear_custom_fuel_model(&mut self, number: i32) -> bool {
        if self.records[number as usize].is_reserved {
            return false;
        }
        self.records[number as usize] = FuelModelRecord::default();
        true
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn set_record(
        &mut self,
        number: i32,
        code: &str,
        name: &str,
        fuelbed_depth: f64,
        moisture_of_extinction_dead: f64,
        heat_of_combustion_dead: f64,
        heat_of_combustion_live: f64,
        fuel_load_one_hour: f64,
        fuel_load_ten_hour: f64,
        fuel_load_hundred_hour: f64,
        fuel_load_live_herbaceous: f64,
        fuel_load_live_woody: f64,
        savr_one_hour: f64,
        savr_live_herbaceous: f64,
        savr_live_woody: f64,
        is_dynamic: bool,
        is_reserved: bool,
    ) {
        let r = &mut self.records[number as usize];
        r.number = number;
        r.code = code.to_string();
        r.name = name.to_string();
        r.fuelbed_depth = fuelbed_depth;
        r.moisture_of_extinction_dead = moisture_of_extinction_dead;
        r.heat_of_combustion_dead = heat_of_combustion_dead;
        r.heat_of_combustion_live = heat_of_combustion_live;
        r.fuel_load_one_hour = fuel_load_one_hour;
        r.fuel_load_ten_hour = fuel_load_ten_hour;
        r.fuel_load_hundred_hour = fuel_load_hundred_hour;
        r.fuel_load_live_herbaceous = fuel_load_live_herbaceous;
        r.fuel_load_live_woody = fuel_load_live_woody;
        r.savr_one_hour = savr_one_hour;
        r.savr_live_herbaceous = savr_live_herbaceous;
        r.savr_live_woody = savr_live_woody;
        r.is_dynamic = is_dynamic;
        r.is_reserved = is_reserved;
        r.is_defined = true;
    }

    fn mark_as_reserved(&mut self, number: i32) {
        self.records[number as usize].is_reserved = true;
    }

    fn populate_fuel_models(&mut self) {
        // Index 0 — unused sentinel
        self.set_record(0, "NO_CODE", "NO_NAME", 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, false, false);
        self.records[0].is_defined = false;

        // =================================================================
        // Original 13 Fuel Models (FM1–FM13)
        // Values already in base units (ft, fraction, Btu/lb, lb/ft², ft²/ft³)
        // =================================================================

        self.set_record(1, "FM1", "Short grass [1]",
            1.0, 0.12, 8000.0, 8000.0,
            0.034, 0.0, 0.0, 0.0, 0.0,
            3500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(2, "FM2", "Timber grass and understory [2]",
            1.0, 0.15, 8000.0, 8000.0,
            0.092, 0.046, 0.023, 0.023, 0.0,
            3000.0, 1500.0, 1500.0,
            false, true);

        self.set_record(3, "FM3", "Tall grass [3]",
            2.5, 0.25, 8000.0, 8000.0,
            0.138, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(4, "FM4", "Chaparral [4]",
            6.0, 0.2, 8000.0, 8000.0,
            0.230, 0.184, 0.092, 0.0, 0.230,
            2000.0, 1500.0, 1500.0,
            false, true);

        self.set_record(5, "FM5", "Brush [5]",
            2.0, 0.20, 8000.0, 8000.0,
            0.046, 0.023, 0.0, 0.0, 0.092,
            2000.0, 1500.0, 1500.0,
            false, true);

        self.set_record(6, "FM6", "Dormant brush, hardwood slash [6]",
            2.5, 0.25, 8000.0, 8000.0,
            0.069, 0.115, 0.092, 0.0, 0.0,
            1750.0, 1500.0, 1500.0,
            false, true);

        self.set_record(7, "FM7", "Southern rough [7]",
            2.5, 0.40, 8000.0, 8000.0,
            0.052, 0.086, 0.069, 0.0, 0.017,
            1750.0, 1500.0, 1500.0,
            false, true);

        self.set_record(8, "FM8", "Short needle litter [8]",
            0.2, 0.3, 8000.0, 8000.0,
            0.069, 0.046, 0.115, 0.0, 0.0,
            2000.0, 1500.0, 1500.0,
            false, true);

        self.set_record(9, "FM9", "Long needle or hardwood litter [9]",
            0.2, 0.25, 8000.0, 8000.0,
            0.134, 0.019, 0.007, 0.0, 0.0,
            2500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(10, "FM10", "Timber litter & understory [10]",
            1.0, 0.25, 8000.0, 8000.0,
            0.138, 0.092, 0.230, 0.0, 0.092,
            2000.0, 1500.0, 1500.0,
            false, true);

        self.set_record(11, "FM11", "Light logging slash [11]",
            1.0, 0.15, 8000.0, 8000.0,
            0.069, 0.207, 0.253, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(12, "FM12", "Medium logging slash [12]",
            2.3, 0.20, 8000.0, 8000.0,
            0.184, 0.644, 0.759, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(13, "FM13", "Heavy logging slash [13]",
            3.0, 0.25, 8000.0, 8000.0,
            0.322, 1.058, 1.288, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        // 14–89: Available for custom models (no records set)

        // =================================================================
        // Non-Burnable Models (NB1–NB9)
        // =================================================================

        self.set_record(91, "NB1", "Urban, developed [91]",
            1.0, 0.10, 8000.0, 8000.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(92, "NB2", "Snow, ice [92]",
            1.0, 0.10, 8000.0, 8000.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(93, "NB3", "Agricultural [93]",
            1.0, 0.10, 8000.0, 8000.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        // 94–95: Reserved for future NB models
        for i in 94..=95 {
            self.mark_as_reserved(i);
        }

        self.set_record(94, "NB4", "Future standard non-burnable [94]",
            1.0, 0.10, 8000.0, 8000.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(95, "NB5", "Future standard non-burnable [95]",
            1.0, 0.10, 8000.0, 8000.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        // 96–97: Available for custom NB models

        self.set_record(98, "NB8", "Open water [98]",
            1.0, 0.10, 8000.0, 8000.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        self.set_record(99, "NB9", "Bare ground [99]",
            1.0, 0.10, 8000.0, 8000.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
            1500.0, 1500.0, 1500.0,
            false, true);

        // =================================================================
        // Scott & Burgan: Grass (GR1–GR9)
        // Fuel loads in tons/acre, converted with F = 2000/43560
        // =================================================================

        self.set_record(101, "GR1", "Short, sparse, dry climate grass (D)",
            0.4, 0.15, 8000.0, 8000.0,
            0.10*F, 0.0, 0.0, 0.30*F, 0.0,
            2200.0, 2000.0, 1500.0,
            true, true);

        self.set_record(102, "GR2", "Low load, dry climate grass (D)",
            1.0, 0.15, 8000.0, 8000.0,
            0.10*F, 0.0, 0.0, 1.0*F, 0.0,
            2000.0, 1800.0, 1500.0,
            true, true);

        self.set_record(103, "GR3", "Low load, very coarse, humid climate grass (D)",
            2.0, 0.30, 8000.0, 8000.0,
            0.10*F, 0.40*F, 0.0, 1.50*F, 0.0,
            1500.0, 1300.0, 1500.0,
            true, true);

        self.set_record(104, "GR4", "Moderate load, dry climate grass (D)",
            2.0, 0.15, 8000.0, 8000.0,
            0.25*F, 0.0, 0.0, 1.9*F, 0.0,
            2000.0, 1800.0, 1500.0,
            true, true);

        self.set_record(105, "GR5", "Low load, humid climate grass (D)",
            1.5, 0.40, 8000.0, 8000.0,
            0.40*F, 0.0, 0.0, 2.50*F, 0.0,
            1800.0, 1600.0, 1500.0,
            true, true);

        self.set_record(106, "GR6", "Moderate load, humid climate grass (D)",
            1.5, 0.40, 9000.0, 9000.0,
            0.10*F, 0.0, 0.0, 3.4*F, 0.0,
            2200.0, 2000.0, 1500.0,
            true, true);

        self.set_record(107, "GR7", "High load, dry climate grass (D)",
            3.0, 0.15, 8000.0, 8000.0,
            1.0*F, 0.0, 0.0, 5.4*F, 0.0,
            2000.0, 1800.0, 1500.0,
            true, true);

        self.set_record(108, "GR8", "High load, very coarse, humid climate grass (D)",
            4.0, 0.30, 8000.0, 8000.0,
            0.5*F, 1.0*F, 0.0, 7.3*F, 0.0,
            1500.0, 1300.0, 1500.0,
            true, true);

        self.set_record(109, "GR9", "Very high load, humid climate grass (D)",
            5.0, 0.40, 8000.0, 8000.0,
            1.0*F, 1.0*F, 0.0, 9.0*F, 0.0,
            1800.0, 1600.0, 1500.0,
            true, true);

        // International grass models (V-Hb, V-Ha)
        self.set_record(110, "V-Hb", "Short Gass, < 0.5 m (Dynamic)",
            0.35, 24.0, 19000.0, 19000.0,
            0.3*F, 0.0, 0.0, 1.2*F, 0.0,
            6000.0, 6000.0, 6000.0,
            true, true);

        self.set_record(111, "V-Ha", "Tall Grass, > 0.5 m (Dynamic)",
            0.6, 24.0, 19000.0, 19000.0,
            0.5*F, 0.1*F, 0.0, 2.5*F, 0.3*F,
            4000.0, 6000.0, 4000.0,
            true, true);

        // 112: Reserved for future standard grass models
        self.mark_as_reserved(112);

        // 113–119: Available for custom grass models

        // =================================================================
        // Grass-Shrub (GS1–GS4)
        // =================================================================

        self.set_record(121, "GS1", "Low load, dry climate grass-shrub (D)",
            0.9, 0.15, 8000.0, 8000.0,
            0.2*F, 0.0, 0.0, 0.5*F, 0.65*F,
            2000.0, 1800.0, 1800.0,
            true, true);

        self.set_record(122, "GS2", "Moderate load, dry climate grass-shrub (D)",
            1.5, 0.15, 8000.0, 8000.0,
            0.5*F, 0.5*F, 0.0, 0.6*F, 1.0*F,
            2000.0, 1800.0, 1800.0,
            true, true);

        self.set_record(123, "GS3", "Moderate load, humid climate grass-shrub (D)",
            1.8, 0.40, 8000.0, 8000.0,
            0.3*F, 0.25*F, 0.0, 1.45*F, 1.25*F,
            1800.0, 1600.0, 1600.0,
            true, true);

        self.set_record(124, "GS4", "High load, humid climate grass-shrub (D)",
            2.1, 0.40, 8000.0, 8000.0,
            1.9*F, 0.3*F, 0.1*F, 3.4*F, 7.1*F,
            1800.0, 1600.0, 1600.0,
            true, true);

        // 125–130: Reserved for future GS models
        for i in 125..=130 {
            self.mark_as_reserved(i);
        }

        // 131–139: Available for custom GS models

        // =================================================================
        // Shrub (SH1–SH9)
        // =================================================================

        self.set_record(141, "SH1", "Low load, dry climate shrub (D)",
            1.0, 0.15, 8000.0, 8000.0,
            0.25*F, 0.25*F, 0.0, 0.15*F, 1.3*F,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(142, "SH2", "Moderate load, dry climate shrub (S)",
            1.0, 0.15, 8000.0, 8000.0,
            1.35*F, 2.4*F, 0.75*F, 0.0, 3.85*F,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(143, "SH3", "Moderate load, humid climate shrub (S)",
            2.4, 0.40, 8000.0, 8000.0,
            0.45*F, 3.0*F, 0.0, 0.0, 6.2*F,
            1600.0, 1800.0, 1400.0,
            true, true);

        self.set_record(144, "SH4", "Low load, humid climate timber-shrub (S)",
            3.0, 0.30, 8000.0, 8000.0,
            0.85*F, 1.15*F, 0.2*F, 0.0, 2.55*F,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(145, "SH5", "High load, dry climate shrub (S)",
            6.0, 0.15, 8000.0, 8000.0,
            3.6*F, 2.1*F, 0.0, 0.0, 2.9*F,
            750.0, 1800.0, 1600.0,
            true, true);

        self.set_record(146, "SH6", "Low load, humid climate shrub (S)",
            2.0, 0.30, 8000.0, 8000.0,
            2.9*F, 1.45*F, 0.0, 0.0, 1.4*F,
            750.0, 1800.0, 1600.0,
            true, true);

        self.set_record(147, "SH7", "Very high load, dry climate shrub (S)",
            6.0, 0.15, 8000.0, 8000.0,
            3.5*F, 5.3*F, 2.2*F, 0.0, 3.4*F,
            750.0, 1800.0, 1600.0,
            true, true);

        self.set_record(148, "SH8", "High load, humid climate shrub (S)",
            3.0, 0.40, 8000.0, 8000.0,
            2.05*F, 3.4*F, 0.85*F, 0.0, 4.35*F,
            750.0, 1800.0, 1600.0,
            true, true);

        self.set_record(149, "SH9", "Very high load, humid climate shrub (D)",
            4.4, 0.40, 8000.0, 8000.0,
            4.5*F, 2.45*F, 0.0, 1.55*F, 7.0*F,
            750.0, 1800.0, 1500.0,
            true, true);

        // SCAL models (Standard California Shrub)
        self.set_record(150, "SCAL17",
            "Chamise with Moderate Load Grass, 4 feet (Static)",
            4.0, 20.0, 8000.0, 8000.0,
            1.3*F, 1.0*F, 1.0*F, 2.0*F, 2.0*F,
            640.0, 2200.0, 640.0,
            false, true);

        self.set_record(151, "SCAL15",
            "Chamise with Low Load Grass, 3 feet (Static)",
            3.0, 13.0, 10000.0, 10000.0,
            2.0*F, 3.0*F, 1.0*F, 0.5*F, 2.0*F,
            640.0, 2200.0, 640.0,
            false, true);

        self.set_record(152, "SCAL16",
            "North Slope Ceanothus with Moderate Load Grass (Static)",
            6.0, 15.0, 8000.0, 8000.0,
            2.2*F, 4.8*F, 1.8*F, 3.0*F, 2.8*F,
            500.0, 1500.0, 500.0,
            false, true);

        self.set_record(153, "SCAL14",
            "Manzanita/Scrub Oak with Low Load Grass (Static)",
            3.0, 15.0, 9211.0, 9211.0,
            3.0*F, 4.5*F, 1.1*F, 1.4*F, 5.0*F,
            350.0, 1500.0, 250.0,
            false, true);

        self.set_record(154, "SCAL18",
            "Coastal Sage/Buckwheat Scrub with Low Load Grass (Static)",
            3.0, 25.0, 9200.0, 9200.0,
            5.5*F, 0.8*F, 0.1*F, 0.75*F, 2.5*F,
            640.0, 1500.0, 640.0,
            false, true);

        // International shrub models (V-*)
        self.set_record(155, "V-MH",
            "Short Green Shrub < 1 m With Grass, Discontinuous (< 1 m) often discontinuous and with grass (Dynamic)",
            0.55, 25.0, 19500.0, 19500.0,
            1.0*F, 1.0*F, 0.0, 1.5*F, 5.5*F,
            4500.0, 8500.0, 4000.0,
            true, true);

        self.set_record(156, "V-MMb",
            "Short Shrub < 1 m, Low Dead Fraction and/or Thick Foliage (Static)",
            0.9, 20.0, 20500.0, 20500.0,
            4.0*F, 0.5*F, 0.0, 0.0, 7.0*F,
            3000.0, 3000.0, 3000.0,
            false, true);

        self.set_record(157, "V-MAb",
            "Short Shrub < 1 m, High Dead Fraction and/or Thin Fuel (Static)",
            0.5, 35.0, 21000.0, 21000.0,
            6.0*F, 0.5*F, 0.0, 0.0, 7.5*F,
            4500.0, 4500.0, 4500.0,
            false, true);

        self.set_record(158, "V-MMa",
            "Tall Shrub > 1 m, Low Dead Fraction and/or Thick Foliage (Static)",
            1.7, 24.0, 20500.0, 20500.0,
            6.0*F, 4.0*F, 0.0, 0.0, 13.0*F,
            2500.0, 3000.0, 3000.0,
            false, true);

        self.set_record(159, "V-MAa",
            "Tall Shrub > 1 m, High Dead Fraction and/or Thin Fuel (Static)",
            1.05, 35.0, 21000.0, 21000.0,
            9.5*F, 2.5*F, 0.0, 0.0, 14.5*F,
            3500.0, 4000.0, 4000.0,
            false, true);

        // =================================================================
        // Timber-Understory (TU1–TU5)
        // =================================================================

        self.set_record(161, "TU1", "Light load, dry climate timber-grass-shrub (D)",
            0.6, 0.20, 8000.0, 8000.0,
            0.2*F, 0.9*F, 1.5*F, 0.2*F, 0.9*F,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(162, "TU2", "Moderate load, humid climate timber-shrub (S)",
            1.0, 0.30, 8000.0, 8000.0,
            0.95*F, 1.8*F, 1.25*F, 0.0, 0.2*F,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(163, "TU3", "Moderate load, humid climate timber-grass-shrub (D)",
            1.3, 0.30, 8000.0, 8000.0,
            1.1*F, 0.15*F, 0.25*F, 0.65*F, 1.1*F,
            1800.0, 1600.0, 1400.0,
            true, true);

        self.set_record(164, "TU4", "Dwarf conifer understory (S)",
            0.5, 0.12, 8000.0, 8000.0,
            4.5*F, 0.0, 0.0, 0.0, 2.0*F,
            2300.0, 1800.0, 2000.0,
            true, true);

        self.set_record(165, "TU5", "Very high load, dry climate timber-shrub (S)",
            1.0, 0.25, 8000.0, 8000.0,
            4.0*F, 4.0*F, 3.0*F, 0.0, 3.0*F,
            1500.0, 1800.0, 750.0,
            true, true);

        // International timber-understory models (M-*)
        self.set_record(166, "M-EUCd",
            "Discontinuous Litter Eucalyptus Plantation, With or Without Shrub Understory (Static)",
            0.4, 26.0, 21000.0, 20500.0,
            1.37*F, 2.89*F, 1.59*F, 0.0, 1.84*F,
            4500.0, 4200.0, 5000.0,
            false, true);

        self.set_record(167, "M-H",
            "Deciduous or Conifer Litter, Shrub and Herb Understory",
            0.1, 30.0, 20500.0, 20500.0,
            2.71*F, 1.0*F, 0.0, 0.66*F, 0.1*F,
            5500.0, 8000.0, 4500.0,
            true, true);

        self.set_record(168, "M-F",
            "Deciduous or Conifer Litter, Shrub and Fern Understory (Dynamic)",
            0.3, 35.0, 19500.0, 19500.0,
            4.5*F, 1.5*F, 0.5*F, 2.35*F, 0.48*F,
            6000.0, 8000.0, 4500.0,
            true, true);

        self.set_record(169, "M-CAD",
            "Deciduous Litter, Shrub Understory (Static)",
            0.63, 30.0, 20000.0, 20000.0,
            4.54*F, 1.87*F, 0.61*F, 0.0, 9.08*F,
            6000.0, 4921.0, 5000.0,
            false, true);

        self.set_record(170, "M-ESC",
            "Sclerophyll Broadleaf Litter, Shrub Understory (Static)",
            0.5, 27.0, 20500.0, 20500.0,
            5.65*F, 1.5*F, 0.48*F, 0.0, 7.89*F,
            5000.0, 4921.0, 5500.0,
            false, true);

        self.set_record(171, "M-PIN",
            "Medium-Long Needle Pine Litter, Shrub Understory (Static)",
            0.5, 40.0, 20500.0, 21500.0,
            7.21*F, 3.0*F, 0.0, 0.0, 6.89*F,
            5500.0, 5500.0, 6000.0,
            false, true);

        self.set_record(172, "M-EUC",
            "Eucalyptus Litter, Shrub Understory (Static)",
            0.64, 32.0, 21000.0, 21000.0,
            8.37*F, 3.81*F, 0.0, 0.0, 4.51*F,
            4700.0, 4200.0, 5000.0,
            false, true);

        // 173–179: Available for custom timber-understory models

        // =================================================================
        // Timber-Litter (TL1–TL9)
        // =================================================================

        self.set_record(181, "TL1", "Low load, compact conifer litter (S)",
            0.2, 0.30, 8000.0, 8000.0,
            1.0*F, 2.2*F, 3.6*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(182, "TL2", "Low load broadleaf litter (S)",
            0.2, 0.25, 8000.0, 8000.0,
            1.4*F, 2.3*F, 2.2*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(183, "TL3", "Moderate load conifer litter (S)",
            0.3, 0.20, 8000.0, 8000.0,
            0.5*F, 2.2*F, 2.8*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(184, "TL4", "Small downed logs (S)",
            0.4, 0.25, 8000.0, 8000.0,
            0.5*F, 1.5*F, 4.2*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        // NOTE: C++ has savrLiveWoody=160 for TL5 (likely a typo, should be 1600)
        self.set_record(185, "TL5", "High load conifer litter (S)",
            0.6, 0.25, 8000.0, 8000.0,
            1.15*F, 2.5*F, 4.4*F, 0.0, 0.0,
            2000.0, 1800.0, 160.0,
            true, true);

        self.set_record(186, "TL6", "High load broadleaf litter (S)",
            0.3, 0.25, 8000.0, 8000.0,
            2.4*F, 1.2*F, 1.2*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(187, "TL7", "Large downed logs (S)",
            0.4, 0.25, 8000.0, 8000.0,
            0.3*F, 1.4*F, 8.1*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(188, "TL8", "Long-needle litter (S)",
            0.3, 0.35, 8000.0, 8000.0,
            5.8*F, 1.4*F, 1.1*F, 0.0, 0.0,
            1800.0, 1800.0, 1600.0,
            true, true);

        self.set_record(189, "TL9", "Very high load broadleaf litter (S)",
            0.6, 0.35, 8000.0, 8000.0,
            6.65*F, 3.30*F, 4.15*F, 0.0, 0.0,
            1800.0, 1800.0, 1600.0,
            true, true);

        // International timber-litter models (F-*)
        self.set_record(190, "F-RAC",
            "Very Compact Litter, Short Needle Conifers (Static)",
            0.05, 28.0, 20500.0, 20500.0,
            3.75*F, 2.0*F, 1.0*F, 0.0, 1.18*F,
            6500.0, 4921.0, 4500.0,
            false, true);

        self.set_record(191, "F-FOL",
            "Compact Litter, Deciduous or Evergreen Foliage (Static)",
            0.15, 25.0, 20500.0, 20500.0,
            2.67*F, 1.27*F, 0.69*F, 0.0, 1.16*F,
            4500.0, 5500.0, 5000.0,
            false, true);

        self.set_record(192, "F-PIN",
            "Litter from Medium-Long Needle Pine Trees (Static)",
            0.1, 45.0, 20500.0, 21500.0,
            6.5*F, 1.5*F, 0.0, 0.0, 0.0,
            5500.0, 5500.0, 5500.0,
            false, true);

        self.set_record(193, "F-EUC",
            "Pure Eucalyptus Litter, No Understory (Static)",
            0.32, 26.0, 21000.0, 20500.0,
            4.63*F, 2.96*F, 1.27*F, 0.0, 1.12*F,
            4200.0, 4200.0, 5000.0,
            false, true);

        // 194–199: Available for custom timber-litter models

        // =================================================================
        // Slash-Blowdown (SB1–SB4)
        // =================================================================

        self.set_record(201, "SB1", "Low load activity fuel (S)",
            1.0, 0.25, 8000.0, 8000.0,
            1.5*F, 3.0*F, 11.0*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(202, "SB2", "Moderate load activity or low load blowdown (S)",
            1.0, 0.25, 8000.0, 8000.0,
            4.5*F, 4.25*F, 4.0*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(203, "SB3", "High load activity fuel or moderate load blowdown (S)",
            1.2, 0.25, 8000.0, 8000.0,
            5.5*F, 2.75*F, 3.0*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        self.set_record(204, "SB4", "High load blowdown (S)",
            2.7, 0.25, 8000.0, 8000.0,
            5.25*F, 3.5*F, 5.25*F, 0.0, 0.0,
            2000.0, 1800.0, 1600.0,
            true, true);

        // 205–210: Reserved for future SB models
        for i in 205..=210 {
            self.mark_as_reserved(i);
        }

        // 211–255: Available for custom models
    }
}

impl Default for FuelModels {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog() -> FuelModels {
        FuelModels::new()
    }

    // -------------------------------------------------------------------
    // Original 13 fuel models
    // -------------------------------------------------------------------

    #[test]
    fn fm1_properties() {
        let c = catalog();
        assert_eq!(c.fuel_code(1), "FM1");
        assert_eq!(c.fuel_name(1), "Short grass [1]");
        assert_eq!(c.fuelbed_depth(1, LengthUnits::Feet), 1.0);
        assert_eq!(c.moisture_of_extinction_dead(1, FractionUnits::Fraction), 0.12);
        assert_eq!(c.heat_of_combustion_dead(1, HeatOfCombustionUnits::BtusPerPound), 8000.0);
        assert_eq!(c.fuel_load_one_hour(1, LoadingUnits::PoundsPerSquareFoot), 0.034);
        assert_eq!(c.fuel_load_ten_hour(1, LoadingUnits::PoundsPerSquareFoot), 0.0);
        assert_eq!(c.fuel_load_live_herbaceous(1, LoadingUnits::PoundsPerSquareFoot), 0.0);
        assert_eq!(c.savr_one_hour(1, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet), 3500.0);
        assert!(!c.is_dynamic(1));
        assert!(c.is_fuel_model_defined(1));
        assert!(c.is_fuel_model_reserved(1));
    }

    #[test]
    fn fm4_chaparral() {
        let c = catalog();
        assert_eq!(c.fuel_code(4), "FM4");
        assert_eq!(c.fuelbed_depth(4, LengthUnits::Feet), 6.0);
        assert_eq!(c.fuel_load_one_hour(4, LoadingUnits::PoundsPerSquareFoot), 0.230);
        assert_eq!(c.fuel_load_live_woody(4, LoadingUnits::PoundsPerSquareFoot), 0.230);
        assert_eq!(c.savr_one_hour(4, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet), 2000.0);
    }

    #[test]
    fn fm13_heavy_slash() {
        let c = catalog();
        assert_eq!(c.fuel_code(13), "FM13");
        assert_eq!(c.fuelbed_depth(13, LengthUnits::Feet), 3.0);
        assert_eq!(c.fuel_load_one_hour(13, LoadingUnits::PoundsPerSquareFoot), 0.322);
        assert_eq!(c.fuel_load_ten_hour(13, LoadingUnits::PoundsPerSquareFoot), 1.058);
        assert_eq!(c.fuel_load_hundred_hour(13, LoadingUnits::PoundsPerSquareFoot), 1.288);
    }

    // -------------------------------------------------------------------
    // Non-burnable models
    // -------------------------------------------------------------------

    #[test]
    fn nb_models_have_zero_load() {
        let c = catalog();
        for &n in &[91, 92, 93, 94, 95, 98, 99] {
            assert!(c.is_all_fuel_load_zero(n), "NB model {} should have zero loads", n);
            assert!(c.is_fuel_model_defined(n));
            assert!(c.is_fuel_model_reserved(n));
        }
    }

    // -------------------------------------------------------------------
    // Scott & Burgan models — verify tons/acre conversion
    // -------------------------------------------------------------------

    #[test]
    fn gr1_converted_loads() {
        let c = catalog();
        let f = 2000.0 / 43560.0;
        assert_eq!(c.fuel_code(101), "GR1");
        assert!(c.is_dynamic(101));

        let l1h = c.fuel_load_one_hour(101, LoadingUnits::PoundsPerSquareFoot);
        assert!((l1h - 0.10 * f).abs() < 1e-12, "GR1 1-h load: {}", l1h);

        let llh = c.fuel_load_live_herbaceous(101, LoadingUnits::PoundsPerSquareFoot);
        assert!((llh - 0.30 * f).abs() < 1e-12, "GR1 live herb: {}", llh);
    }

    #[test]
    fn gr9_converted_loads() {
        let c = catalog();
        let f = 2000.0 / 43560.0;
        assert_eq!(c.fuel_code(109), "GR9");
        let l1h = c.fuel_load_one_hour(109, LoadingUnits::PoundsPerSquareFoot);
        assert!((l1h - 1.0 * f).abs() < 1e-12);
        let llh = c.fuel_load_live_herbaceous(109, LoadingUnits::PoundsPerSquareFoot);
        assert!((llh - 9.0 * f).abs() < 1e-12);
    }

    #[test]
    fn gs2_grass_shrub() {
        let c = catalog();
        let f = 2000.0 / 43560.0;
        assert_eq!(c.fuel_code(122), "GS2");
        assert_eq!(c.fuelbed_depth(122, LengthUnits::Feet), 1.5);
        let llw = c.fuel_load_live_woody(122, LoadingUnits::PoundsPerSquareFoot);
        assert!((llw - 1.0 * f).abs() < 1e-12);
    }

    #[test]
    fn sh5_shrub() {
        let c = catalog();
        assert_eq!(c.fuel_code(145), "SH5");
        assert_eq!(c.fuelbed_depth(145, LengthUnits::Feet), 6.0);
        assert_eq!(c.savr_one_hour(145, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet), 750.0);
    }

    #[test]
    fn tu1_timber_understory() {
        let c = catalog();
        let f = 2000.0 / 43560.0;
        assert_eq!(c.fuel_code(161), "TU1");
        assert!(c.is_dynamic(161));
        let l1h = c.fuel_load_one_hour(161, LoadingUnits::PoundsPerSquareFoot);
        assert!((l1h - 0.2 * f).abs() < 1e-12);
    }

    #[test]
    fn tl5_preserves_cpp_savr_typo() {
        // C++ has savrLiveWoody=160 for TL5 (likely a bug, should be 1600)
        // We preserve the C++ value for numerical parity
        let c = catalog();
        assert_eq!(
            c.savr_live_woody(185, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet),
            160.0
        );
    }

    #[test]
    fn sb4_slash_blowdown() {
        let c = catalog();
        let f = 2000.0 / 43560.0;
        assert_eq!(c.fuel_code(204), "SB4");
        assert_eq!(c.fuelbed_depth(204, LengthUnits::Feet), 2.7);
        let l100h = c.fuel_load_hundred_hour(204, LoadingUnits::PoundsPerSquareFoot);
        assert!((l100h - 5.25 * f).abs() < 1e-12);
    }

    // -------------------------------------------------------------------
    // International models
    // -------------------------------------------------------------------

    #[test]
    fn v_hb_grass_international() {
        let c = catalog();
        assert_eq!(c.fuel_code(110), "V-Hb");
        assert!(c.is_dynamic(110));
        // NOTE: C++ stores moistureOfExtinctionDead=24.0 (not a fraction!) for international models
        assert_eq!(c.moisture_of_extinction_dead(110, FractionUnits::Fraction), 24.0);
    }

    #[test]
    fn scal14_california_shrub() {
        let c = catalog();
        assert_eq!(c.fuel_code(153), "SCAL14");
        assert_eq!(c.heat_of_combustion_dead(153, HeatOfCombustionUnits::BtusPerPound), 9211.0);
    }

    #[test]
    fn m_euc_eucalyptus() {
        let c = catalog();
        let f = 2000.0 / 43560.0;
        assert_eq!(c.fuel_code(172), "M-EUC");
        let l1h = c.fuel_load_one_hour(172, LoadingUnits::PoundsPerSquareFoot);
        assert!((l1h - 8.37 * f).abs() < 1e-12);
    }

    #[test]
    fn f_pin_pine_litter() {
        let c = catalog();
        assert_eq!(c.fuel_code(192), "F-PIN");
        assert_eq!(c.heat_of_combustion_live(192, HeatOfCombustionUnits::BtusPerPound), 21500.0);
        // F-PIN has no live woody load
        assert_eq!(c.fuel_load_live_woody(192, LoadingUnits::PoundsPerSquareFoot), 0.0);
    }

    // -------------------------------------------------------------------
    // Undefined / invalid model queries
    // -------------------------------------------------------------------

    #[test]
    fn index_zero_not_defined() {
        let c = catalog();
        assert!(!c.is_fuel_model_defined(0));
    }

    #[test]
    fn undefined_slot_not_defined() {
        let c = catalog();
        // Slot 50 is available for custom models, not pre-populated
        assert!(!c.is_fuel_model_defined(50));
    }

    #[test]
    fn invalid_model_number() {
        let c = catalog();
        assert!(!c.is_fuel_model_defined(-1));
        assert!(!c.is_fuel_model_defined(257));
        assert!(!c.is_dynamic(-1));
        assert!(!c.is_fuel_model_reserved(0));
    }

    // -------------------------------------------------------------------
    // Reserved model ranges
    // -------------------------------------------------------------------

    #[test]
    fn reserved_ranges() {
        let c = catalog();
        // 94–95 reserved for future NB
        assert!(c.is_fuel_model_reserved(94));
        assert!(c.is_fuel_model_reserved(95));
        // 112 reserved for future grass
        assert!(c.is_fuel_model_reserved(112));
        // 125–130 reserved for future GS
        for i in 125..=130 {
            assert!(c.is_fuel_model_reserved(i), "model {} should be reserved", i);
        }
        // 205–210 reserved for future SB
        for i in 205..=210 {
            assert!(c.is_fuel_model_reserved(i), "model {} should be reserved", i);
        }
    }

    // -------------------------------------------------------------------
    // Custom fuel model support
    // -------------------------------------------------------------------

    #[test]
    fn set_custom_model_in_open_slot() {
        let mut c = catalog();
        let ok = c.set_custom_fuel_model(
            50, "CUS", "My custom model",
            2.0, LengthUnits::Feet,
            0.20, FractionUnits::Fraction,
            8000.0, 8000.0, HeatOfCombustionUnits::BtusPerPound,
            0.1, 0.05, 0.02, 0.03, 0.01, LoadingUnits::PoundsPerSquareFoot,
            2000.0, 1500.0, 1500.0, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet,
            true,
        );
        assert!(ok);
        assert!(c.is_fuel_model_defined(50));
        assert_eq!(c.fuel_code(50), "CUS");
        assert_eq!(c.fuelbed_depth(50, LengthUnits::Feet), 2.0);
        assert!(c.is_dynamic(50));
    }

    #[test]
    fn cannot_set_custom_in_reserved_slot() {
        let mut c = catalog();
        let ok = c.set_custom_fuel_model(
            1, "XX", "Should fail",
            1.0, LengthUnits::Feet,
            0.10, FractionUnits::Fraction,
            8000.0, 8000.0, HeatOfCombustionUnits::BtusPerPound,
            0.0, 0.0, 0.0, 0.0, 0.0, LoadingUnits::PoundsPerSquareFoot,
            1500.0, 1500.0, 1500.0, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet,
            false,
        );
        assert!(!ok);
        // Original FM1 data preserved
        assert_eq!(c.fuel_code(1), "FM1");
    }

    #[test]
    fn clear_custom_model() {
        let mut c = catalog();
        c.set_custom_fuel_model(
            50, "CUS", "Test",
            1.0, LengthUnits::Feet,
            0.15, FractionUnits::Fraction,
            8000.0, 8000.0, HeatOfCombustionUnits::BtusPerPound,
            0.05, 0.0, 0.0, 0.0, 0.0, LoadingUnits::PoundsPerSquareFoot,
            2000.0, 1500.0, 1500.0, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet,
            false,
        );
        assert!(c.is_fuel_model_defined(50));
        let ok = c.clear_custom_fuel_model(50);
        assert!(ok);
        assert!(!c.is_fuel_model_defined(50));
    }

    #[test]
    fn cannot_clear_reserved_model() {
        let mut c = catalog();
        let ok = c.clear_custom_fuel_model(1);
        assert!(!ok);
        assert!(c.is_fuel_model_defined(1));
    }

    #[test]
    fn custom_model_with_unit_conversion() {
        let mut c = catalog();
        // Set a custom model using metric units
        let ok = c.set_custom_fuel_model(
            50, "MET", "Metric test",
            0.3048, LengthUnits::Meters,  // 0.3048 m = 1 ft
            15.0, FractionUnits::Percent,  // 15% = 0.15 fraction
            8000.0, 8000.0, HeatOfCombustionUnits::BtusPerPound,
            0.034, 0.0, 0.0, 0.0, 0.0, LoadingUnits::PoundsPerSquareFoot,
            2000.0, 1500.0, 1500.0, SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet,
            false,
        );
        assert!(ok);

        // Read back in base units: depth should be ~1 ft
        let depth_ft = c.fuelbed_depth(50, LengthUnits::Feet);
        assert!((depth_ft - 1.0).abs() < 0.001, "depth_ft = {}", depth_ft);

        // Moisture should be ~0.15 fraction
        let moe = c.moisture_of_extinction_dead(50, FractionUnits::Fraction);
        assert!((moe - 0.15).abs() < 1e-10, "moe = {}", moe);
    }

    // -------------------------------------------------------------------
    // Total model count
    // -------------------------------------------------------------------

    #[test]
    fn total_defined_models() {
        let c = catalog();
        let count = (0..MAX_FUEL_MODELS)
            .filter(|&i| c.records[i].is_defined)
            .count();
        // 13 original + 7 NB (91-95,98,99) + 9 GR + 2 V-H + 4 GS + 9 SH
        // + 5 SCAL + 5 V-M + 5 TU + 7 M- + 9 TL + 4 F- + 4 SB = 83
        assert_eq!(count, 83, "expected 83 defined models, got {}", count);
    }
}
