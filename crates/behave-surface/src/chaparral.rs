//! Chaparral fuel model calculations.
//!
//! C++ source: chaparralFuel.h / chaparralFuel.cpp
//!
//! Implements the Rothermel & Philpot (1973) chaparral flammability model
//! with modifications by Jack Cohen (FIRECAST).

use firelab_base::{FuelConstants, FuelLifeState};

/// Chaparral vegetation type.
///
/// C++ source: `ChaparralFuelType` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaparralFuelType {
    NotSet = 0,
    Chamise = 1,
    MixedBrush = 2,
}

/// How chaparral fuel load is specified.
///
/// C++ source: `ChaparralFuelLoadInputMode` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaparralFuelLoadInputMode {
    DirectFuelLoad = 1,
    FuelLoadFromDepthAndChaparralType = 2,
}

/// Number of chaparral size classes.
pub const CHAPARRAL_NUM_FUEL_CLASSES: usize = 5;

const NUM_LIFE: usize = FuelConstants::MAX_LIFE_STATES;
const NUM_SIZE: usize = CHAPARRAL_NUM_FUEL_CLASSES;
const DEAD: usize = FuelLifeState::Dead as usize;
const LIVE: usize = FuelLifeState::Live as usize;

/// Chaparral fuel loading and depth calculator.
///
/// Fuel properties are stored as [life_state][size_class] arrays.
/// Life state: 0=Dead, 1=Live. Size classes: 0-4.
/// For live fuels, size class 0 = leaves.
#[derive(Debug, Clone)]
pub struct ChaparralFuel {
    age: f64,
    days: f64, // days since May 1
    dead_fuel_fraction: f64,
    dead_moisture_of_extinction: f64,
    live_moisture_of_extinction: f64,
    fuel_bed_depth: f64,   // ft
    total_dead_load: f64,  // lb/ft²
    total_fuel_load: f64,  // lb/ft²
    total_live_load: f64,  // lb/ft²
    fuel_type: ChaparralFuelType,

    density: [[f64; NUM_SIZE]; NUM_LIFE],
    heat_of_combustion: [[f64; NUM_SIZE]; NUM_LIFE],
    load: [[f64; NUM_SIZE]; NUM_LIFE],
    moisture: [[f64; NUM_SIZE]; NUM_LIFE],
    savr: [[f64; NUM_SIZE]; NUM_LIFE],
    effective_silica: [[f64; NUM_SIZE]; NUM_LIFE],
    total_silica: [[f64; NUM_SIZE]; NUM_LIFE],
}

impl ChaparralFuel {
    pub fn new() -> Self {
        let mut fuel = Self {
            age: 0.0,
            days: 0.0,
            dead_fuel_fraction: 0.0,
            dead_moisture_of_extinction: 0.3,
            live_moisture_of_extinction: 0.65,
            fuel_bed_depth: 0.0,
            total_dead_load: 0.0,
            total_fuel_load: 0.0,
            total_live_load: 0.0,
            fuel_type: ChaparralFuelType::NotSet,
            density: [[46.0; NUM_SIZE]; NUM_LIFE],
            heat_of_combustion: [[8000.0; NUM_SIZE]; NUM_LIFE],
            load: [[0.0; NUM_SIZE]; NUM_LIFE],
            moisture: [[1.0; NUM_SIZE]; NUM_LIFE],
            savr: [[0.0; NUM_SIZE]; NUM_LIFE],
            effective_silica: [[0.015; NUM_SIZE]; NUM_LIFE],
            total_silica: [[0.055; NUM_SIZE]; NUM_LIFE],
        };
        fuel.initialize_fuel_arrays();
        fuel
    }

    fn initialize_fuel_arrays(&mut self) {
        // Live leaf density
        self.density[LIVE][0] = 32.0;
        self.effective_silica[LIVE][0] = 0.035;

        // Live heat of combustion (Bevins, BehavePlus 6)
        self.heat_of_combustion[LIVE][0] = 10500.0; // leaf
        self.heat_of_combustion[LIVE][1] = 10500.0; // stem
        self.heat_of_combustion[LIVE][2] = 9500.0;
        self.heat_of_combustion[LIVE][3] = 9500.0;
        self.heat_of_combustion[LIVE][4] = 9500.0;

        // Surface area-to-volume ratios
        self.savr[LIVE][0] = 2200.0; // leaf
        self.savr[DEAD][0] = 640.0;
        self.savr[LIVE][1] = 640.0;
        self.savr[DEAD][1] = 127.0;
        self.savr[LIVE][2] = 127.0;
        self.savr[DEAD][2] = 61.0;
        self.savr[LIVE][3] = 61.0;
        self.savr[DEAD][3] = 27.0;
        self.savr[LIVE][4] = 27.0;
        self.savr[DEAD][4] = 0.0;
    }

