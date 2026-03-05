//! Unit conversion types for the BehavePlus fire behavior library.
//!
//! Each unit enum corresponds to a physical quantity with a designated base unit.
//! The `UnitConversion` trait provides `to_base` / `from_base` for converting
//! between unit systems. Conversion constants are ported directly from the C++
//! source to guarantee numerical parity.
//!
//! C++ source: behaveUnits.h / behaveUnits.cpp

use std::f64::consts::PI;

/// Trait for converting between unit variants and a canonical base unit.
pub trait UnitConversion {
    /// Convert a value in this unit to the base unit.
    fn to_base(&self, value: f64) -> f64;
    /// Convert a value from the base unit to this unit.
    fn from_base(&self, value: f64) -> f64;
}

// ---------------------------------------------------------------------------
// Area — base: SquareFeet
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AreaUnits {
    SquareFeet,
    Acres,
    Hectares,
    SquareMeters,
    SquareMiles,
    SquareKilometers,
}

impl UnitConversion for AreaUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::SquareFeet => value,
            Self::Acres => value * 43560.002160576107,
            Self::Hectares => value * 107639.10416709723,
            Self::SquareMeters => value * 10.76391041671,
            Self::SquareMiles => value * 27878400.0,
            Self::SquareKilometers => value * 10763910.416709721,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::SquareFeet => value,
            Self::Acres => value * 2.295684e-05,
            Self::Hectares => value * 0.0000092903036,
            Self::SquareMeters => value * 0.0929030353835,
            Self::SquareMiles => value * 3.5870064279e-08,
            Self::SquareKilometers => value * 9.290304e-08,
        }
    }
}

// ---------------------------------------------------------------------------
// Basal Area — base: SquareFeetPerAcre
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasalAreaUnits {
    SquareFeetPerAcre,
    SquareMetersPerHectare,
}

impl UnitConversion for BasalAreaUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::SquareFeetPerAcre => value,
            // C++ uses this constant (ft²/ac → m²/ha direction name, but used in toBase)
            Self::SquareMetersPerHectare => value * 0.229568,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::SquareFeetPerAcre => value,
            Self::SquareMetersPerHectare => value * 4.356,
        }
    }
}

// ---------------------------------------------------------------------------
// Length — base: Feet
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnits {
    Feet,
    Inches,
    Millimeters,
    Centimeters,
    Meters,
    Chains,
    Miles,
    Kilometers,
}

impl UnitConversion for LengthUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Feet => value,
            Self::Inches => value * 0.08333333333333,
            Self::Millimeters => value * 0.003280839895,
            Self::Centimeters => value * 0.03280839895,
            Self::Meters => value * 3.2808398950131,
            Self::Chains => value * 66.0,
            Self::Miles => value * 5280.0,
            Self::Kilometers => value * 3280.8398950131,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Feet => value,
            Self::Inches => value * 12.0,
            Self::Millimeters => value * 304.8,
            Self::Centimeters => value * 30.480,
            Self::Meters => value * 0.3048,
            Self::Chains => value * 0.0151515151515,
            Self::Miles => value * 0.0001893939393939394,
            Self::Kilometers => value * 0.0003048,
        }
    }
}

// ---------------------------------------------------------------------------
// Loading — base: PoundsPerSquareFoot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingUnits {
    PoundsPerSquareFoot,
    TonsPerAcre,
    TonnesPerHectare,
    KilogramsPerSquareMeter,
}

impl UnitConversion for LoadingUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::PoundsPerSquareFoot => value,
            Self::TonsPerAcre => value * 0.045913682277318638,
            Self::TonnesPerHectare => value * 0.02048161436225217,
            Self::KilogramsPerSquareMeter => value * 0.2048161436225217,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::PoundsPerSquareFoot => value,
            Self::TonsPerAcre => value * 21.78,
            Self::TonnesPerHectare => value * 48.8242763638305,
            Self::KilogramsPerSquareMeter => value * 4.88242763638305,
        }
    }
}

// ---------------------------------------------------------------------------
// Pressure — base: Pascal
// NOTE: C++ toBaseUnits divides instead of multiplying (and fromBaseUnits
// multiplies instead of dividing). We preserve C++ behavior for parity.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureUnits {
    Pascal,
    HectoPascal,
    KiloPascal,
    MegaPascal,
    GigaPascal,
    Bar,
    Atmosphere,
    TechnicalAtmosphere,
    PoundPerSquareInch,
}

