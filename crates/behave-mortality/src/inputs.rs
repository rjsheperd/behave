//! Mortality input parameters.
//!
//! C++ source: mortality_inputs.h / mortality_inputs.cpp

use firelab_base::{
    AreaUnits, FractionUnits, LengthUnits, SpeedUnits, TemperatureUnits,
    FirelineIntensityUnits, UnitConversion,
};

use crate::equations::{
    CrownDamageEquationCode, CrownDamageType, EquationType, NUM_REQUIRED_FIELDS,
};
use crate::species::Gacc;

/// Beetle damage classification.
///
/// C++ source: `BeetleDamage` in mortality_inputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeetleDamage {
    NotSet = -1,
    No = 0,
    Yes = 1,
}

/// Fire severity classification for mortality equation #4.
///
/// C++ source: `FireSeverity` in mortality_inputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireSeverity {
    NotSet = -1,
    Empty = 0,
    Low = 1,
}

/// Whether input uses flame length or scorch height.
///
/// C++ source: `FlameLengthOrScorchHeightSwitch` in mortality_inputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlameLengthOrScorchHeightSwitch {
    FlameLength,
    ScorchHeight,
}

/// All user-facing inputs for a mortality calculation.
///
/// C++ class: `MortalityInputs`
#[derive(Debug, Clone)]
pub struct MortalityInputs {
    // --- Core inputs (stored in base units) ---
    region: Gacc,
    species_code: String,
    equation_type: EquationType,
    flame_length_or_scorch_height_switch: FlameLengthOrScorchHeightSwitch,
    flame_length_or_scorch_height_value: f64, // deprecated, base units (ft)
    flame_length: f64,       // ft (base)
    scorch_height: f64,      // ft (base)
    density_per_acre: f64,   // trees per acre (stored via area base units)
    dbh: f64,                // ft (base)
    tree_height: f64,        // ft (base)
    crown_ratio: f64,        // fraction 0-1 (base)
    crown_damage: f64,       // percent 0-100
    cambium_kill_rating: f64, // 0-4
    beetle_damage: BeetleDamage,
    bole_char_height: f64,   // ft (base)
    crown_scorch_or_bole_char_equation_number: i32,
    crown_damage_equation_code: CrownDamageEquationCode,
    crown_damage_type: CrownDamageType,
    fire_severity: FireSeverity,
    bark_thickness: f64,     // ft (base) — calculated, not user input
    fireline_intensity: f64, // btu/ft/s (base)
    mid_flame_wind_speed: f64, // ft/min (base)
    air_temperature: f64,    // °F (base)

    /// Required field bitvector for the current equation.
    pub is_field_required: [bool; NUM_REQUIRED_FIELDS],
}

impl MortalityInputs {
    pub fn new() -> Self {
        let mut fields = [false; NUM_REQUIRED_FIELDS];
        // Always required
        fields[0] = true; // Region
        fields[1] = true; // FlameLengthOrScorchHeightSwitch
        fields[2] = true; // FlameLengthOrScorchHeightValue
        fields[3] = true; // EquationType

        Self {
            region: Gacc::NotSet,
            species_code: String::new(),
            equation_type: EquationType::NotSet,
            flame_length_or_scorch_height_switch: FlameLengthOrScorchHeightSwitch::FlameLength,
            flame_length_or_scorch_height_value: -1.0,
            flame_length: -1.0,
            scorch_height: -1.0,
            density_per_acre: -1.0,
            dbh: -1.0,
            tree_height: -1.0,
            crown_ratio: -1.0,
            crown_damage: -1.0,
            cambium_kill_rating: -1.0,
            beetle_damage: BeetleDamage::NotSet,
            bole_char_height: -1.0,
            crown_scorch_or_bole_char_equation_number: -1,
            crown_damage_equation_code: CrownDamageEquationCode::NotSet,
            crown_damage_type: CrownDamageType::NotSet,
            fire_severity: FireSeverity::NotSet,
            bark_thickness: -1.0,
            fireline_intensity: -1.0,
            mid_flame_wind_speed: -1.0,
            air_temperature: -1.0,
            is_field_required: fields,
        }
    }

