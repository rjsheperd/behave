//! Ignition input parameters.
//!
//! C++ source: igniteInputs.h / igniteInputs.cpp

use firelab_base::{FractionUnits, LengthUnits, TemperatureUnits, UnitConversion};

/// Fuel bed type for ignition probability.
///
/// C++ source: `IgnitionFuelBedType` in igniteInputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnitionFuelBedType {
    PonderosaPineLitter = 0,
    PunkyWoodRottenChunky = 1,
    PunkyWoodPowderDeep = 2,
    PunkWoodPowderShallow = 3,
    LodgepolePineDuff = 4,
    DouglasFirDuff = 5,
    HighAltitudeMixed = 6,
    PeatMoss = 7,
}

/// Lightning charge type.
///
/// C++ source: `LightningCharge` in igniteInputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningCharge {
    Negative = 0,
    Positive = 1,
    Unknown = 2,
}

/// Inputs for ignition probability calculations.
///
/// All values stored in base units (fraction 0-1 for moisture/shade,
/// °F for temperature, ft for duff depth).
///
/// C++ class: `IgniteInputs`
#[derive(Debug, Clone)]
pub struct IgniteInputs {
    moisture_one_hour: f64,       // fraction 0-1
    moisture_hundred_hour: f64,   // fraction 0-1
    air_temperature: f64,         // °F
    sun_shade: f64,               // fraction 0-1
    fuel_bed_type: IgnitionFuelBedType,
    duff_depth: f64,              // ft
    lightning_charge_type: LightningCharge,
}

impl IgniteInputs {
    pub fn new() -> Self {
        Self {
            moisture_one_hour: 0.0,
            moisture_hundred_hour: 0.0,
            air_temperature: 0.0,
            sun_shade: 0.0,
            fuel_bed_type: IgnitionFuelBedType::PonderosaPineLitter,
            duff_depth: 0.0,
            lightning_charge_type: LightningCharge::Unknown,
        }
    }

    pub fn initialize_members(&mut self) {
        self.moisture_one_hour = 0.0;
        self.moisture_hundred_hour = 0.0;
        self.air_temperature = 0.0;
        self.sun_shade = 0.0;
        self.fuel_bed_type = IgnitionFuelBedType::PonderosaPineLitter;
        self.duff_depth = 0.0;
        self.lightning_charge_type = LightningCharge::Unknown;
    }

    // --- Setters ---

    pub fn set_moisture_one_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_one_hour = units.to_base(moisture);
    }

    pub fn set_moisture_hundred_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.moisture_hundred_hour = units.to_base(moisture);
    }

    pub fn set_air_temperature(&mut self, temperature: f64, units: TemperatureUnits) {
        self.air_temperature = units.to_base(temperature);
    }

    pub fn set_sun_shade(&mut self, shade: f64, units: FractionUnits) {
        self.sun_shade = units.to_base(shade);
    }

    pub fn set_fuel_bed_type(&mut self, fuel_bed_type: IgnitionFuelBedType) {
        self.fuel_bed_type = fuel_bed_type;
    }

    pub fn set_duff_depth(&mut self, depth: f64, units: LengthUnits) {
        self.duff_depth = units.to_base(depth);
    }

    pub fn set_lightning_charge_type(&mut self, charge: LightningCharge) {
        self.lightning_charge_type = charge;
    }

    /// Bulk update setter matching C++ `updateIgniteInputs`.
    pub fn update_ignite_inputs(
        &mut self,
        moisture_one_hour: f64,
        moisture_hundred_hour: f64,
        moisture_units: FractionUnits,
        air_temperature: f64,
        temperature_units: TemperatureUnits,
        sun_shade: f64,
        sun_shade_units: FractionUnits,
        fuel_bed_type: IgnitionFuelBedType,
        duff_depth: f64,
        duff_depth_units: LengthUnits,
        lightning_charge_type: LightningCharge,
    ) {
        self.set_moisture_one_hour(moisture_one_hour, moisture_units);
        self.set_moisture_hundred_hour(moisture_hundred_hour, moisture_units);
        self.set_air_temperature(air_temperature, temperature_units);
        self.set_sun_shade(sun_shade, sun_shade_units);
        self.fuel_bed_type = fuel_bed_type;
        self.set_duff_depth(duff_depth, duff_depth_units);
        self.lightning_charge_type = lightning_charge_type;
    }

    // --- Getters ---

    pub fn moisture_one_hour(&self, units: FractionUnits) -> f64 {
        units.from_base(self.moisture_one_hour)
    }

    pub fn moisture_hundred_hour(&self, units: FractionUnits) -> f64 {
        units.from_base(self.moisture_hundred_hour)
    }

    pub fn air_temperature(&self, units: TemperatureUnits) -> f64 {
        units.from_base(self.air_temperature)
    }

    pub fn sun_shade(&self, units: FractionUnits) -> f64 {
        units.from_base(self.sun_shade)
    }

    pub fn fuel_bed_type(&self) -> IgnitionFuelBedType {
        self.fuel_bed_type
    }

    pub fn duff_depth(&self, units: LengthUnits) -> f64 {
        units.from_base(self.duff_depth)
    }

    pub fn lightning_charge_type(&self) -> LightningCharge {
        self.lightning_charge_type
    }
}

impl Default for IgniteInputs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignite_inputs_default() {
        let ii = IgniteInputs::new();
        assert_eq!(ii.fuel_bed_type(), IgnitionFuelBedType::PonderosaPineLitter);
        assert_eq!(ii.lightning_charge_type(), LightningCharge::Unknown);
        assert_eq!(ii.moisture_one_hour(FractionUnits::Fraction), 0.0);
    }

    #[test]
    fn ignite_inputs_set_and_get() {
        let mut ii = IgniteInputs::new();
        ii.set_moisture_one_hour(0.06, FractionUnits::Fraction);
        ii.set_moisture_hundred_hour(0.08, FractionUnits::Fraction);
        ii.set_air_temperature(80.0, TemperatureUnits::Fahrenheit);
        ii.set_sun_shade(50.0, FractionUnits::Percent);
        ii.set_fuel_bed_type(IgnitionFuelBedType::DouglasFirDuff);
        ii.set_duff_depth(6.0, LengthUnits::Inches);
        ii.set_lightning_charge_type(LightningCharge::Unknown);

        assert!((ii.moisture_one_hour(FractionUnits::Fraction) - 0.06).abs() < 1e-10);
        assert!((ii.moisture_hundred_hour(FractionUnits::Fraction) - 0.08).abs() < 1e-10);
        assert!((ii.air_temperature(TemperatureUnits::Fahrenheit) - 80.0).abs() < 0.01);
        assert!((ii.sun_shade(FractionUnits::Percent) - 50.0).abs() < 0.01);
        assert_eq!(ii.fuel_bed_type(), IgnitionFuelBedType::DouglasFirDuff);
        assert!((ii.duff_depth(LengthUnits::Inches) - 6.0).abs() < 1e-10);
        assert_eq!(ii.lightning_charge_type(), LightningCharge::Unknown);
    }
}
