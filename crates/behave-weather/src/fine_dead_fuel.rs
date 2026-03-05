//! Fine dead fuel moisture lookup tool.
//!
//! C++ source: fineDeadFuelMoistureTool.h / fineDeadFuelMoistureTool.cpp
//!
//! Calculates fine dead fuel moisture as the sum of a reference moisture
//! (from dry bulb temperature and relative humidity) and a correction
//! (from month, time of day, elevation, slope, aspect, and shading).

use firelab_base::{FractionUnits, UnitConversion};

/// Aspect index for FDFM lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectIndex {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

/// Dry bulb temperature range index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DryBulbIndex {
    F10To29 = 0,
    F30To49 = 1,
    F50To69 = 2,
    F70To89 = 3,
    F90To109 = 4,
    GreaterThan109 = 5,
}

/// Elevation difference index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevationIndex {
    Below1000To2000 = 0,
    LevelWithin1000 = 1,
    Above1000To2000 = 2,
}

/// Month range index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonthIndex {
    MayJunJul = 0,
    FebMarAprAugSepOct = 1,
    NovDecJan = 2,
}

/// Relative humidity range index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RHIndex {
    Pct0To4 = 0,
    Pct5To9 = 1,
    Pct10To14 = 2,
    Pct15To19 = 3,
    Pct20To24 = 4,
    Pct25To29 = 5,
    Pct30To34 = 6,
    Pct35To39 = 7,
    Pct40To44 = 8,
    Pct45To49 = 9,
    Pct50To54 = 10,
    Pct55To59 = 11,
    Pct60To64 = 12,
    Pct65To69 = 13,
    Pct70To74 = 14,
    Pct75To79 = 15,
    Pct80To84 = 16,
    Pct85To89 = 17,
    Pct90To94 = 18,
    Pct95To99 = 19,
    Pct100 = 20,
}

/// Shading index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadingIndex {
    Exposed = 0,
    Shaded = 1,
}

/// Slope steepness index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlopeIndex {
    ZeroTo30Percent = 0,
    GreaterThan30Percent = 1,
}

/// Time of day index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfDayIndex {
    H0800To0959 = 0,
    H1000To1159 = 1,
    H1200To1359 = 2,
    H1400To1559 = 3,
    H1600To1759 = 4,
    H1800ToSunset = 5,
}

/// Reference moisture table: 6 dry-bulb rows × 21 RH columns.
/// Values are fractions (0.01 = 1%).
const REFERENCE_MOISTURES: [[f64; 21]; 6] = [
    [0.01, 0.02, 0.02, 0.03, 0.04, 0.05, 0.05, 0.06, 0.07, 0.08, 0.08, 0.08, 0.09, 0.09, 0.10, 0.11, 0.12, 0.12, 0.13, 0.13, 0.14],
    [0.01, 0.02, 0.02, 0.03, 0.04, 0.05, 0.05, 0.06, 0.07, 0.07, 0.07, 0.08, 0.09, 0.09, 0.10, 0.10, 0.11, 0.12, 0.13, 0.13, 0.13],
    [0.01, 0.02, 0.02, 0.03, 0.04, 0.05, 0.05, 0.06, 0.06, 0.07, 0.07, 0.08, 0.08, 0.09, 0.09, 0.10, 0.11, 0.12, 0.12, 0.12, 0.13],
    [0.01, 0.01, 0.02, 0.02, 0.03, 0.04, 0.05, 0.05, 0.06, 0.07, 0.07, 0.08, 0.08, 0.08, 0.09, 0.10, 0.10, 0.11, 0.12, 0.12, 0.13],
    [0.01, 0.01, 0.02, 0.02, 0.03, 0.04, 0.04, 0.05, 0.06, 0.07, 0.07, 0.08, 0.08, 0.08, 0.09, 0.10, 0.10, 0.11, 0.12, 0.12, 0.13],
    [0.01, 0.01, 0.02, 0.02, 0.03, 0.04, 0.04, 0.05, 0.06, 0.07, 0.07, 0.08, 0.08, 0.08, 0.09, 0.10, 0.10, 0.11, 0.12, 0.12, 0.12],
];

