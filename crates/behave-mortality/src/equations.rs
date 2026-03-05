//! Mortality equation table and equation type definitions.
//!
//! Defines the required input fields for each mortality equation type
//! (crown scorch, bole char, crown damage species-specific).
//!
//! C++ source: mortality_equation_table.h / mortality_equation_table.cpp

/// Crown damage type for post-fire equations.
///
/// C++ source: `CrownDamageType` in mortality_equation_table.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrownDamageType {
    NotSet = -1,
    CrownLength = 0,
    CrownVolume = 1,
    CrownKill = 2,
}

/// Crown damage equation species codes.
///
/// C++ source: `CrownDamageEquationCode` in mortality_equation_table.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrownDamageEquationCode {
    NotSet = -1,
    WhiteFir = 0,
    SubalpineFir = 1,
    IncenseCedar = 2,
    WesternLarch = 3,
    WhitebarkPine = 4,
    EngelmannSpruce = 5,
    SugarPine = 6,
    RedFir = 7,
    PonderosaPine = 8,
    PonderosaKill = 9,
    DouglasFir = 10,
}

/// Primary mortality equation classification.
///
/// C++ source: `EquationType` in mortality_equation_table.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquationType {
    NotSet = -1,
    CrownScorch = 0,
    BoleChar = 1,
    CrownDamage = 2,
}

/// Required input field identifiers.
///
/// C++ source: `RequiredFieldNames` in mortality_equation_table.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredFieldName {
    Region = 0,
    FlameLengthOrScorchHeightSwitch = 1,
    FlameLengthOrScorchHeightValue = 2,
    EquationType = 3,
    Dbh = 4,
    TreeHeight = 5,
    CrownRatio = 6,
    CrownDamage = 7,
    CambiumKillRating = 8,
    BeetleDamage = 9,
    BoleCharHeight = 10,
    BarkThickness = 11,
    FireSeverity = 12,
}

/// Number of input fields (used for required-field bitvector sizing).
pub const NUM_REQUIRED_FIELDS: usize = 13;

/// Bole char equation coefficients.
///
/// C++ struct: `BoleCharCoefficientTableRecord`
#[derive(Debug, Clone)]
pub struct BoleCharCoefficientRecord {
    pub equation_number: i32,
    pub b1: f64,
    pub b2: f64,
    pub b3: f64,
}

/// A single row in the equation required field table.
///
/// C++ class: `EquationRequiredFieldTableRecord`
#[derive(Debug, Clone)]
pub struct EquationRequiredFieldRecord {
    pub equation_type: EquationType,
    pub crown_damage_type: CrownDamageType,
    pub crown_damage_equation_code: CrownDamageEquationCode,
    pub required_fields: [bool; NUM_REQUIRED_FIELDS],
}

/// Table mapping equation types → required input fields.
///
/// C++ class: `EquationRequiredFieldTable`
#[derive(Debug, Clone)]
pub struct EquationRequiredFieldTable {
    records: Vec<EquationRequiredFieldRecord>,
}

impl EquationRequiredFieldTable {
    pub fn new() -> Self {
        let mut table = Self { records: Vec::new() };
        table.populate();
        table
    }

    /// Get the required field bitvector for a given equation type and crown damage code.
    pub fn get_required_fields(
        &self,
        equation_type: EquationType,
        crown_damage_code: CrownDamageEquationCode,
    ) -> [bool; NUM_REQUIRED_FIELDS] {
        for r in &self.records {
            if r.equation_type == equation_type
                && r.crown_damage_equation_code == crown_damage_code
            {
                return r.required_fields;
            }
        }
        // Default: only always-required fields
        Self::default_required()
    }

