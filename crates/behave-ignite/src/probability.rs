//! Ignition probability calculations for firebrand and lightning ignition.
//!
//! C++ source: ignite.h / ignite.cpp

use firelab_base::{FractionUnits, LengthUnits, TemperatureUnits, UnitConversion};

use crate::inputs::{IgniteInputs, IgnitionFuelBedType, LightningCharge};

/// Ignition probability calculator.
///
/// Calculates probability of ignition from firebrands and lightning
/// based on fuel bed type, moisture, temperature, shade, and duff depth.
///
/// C++ class: `Ignite`
#[derive(Debug, Clone)]
pub struct Ignite {
    ignite_inputs: IgniteInputs,
    fuel_temperature: f64, // °F (base unit)
}

impl Ignite {
    pub fn new() -> Self {
        Self {
            ignite_inputs: IgniteInputs::new(),
            fuel_temperature: 0.0,
        }
    }

    pub fn initialize_members(&mut self) {
        self.ignite_inputs.initialize_members();
        self.fuel_temperature = 0.0;
    }

    // ---------------------------------------------------------------
    // Calculation methods
    // ---------------------------------------------------------------

    /// Calculate fuel temperature from air temperature and sun shade.
    /// Returns fuel temperature in °F (stored internally).
    fn calculate_fuel_temperature(&mut self) -> f64 {
        let sun_shade = self.ignite_inputs.sun_shade(FractionUnits::Fraction);
        let air_temperature = self.ignite_inputs.air_temperature(TemperatureUnits::Fahrenheit);

        let temperature_differential = 25.0 - (20.0 * sun_shade);
        self.fuel_temperature = air_temperature + temperature_differential;
        self.fuel_temperature
    }

    /// Calculate probability of ignition from a firebrand.
    pub fn calculate_firebrand_ignition_probability(
        &mut self,
        desired_units: FractionUnits,
    ) -> f64 {
        // Calculate fuel temperature in Celsius
        self.calculate_fuel_temperature();
        let fuel_temperature = TemperatureUnits::Celsius.from_base(self.fuel_temperature);

        // Use one hour moisture
        let fuel_moisture = self.ignite_inputs.moisture_one_hour(FractionUnits::Fraction);

        // Calculate heat of ignition
        let mut heat_of_ignition = 144.51
            - 0.26600 * fuel_temperature
            - 0.00058 * fuel_temperature * fuel_temperature
            - fuel_temperature * fuel_moisture
            + 18.5400 * (1.0 - (-15.1 * fuel_moisture).exp())
            + 640.000 * fuel_moisture;

        if heat_of_ignition > 400.0 {
            heat_of_ignition = 400.0;
        }

        let x = 0.1 * (400.0 - heat_of_ignition);
        let mut probability = (0.000048 * x.powf(4.3)) / 50.0;

        if probability > 1.0 {
            probability = 1.0;
        } else if probability < 0.0 {
            probability = 0.0;
        }

        desired_units.from_base(probability)
    }

