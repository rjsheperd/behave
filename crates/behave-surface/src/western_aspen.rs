//! Western Aspen fuel model calculations.
//!
//! C++ source: westernAspen.h / westernAspen.cpp
//!
//! Provides fuel property adjustments (loads, SAVR) based on aspen
//! fuel model number and curing level via interpolation tables.

/// Aspen fire severity level.
///
/// C++ source: `AspenFireSeverity` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspenFireSeverity {
    Low = 0,
    Moderate = 1,
}

/// Curing level breakpoints for interpolation.
const CURING_ARRAY: [f64; 6] = [0.0, 0.3, 0.5, 0.7, 0.9, 1.000000001];

/// Fuel bed depths by aspen fuel model number (1-5).
const DEPTH: [f64; 5] = [0.65, 0.30, 0.18, 0.50, 0.18];

/// Tons per acre to lb/ft² conversion factor.
const TPA_TO_LB_PER_FT2: f64 = 2000.0 / 43560.0;

// Load tables [model_index][curing_bracket]
const LOAD_DEAD_1HR: [[f64; 6]; 5] = [
    [0.800, 0.893, 1.056, 1.218, 1.379, 1.4595],
    [0.738, 0.930, 1.056, 1.183, 1.309, 1.3720],
    [0.601, 0.645, 0.671, 0.699, 0.730, 0.7455],
    [0.880, 0.906, 1.037, 1.167, 1.300, 1.3665],
    [0.754, 0.797, 0.825, 0.854, 0.884, 0.8990],
];

const LOAD_DEAD_10HR: [f64; 5] = [0.975, 0.475, 1.035, 1.340, 1.115];

const LOAD_LIVE_HERB: [[f64; 6]; 5] = [
    [0.335, 0.234, 0.167, 0.100, 0.033, 0.000],
    [0.665, 0.465, 0.332, 0.199, 0.067, 0.000],
    [0.150, 0.105, 0.075, 0.045, 0.015, 0.000],
    [0.100, 0.070, 0.050, 0.030, 0.010, 0.000],
    [0.150, 0.105, 0.075, 0.045, 0.015, 0.000],
];

const LOAD_LIVE_WOODY: [[f64; 6]; 5] = [
    [0.403, 0.403, 0.333, 0.283, 0.277, 0.2740],
    [0.000, 0.000, 0.000, 0.000, 0.000, 0.0000],
    [0.000, 0.000, 0.000, 0.000, 0.000, 0.0000],
    [0.455, 0.455, 0.364, 0.290, 0.261, 0.2465],
    [0.000, 0.000, 0.000, 0.000, 0.000, 0.0000],
];

const SAVR_DEAD_1HR: [[f64; 6]; 5] = [
    [1440.0, 1620.0, 1910.0, 2090.0, 2220.0, 2285.0],
    [1480.0, 1890.0, 2050.0, 2160.0, 2240.0, 2280.0],
    [1400.0, 1540.0, 1620.0, 1690.0, 1750.0, 1780.0],
    [1350.0, 1420.0, 1710.0, 1910.0, 2060.0, 2135.0],
    [1420.0, 1540.0, 1610.0, 1670.0, 1720.0, 1745.0],
];

const SAVR_LIVE_WOODY: [[f64; 6]; 5] = [
    [2440.0, 2440.0, 2310.0, 2090.0, 1670.0, 1670.0],
    [2440.0, 2440.0, 2440.0, 2440.0, 2440.0, 2440.0],
    [2440.0, 2440.0, 2440.0, 2440.0, 2440.0, 2440.0],
    [2530.0, 2530.0, 2410.0, 2210.0, 1800.0, 1800.0],
    [2440.0, 2440.0, 2440.0, 2440.0, 2440.0, 2440.0],
];

/// Interpolate a value from a 6-element array based on curing level.
fn aspen_interpolate(curing: f64, values: &[f64; 6]) -> f64 {
    let c = curing.clamp(0.0, 1.0);
    let mut fraction = 0.0;
    let mut idx = 1;
    for i in 1..CURING_ARRAY.len() {
        if c < CURING_ARRAY[i] {
            fraction = 1.0 - (CURING_ARRAY[i] - c) / (CURING_ARRAY[i] - CURING_ARRAY[i - 1]);
            idx = i;
            break;
        }
    }
    values[idx - 1] + fraction * (values[idx] - values[idx - 1])
}