    fn is_good_index(&self, life: usize, size: usize) -> bool {
        life < NUM_LIFE && size < NUM_SIZE
    }

    // --- Getters ---

    pub fn get_age(&self) -> f64 { self.age }
    pub fn get_days_since_may_first(&self) -> f64 { self.days }
    pub fn get_dead_fuel_fraction(&self) -> f64 { self.dead_fuel_fraction }
    pub fn get_dead_moisture_of_extinction(&self) -> f64 { self.dead_moisture_of_extinction }
    pub fn get_live_moisture_of_extinction(&self) -> f64 { self.live_moisture_of_extinction }
    pub fn get_fuel_bed_depth(&self) -> f64 { self.fuel_bed_depth }
    pub fn get_total_dead_fuel_load(&self) -> f64 { self.total_dead_load }
    pub fn get_total_fuel_load(&self) -> f64 { self.total_fuel_load }
    pub fn get_total_live_fuel_load(&self) -> f64 { self.total_live_load }
    pub fn get_chaparral_fuel_type(&self) -> ChaparralFuelType { self.fuel_type }

    pub fn get_density(&self, life: FuelLifeState, size: usize) -> f64 {
        let l = life as usize;
        if self.is_good_index(l, size) { self.density[l][size] } else { -1.0 }
    }

    pub fn get_heat_of_combustion(&self, life: FuelLifeState, size: usize) -> f64 {
        let l = life as usize;
        if self.is_good_index(l, size) { self.heat_of_combustion[l][size] } else { -1.0 }
    }

    pub fn get_load(&self, life: FuelLifeState, size: usize) -> f64 {
        let l = life as usize;
        if self.is_good_index(l, size) { self.load[l][size] } else { -1.0 }
    }

    pub fn get_moisture(&self, life: FuelLifeState, size: usize) -> f64 {
        let l = life as usize;
        if self.is_good_index(l, size) { self.moisture[l][size] } else { -1.0 }
    }

    pub fn get_savr(&self, life: FuelLifeState, size: usize) -> f64 {
        let l = life as usize;
        if self.is_good_index(l, size) { self.savr[l][size] } else { -1.0 }
    }

    pub fn get_effective_silica(&self, life: FuelLifeState, size: usize) -> f64 {
        let l = life as usize;
        if self.is_good_index(l, size) { self.effective_silica[l][size] } else { -1.0 }
    }

    pub fn get_total_silica(&self, life: FuelLifeState, size: usize) -> f64 {
        let l = life as usize;
        if self.is_good_index(l, size) { self.total_silica[l][size] } else { -1.0 }
    }

    // --- Simple setters ---

    pub fn set_depth(&mut self, depth: f64) { self.fuel_bed_depth = depth; }
    pub fn set_dead_fuel_fraction(&mut self, frac: f64) { self.dead_fuel_fraction = frac; }
    pub fn set_chaparral_fuel_type(&mut self, ft: ChaparralFuelType) { self.fuel_type = ft; }
    pub fn set_total_fuel_load(&mut self, load: f64) { self.total_fuel_load = load; }

    pub fn set_moisture(&mut self, value: f64, life: FuelLifeState, size: usize) {
        let l = life as usize;
        if self.is_good_index(l, size) {
            self.moisture[l][size] = value;
        }
    }

    pub fn set_live_fuel_heat_of_combustion(&mut self, leaf_heat: f64, wood_heat: f64) {
        self.heat_of_combustion[LIVE][0] = leaf_heat;
        for s in 1..NUM_SIZE {
            self.heat_of_combustion[LIVE][s] = wood_heat;
        }
    }

    pub fn set_live_fuel_moisture(&mut self, leaf_moisture: f64, wood_moisture: f64) {
        self.moisture[LIVE][0] = leaf_moisture;
        for s in 1..NUM_SIZE {
            self.moisture[LIVE][s] = wood_moisture;
        }
    }