    /// Calculate probability of ignition from lightning.
    pub fn calculate_lightning_ignition_probability(
        &mut self,
        desired_units: FractionUnits,
    ) -> f64 {
        // Probability of continuing current by charge type (Latham)
        const CC_NEG: f64 = 0.2;
        const CC_POS: f64 = 0.9;

        // Relative frequency by charge type (Latham and Schlieter)
        const FREQ_NEG: f64 = 0.723;
        const FREQ_POS: f64 = 0.277;

        // Convert duff depth to cm and multiply by 2.54 (matching C++),
        // then restrict to maximum of 10 cm.
        let mut duff_depth =
            self.ignite_inputs.duff_depth(LengthUnits::Centimeters);
        duff_depth *= 2.54;
        if duff_depth > 10.0 {
            duff_depth = 10.0;
        }

        // Use hundred hour moisture as duff moisture, convert to percent,
        // restrict to maximum of 40%.
        let mut fuel_moisture =
            self.ignite_inputs.moisture_hundred_hour(FractionUnits::Percent);
        if fuel_moisture > 40.0 {
            fuel_moisture = 40.0;
        }

        let fuel_type = self.ignite_inputs.fuel_bed_type();

        let (p_pos, p_neg) = match fuel_type {
            IgnitionFuelBedType::PonderosaPineLitter => (
                0.92 * (-0.087 * fuel_moisture).exp(),
                1.04 * (-0.054 * fuel_moisture).exp(),
            ),
            IgnitionFuelBedType::PunkyWoodRottenChunky => (
                0.44 * (-0.110 * fuel_moisture).exp(),
                0.59 * (-0.094 * fuel_moisture).exp(),
            ),
            IgnitionFuelBedType::PunkyWoodPowderDeep => (
                0.86 * (-0.060 * fuel_moisture).exp(),
                0.90 * (-0.056 * fuel_moisture).exp(),
            ),
            IgnitionFuelBedType::PunkWoodPowderShallow => (
                0.60 - (0.011 * fuel_moisture),
                0.73 - (0.011 * fuel_moisture),
            ),
            IgnitionFuelBedType::LodgepolePineDuff => (
                1.0 / (1.0 + (5.13 - 0.68 * duff_depth).exp()),
                1.0 / (1.0 + (3.84 - 0.60 * duff_depth).exp()),
            ),
            IgnitionFuelBedType::DouglasFirDuff => (
                1.0 / (1.0 + (6.69 - 1.39 * duff_depth).exp()),
                1.0 / (1.0 + (5.48 - 1.28 * duff_depth).exp()),
            ),
            IgnitionFuelBedType::HighAltitudeMixed => (
                0.62 * (-0.050 * fuel_moisture).exp(),
                0.80 - (0.014 * fuel_moisture),
            ),
            IgnitionFuelBedType::PeatMoss => (
                0.71 * (-0.070 * fuel_moisture).exp(),
                0.84 * (-0.060 * fuel_moisture).exp(),
            ),
        };

        let charge = self.ignite_inputs.lightning_charge_type();
        let mut probability = match charge {
            LightningCharge::Negative => CC_NEG * p_neg,
            LightningCharge::Positive => CC_POS * p_pos,
            LightningCharge::Unknown => {
                FREQ_POS * CC_POS * p_pos + FREQ_NEG * CC_NEG * p_neg
            }
        };

        if probability < 0.0 {
            probability = 0.0;
        }
        if probability > 1.0 {
            probability = 1.0;
        }

        desired_units.from_base(probability)
    }

    // ---------------------------------------------------------------
    // Pass-through setters
    // ---------------------------------------------------------------

    pub fn set_moisture_one_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.ignite_inputs.set_moisture_one_hour(moisture, units);
    }

    pub fn set_moisture_hundred_hour(&mut self, moisture: f64, units: FractionUnits) {
        self.ignite_inputs.set_moisture_hundred_hour(moisture, units);
    }

    pub fn set_air_temperature(&mut self, temperature: f64, units: TemperatureUnits) {
        self.ignite_inputs.set_air_temperature(temperature, units);
    }

    pub fn set_sun_shade(&mut self, shade: f64, units: FractionUnits) {
        self.ignite_inputs.set_sun_shade(shade, units);
    }

    pub fn set_fuel_bed_type(&mut self, fuel_bed_type: IgnitionFuelBedType) {
        self.ignite_inputs.set_fuel_bed_type(fuel_bed_type);
    }

    pub fn set_duff_depth(&mut self, depth: f64, units: LengthUnits) {
        self.ignite_inputs.set_duff_depth(depth, units);
    }

    pub fn set_lightning_charge_type(&mut self, charge: LightningCharge) {
        self.ignite_inputs.set_lightning_charge_type(charge);
    }

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
        self.ignite_inputs.update_ignite_inputs(
            moisture_one_hour, moisture_hundred_hour, moisture_units,
            air_temperature, temperature_units,
            sun_shade, sun_shade_units,
            fuel_bed_type,
            duff_depth, duff_depth_units,
            lightning_charge_type,
        );
    }

    // ---------------------------------------------------------------
    // Pass-through getters
    // ---------------------------------------------------------------

    pub fn air_temperature(&self, units: TemperatureUnits) -> f64 {
        self.ignite_inputs.air_temperature(units)
    }

    pub fn fuel_temperature(&self, units: TemperatureUnits) -> f64 {
        units.from_base(self.fuel_temperature)
    }

    pub fn moisture_one_hour(&self, units: FractionUnits) -> f64 {
        self.ignite_inputs.moisture_one_hour(units)
    }

    pub fn moisture_hundred_hour(&self, units: FractionUnits) -> f64 {
        self.ignite_inputs.moisture_hundred_hour(units)
    }

    pub fn sun_shade(&self, units: FractionUnits) -> f64 {
        self.ignite_inputs.sun_shade(units)
    }

    pub fn fuel_bed_type(&self) -> IgnitionFuelBedType {
        self.ignite_inputs.fuel_bed_type()
    }

    pub fn duff_depth(&self, units: LengthUnits) -> f64 {
        self.ignite_inputs.duff_depth(units)
    }

    pub fn lightning_charge_type(&self) -> LightningCharge {
        self.ignite_inputs.lightning_charge_type()
    }

    /// Whether fuel depth input is needed for the current fuel bed type.
    pub fn is_fuel_depth_needed(&self) -> bool {
        matches!(
            self.ignite_inputs.fuel_bed_type(),
            IgnitionFuelBedType::LodgepolePineDuff | IgnitionFuelBedType::DouglasFirDuff
        )
    }
}

