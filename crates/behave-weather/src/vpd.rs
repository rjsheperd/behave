//! Vapor Pressure Deficit calculator.
//!
//! C++ source: vaporPressureDeficitCalculator.h / vaporPressureDeficitCalculator.cpp

use firelab_base::{FractionUnits, PressureUnits, TemperatureUnits, UnitConversion};

/// Calculator for Vapor Pressure Deficit (VPD).
///
/// Uses the Magnus formula to compute saturated vapor pressure from
/// temperature, then derives actual vapor pressure from relative humidity.
#[derive(Debug, Clone)]
pub struct VaporPressureDeficitCalculator {
    temperature: f64,              // base units (°F)
    relative_humidity: f64,        // base units (fraction 0-1)
    actual_vapor_pressure: f64,    // base units (Pa)
    saturated_vapor_pressure: f64, // base units (Pa)
    vapor_pressure_deficit: f64,   // base units (Pa)
}

impl VaporPressureDeficitCalculator {
    pub fn new() -> Self {
        Self {
            temperature: 0.0,
            relative_humidity: 0.0,
            actual_vapor_pressure: 0.0,
            saturated_vapor_pressure: 0.0,
            vapor_pressure_deficit: 0.0,
        }
    }

    pub fn set_temperature(&mut self, temperature: f64, units: TemperatureUnits) {
        self.temperature = units.to_base(temperature);
    }

    pub fn set_relative_humidity(&mut self, rh: f64, units: FractionUnits) {
        self.relative_humidity = units.to_base(rh);
    }

    pub fn run_calculation(&mut self) {
        // Convert temperature to Celsius
        let temp_celsius = TemperatureUnits::Celsius.from_base(self.temperature);

        // Magnus formula denominator constant (NOT Kelvin conversion)
        let temp_k = temp_celsius + 237.3;

        // Saturated vapor pressure in hPa
        let svp = 6.11 * 10.0_f64.powf((7.5 * temp_celsius) / temp_k);

        // Store in base units (Pa)
        self.saturated_vapor_pressure = PressureUnits::HectoPascal.to_base(svp);

        // Actual vapor pressure in hPa
        let avp = self.relative_humidity * svp;
        self.actual_vapor_pressure = PressureUnits::HectoPascal.to_base(avp);

        // VPD in hPa, then to base units (Pa)
        let vpd = svp - avp;
        self.vapor_pressure_deficit = PressureUnits::HectoPascal.to_base(vpd);
    }

    pub fn get_vapor_pressure_deficit(&self, units: PressureUnits) -> f64 {
        units.from_base(self.vapor_pressure_deficit)
    }

    pub fn get_actual_vapor_pressure(&self, units: PressureUnits) -> f64 {
        units.from_base(self.actual_vapor_pressure)
    }

    pub fn get_saturated_vapor_pressure(&self, units: PressureUnits) -> f64 {
        units.from_base(self.saturated_vapor_pressure)
    }
}

impl Default for VaporPressureDeficitCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test cases from testBehave.cpp testVaporPressureDeficitCalculator (lines 1926–1942).
    /// All at 50°F, varying RH, tolerance = 1e-03.
    #[test]
    fn vpd_at_50f_varying_rh() {
        let expected: &[(f64, f64)] = &[
            (100.0, 0.000),
            (90.0, 1.228),
            (80.0, 2.457),
            (70.0, 3.685),
            (60.0, 4.913),
            (50.0, 6.142),
            (40.0, 7.370),
            (30.0, 8.598),
            (20.0, 9.827),
            (10.0, 11.055),
        ];

        for &(rh_pct, expected_vpd) in expected {
            let mut calc = VaporPressureDeficitCalculator::new();
            calc.set_temperature(50.0, TemperatureUnits::Fahrenheit);
            calc.set_relative_humidity(rh_pct, FractionUnits::Percent);
            calc.run_calculation();

            let vpd = calc.get_vapor_pressure_deficit(PressureUnits::HectoPascal);
            assert!(
                (vpd - expected_vpd).abs() < 1e-03,
                "RH={rh_pct}%: expected VPD={expected_vpd} hPa, got {vpd}"
            );
        }
    }
}