    /// Get crown damage type for a given equation type and code.
    /// Returns NotSet for crown_damage equation types (matching C++ behavior).
    pub fn get_crown_damage_type(
        &self,
        equation_type: EquationType,
        crown_damage_code: CrownDamageEquationCode,
    ) -> CrownDamageType {
        if equation_type == EquationType::CrownDamage {
            // C++ returns not_set for crown_damage equation type (conditional check is inverted)
            // Actually the C++ checks `if(equationType != EquationType::crown_damage)` then searches
            return CrownDamageType::NotSet;
        }
        for r in &self.records {
            if r.equation_type == equation_type
                && r.crown_damage_equation_code == crown_damage_code
            {
                return r.crown_damage_type;
            }
        }
        CrownDamageType::NotSet
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    // --- Internal ---

    fn default_required() -> [bool; NUM_REQUIRED_FIELDS] {
        let mut fields = [false; NUM_REQUIRED_FIELDS];
        fields[RequiredFieldName::Region as usize] = true;
        fields[RequiredFieldName::EquationType as usize] = true;
        fields[RequiredFieldName::FlameLengthOrScorchHeightSwitch as usize] = true;
        fields[RequiredFieldName::FlameLengthOrScorchHeightValue as usize] = true;
        fields
    }

    fn add_record(
        &mut self,
        equation_type: EquationType,
        crown_damage_type: CrownDamageType,
        crown_damage_code: CrownDamageEquationCode,
        extra_fields: &[RequiredFieldName],
    ) {
        let mut fields = Self::default_required();
        for f in extra_fields {
            fields[*f as usize] = true;
        }
        self.records.push(EquationRequiredFieldRecord {
            equation_type,
            crown_damage_type,
            crown_damage_equation_code: crown_damage_code,
            required_fields: fields,
        });
    }

    fn populate(&mut self) {
        use CrownDamageEquationCode as CD;
        use CrownDamageType as CDT;
        use EquationType as ET;
        use RequiredFieldName as F;

        // Crown Scorch: dbh, tree_height, crown_ratio
        self.add_record(ET::CrownScorch, CDT::NotSet, CD::NotSet,
            &[F::Dbh, F::TreeHeight, F::CrownRatio]);

        // Bole Char: dbh, bole_char_height
        self.add_record(ET::BoleChar, CDT::NotSet, CD::NotSet,
            &[F::Dbh, F::BoleCharHeight]);

        // White Fir: crown_length, dbh, crown_damage, cambium_kill, beetle_damage
        self.add_record(ET::CrownDamage, CDT::CrownLength, CD::WhiteFir,
            &[F::CrownDamage, F::Dbh, F::CambiumKillRating, F::BeetleDamage]);

        // Subalpine Fir: crown_volume, dbh, crown_damage, cambium_kill
        self.add_record(ET::CrownDamage, CDT::CrownVolume, CD::SubalpineFir,
            &[F::CrownDamage, F::Dbh, F::CambiumKillRating]);

        // Incense Cedar: crown_length, crown_damage, cambium_kill
        self.add_record(ET::CrownDamage, CDT::CrownLength, CD::IncenseCedar,
            &[F::CrownDamage, F::CambiumKillRating]);

        // Western Larch: crown_volume, crown_damage, cambium_kill
        self.add_record(ET::CrownDamage, CDT::CrownVolume, CD::WesternLarch,
            &[F::CrownDamage, F::CambiumKillRating]);

        // Whitebark Pine: crown_volume, dbh, crown_damage, cambium_kill
        self.add_record(ET::CrownDamage, CDT::CrownVolume, CD::WhitebarkPine,
            &[F::CrownDamage, F::Dbh, F::CambiumKillRating]);

        // Engelmann Spruce: crown_volume, crown_damage, cambium_kill
        self.add_record(ET::CrownDamage, CDT::CrownVolume, CD::EngelmannSpruce,
            &[F::CrownDamage, F::CambiumKillRating]);

        // Sugar Pine: crown_length, dbh, crown_damage, cambium_kill, beetle_damage
        self.add_record(ET::CrownDamage, CDT::CrownLength, CD::SugarPine,
            &[F::CrownDamage, F::Dbh, F::CambiumKillRating, F::BeetleDamage]);

        // Red Fir: crown_length, crown_damage, cambium_kill
        self.add_record(ET::CrownDamage, CDT::CrownLength, CD::RedFir,
            &[F::CrownDamage, F::CambiumKillRating]);

        // Ponderosa Pine: crown_volume, crown_damage, cambium_kill, beetle_damage
        self.add_record(ET::CrownDamage, CDT::CrownVolume, CD::PonderosaPine,
            &[F::CrownDamage, F::CambiumKillRating, F::BeetleDamage]);

        // Ponderosa Kill: crown_kill, crown_damage, cambium_kill, beetle_damage
        self.add_record(ET::CrownDamage, CDT::CrownKill, CD::PonderosaKill,
            &[F::CrownDamage, F::CambiumKillRating, F::BeetleDamage]);

        // Douglas Fir: crown_volume, dbh, crown_damage, cambium_kill, bark_thickness
        self.add_record(ET::CrownDamage, CDT::CrownVolume, CD::DouglasFir,
            &[F::CrownDamage, F::Dbh, F::CambiumKillRating, F::BarkThickness]);
    }
}

impl Default for EquationRequiredFieldTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_13_records() {
        let t = EquationRequiredFieldTable::new();
        assert_eq!(t.record_count(), 13);
    }

