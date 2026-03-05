//! Relative humidity calculation from dry/wet bulb temperatures.
//!
//! C++ source: relativeHumidity.h / relativeHumidity.cpp

use firelab_base::{FractionUnits, LengthUnits, TemperatureUnits, UnitConversion};

/// Calculates relative humidity and dew point from dry-bulb temperature,
/// wet-bulb temperature, and site elevation.
#[derive(Debug, Clone)]
pub struct RelativeHumidityTool {
    // Inputs (base units)
    dry_bulb_temperature: f64, // °F
    wet_bulb_temperature: f64, // °F
    site_elevation: f64,       // ft

    // Outputs (base units)
    dew_point_temperature: f64, // °F
    relative_humidity: f64,     // fraction 0-1
}

impl RelativeHumidityTool {
    pub fn new() -> Self {
        Self {
            dry_bulb_temperature: 0.0,
            wet_bulb_temperature: 0.0,
            site_elevation: 0.0,
            dew_point_temperature: 0.0,
            relative_humidity: 0.0,
        }
    }

    pub fn set_dry_bulb_temperature(&mut self, temp: f64, units: TemperatureUnits) {
        self.dry_bulb_temperature = units.to_base(temp);
    }

    pub fn set_wet_bulb_temperature(&mut self, temp: f64, units: TemperatureUnits) {
        self.wet_bulb_temperature = units.to_base(temp);
    }

    pub fn set_site_elevation(&mut self, elevation: f64, units: LengthUnits) {
        self.site_elevation = units.to_base(elevation);
    }

    pub fn calculate(&mut self) {
        let dry_bulb_c = TemperatureUnits::Celsius.from_base(self.dry_bulb_temperature);
        let wet_bulb_c = TemperatureUnits::Celsius.from_base(self.wet_bulb_temperature);
        let elevation_m = LengthUnits::Meters.from_base(self.site_elevation);

        // Calculate dew point
        let mut dew_point_c = dry_bulb_c;

        if wet_bulb_c < dry_bulb_c {
            let e2 = if wet_bulb_c < 0.0 {
                6.1115 * (22.452 * wet_bulb_c / (272.55 + wet_bulb_c)).exp()
            } else {
                6.1121 * (17.502 * wet_bulb_c / (240.97 + wet_bulb_c)).exp()
            };

            let p = 1013.0 * (-0.0000375 * elevation_m).exp();
            let d = 0.66 * (1.0 + 0.00115 * wet_bulb_c) * (dry_bulb_c - wet_bulb_c);
            let e3 = (e2 - d * p / 1000.0).max(0.001);

            dew_point_c = -240.97 / (1.0 - 17.502 / (e3 / 6.1121).ln());

            if dew_point_c < -40.0 {
                dew_point_c = -40.0;
            }
        }

        // Calculate relative humidity
        let dew_point_f = dew_point_c * 9.0 / 5.0 + 32.0;
        let dry_bulb_f = dry_bulb_c * 9.0 / 5.0 + 32.0;

        let rh = if dew_point_f >= dry_bulb_f {
            1.0
        } else {
            (-7469.0 / (dew_point_f + 398.0) + 7469.0 / (dry_bulb_f + 398.0)).exp()
        };

        // Store results
        self.dew_point_temperature = TemperatureUnits::Celsius.to_base(dew_point_c);
        self.relative_humidity = FractionUnits::Fraction.to_base(rh);
    }

    pub fn get_dry_bulb_temperature(&self, units: TemperatureUnits) -> f64 {
        units.from_base(self.dry_bulb_temperature)
    }

    pub fn get_wet_bulb_temperature(&self, units: TemperatureUnits) -> f64 {
        units.from_base(self.wet_bulb_temperature)
    }

    pub fn get_site_elevation(&self, units: LengthUnits) -> f64 {
        units.from_base(self.site_elevation)
    }

    pub fn get_dew_point_temperature(&self, units: TemperatureUnits) -> f64 {
        units.from_base(self.dew_point_temperature)
    }

    pub fn get_relative_humidity(&self, units: FractionUnits) -> f64 {
        units.from_base(self.relative_humidity)
    }

    pub fn get_wet_bulb_depression(&self, units: TemperatureUnits) -> f64 {
        let dry_c = TemperatureUnits::Celsius.from_base(self.dry_bulb_temperature);
        let wet_c = TemperatureUnits::Celsius.from_base(self.wet_bulb_temperature);
        let dep_c = dry_c - wet_c;

        match units {
            TemperatureUnits::Celsius => dep_c,
            _ => dep_c * 1.8, // Fahrenheit difference
        }
    }
}

impl Default for RelativeHumidityTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rh_equal_temps_gives_100_percent() {
        let mut tool = RelativeHumidityTool::new();
        tool.set_dry_bulb_temperature(70.0, TemperatureUnits::Fahrenheit);
        tool.set_wet_bulb_temperature(70.0, TemperatureUnits::Fahrenheit);
        tool.set_site_elevation(0.0, LengthUnits::Feet);
        tool.calculate();

        let rh = tool.get_relative_humidity(FractionUnits::Percent);
        assert!(
            (rh - 100.0).abs() < 0.1,
            "expected ~100%, got {rh}"
        );
    }

    #[test]
    fn rh_basic_calculation() {
        let mut tool = RelativeHumidityTool::new();
        tool.set_dry_bulb_temperature(80.0, TemperatureUnits::Fahrenheit);
        tool.set_wet_bulb_temperature(60.0, TemperatureUnits::Fahrenheit);
        tool.set_site_elevation(5000.0, LengthUnits::Feet);
        tool.calculate();

        let rh = tool.get_relative_humidity(FractionUnits::Percent);
        // RH should be somewhere between 0 and 100 with wet bulb < dry bulb
        assert!(rh > 0.0 && rh < 100.0, "RH should be between 0 and 100, got {rh}");

        let dew_point = tool.get_dew_point_temperature(TemperatureUnits::Fahrenheit);
        // Dew point should be below dry bulb
        assert!(dew_point < 80.0, "dew point should be < dry bulb, got {dew_point}");
    }

    #[test]
    fn wet_bulb_depression() {
        let mut tool = RelativeHumidityTool::new();
        tool.set_dry_bulb_temperature(80.0, TemperatureUnits::Fahrenheit);
        tool.set_wet_bulb_temperature(70.0, TemperatureUnits::Fahrenheit);

        let dep_f = tool.get_wet_bulb_depression(TemperatureUnits::Fahrenheit);
        // 80°F = 26.667°C, 70°F = 21.111°C, diff = 5.556°C * 1.8 = 10°F
        assert!(
            (dep_f - 10.0).abs() < 0.1,
            "expected ~10°F depression, got {dep_f}"
        );
    }
}