    // --- Age/date setters that update dependent values ---

    pub fn set_age(&mut self, years: f64) {
        self.age = years;
        self.update_total_fuel_load_from_age();
        self.update_dead_fuel_fraction_from_age();
        self.update_fuel_bed_depth_from_age();
        self.update_fuel_loads();
    }

    pub fn set_date_days_since_may(&mut self, days: i32) {
        self.days = if days > 184 { 184.0 } else { days as f64 };
        self.update_live_fuel_moisture_from_date();
        self.update_live_fuel_heat_from_date();
    }

    pub fn set_date_month_day(&mut self, month: i32, day: i32) {
        let days_in_months = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let m = month.clamp(1, 12) as usize;
        let d = day.clamp(1, 31);
        let mut days = days_in_months[m - 1] + d;
        days = days.min(304); // Don't extend past Oct 31
        let mut days_since_may = days - 121;
        if days_since_may < 0 { days_since_may = 0; }
        self.days = days_since_may as f64;
        self.update_live_fuel_moisture_from_date();
        self.update_live_fuel_heat_from_date();
    }

    // --- Update methods ---

    pub fn update_fuel_load_from_depth_and_dead_fuel_fraction(&mut self) {
        self.update_fuel_loads();
    }

    pub fn update_fuel_load_from_depth_and_fuel_type(&mut self) {
        self.update_age_from_depth();
        self.update_total_fuel_load_from_age();
        self.update_fuel_loads();
    }

    pub fn update_dead_fuel_fraction_from_age(&mut self) {
        self.dead_fuel_fraction = 0.0694 * (0.0402 * self.age).exp();
    }

    pub fn update_fuel_loads(&mut self) {
        let df = self.dead_fuel_fraction;
        let tl = self.total_fuel_load;

        self.load[DEAD][0] = 0.347 * df * tl;
        self.load[DEAD][1] = 0.364 * df * tl;
        self.load[DEAD][2] = 0.207 * df * tl;
        self.load[DEAD][3] = 0.082 * df * tl; // Bevins uses 0.082 (not 0.085) to total 1.000
        self.load[DEAD][4] = 0.0;

        self.load[LIVE][0] = tl * (0.1957 - 0.3050 * df);
        self.load[LIVE][1] = tl * (0.2416 - 0.2560 * df);
        self.load[LIVE][2] = tl * (0.1918 - 0.2560 * df);
        self.load[LIVE][3] = tl * (0.2648 - 0.0500 * df);
        // Bevins method: make live loads add up
        self.total_live_load = (1.0 - df) * tl;
        self.load[LIVE][4] = self.total_live_load
            - self.load[LIVE][0]
            - self.load[LIVE][1]
            - self.load[LIVE][2]
            - self.load[LIVE][3];
        if self.load[LIVE][4] < 0.0 {
            self.load[LIVE][4] = 0.0;
        }

        self.total_dead_load = 0.0;
        self.total_live_load = 0.0;
        for s in 0..NUM_SIZE {
            self.total_dead_load += self.load[DEAD][s];
            self.total_live_load += self.load[LIVE][s];
        }
    }

    pub fn update_live_fuel_heat_from_date(&mut self) {
        let d = self.days;
        self.heat_of_combustion[LIVE][0] =
            9613.0 - 1.00 * d + 0.1369 * d * d - 0.000365 * d * d * d;
        let live_heat = 9509.0 - 10.74 * d + 0.1359 * d * d - 0.000405 * d * d * d;
        for s in 1..NUM_SIZE {
            self.heat_of_combustion[LIVE][s] = live_heat;
        }
    }

    pub fn update_live_fuel_moisture_from_date(&mut self) {
        self.moisture[LIVE][0] = 1.0 / (0.726 + 0.00877 * self.days);
        let live_mc = 1.0 / (1.454 + 0.00650 * self.days);
        for s in 1..NUM_SIZE {
            self.moisture[LIVE][s] = live_mc;
        }
    }

    pub fn update_age_from_depth(&mut self) {
        match self.fuel_type {
            ChaparralFuelType::Chamise => {
                self.age = (3.912023 * (self.fuel_bed_depth / 7.5).sqrt()).exp();
            }
            ChaparralFuelType::MixedBrush => {
                self.age = (3.912023 * (self.fuel_bed_depth / 10.0).sqrt()).exp();
            }
            ChaparralFuelType::NotSet => {
                self.age = -1.0;
            }
        }
    }