impl Default for Ignite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firebrand_ignition_douglas_fir_duff() {
        let mut ignite = Ignite::new();
        ignite.update_ignite_inputs(
            6.0, 8.0, FractionUnits::Percent,
            80.0, TemperatureUnits::Fahrenheit,
            50.0, FractionUnits::Percent,
            IgnitionFuelBedType::DouglasFirDuff,
            6.0, LengthUnits::Inches,
            LightningCharge::Unknown,
        );

        let prob = ignite.calculate_firebrand_ignition_probability(FractionUnits::Fraction);
        assert!(
            (prob - 0.54831705).abs() < 1e-6,
            "firebrand ignition Douglas fir: expected 0.54831705, got {prob}"
        );
    }

    #[test]
    fn lightning_ignition_douglas_fir_duff() {
        let mut ignite = Ignite::new();
        ignite.update_ignite_inputs(
            6.0, 8.0, FractionUnits::Percent,
            80.0, TemperatureUnits::Fahrenheit,
            50.0, FractionUnits::Percent,
            IgnitionFuelBedType::DouglasFirDuff,
            6.0, LengthUnits::Inches,
            LightningCharge::Unknown,
        );

        let prob = ignite.calculate_lightning_ignition_probability(FractionUnits::Fraction);
        assert!(
            (prob - 0.39362018).abs() < 1e-6,
            "lightning ignition Douglas fir: expected 0.39362018, got {prob}"
        );
    }

    #[test]
    fn firebrand_ignition_lodgepole_pine_duff() {
        let mut ignite = Ignite::new();
        ignite.update_ignite_inputs(
            7.0, 9.0, FractionUnits::Percent,
            90.0, TemperatureUnits::Fahrenheit,
            25.0, FractionUnits::Percent,
            IgnitionFuelBedType::LodgepolePineDuff,
            8.0, LengthUnits::Inches,
            LightningCharge::Negative,
        );

        let prob = ignite.calculate_firebrand_ignition_probability(FractionUnits::Percent);
        assert!(
            (prob - 50.717573).abs() < 1e-4,
            "firebrand ignition lodgepole: expected 50.717573, got {prob}"
        );
    }

    #[test]
    fn lightning_ignition_lodgepole_pine_duff() {
        let mut ignite = Ignite::new();
        ignite.update_ignite_inputs(
            7.0, 9.0, FractionUnits::Percent,
            90.0, TemperatureUnits::Fahrenheit,
            25.0, FractionUnits::Percent,
            IgnitionFuelBedType::LodgepolePineDuff,
            8.0, LengthUnits::Inches,
            LightningCharge::Negative,
        );

        let prob = ignite.calculate_lightning_ignition_probability(FractionUnits::Percent);
        assert!(
            (prob - 17.931991).abs() < 1e-4,
            "lightning ignition lodgepole: expected 17.931991, got {prob}"
        );
    }

    #[test]
    fn is_fuel_depth_needed_check() {
        let mut ignite = Ignite::new();
        ignite.set_fuel_bed_type(IgnitionFuelBedType::DouglasFirDuff);
        assert!(ignite.is_fuel_depth_needed());

        ignite.set_fuel_bed_type(IgnitionFuelBedType::LodgepolePineDuff);
        assert!(ignite.is_fuel_depth_needed());

        ignite.set_fuel_bed_type(IgnitionFuelBedType::PonderosaPineLitter);
        assert!(!ignite.is_fuel_depth_needed());
    }
}
