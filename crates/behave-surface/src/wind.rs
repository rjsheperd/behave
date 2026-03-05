//! Wind adjustment factor and wind speed utility calculations.
//!
//! The wind adjustment factor (WAF) converts 20-ft or 10-m wind speed to
//! midflame wind speed based on canopy cover, canopy height, crown ratio,
//! and fuel bed depth.
//!
//! C++ sources: windAdjustmentFactor.h / windAdjustmentFactor.cpp,
//!              windSpeedUtility.h / windSpeedUtility.cpp

/// Method for sheltering effects on WAF.
///
/// C++ source: `WindAdjustmentFactorShelterMethod` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindAdjustmentFactorShelterMethod {
    Unsheltered = 0,
    Sheltered = 1,
}

/// Calculates the wind adjustment factor (WAF) for converting 20-ft or 10-m
/// wind speed to midflame wind speed.
///
/// Based on Albini and Baughman (1979) when crown ratio is used,
/// Finney (1998, 2004) when crown ratio is not used.
///
/// C++ class: `WindAjustmentFactor` (note: C++ has a typo in class name)
#[derive(Debug, Clone)]
pub struct WindAdjustmentFactor {
    wind_adjustment_factor: f64,
    canopy_crown_fraction: f64,
    shelter_method: WindAdjustmentFactorShelterMethod,
}

impl Default for WindAdjustmentFactor {
    fn default() -> Self {
        Self::new()
    }
}

impl WindAdjustmentFactor {
    pub fn new() -> Self {
        Self {
            wind_adjustment_factor: 0.0,
            canopy_crown_fraction: 0.0,
            shelter_method: WindAdjustmentFactorShelterMethod::Unsheltered,
        }
    }

    /// Calculate WAF using crown ratio (Albini and Baughman 1979).
    ///
    /// All inputs in base units: canopy_cover as fraction [0,1],
    /// canopy_height in feet, crown_ratio as fraction [0,1],
    /// fuelbed_depth in feet.
    pub fn calculate_with_crown_ratio(
        &mut self,
        canopy_cover: f64,
        canopy_height: f64,
        crown_ratio: f64,
        fuelbed_depth: f64,
    ) -> f64 {
        // canopy_crown_fraction = fraction of volume under canopy top filled
        // with tree crowns (division by 3 assumes conical crown shapes)
        self.canopy_crown_fraction = crown_ratio * canopy_cover / 3.0;

        self.calculate_shelter_method(canopy_cover, canopy_height, fuelbed_depth);
        self.apply_log_profile(canopy_cover, canopy_height, fuelbed_depth);

        self.wind_adjustment_factor
    }

    /// Calculate WAF without crown ratio (Finney 1998, 2004).
    ///
    /// All inputs in base units: canopy_cover as fraction [0,1],
    /// canopy_height in feet, fuelbed_depth in feet.
    pub fn calculate_without_crown_ratio(
        &mut self,
        canopy_cover: f64,
        canopy_height: f64,
        fuelbed_depth: f64,
    ) -> f64 {
        // RMRS-RP-4, eq. 45
        self.canopy_crown_fraction = (canopy_cover * std::f64::consts::PI) / 12.0;

        self.calculate_shelter_method(canopy_cover, canopy_height, fuelbed_depth);
        self.apply_log_profile(canopy_cover, canopy_height, fuelbed_depth);

        self.wind_adjustment_factor
    }

    pub fn wind_adjustment_factor(&self) -> f64 {
        self.wind_adjustment_factor
    }

    pub fn canopy_crown_fraction(&self) -> f64 {
        self.canopy_crown_fraction
    }

    pub fn shelter_method(&self) -> WindAdjustmentFactorShelterMethod {
        self.shelter_method
    }

    // --- Internal ---

    fn calculate_shelter_method(
        &mut self,
        canopy_cover: f64,
        canopy_height: f64,
        _fuelbed_depth: f64,
    ) {
        // Unsheltered if: no canopy cover, or crown fraction < 5%, or canopy < 6 ft
        if canopy_cover < 1e-07
            || self.canopy_crown_fraction < 0.05
            || canopy_height < 6.0
        {
            self.shelter_method = WindAdjustmentFactorShelterMethod::Unsheltered;
        } else {
            self.shelter_method = WindAdjustmentFactorShelterMethod::Sheltered;
        }
    }