/// Correction moisture table: 36 rows × 18 columns.
/// Row index = f(month, shading, slope, aspect).
/// Column index = f(elevation, time_of_day).
/// Values are fractions.
const CORRECTION_MOISTURES: [[f64; 18]; 36] = [
    // May-Jun-Jul Exposed (rows 0-7)
    [0.02, 0.03, 0.04, 0.01, 0.01, 0.01, 0.0, 0.0, 0.01, 0.0, 0.0, 0.01, 0.01, 0.01, 0.01, 0.02, 0.03, 0.04],
    [0.03, 0.04, 0.04, 0.01, 0.02, 0.02, 0.01, 0.01, 0.02, 0.01, 0.01, 0.02, 0.01, 0.02, 0.02, 0.03, 0.04, 0.04],
    [0.02, 0.02, 0.03, 0.01, 0.01, 0.01, 0.0, 0.0, 0.01, 0.0, 0.0, 0.01, 0.01, 0.01, 0.02, 0.03, 0.04, 0.04],
    [0.01, 0.02, 0.02, 0.0, 0.0, 0.01, 0.0, 0.0, 0.01, 0.01, 0.01, 0.02, 0.02, 0.03, 0.04, 0.04, 0.05, 0.06],
    [0.02, 0.03, 0.03, 0.01, 0.01, 0.01, 0.0, 0.0, 0.01, 0.0, 0.0, 0.01, 0.01, 0.01, 0.01, 0.02, 0.03, 0.03],
    [0.02, 0.03, 0.03, 0.01, 0.01, 0.02, 0.0, 0.01, 0.01, 0.0, 0.01, 0.01, 0.01, 0.01, 0.02, 0.02, 0.03, 0.03],
    [0.02, 0.03, 0.04, 0.01, 0.01, 0.02, 0.0, 0.0, 0.01, 0.0, 0.0, 0.01, 0.0, 0.01, 0.01, 0.02, 0.03, 0.03],
    [0.04, 0.05, 0.06, 0.02, 0.03, 0.04, 0.01, 0.01, 0.02, 0.0, 0.0, 0.01, 0.0, 0.0, 0.01, 0.01, 0.02, 0.02],
    // May-Jun-Jul Shaded (rows 8-11)
    [0.04, 0.05, 0.05, 0.03, 0.04, 0.05, 0.03, 0.03, 0.04, 0.03, 0.03, 0.04, 0.03, 0.04, 0.05, 0.04, 0.05, 0.05],
    [0.04, 0.04, 0.05, 0.03, 0.04, 0.05, 0.03, 0.03, 0.04, 0.03, 0.04, 0.04, 0.03, 0.04, 0.05, 0.04, 0.05, 0.06],
    [0.04, 0.04, 0.05, 0.03, 0.04, 0.05, 0.03, 0.03, 0.04, 0.03, 0.03, 0.04, 0.03, 0.04, 0.05, 0.04, 0.05, 0.05],
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.03, 0.03, 0.04, 0.03, 0.03, 0.04, 0.03, 0.04, 0.05, 0.04, 0.04, 0.05],
    // Feb-Mar-Apr/Aug-Sep-Oct Exposed (rows 12-19)
    [0.03, 0.04, 0.05, 0.01, 0.02, 0.03, 0.01, 0.01, 0.02, 0.01, 0.01, 0.02, 0.01, 0.02, 0.03, 0.03, 0.04, 0.05],
    [0.03, 0.04, 0.05, 0.03, 0.03, 0.04, 0.02, 0.03, 0.04, 0.02, 0.03, 0.04, 0.03, 0.03, 0.04, 0.03, 0.04, 0.05],
    [0.03, 0.04, 0.05, 0.01, 0.02, 0.03, 0.01, 0.01, 0.01, 0.01, 0.01, 0.02, 0.01, 0.02, 0.03, 0.03, 0.04, 0.05],
    [0.03, 0.03, 0.04, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.02, 0.03, 0.03, 0.04, 0.05, 0.03, 0.04, 0.06],
    [0.03, 0.04, 0.05, 0.01, 0.02, 0.02, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.02, 0.03, 0.03, 0.04, 0.05],
    [0.03, 0.04, 0.05, 0.01, 0.02, 0.02, 0.0, 0.01, 0.01, 0.0, 0.01, 0.01, 0.01, 0.02, 0.02, 0.03, 0.04, 0.05],
    [0.03, 0.04, 0.05, 0.01, 0.02, 0.03, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.02, 0.03, 0.03, 0.04, 0.05],
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.01, 0.02, 0.03, 0.01, 0.01, 0.01, 0.01, 0.01, 0.01, 0.03, 0.03, 0.04],
    // Feb-Mar-Apr/Aug-Sep-Oct Shaded (rows 20-23)
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.05, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.04, 0.05, 0.05, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.03, 0.04, 0.05, 0.04, 0.05, 0.06],
    // Nov-Dec-Jan Exposed (rows 24-31)
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.02, 0.03, 0.04, 0.02, 0.03, 0.04, 0.03, 0.04, 0.05, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.04, 0.02, 0.03, 0.03, 0.02, 0.03, 0.03, 0.03, 0.04, 0.05, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.02, 0.03, 0.04, 0.02, 0.02, 0.03, 0.03, 0.04, 0.04, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.02, 0.03, 0.03, 0.02, 0.02, 0.03, 0.03, 0.04, 0.04, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.02, 0.03, 0.03, 0.01, 0.01, 0.02, 0.01, 0.01, 0.02, 0.02, 0.03, 0.03, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.03, 0.04, 0.05, 0.02, 0.03, 0.03, 0.02, 0.03, 0.03, 0.03, 0.04, 0.04, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.03, 0.04, 0.04, 0.02, 0.02, 0.03, 0.02, 0.03, 0.04, 0.04, 0.05, 0.06],
    // Nov-Dec-Jan Shaded (rows 32-35)
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06],
    [0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06, 0.04, 0.05, 0.06],
];