impl UnitConversion for PressureUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Pascal => value,
            Self::HectoPascal => value / 1e2,
            Self::KiloPascal => value / 1e3,
            Self::MegaPascal => value / 1e6,
            Self::GigaPascal => value / 1e9,
            Self::Bar => value / 1e5,
            Self::Atmosphere => value / 101325.0,
            Self::TechnicalAtmosphere => value / 98066.5,
            Self::PoundPerSquareInch => value / 6894.757,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Pascal => value,
            Self::HectoPascal => value * 1e2,
            Self::KiloPascal => value * 1e3,
            Self::MegaPascal => value * 1e6,
            Self::GigaPascal => value * 1e9,
            Self::Bar => value * 1e5,
            Self::Atmosphere => value * 101325.0,
            Self::TechnicalAtmosphere => value * 98066.5,
            Self::PoundPerSquareInch => value * 6894.757,
        }
    }
}

// ---------------------------------------------------------------------------
// Surface-Area-to-Volume Ratio — base: SquareFeetOverCubicFeet (1/ft)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceAreaToVolumeUnits {
    SquareFeetOverCubicFeet,
    SquareMetersOverCubicMeters,
    SquareInchesOverCubicInches,
    SquareCentimetersOverCubicCentimeters,
}

impl UnitConversion for SurfaceAreaToVolumeUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::SquareFeetOverCubicFeet => value,
            Self::SquareMetersOverCubicMeters => value * 3.280839895013123,
            Self::SquareInchesOverCubicInches => value * 0.083333333333333,
            Self::SquareCentimetersOverCubicCentimeters => value * 0.03280839895013123,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::SquareFeetOverCubicFeet => value,
            Self::SquareMetersOverCubicMeters => value * 0.3048,
            Self::SquareInchesOverCubicInches => value * 12.0,
            Self::SquareCentimetersOverCubicCentimeters => value * 30.48,
        }
    }
}

// ---------------------------------------------------------------------------
// Speed — base: FeetPerMinute
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedUnits {
    FeetPerMinute,
    ChainsPerHour,
    MetersPerSecond,
    MetersPerMinute,
    MetersPerHour,
    MilesPerHour,
    KilometersPerHour,
}

impl UnitConversion for SpeedUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::FeetPerMinute => value,
            Self::ChainsPerHour => value * 1.1,
            Self::MetersPerSecond => value * 196.8503937,
            Self::MetersPerMinute => value * 3.28084,
            Self::MetersPerHour => value * 0.0547,
            Self::MilesPerHour => value * 88.0,
            Self::KilometersPerHour => value * 54.680665,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::FeetPerMinute => value,
            Self::ChainsPerHour => value * (10.0 / 11.0),
            Self::MetersPerSecond => value * 0.00508,
            Self::MetersPerMinute => value * 0.3048,
            Self::MetersPerHour => value * 18.288,
            Self::MilesPerHour => value * 0.01136363636,
            Self::KilometersPerHour => value * 0.018288,
        }
    }
}

// ---------------------------------------------------------------------------
// Fraction — base: Fraction (0..1)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractionUnits {
    Fraction,
    Percent,
}

impl UnitConversion for FractionUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Fraction => value,
            Self::Percent => value / 100.0,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Fraction => value,
            Self::Percent => value * 100.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Slope — base: Degrees
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlopeUnits {
    Degrees,
    Percent,
}

impl UnitConversion for SlopeUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Degrees => value,
            Self::Percent => (180.0 / PI) * (value / 100.0).atan(),
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Degrees => value,
            Self::Percent => (value * (PI / 180.0)).tan() * 100.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Density — base: PoundsPerCubicFoot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DensityUnits {
    PoundsPerCubicFoot,
    KilogramsPerCubicMeter,
}

impl UnitConversion for DensityUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::PoundsPerCubicFoot => value,
            Self::KilogramsPerCubicMeter => value * 0.06242781786,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::PoundsPerCubicFoot => value,
            Self::KilogramsPerCubicMeter => value * 16.0185,
        }
    }
}