    // --- Setters ---

    pub fn set_region(&mut self, region: Gacc) {
        self.region = region;
    }

    pub fn set_species_code(&mut self, code: &str) {
        self.species_code = code.to_string();
    }

    pub fn set_equation_type(&mut self, eq_type: EquationType) {
        self.equation_type = eq_type;
    }

    pub fn set_flame_length_or_scorch_height_switch(
        &mut self,
        switch: FlameLengthOrScorchHeightSwitch,
    ) {
        self.flame_length_or_scorch_height_switch = switch;
    }

    pub fn set_flame_length_or_scorch_height_value(&mut self, value: f64, units: LengthUnits) {
        self.flame_length_or_scorch_height_value = units.to_base(value);
    }

    pub fn set_flame_length(&mut self, flame_length: f64, units: LengthUnits) {
        self.flame_length = units.to_base(flame_length);
    }

    pub fn set_scorch_height(&mut self, scorch_height: f64, units: LengthUnits) {
        self.scorch_height = units.to_base(scorch_height);
    }

    pub fn set_tree_density_per_unit_area(&mut self, number_of_trees: f64, units: AreaUnits) {
        self.density_per_acre = units.to_base(number_of_trees);
    }

    pub fn set_dbh(&mut self, dbh: f64, units: LengthUnits) {
        self.dbh = units.to_base(dbh);
    }

    pub fn set_tree_height(&mut self, tree_height: f64, units: LengthUnits) {
        self.tree_height = units.to_base(tree_height);
    }

    pub fn set_crown_ratio(&mut self, crown_ratio: f64, units: FractionUnits) {
        self.crown_ratio = units.to_base(crown_ratio);
    }

    pub fn set_crown_damage(&mut self, crown_damage: f64) {
        self.crown_damage = crown_damage;
    }

    pub fn set_cambium_kill_rating(&mut self, cambium_kill_rating: f64) {
        self.cambium_kill_rating = cambium_kill_rating;
    }

    pub fn set_beetle_damage(&mut self, beetle_damage: BeetleDamage) {
        self.beetle_damage = beetle_damage;
    }

    pub fn set_bole_char_height(&mut self, bole_char_height: f64, units: LengthUnits) {
        self.bole_char_height = units.to_base(bole_char_height);
    }

    pub fn set_crown_scorch_or_bole_char_equation_number(&mut self, number: i32) {
        self.crown_scorch_or_bole_char_equation_number = number;
    }

    pub fn set_crown_damage_equation_code(&mut self, code: CrownDamageEquationCode) {
        self.crown_damage_equation_code = code;
    }

    pub fn set_crown_damage_type(&mut self, damage_type: CrownDamageType) {
        self.crown_damage_type = damage_type;
    }

    pub fn set_fire_severity(&mut self, severity: FireSeverity) {
        self.fire_severity = severity;
    }

    pub fn set_bark_thickness(&mut self, bark_thickness: f64, units: LengthUnits) {
        self.bark_thickness = units.to_base(bark_thickness);
    }

    pub fn set_fireline_intensity(&mut self, intensity: f64, units: FirelineIntensityUnits) {
        self.fireline_intensity = units.to_base(intensity);
    }

    pub fn set_mid_flame_wind_speed(&mut self, speed: f64, units: SpeedUnits) {
        self.mid_flame_wind_speed = units.to_base(speed);
    }

    pub fn set_air_temperature(&mut self, temp: f64, units: TemperatureUnits) {
        self.air_temperature = units.to_base(temp);
    }

    // --- Getters ---

    pub fn get_region(&self) -> Gacc {
        self.region
    }