    pub fn update_fuel_bed_depth_from_age(&mut self) {
        match self.fuel_type {
            ChaparralFuelType::Chamise => {
                let x = self.age.ln() / 3.912023;
                self.fuel_bed_depth = 7.5 * x * x;
            }
            ChaparralFuelType::MixedBrush => {
                let x = self.age.ln() / 3.912023;
                self.fuel_bed_depth = 10.0 * x * x;
            }
            ChaparralFuelType::NotSet => {
                self.fuel_bed_depth = -1.0;
            }
        }
    }

    pub fn update_total_fuel_load_from_age(&mut self) {
        match self.fuel_type {
            ChaparralFuelType::Chamise => {
                let chamise_load_factor = 0.0347; // Cohen's FIRECAST
                let tpa = self.age / (1.4459 + chamise_load_factor * self.age);
                self.total_fuel_load = tpa * 2000.0 / 43560.0;
            }
            ChaparralFuelType::MixedBrush => {
                let tpa = self.age / (0.4849 + 0.0170 * self.age);
                self.total_fuel_load = tpa * 2000.0 / 43560.0;
            }
            ChaparralFuelType::NotSet => {
                self.total_fuel_load = -1.0;
            }
        }
    }
}

impl Default for ChaparralFuel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let fuel = ChaparralFuel::new();
        assert_eq!(fuel.get_chaparral_fuel_type(), ChaparralFuelType::NotSet);
        assert!((fuel.get_dead_moisture_of_extinction() - 0.3).abs() < 1e-6);
        assert!((fuel.get_live_moisture_of_extinction() - 0.65).abs() < 1e-6);
        // Live leaf density should be 32.0
        assert!((fuel.get_density(FuelLifeState::Live, 0) - 32.0).abs() < 1e-6);
        // Dead density should be 46.0
        assert!((fuel.get_density(FuelLifeState::Dead, 0) - 46.0).abs() < 1e-6);
    }

    #[test]
    fn chamise_age_20() {
        let mut fuel = ChaparralFuel::new();
        fuel.set_chaparral_fuel_type(ChaparralFuelType::Chamise);
        fuel.set_age(20.0);

        // Total fuel load should be positive
        assert!(fuel.get_total_fuel_load() > 0.0);
        // Dead fraction should increase with age
        assert!(fuel.get_dead_fuel_fraction() > 0.0);
        // Depth should be positive
        assert!(fuel.get_fuel_bed_depth() > 0.0);
        // Dead + live loads should sum close to total
        let total = fuel.get_total_dead_fuel_load() + fuel.get_total_live_fuel_load();
        assert!(
            (total - fuel.get_total_fuel_load()).abs() < 0.001,
            "dead+live={total} vs total={}",
            fuel.get_total_fuel_load()
        );
    }

    #[test]
    fn mixed_brush_age_30() {
        let mut fuel = ChaparralFuel::new();
        fuel.set_chaparral_fuel_type(ChaparralFuelType::MixedBrush);
        fuel.set_age(30.0);

        assert!(fuel.get_total_fuel_load() > 0.0);
        assert!(fuel.get_fuel_bed_depth() > 0.0);
    }

    #[test]
    fn depth_to_age_round_trip_chamise() {
        let mut fuel = ChaparralFuel::new();
        fuel.set_chaparral_fuel_type(ChaparralFuelType::Chamise);
        fuel.set_age(20.0);
        let depth = fuel.get_fuel_bed_depth();

        // Now derive age from depth
        let mut fuel2 = ChaparralFuel::new();
        fuel2.set_chaparral_fuel_type(ChaparralFuelType::Chamise);
        fuel2.set_depth(depth);
        fuel2.update_age_from_depth();

        assert!(
            (fuel2.get_age() - 20.0).abs() < 0.01,
            "expected ~20, got {}",
            fuel2.get_age()
        );
    }

    #[test]
    fn savr_values() {
        let fuel = ChaparralFuel::new();
        assert!((fuel.get_savr(FuelLifeState::Live, 0) - 2200.0).abs() < 1e-6);
        assert!((fuel.get_savr(FuelLifeState::Dead, 0) - 640.0).abs() < 1e-6);
        assert!((fuel.get_savr(FuelLifeState::Dead, 1) - 127.0).abs() < 1e-6);
    }
}