// ---------------------------------------------------------------------------
// Heat of Combustion — base: BtusPerPound
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatOfCombustionUnits {
    BtusPerPound,
    KilojoulesPerKilogram,
}

impl UnitConversion for HeatOfCombustionUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerPound => value,
            Self::KilojoulesPerKilogram => value * 0.429592,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerPound => value,
            Self::KilojoulesPerKilogram => value * 2.32779,
        }
    }
}

// ---------------------------------------------------------------------------
// Heat Sink — base: BtusPerCubicFoot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatSinkUnits {
    BtusPerCubicFoot,
    KilojoulesPerCubicMeter,
}

impl UnitConversion for HeatSinkUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerCubicFoot => value,
            Self::KilojoulesPerCubicMeter => value * 0.02681849745789,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerCubicFoot => value,
            Self::KilojoulesPerCubicMeter => value * 37.28769673134085,
        }
    }
}

// ---------------------------------------------------------------------------
// Heat Per Unit Area — base: BtusPerSquareFoot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatPerUnitAreaUnits {
    BtusPerSquareFoot,
    KilojoulesPerSquareMeter,
    KilowattSecondsPerSquareMeter,
}

impl UnitConversion for HeatPerUnitAreaUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerSquareFoot => value,
            Self::KilojoulesPerSquareMeter => value * 0.0879872,
            Self::KilowattSecondsPerSquareMeter => value * 0.0879872,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerSquareFoot => value,
            Self::KilojoulesPerSquareMeter => value * 11.3653,
            Self::KilowattSecondsPerSquareMeter => value * 11.3653,
        }
    }
}

// ---------------------------------------------------------------------------
// Heat Source / Reaction Intensity — base: BtusPerSquareFootPerMinute
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeatSourceAndReactionIntensityUnits {
    BtusPerSquareFootPerMinute,
    BtusPerSquareFootPerSecond,
    KilojoulesPerSquareMeterPerSecond,
    KilojoulesPerSquareMeterPerMinute,
    KilowattsPerSquareMeter,
}

impl UnitConversion for HeatSourceAndReactionIntensityUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerSquareFootPerMinute => value,
            Self::BtusPerSquareFootPerSecond => value * 60.0,
            Self::KilojoulesPerSquareMeterPerSecond => value * 5.27921783108615,
            Self::KilojoulesPerSquareMeterPerMinute => value * 0.0880549963329497,
            Self::KilowattsPerSquareMeter => value * 5.27921783108615,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerSquareFootPerMinute => value,
            Self::BtusPerSquareFootPerSecond => value * 0.01666666666666667,
            Self::KilojoulesPerSquareMeterPerSecond => value * 0.189422,
            Self::KilojoulesPerSquareMeterPerMinute => value * 11.356539,
            Self::KilowattsPerSquareMeter => value * 0.189422,
        }
    }
}

// ---------------------------------------------------------------------------
// Fireline Intensity — base: BtusPerFootPerSecond
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirelineIntensityUnits {
    BtusPerFootPerSecond,
    BtusPerFootPerMinute,
    KilojoulesPerMeterPerSecond,
    KilojoulesPerMeterPerMinute,
    KilowattsPerMeter,
}

impl UnitConversion for FirelineIntensityUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerFootPerSecond => value,
            Self::BtusPerFootPerMinute => value * 0.01666666666666667,
            // C++ uses kW/m constant for both kJ/m/s and kW/m
            Self::KilojoulesPerMeterPerSecond => value * 0.2886719,
            Self::KilojoulesPerMeterPerMinute => value * 0.00481120819,
            Self::KilowattsPerMeter => value * 0.2886719,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::BtusPerFootPerSecond => value,
            Self::BtusPerFootPerMinute => value * 60.0,
            // C++ uses kW/m constant for both kJ/m/s and kW/m
            Self::KilojoulesPerMeterPerSecond => value * 3.464140419,
            Self::KilojoulesPerMeterPerMinute => value * 207.848,
            Self::KilowattsPerMeter => value * 3.464140419,
        }
    }
}

// ---------------------------------------------------------------------------
// Temperature — base: Fahrenheit
// NOTE: Temperature does NOT short-circuit on value==0 (C++ behavior)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureUnits {
    Fahrenheit,
    Celsius,
    Kelvin,
}