    #[test]
    fn crown_scorch_requires_dbh_tree_height_crown_ratio() {
        let t = EquationRequiredFieldTable::new();
        let fields = t.get_required_fields(EquationType::CrownScorch, CrownDamageEquationCode::NotSet);

        // Always required
        assert!(fields[RequiredFieldName::Region as usize]);
        assert!(fields[RequiredFieldName::EquationType as usize]);
        assert!(fields[RequiredFieldName::FlameLengthOrScorchHeightSwitch as usize]);
        assert!(fields[RequiredFieldName::FlameLengthOrScorchHeightValue as usize]);

        // Equation-specific
        assert!(fields[RequiredFieldName::Dbh as usize]);
        assert!(fields[RequiredFieldName::TreeHeight as usize]);
        assert!(fields[RequiredFieldName::CrownRatio as usize]);

        // Not required
        assert!(!fields[RequiredFieldName::CrownDamage as usize]);
        assert!(!fields[RequiredFieldName::BoleCharHeight as usize]);
    }

    #[test]
    fn bole_char_requires_dbh_bole_char_height() {
        let t = EquationRequiredFieldTable::new();
        let fields = t.get_required_fields(EquationType::BoleChar, CrownDamageEquationCode::NotSet);
        assert!(fields[RequiredFieldName::Dbh as usize]);
        assert!(fields[RequiredFieldName::BoleCharHeight as usize]);
        assert!(!fields[RequiredFieldName::TreeHeight as usize]);
    }

    #[test]
    fn white_fir_crown_damage() {
        let t = EquationRequiredFieldTable::new();
        let fields = t.get_required_fields(EquationType::CrownDamage, CrownDamageEquationCode::WhiteFir);
        assert!(fields[RequiredFieldName::CrownDamage as usize]);
        assert!(fields[RequiredFieldName::Dbh as usize]);
        assert!(fields[RequiredFieldName::CambiumKillRating as usize]);
        assert!(fields[RequiredFieldName::BeetleDamage as usize]);
    }

    #[test]
    fn incense_cedar_no_dbh() {
        let t = EquationRequiredFieldTable::new();
        let fields = t.get_required_fields(EquationType::CrownDamage, CrownDamageEquationCode::IncenseCedar);
        assert!(fields[RequiredFieldName::CrownDamage as usize]);
        assert!(fields[RequiredFieldName::CambiumKillRating as usize]);
        assert!(!fields[RequiredFieldName::Dbh as usize]);
    }

    #[test]
    fn douglas_fir_requires_bark_thickness() {
        let t = EquationRequiredFieldTable::new();
        let fields = t.get_required_fields(EquationType::CrownDamage, CrownDamageEquationCode::DouglasFir);
        assert!(fields[RequiredFieldName::Dbh as usize]);
        assert!(fields[RequiredFieldName::CrownDamage as usize]);
        assert!(fields[RequiredFieldName::CambiumKillRating as usize]);
        assert!(fields[RequiredFieldName::BarkThickness as usize]);
        assert!(!fields[RequiredFieldName::BeetleDamage as usize]);
    }

    #[test]
    fn crown_damage_type_for_non_crown_damage_returns_not_set() {
        let t = EquationRequiredFieldTable::new();
        // CrownDamage equation type returns NotSet (C++ quirk)
        assert_eq!(
            t.get_crown_damage_type(EquationType::CrownDamage, CrownDamageEquationCode::WhiteFir),
            CrownDamageType::NotSet
        );
    }

    #[test]
    fn unknown_equation_returns_defaults() {
        let t = EquationRequiredFieldTable::new();
        let fields = t.get_required_fields(EquationType::NotSet, CrownDamageEquationCode::NotSet);
        // Only always-required fields
        assert!(fields[RequiredFieldName::Region as usize]);
        assert!(!fields[RequiredFieldName::Dbh as usize]);
    }
}