    fn apply_log_profile(
        &mut self,
        _canopy_cover: f64,
        canopy_height: f64,
        fuelbed_depth: f64,
    ) {
        if self.shelter_method == WindAdjustmentFactorShelterMethod::Unsheltered {
            if fuelbed_depth > 1e-07 {
                self.wind_adjustment_factor = 1.83
                    / ((20.0 + 0.36 * fuelbed_depth) / (0.13 * fuelbed_depth)).ln();
            }
        } else {
            // Sheltered
            self.wind_adjustment_factor = 0.555
                / ((self.canopy_crown_fraction * canopy_height).sqrt()
                    * ((20.0 + 0.36 * canopy_height) / (0.13 * canopy_height)).ln());
        }
    }
}

/// Utility for converting wind speeds between different height standards.
///
/// C++ class: `WindSpeedUtility`
pub struct WindSpeedUtility;

impl WindSpeedUtility {
    /// Convert 20-ft wind speed to midflame using wind adjustment factor.
    pub fn twenty_foot_to_midflame(twenty_foot_wind_speed: f64, waf: f64) -> f64 {
        twenty_foot_wind_speed * waf
    }

    /// Convert 10-m wind speed to 20-ft wind speed.
    /// The standard conversion factor is 1/1.15 (Lawson & Armitage 2008).
    pub fn ten_meter_to_twenty_foot(ten_meter_wind_speed: f64) -> f64 {
        ten_meter_wind_speed / 1.15
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

    #[test]
    fn unsheltered_no_canopy() {
        let mut waf = WindAdjustmentFactor::new();
        let result = waf.calculate_with_crown_ratio(0.0, 0.0, 0.0, 1.0);
        // No canopy cover → unsheltered, use log profile with fuelbed depth
        assert!(result > 0.0);
        assert_eq!(waf.shelter_method(), WindAdjustmentFactorShelterMethod::Unsheltered);
    }

    #[test]
    fn unsheltered_log_profile_depth_1ft() {
        let mut waf = WindAdjustmentFactor::new();
        let result = waf.calculate_with_crown_ratio(0.0, 0.0, 0.0, 1.0);
        // WAF = 1.83 / ln((20 + 0.36*1) / (0.13*1))
        // = 1.83 / ln(20.36/0.13) = 1.83 / ln(156.615)
        // = 1.83 / 5.055 = 0.362
        assert_near(result, 0.362, 0.01);
    }

    #[test]
    fn sheltered_with_crown_ratio() {
        let mut waf = WindAdjustmentFactor::new();
        // canopy_cover=0.5, height=60ft, crown_ratio=0.5, depth=1ft
        // crown_fraction = 0.5 * 0.5 / 3 = 0.0833
        // Sheltered: cover>0, fraction>=0.05, height>=6
        let result = waf.calculate_with_crown_ratio(0.5, 60.0, 0.5, 1.0);
        assert_eq!(waf.shelter_method(), WindAdjustmentFactorShelterMethod::Sheltered);
        assert!(result > 0.0 && result < 0.5, "WAF={}", result);
    }

    #[test]
    fn without_crown_ratio_finney() {
        let mut waf = WindAdjustmentFactor::new();
        // canopy_cover=0.5, height=60ft, depth=1ft
        // crown_fraction = (0.5 * PI) / 12 = 0.1309
        let result = waf.calculate_without_crown_ratio(0.5, 60.0, 1.0);
        assert_eq!(waf.shelter_method(), WindAdjustmentFactorShelterMethod::Sheltered);
        assert!(result > 0.0 && result < 0.5, "WAF={}", result);
    }

    #[test]
    fn low_canopy_is_unsheltered() {
        let mut waf = WindAdjustmentFactor::new();
        // canopy height < 6 ft → unsheltered regardless of cover/crown_ratio
        let _result = waf.calculate_with_crown_ratio(0.8, 5.0, 0.9, 1.0);
        assert_eq!(waf.shelter_method(), WindAdjustmentFactorShelterMethod::Unsheltered);
    }

    #[test]
    fn ten_meter_to_twenty_foot() {
        let result = WindSpeedUtility::ten_meter_to_twenty_foot(11.5);
        assert_near(result, 10.0, 0.001);
    }

    #[test]
    fn twenty_foot_to_midflame() {
        let result = WindSpeedUtility::twenty_foot_to_midflame(10.0, 0.4);
        assert_near(result, 4.0, 0.001);
    }
}