impl UnitConversion for TemperatureUnits {
    fn to_base(&self, value: f64) -> f64 {
        match self {
            Self::Fahrenheit => value,
            Self::Celsius => ((value * 9.0) / 5.0) + 32.0,
            Self::Kelvin => (((value - 273.15) * 9.0) / 5.0) + 32.0,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        match self {
            Self::Fahrenheit => value,
            Self::Celsius => ((value - 32.0) * 5.0) / 9.0,
            Self::Kelvin => (((value - 32.0) * 5.0) / 9.0) + 273.15,
        }
    }
}

// ---------------------------------------------------------------------------
// Time — base: Minutes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnits {
    Minutes,
    Seconds,
    Hours,
    Days,
    Years,
}

impl UnitConversion for TimeUnits {
    fn to_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Minutes => value,
            Self::Seconds => value / 60.0,
            Self::Hours => value * 60.0,
            Self::Days => value * 1440.0,
            Self::Years => value * 525600.0,
        }
    }

    fn from_base(&self, value: f64) -> f64 {
        if value == 0.0 { return 0.0; }
        match self {
            Self::Minutes => value,
            Self::Seconds => value * 60.0,
            Self::Hours => value / 60.0,
            Self::Days => value / 1440.0,
            Self::Years => value / 525600.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1e-6;

    fn assert_near(a: f64, b: f64, tol: f64) {
        assert!(
            (a - b).abs() < tol,
            "expected {b}, got {a} (diff {})",
            (a - b).abs()
        );
    }

    // --- Speed unit tests (from RUST_PORT.org testSpeedUnitConversion) ---
    // 8.876216 ch/hr = 9.763838 ft/min (base)
    #[test]
    fn speed_chains_per_hour_to_base() {
        let fpm = SpeedUnits::ChainsPerHour.to_base(8.876216);
        assert_near(fpm, 9.763838, 0.001);
    }

    #[test]
    fn speed_base_to_chains_per_hour() {
        let cph = SpeedUnits::ChainsPerHour.from_base(9.763838);
        assert_near(cph, 8.876216, 0.001);
    }

    #[test]
    fn speed_base_to_km_per_hr() {
        let kph = SpeedUnits::KilometersPerHour.from_base(9.763838);
        assert_near(kph, 0.178561, 0.001);
    }

    #[test]
    fn speed_base_to_m_per_min() {
        let mpm = SpeedUnits::MetersPerMinute.from_base(9.763838);
        assert_near(mpm, 2.976018, 0.001);
    }

    #[test]
    fn speed_base_to_m_per_s() {
        let mps = SpeedUnits::MetersPerSecond.from_base(9.763838);
        assert_near(mps, 0.049600, 0.001);
    }

    #[test]
    fn speed_base_to_mph() {
        let mph = SpeedUnits::MilesPerHour.from_base(9.763838);
        assert_near(mph, 0.110953, 0.001);
    }

    // --- Length ---
    #[test]
    fn length_feet_round_trip() {
        assert_eq!(LengthUnits::Feet.to_base(100.0), 100.0);
        assert_eq!(LengthUnits::Feet.from_base(100.0), 100.0);
    }

    #[test]
    fn length_meters_to_feet() {
        assert_near(LengthUnits::Meters.to_base(1.0), 3.2808398950131, TOL);
    }

    #[test]
    fn length_feet_to_meters() {
        assert_near(LengthUnits::Meters.from_base(1.0), 0.3048, TOL);
    }

    #[test]
    fn length_chains_to_feet() {
        assert_near(LengthUnits::Chains.to_base(1.0), 66.0, TOL);
    }

    #[test]
    fn length_miles_to_feet() {
        assert_near(LengthUnits::Miles.to_base(1.0), 5280.0, TOL);
    }

    // --- Area ---
    #[test]
    fn area_acres_to_sq_feet() {
        assert_near(AreaUnits::Acres.to_base(1.0), 43560.002160576107, 0.01);
    }

    #[test]
    fn area_sq_feet_to_acres() {
        assert_near(AreaUnits::Acres.from_base(43560.0), 1.0, 0.001);
    }

    // --- Fraction ---
    #[test]
    fn fraction_percent_to_fraction() {
        assert_near(FractionUnits::Percent.to_base(50.0), 0.5, TOL);
    }

    #[test]
    fn fraction_fraction_to_percent() {
        assert_near(FractionUnits::Percent.from_base(0.5), 50.0, TOL);
    }

    // --- Slope ---
    #[test]
    fn slope_percent_to_degrees() {
        // 100% slope = 45 degrees
        assert_near(SlopeUnits::Percent.to_base(100.0), 45.0, 0.001);
    }

    #[test]
    fn slope_degrees_to_percent() {
        // 45 degrees = 100% slope
        assert_near(SlopeUnits::Percent.from_base(45.0), 100.0, 0.001);
    }

    #[test]
    fn slope_30_percent_round_trip() {
        let deg = SlopeUnits::Percent.to_base(30.0);
        let pct = SlopeUnits::Percent.from_base(deg);
        assert_near(pct, 30.0, 0.001);
    }

    // --- Temperature ---
    #[test]
    fn temperature_celsius_to_fahrenheit() {
        assert_near(TemperatureUnits::Celsius.to_base(0.0), 32.0, TOL);
        assert_near(TemperatureUnits::Celsius.to_base(100.0), 212.0, TOL);
    }

    #[test]
    fn temperature_fahrenheit_to_celsius() {
        assert_near(TemperatureUnits::Celsius.from_base(32.0), 0.0, TOL);
        assert_near(TemperatureUnits::Celsius.from_base(212.0), 100.0, TOL);
    }

    #[test]
    fn temperature_kelvin_to_fahrenheit() {
        // 273.15 K = 0°C = 32°F
        assert_near(TemperatureUnits::Kelvin.to_base(273.15), 32.0, TOL);
    }

    #[test]
    fn temperature_fahrenheit_to_kelvin() {
        assert_near(TemperatureUnits::Kelvin.from_base(32.0), 273.15, TOL);
    }

    // --- Time ---
    #[test]
    fn time_hours_to_minutes() {
        assert_near(TimeUnits::Hours.to_base(1.0), 60.0, TOL);
    }

    #[test]
    fn time_seconds_to_minutes() {
        assert_near(TimeUnits::Seconds.to_base(120.0), 2.0, TOL);
    }

    #[test]
    fn time_minutes_to_hours() {
        assert_near(TimeUnits::Hours.from_base(60.0), 1.0, TOL);
    }

    // --- Density ---
    #[test]
    fn density_kg_m3_to_lb_ft3() {
        assert_near(DensityUnits::KilogramsPerCubicMeter.to_base(1.0), 0.06242781786, TOL);
    }

    // --- Heat of Combustion ---
    #[test]
    fn heat_combustion_kj_kg_to_btu_lb() {
        assert_near(HeatOfCombustionUnits::KilojoulesPerKilogram.to_base(1.0), 0.429592, TOL);
    }

    // --- Fireline Intensity (from RUST_PORT.org testFirelineIntensity) ---
    #[test]
    fn fireline_intensity_btu_ft_s_to_kw_m() {
        // 598.339039 Btu/ft/s → 2072.730450 kW/m
        let kwm = FirelineIntensityUnits::KilowattsPerMeter.from_base(598.339039);
        assert_near(kwm, 2072.730450, 1.0);
    }

    // --- Zero passthrough ---
    #[test]
    fn zero_passthrough() {
        assert_eq!(LengthUnits::Meters.to_base(0.0), 0.0);
        assert_eq!(SpeedUnits::MilesPerHour.to_base(0.0), 0.0);
        assert_eq!(FractionUnits::Percent.to_base(0.0), 0.0);
        assert_eq!(TimeUnits::Hours.from_base(0.0), 0.0);
    }

    // --- Loading ---
    #[test]
    fn loading_tons_per_acre_round_trip() {
        let base = LoadingUnits::TonsPerAcre.to_base(1.0);
        let back = LoadingUnits::TonsPerAcre.from_base(base);
        assert_near(back, 1.0, 0.01);
    }

    // --- SAVR ---
    #[test]
    fn savr_m_to_ft_round_trip() {
        let base = SurfaceAreaToVolumeUnits::SquareMetersOverCubicMeters.to_base(1.0);
        let back = SurfaceAreaToVolumeUnits::SquareMetersOverCubicMeters.from_base(base);
        assert_near(back, 1.0, 0.01);
    }
}