/// Label arrays for UI/debugging.
const ASPECT_LABELS: &[&str] = &["North", "East", "South", "West"];
const DRY_BULB_LABELS: &[&str] = &[
    "10 - 29 °F", "30 - 49 °F", "50 - 69 °F",
    "70 - 89 °F", "90 - 109 °F", "> 109 °F",
];
const ELEVATION_LABELS: &[&str] = &[
    "Below (1000 - 2000 ft)", "Level (within 1000 ft)", "Above (1000 - 2000 ft)",
];
const MONTH_LABELS: &[&str] = &[
    "May June July", "Feb Mar Apr Aug Sep Oct", "Nov Dec Jan",
];
const RH_LABELS: &[&str] = &[
    "0 - 4 %", "5 - 9 %", "10 - 14 %", "15 - 19 %", "20 - 24 %",
    "25 - 29 %", "30 - 34 %", "35 - 39 %", "40 - 44 %", "45 - 49 %",
    "50 - 54 %", "55 - 59 %", "60 - 64 %", "65 - 69 %", "70 - 74 %",
    "75 - 79 %", "80 - 84 %", "85 - 89 %", "90 - 94 %", "95 - 99 %",
    "100 %",
];
const SHADING_LABELS: &[&str] = &["Exposed (< 50% shading)", "Shaded (>=50% shading)"];
const SLOPE_LABELS: &[&str] = &["0 - 30%", "31+ %"];
const TIME_OF_DAY_LABELS: &[&str] = &[
    "08:00 - 09:59", "10:00 - 11:59", "12:00 - 13:59",
    "14:00 - 15:59", "16:00 - 17:59", "18:00 - Sunset",
];

/// Fine dead fuel moisture lookup tool.
///
/// Calculates fuel moisture as reference moisture + correction moisture,
/// using 8-dimensional lookup tables indexed by aspect, dry bulb temperature,
/// elevation, month, relative humidity, shading, slope, and time of day.
#[derive(Debug, Clone)]
pub struct FineDeadFuelMoistureTool {
    reference_moisture: f64,   // fraction (base)
    correction_moisture: f64,  // fraction (base)
    fine_dead_fuel_moisture: f64, // fraction (base)
}

impl FineDeadFuelMoistureTool {
    pub fn new() -> Self {
        Self {
            reference_moisture: -0.01,
            correction_moisture: -0.01,
            fine_dead_fuel_moisture: -0.01,
        }
    }

    /// Calculate using typed enum indices.
    pub fn calculate(
        &mut self,
        aspect: AspectIndex,
        dry_bulb: DryBulbIndex,
        elevation: ElevationIndex,
        month: MonthIndex,
        rh: RHIndex,
        shading: ShadingIndex,
        slope: SlopeIndex,
        time_of_day: TimeOfDayIndex,
    ) {
        self.calculate_by_index(
            aspect as usize,
            dry_bulb as usize,
            elevation as usize,
            month as usize,
            rh as usize,
            shading as usize,
            slope as usize,
            time_of_day as usize,
        );
    }