/// Western Aspen fuel property adjustment calculator.
#[derive(Debug, Clone)]
pub struct WesternAspen {
    mortality: f64,
    dead_one_hour: f64,
    dead_ten_hour: f64,
    live_herbaceous: f64,
    live_woody: f64,
    savr_dead_one_hour: f64,
    savr_dead_ten_hour: f64,
    savr_live_herbaceous: f64,
    savr_live_woody: f64,
}

impl WesternAspen {
    pub fn new() -> Self {
        Self {
            mortality: 0.0,
            dead_one_hour: 0.0,
            dead_ten_hour: 0.0,
            live_herbaceous: 0.0,
            live_woody: 0.0,
            savr_dead_one_hour: 1440.0,
            savr_dead_ten_hour: 109.0,
            savr_live_herbaceous: 2800.0,
            savr_live_woody: 2440.0,
        }
    }

    // --- Getters ---

    pub fn get_aspen_mortality(&self) -> f64 { self.mortality }
    pub fn get_aspen_fuel_model_number(&self) -> i32 { 0 }

    pub fn get_aspen_fuel_bed_depth(model_number: i32) -> f64 {
        let idx = (model_number - 1) as usize;
        if idx < 5 { DEPTH[idx] } else { -1.0 }
    }

    pub fn get_aspen_heat_of_combustion_dead() -> f64 { 8000.0 }
    pub fn get_aspen_heat_of_combustion_live() -> f64 { 8000.0 }
    pub fn get_aspen_moisture_of_extinction_dead() -> f64 { 0.25 }

    pub fn get_aspen_load_dead_one_hour(&self) -> f64 { self.dead_one_hour }
    pub fn get_aspen_load_dead_ten_hour(&self) -> f64 { self.dead_ten_hour }
    pub fn get_aspen_load_live_herbaceous(&self) -> f64 { self.live_herbaceous }
    pub fn get_aspen_load_live_woody(&self) -> f64 { self.live_woody }
    pub fn get_aspen_savr_dead_one_hour(&self) -> f64 { self.savr_dead_one_hour }
    pub fn get_aspen_savr_dead_ten_hour(&self) -> f64 { self.savr_dead_ten_hour }
    pub fn get_aspen_savr_live_herbaceous(&self) -> f64 { self.savr_live_herbaceous }
    pub fn get_aspen_savr_live_woody(&self) -> f64 { self.savr_live_woody }

    // --- Calculate methods ---

    pub fn calculate_load_dead_one_hour(&mut self, model: i32, curing: f64) -> f64 {
        let idx = (model - 1) as usize;
        self.dead_one_hour = if idx < 5 {
            aspen_interpolate(curing, &LOAD_DEAD_1HR[idx])
        } else {
            0.0
        };
        self.dead_one_hour *= TPA_TO_LB_PER_FT2;
        self.dead_one_hour
    }

    pub fn calculate_load_dead_ten_hour(&mut self, model: i32) -> f64 {
        let idx = (model - 1) as usize;
        self.dead_ten_hour = if idx < 5 { LOAD_DEAD_10HR[idx] } else { 0.0 };
        self.dead_ten_hour *= TPA_TO_LB_PER_FT2;
        self.dead_ten_hour
    }

    pub fn calculate_load_live_herbaceous(&mut self, model: i32, curing: f64) -> f64 {
        let idx = (model - 1) as usize;
        self.live_herbaceous = if idx < 5 {
            aspen_interpolate(curing, &LOAD_LIVE_HERB[idx])
        } else {
            0.0
        };
        self.live_herbaceous *= TPA_TO_LB_PER_FT2;
        self.live_herbaceous
    }

    pub fn calculate_load_live_woody(&mut self, model: i32, curing: f64) -> f64 {
        let idx = (model - 1) as usize;
        self.live_woody = if idx < 5 {
            aspen_interpolate(curing, &LOAD_LIVE_WOODY[idx])
        } else {
            0.0
        };
        self.live_woody *= TPA_TO_LB_PER_FT2;
        self.live_woody
    }