    pub fn get_species_code(&self) -> &str {
        &self.species_code
    }

    pub fn get_equation_type(&self) -> EquationType {
        self.equation_type
    }

    pub fn get_flame_length_or_scorch_height_switch(&self) -> FlameLengthOrScorchHeightSwitch {
        self.flame_length_or_scorch_height_switch
    }

    pub fn get_flame_length_or_scorch_height_value(&self, units: LengthUnits) -> f64 {
        units.from_base(self.flame_length_or_scorch_height_value)
    }

    pub fn get_flame_length(&self, units: LengthUnits) -> f64 {
        units.from_base(self.flame_length)
    }

    pub fn get_scorch_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.scorch_height)
    }

    pub fn get_tree_density_per_unit_area(&self, units: AreaUnits) -> f64 {
        units.from_base(self.density_per_acre)
    }

    pub fn get_dbh(&self, units: LengthUnits) -> f64 {
        units.from_base(self.dbh)
    }

    pub fn get_tree_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.tree_height)
    }

    pub fn get_crown_ratio(&self, units: FractionUnits) -> f64 {
        units.from_base(self.crown_ratio)
    }

    pub fn get_crown_damage(&self) -> f64 {
        self.crown_damage
    }

    pub fn get_cambium_kill_rating(&self) -> f64 {
        self.cambium_kill_rating
    }

    pub fn get_beetle_damage(&self) -> BeetleDamage {
        self.beetle_damage
    }

    pub fn get_bole_char_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.bole_char_height)
    }

    pub fn get_crown_scorch_or_bole_char_equation_number(&self) -> i32 {
        self.crown_scorch_or_bole_char_equation_number
    }

    pub fn get_crown_damage_equation_code(&self) -> CrownDamageEquationCode {
        self.crown_damage_equation_code
    }

    pub fn get_crown_damage_type(&self) -> CrownDamageType {
        self.crown_damage_type
    }

    pub fn get_fire_severity(&self) -> FireSeverity {
        self.fire_severity
    }

    pub fn get_bark_thickness(&self, units: LengthUnits) -> f64 {
        units.from_base(self.bark_thickness)
    }

    pub fn get_fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        if self.fireline_intensity == -1.0 {
            -1.0
        } else {
            units.from_base(self.fireline_intensity)
        }
    }

    pub fn get_mid_flame_wind_speed(&self, units: SpeedUnits) -> f64 {
        if self.mid_flame_wind_speed == -1.0 {
            -1.0
        } else {
            units.from_base(self.mid_flame_wind_speed)
        }
    }

    pub fn get_air_temperature(&self, units: TemperatureUnits) -> f64 {
        if self.air_temperature == -1.0 {
            -1.0
        } else {
            units.from_base(self.air_temperature)
        }
    }
}

impl Default for MortalityInputs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inputs_default_values() {
        let inputs = MortalityInputs::new();
        assert_eq!(inputs.get_equation_type(), EquationType::NotSet);
        assert_eq!(inputs.get_beetle_damage(), BeetleDamage::NotSet);
        assert_eq!(inputs.get_fire_severity(), FireSeverity::NotSet);
        assert!((inputs.get_dbh(LengthUnits::Feet) - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn set_and_get_dbh() {
        let mut inputs = MortalityInputs::new();
        inputs.set_dbh(10.0, LengthUnits::Inches);
        let dbh = inputs.get_dbh(LengthUnits::Inches);
        assert!((dbh - 10.0).abs() < 1e-6, "expected 10.0, got {dbh}");
    }

    #[test]
    fn set_and_get_crown_ratio() {
        let mut inputs = MortalityInputs::new();
        inputs.set_crown_ratio(0.5, FractionUnits::Fraction);
        let cr = inputs.get_crown_ratio(FractionUnits::Fraction);
        assert!((cr - 0.5).abs() < 1e-6, "expected 0.5, got {cr}");
    }
}