    /// Calculate using raw integer indices.
    pub fn calculate_by_index(
        &mut self,
        aspect: usize,
        dry_bulb: usize,
        elevation: usize,
        month: usize,
        rh: usize,
        shading: usize,
        slope: usize,
        time_of_day: usize,
    ) {
        self.reference_moisture = -0.01;
        self.correction_moisture = -0.01;
        self.fine_dead_fuel_moisture = -0.01;

        // Bounds check
        if aspect < ASPECT_LABELS.len()
            && dry_bulb < DRY_BULB_LABELS.len()
            && elevation < ELEVATION_LABELS.len()
            && month < MONTH_LABELS.len()
            && rh < RH_LABELS.len()
            && shading < SHADING_LABELS.len()
            && slope < SLOPE_LABELS.len()
            && time_of_day < TIME_OF_DAY_LABELS.len()
        {
            // Reference moisture from dry bulb temp and RH
            self.reference_moisture = REFERENCE_MOISTURES[dry_bulb][rh];

            // Correction table row
            let row = if shading == 0 {
                slope + 2 * aspect
            } else {
                8 + aspect
            } + 12 * month;

            // Correction table column
            let column = elevation + 3 * time_of_day;

            self.correction_moisture = CORRECTION_MOISTURES[row][column];
            self.fine_dead_fuel_moisture = self.reference_moisture + self.correction_moisture;
        }
    }

    // --- Index size getters ---

    pub fn get_aspect_index_size(&self) -> usize { ASPECT_LABELS.len() }
    pub fn get_dry_bulb_index_size(&self) -> usize { DRY_BULB_LABELS.len() }
    pub fn get_elevation_index_size(&self) -> usize { ELEVATION_LABELS.len() }
    pub fn get_month_index_size(&self) -> usize { MONTH_LABELS.len() }
    pub fn get_rh_index_size(&self) -> usize { RH_LABELS.len() }
    pub fn get_shading_index_size(&self) -> usize { SHADING_LABELS.len() }
    pub fn get_slope_index_size(&self) -> usize { SLOPE_LABELS.len() }
    pub fn get_time_of_day_index_size(&self) -> usize { TIME_OF_DAY_LABELS.len() }

    // --- Label getters ---

    pub fn get_aspect_label(&self, index: usize) -> &'static str {
        ASPECT_LABELS.get(index).copied().unwrap_or("error")
    }

    pub fn get_dry_bulb_label(&self, index: usize) -> &'static str {
        DRY_BULB_LABELS.get(index).copied().unwrap_or("error")
    }

    pub fn get_elevation_label(&self, index: usize) -> &'static str {
        ELEVATION_LABELS.get(index).copied().unwrap_or("error")
    }

    pub fn get_month_label(&self, index: usize) -> &'static str {
        MONTH_LABELS.get(index).copied().unwrap_or("error")
    }

    pub fn get_rh_label(&self, index: usize) -> &'static str {
        RH_LABELS.get(index).copied().unwrap_or("error")
    }

    pub fn get_shading_label(&self, index: usize) -> &'static str {
        SHADING_LABELS.get(index).copied().unwrap_or("error")
    }

    pub fn get_slope_label(&self, index: usize) -> &'static str {
        SLOPE_LABELS.get(index).copied().unwrap_or("error")
    }

    pub fn get_time_of_day_label(&self, index: usize) -> &'static str {
        TIME_OF_DAY_LABELS.get(index).copied().unwrap_or("error")
    }

    // --- Output getters ---

    pub fn get_reference_moisture(&self, units: FractionUnits) -> f64 {
        units.from_base(self.reference_moisture)
    }

    pub fn get_correction_moisture(&self, units: FractionUnits) -> f64 {
        units.from_base(self.correction_moisture)
    }

    pub fn get_fine_dead_fuel_moisture(&self, units: FractionUnits) -> f64 {
        units.from_base(self.fine_dead_fuel_moisture)
    }
}

impl Default for FineDeadFuelMoistureTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test cases from testBehave.cpp testFineDeadFuelMoistureTool (lines 1690–1797).
    #[test]
    fn all_zero_indices() {
        let mut tool = FineDeadFuelMoistureTool::new();
        tool.calculate_by_index(0, 0, 0, 0, 0, 0, 0, 0);

        let ref_m = tool.get_reference_moisture(FractionUnits::Percent);
        let corr = tool.get_correction_moisture(FractionUnits::Percent);
        let fdfm = tool.get_fine_dead_fuel_moisture(FractionUnits::Percent);

        assert!(
            (ref_m - 1.0).abs() < 0.1,
            "ref moisture: expected ~1%, got {ref_m}"
        );
        assert!(
            (corr - 2.0).abs() < 0.1,
            "correction: expected ~2%, got {corr}"
        );
        assert!(
            (fdfm - 3.0).abs() < 0.1,
            "FDFM: expected ~3%, got {fdfm}"
        );
    }

    #[test]
    fn all_one_indices() {
        let mut tool = FineDeadFuelMoistureTool::new();
        tool.calculate_by_index(1, 1, 1, 1, 1, 1, 1, 1);

        let ref_m = tool.get_reference_moisture(FractionUnits::Percent);
        let corr = tool.get_correction_moisture(FractionUnits::Percent);
        let fdfm = tool.get_fine_dead_fuel_moisture(FractionUnits::Percent);

        assert!(
            (ref_m - 2.0).abs() < 0.1,
            "ref moisture: expected ~2%, got {ref_m}"
        );
        assert!(
            (corr - 4.0).abs() < 0.1,
            "correction: expected ~4%, got {corr}"
        );
        assert!(
            (fdfm - 6.0).abs() < 0.1,
            "FDFM: expected ~6%, got {fdfm}"
        );
    }

    #[test]
    fn all_max_indices() {
        let mut tool = FineDeadFuelMoistureTool::new();
        // Max valid indices: aspect=3, dryBulb=5, elevation=2, month=2, rh=20, shading=1, slope=1, timeOfDay=5
        tool.calculate_by_index(3, 5, 2, 2, 20, 1, 1, 5);

        let ref_m = tool.get_reference_moisture(FractionUnits::Percent);
        let corr = tool.get_correction_moisture(FractionUnits::Percent);
        let fdfm = tool.get_fine_dead_fuel_moisture(FractionUnits::Percent);

        assert!(
            (ref_m - 12.0).abs() < 0.1,
            "ref moisture: expected ~12%, got {ref_m}"
        );
        assert!(
            (corr - 6.0).abs() < 0.1,
            "correction: expected ~6%, got {corr}"
        );
        assert!(
            (fdfm - 18.0).abs() < 0.1,
            "FDFM: expected ~18%, got {fdfm}"
        );
    }

    #[test]
    fn out_of_bounds_indices() {
        let mut tool = FineDeadFuelMoistureTool::new();
        tool.calculate_by_index(99, 99, 99, 99, 99, 99, 99, 99);

        let ref_m = tool.get_reference_moisture(FractionUnits::Percent);
        let corr = tool.get_correction_moisture(FractionUnits::Percent);
        let fdfm = tool.get_fine_dead_fuel_moisture(FractionUnits::Percent);

        assert!(
            (ref_m - (-1.0)).abs() < 0.1,
            "ref moisture: expected ~-1%, got {ref_m}"
        );
        assert!(
            (corr - (-1.0)).abs() < 0.1,
            "correction: expected ~-1%, got {corr}"
        );
        assert!(
            (fdfm - (-1.0)).abs() < 0.1,
            "FDFM: expected ~-1%, got {fdfm}"
        );
    }

    #[test]
    fn typed_enum_calculate() {
        let mut tool = FineDeadFuelMoistureTool::new();
        tool.calculate(
            AspectIndex::North,
            DryBulbIndex::F10To29,
            ElevationIndex::Below1000To2000,
            MonthIndex::MayJunJul,
            RHIndex::Pct0To4,
            ShadingIndex::Exposed,
            SlopeIndex::ZeroTo30Percent,
            TimeOfDayIndex::H0800To0959,
        );

        let fdfm = tool.get_fine_dead_fuel_moisture(FractionUnits::Percent);
        assert!(
            (fdfm - 3.0).abs() < 0.1,
            "FDFM: expected ~3%, got {fdfm}"
        );
    }
}