    pub fn calculate_savr_dead_one_hour(&mut self, model: i32, curing: f64) -> f64 {
        let idx = (model - 1) as usize;
        self.savr_dead_one_hour = if idx < 5 {
            aspen_interpolate(curing, &SAVR_DEAD_1HR[idx])
        } else {
            1440.0
        };
        self.savr_dead_one_hour
    }

    pub fn calculate_savr_dead_ten_hour(&mut self) -> f64 {
        self.savr_dead_ten_hour = 109.0;
        self.savr_dead_ten_hour
    }

    pub fn calculate_savr_live_herbaceous(&mut self) -> f64 {
        self.savr_live_herbaceous = 2800.0;
        self.savr_live_herbaceous
    }

    pub fn calculate_savr_live_woody(&mut self, model: i32, curing: f64) -> f64 {
        let idx = (model - 1) as usize;
        self.savr_live_woody = if idx < 5 {
            aspen_interpolate(curing, &SAVR_LIVE_WOODY[idx])
        } else {
            2440.0
        };
        self.savr_live_woody
    }

    /// Calculate aspen mortality from flame length and DBH.
    /// Mortality must be calculated AFTER spread rate.
    pub fn calculate_mortality(
        &mut self,
        severity: AspenFireSeverity,
        flame_length: f64,
        dbh: f64,
    ) -> f64 {
        let char_height = flame_length / 1.8;
        let mortality = match severity {
            AspenFireSeverity::Low => {
                1.0 / (1.0 + (-4.407 + 0.638 * dbh - 2.134 * char_height).exp())
            }
            AspenFireSeverity::Moderate => {
                1.0 / (1.0 + (-2.157 + 0.218 * dbh - 3.600 * char_height).exp())
            }
        };
        self.mortality = mortality.clamp(0.0, 1.0);
        self.mortality
    }
}

impl Default for WesternAspen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_at_zero_curing() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let result = aspen_interpolate(0.0, &values);
        assert!((result - 1.0).abs() < 1e-6, "at curing=0 should be first value, got {result}");
    }

    #[test]
    fn interpolate_at_one_curing() {
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let result = aspen_interpolate(1.0, &values);
        assert!((result - 6.0).abs() < 0.01, "at curing=1 should be last value, got {result}");
    }

    #[test]
    fn fuel_bed_depth() {
        assert!((WesternAspen::get_aspen_fuel_bed_depth(1) - 0.65).abs() < 1e-6);
        assert!((WesternAspen::get_aspen_fuel_bed_depth(3) - 0.18).abs() < 1e-6);
    }

    #[test]
    fn dead_one_hour_load() {
        let mut aspen = WesternAspen::new();
        let load = aspen.calculate_load_dead_one_hour(1, 0.5);
        // At model 1, curing 0.5: should interpolate to 1.056 tpa → * TPA_TO_LB_PER_FT2
        let expected = 1.056 * TPA_TO_LB_PER_FT2;
        assert!(
            (load - expected).abs() < 1e-4,
            "expected {expected}, got {load}"
        );
    }

    #[test]
    fn mortality_low_severity() {
        let mut aspen = WesternAspen::new();
        let mort = aspen.calculate_mortality(AspenFireSeverity::Low, 5.0, 10.0);
        // Should be between 0 and 1
        assert!(mort >= 0.0 && mort <= 1.0, "mortality out of range: {mort}");
    }

    #[test]
    fn savr_constants() {
        let mut aspen = WesternAspen::new();
        assert!((aspen.calculate_savr_dead_ten_hour() - 109.0).abs() < 1e-6);
        assert!((aspen.calculate_savr_live_herbaceous() - 2800.0).abs() < 1e-6);
    }

    #[test]
    fn heat_and_extinction() {
        assert!((WesternAspen::get_aspen_heat_of_combustion_dead() - 8000.0).abs() < 1e-6);
        assert!((WesternAspen::get_aspen_heat_of_combustion_live() - 8000.0).abs() < 1e-6);
        assert!((WesternAspen::get_aspen_moisture_of_extinction_dead() - 0.25).abs() < 1e-6);
    }
}
