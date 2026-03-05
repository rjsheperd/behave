//! Top-level mortality calculator.
//!
//! C++ source: mortality.h / mortality.cpp
//!
//! Orchestrates mortality calculations using inputs, equations, species tables,
//! canopy coefficient tables, and bark thickness equations.

use firelab_base::{
    AreaUnits, FirelineIntensityUnits, FractionUnits, LengthUnits, SpeedUnits, TemperatureUnits,
    UnitConversion,
};

use crate::canopy::CanopyCoefficientTable;
use crate::equations::{
    BoleCharCoefficientRecord, CrownDamageEquationCode, EquationRequiredFieldTable, EquationType,
};
use crate::inputs::{BeetleDamage, FireSeverity, FlameLengthOrScorchHeightSwitch, MortalityInputs};
use crate::species::SpeciesMasterTable;

/// Bole char coefficient table (C++ initializer in Mortality constructor).
const BOLE_CHAR_TABLE: &[BoleCharCoefficientRecord] = &[
    BoleCharCoefficientRecord { equation_number: 100, b1:  2.3014, b2: -0.3267, b3:  1.1137 },
    BoleCharCoefficientRecord { equation_number: 101, b1: -0.8727, b2: -0.1814, b3:  4.1947 },
    BoleCharCoefficientRecord { equation_number: 102, b1:  2.7899, b2: -0.5511, b3:  1.2888 },
    BoleCharCoefficientRecord { equation_number: 103, b1:  1.9438, b2: -0.4602, b3:  1.6352 },
    BoleCharCoefficientRecord { equation_number: 104, b1: -1.8137, b2: -0.0603, b3:  0.8666 },
    BoleCharCoefficientRecord { equation_number: 105, b1: -1.6262, b2: -0.0339, b3:  0.6901 },
    BoleCharCoefficientRecord { equation_number: 106, b1:  0.3714, b2: -0.1005, b3:  1.5577 },
    BoleCharCoefficientRecord { equation_number: 107, b1: -1.4416, b2: -0.1469, b3:  1.3159 },
    BoleCharCoefficientRecord { equation_number: 108, b1:  0.1122, b2: -0.1287, b3:  1.2612 },
    BoleCharCoefficientRecord { equation_number: 109, b1:  1.6779, b2: -1.0299, b3: 10.2855 },
    BoleCharCoefficientRecord { equation_number:  -1, b1:  0.0,    b2:  0.0,    b3:  0.0    },
];

/// Bark thickness coefficients indexed by bark equation number.
/// barkThickness = BARK_FACTOR[barkEquation] * DBH(inches)
const BARK_FACTOR: &[(i32, f64)] = &[
    (1, 0.019), (2, 0.022), (3, 0.024), (4, 0.025), (5, 0.026),
    (6, 0.027), (7, 0.028), (8, 0.029), (9, 0.03), (10, 0.031),
    (11, 0.032), (12, 0.033), (13, 0.034), (14, 0.035), (15, 0.036),
    (16, 0.037), (17, 0.038), (18, 0.039), (19, 0.04), (20, 0.041),
    (21, 0.042), (22, 0.043), (23, 0.044), (24, 0.045), (25, 0.046),
    (26, 0.047), (27, 0.048), (28, 0.049), (29, 0.05), (30, 0.052),
    (31, 0.055), (32, 0.057), (33, 0.059), (34, 0.06), (35, 0.062),
    (36, 0.063), (37, 0.068), (38, 0.072), (39, 0.081), (100, 0.0),
];

/// Orchestrates mortality calculations using inputs, equations, and species tables.
#[derive(Debug, Clone)]
pub struct MortalityCalculator {
    pub(crate) inputs: MortalityInputs,
    species_table: SpeciesMasterTable,
    equation_required_field_table: EquationRequiredFieldTable,
    canopy_coefficient_table: CanopyCoefficientTable,

    // Per-species outputs
    tree_crown_length_scorched: f64,
    tree_crown_volume_scorched: f64,
    probability_of_mortality: f64,
    killed_trees: f64,
    total_prefire_trees: f64,

    basal_area_prefire: f64,
    basal_area_killed: f64,
    basal_area_postfire: f64,

    prefire_canopy_cover: f64,
    postfire_canopy_cover: f64,

    // Stand-level totals
    average_mortality: f64,
    average_mortality_greater_than_4_dbh: f64,
    total_killed: f64,
    average_dbh_killed: f64,

    total_basal_area_prefire: f64,
    total_basal_area_killed: f64,
    total_basal_area_postfire: f64,

    total_prefire_canopy_cover: f64,
    total_postfire_canopy_cover: f64,

    // Global running totals
    global_total_probability_of_mortality: f64,
    global_divisor_for_avg_total_probability: f64,
    global_total_mort_greater_than_4_dbh: f64,
    global_divisor_for_avg_total_mort_greater_than_4_dbh: f64,
    global_killed_trees_times_dbh: f64,
    global_total_coverage_prefire_live: f64,
    global_total_cover_postfire_live: f64,
}

impl MortalityCalculator {
    pub fn new(species_table: SpeciesMasterTable) -> Self {
        Self {
            inputs: MortalityInputs::new(),
            species_table,
            equation_required_field_table: EquationRequiredFieldTable::new(),
            canopy_coefficient_table: CanopyCoefficientTable::new(),

            tree_crown_length_scorched: -1.0,
            tree_crown_volume_scorched: -1.0,
            probability_of_mortality: 0.0,
            killed_trees: 0.0,
            total_prefire_trees: 0.0,

            basal_area_prefire: 0.0,
            basal_area_killed: 0.0,
            basal_area_postfire: 0.0,

            prefire_canopy_cover: 0.0,
            postfire_canopy_cover: 0.0,

            average_mortality: 0.0,
            average_mortality_greater_than_4_dbh: 0.0,
            total_killed: 0.0,
            average_dbh_killed: 0.0,

            total_basal_area_prefire: 0.0,
            total_basal_area_killed: 0.0,
            total_basal_area_postfire: 0.0,

            total_prefire_canopy_cover: 0.0,
            total_postfire_canopy_cover: 0.0,

            global_total_probability_of_mortality: 0.0,
            global_divisor_for_avg_total_probability: 0.0,
            global_total_mort_greater_than_4_dbh: 0.0,
            global_divisor_for_avg_total_mort_greater_than_4_dbh: 0.0,
            global_killed_trees_times_dbh: 0.0,
            global_total_coverage_prefire_live: 0.0,
            global_total_cover_postfire_live: 0.0,
        }
    }

    // -----------------------------------------------------------------------
    // Input setters (delegate to MortalityInputs, with species table lookups)
    // -----------------------------------------------------------------------

    pub fn set_species_code(&mut self, species_code: &str) {
        self.inputs.set_species_code(species_code);
        let eq_type = self.inputs.get_equation_type();

        if !species_code.is_empty() {
            let index = self.species_table.index_from_species_code(species_code);
            if index >= 0 && eq_type == EquationType::NotSet {
                let default_eq = self.species_table.records[index as usize].equation_type;
                self.update_inputs_for_species_code_and_equation_type(species_code, default_eq);
            } else if eq_type != EquationType::NotSet {
                self.update_inputs_for_species_code_and_equation_type(species_code, eq_type);
            }
        }
    }

    pub fn set_equation_type(&mut self, eq_type: EquationType) {
        self.inputs.set_equation_type(eq_type);
        let species_code = self.inputs.get_species_code().to_string();

        if !species_code.is_empty() && eq_type != EquationType::NotSet {
            self.update_inputs_for_species_code_and_equation_type(&species_code, eq_type);
        }
    }

    fn update_inputs_for_species_code_and_equation_type(
        &mut self,
        species_code: &str,
        equation_type: EquationType,
    ) -> bool {
        let index = self
            .species_table
            .index_from_species_code_and_equation_type(species_code, equation_type);

        if index >= 0 {
            let record = &self.species_table.records[index as usize];
            self.inputs.set_equation_type(record.equation_type);
            self.inputs
                .set_crown_scorch_or_bole_char_equation_number(record.mortality_equation_number);
            self.inputs
                .set_crown_damage_equation_code(record.crown_damage_equation_code);

            let eq_type = self.inputs.get_equation_type();
            let cd_code = self.inputs.get_crown_damage_equation_code();
            self.inputs.is_field_required =
                self.equation_required_field_table.get_required_fields(eq_type, cd_code);
            let cdt = self.equation_required_field_table.get_crown_damage_type(eq_type, cd_code);
            self.inputs.set_crown_damage_type(cdt);
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Scorch height calculation (standalone, no species needed)
    // -----------------------------------------------------------------------

    /// Calculates scorch height from fireline intensity, wind speed, and air temperature.
    ///
    /// C++ function: `Mortality::calculateScorchHeight`
    pub fn calculate_scorch_height(
        &self,
        fireline_intensity: f64,
        intensity_units: FirelineIntensityUnits,
        mid_flame_wind_speed: f64,
        wind_units: SpeedUnits,
        air_temperature: f64,
        temp_units: TemperatureUnits,
        scorch_height_units: LengthUnits,
    ) -> f64 {
        let fli = intensity_units.to_base(fireline_intensity);

        // Wind speed needs to be in mph for the formula
        let ws_mph = if wind_units != SpeedUnits::MilesPerHour {
            let ws_base = wind_units.to_base(mid_flame_wind_speed);
            SpeedUnits::MilesPerHour.from_base(ws_base)
        } else {
            mid_flame_wind_speed
        };

        let temp_f = temp_units.to_base(air_temperature);

        let scorch_height = if fli < 1.0e-07 {
            0.0
        } else {
            (63.0 / (140.0 - temp_f))
                * fli.powf(1.166667)
                / (fli + ws_mph * ws_mph * ws_mph).sqrt()
        };

        scorch_height_units.from_base(scorch_height)
    }

    // -----------------------------------------------------------------------
    // Bark thickness calculation
    // -----------------------------------------------------------------------

    /// Calculate bark thickness from DBH and species bark equation.
    /// Returns bark thickness in inches, or -1 if no equation found.
    fn calculate_bark_thickness_internal(&self) -> f64 {
        let index = self.species_table.index_from_species_code_and_equation_type(
            self.inputs.get_species_code(),
            self.inputs.get_equation_type(),
        );
        if index < 0 {
            return -1.0;
        }

        let bark_equation = self.species_table.records[index as usize].bark_equation_number;
        let factor = BARK_FACTOR
            .iter()
            .find(|(eq, _)| *eq == bark_equation)
            .map(|(_, f)| *f);

        match factor {
            Some(f) => f * self.inputs.get_dbh(LengthUnits::Inches),
            None => -1.0,
        }
    }

    /// Calculate bark thickness for a given DBH and species code.
    pub fn calculate_bark_thickness(&self, dbh: f64, species_code: &str) -> f64 {
        let index = self.species_table.index_from_species_code(species_code);
        if index < 0 {
            return -1.0;
        }

        let bark_equation = self.species_table.records[index as usize].bark_equation_number;
        let factor = BARK_FACTOR
            .iter()
            .find(|(eq, _)| *eq == bark_equation)
            .map(|(_, f)| *f);

        match factor {
            Some(f) => f * dbh,
            None => -1.0,
        }
    }

    // -----------------------------------------------------------------------
    // Scorch/flame conversions (internal)
    // -----------------------------------------------------------------------

    /// Convert flame length to scorch height.
    /// FORTRAN: FLAME_TO_SCORCH = ((flame / 0.45)^2.174)^0.667
    fn calc_scorch(flame: f64) -> f64 {
        let f = flame / 0.45;
        let g = f.powf(2.174);
        g.powf(0.667)
    }

    /// Convert scorch height to flame length.
    /// FORTRAN: SCORCH_TO_FLAME = 0.45 * (scorch^1.5)^0.46
    fn calc_flame(scorch_height: f64) -> f64 {
        let g = scorch_height.powf(1.5);
        0.45 * g.powf(0.46)
    }

    /// Get the effective flame length (may derive from scorch height).
    fn get_flame_length(&self, units: LengthUnits) -> f64 {
        let flame_length = self.inputs.get_flame_length(units);
        let scorch_height = self.inputs.get_scorch_height(units);
        let switch = self.inputs.get_flame_length_or_scorch_height_switch();

        if scorch_height != -1.0
            && switch == FlameLengthOrScorchHeightSwitch::ScorchHeight
        {
            Self::calc_flame(scorch_height)
        } else {
            flame_length
        }
    }

    /// Get the effective scorch height (may derive from flame length or fireline intensity).
    fn get_scorch_height_internal(&self, units: LengthUnits) -> f64 {
        let flame_length = self.inputs.get_flame_length(LengthUnits::Feet);
        let scorch_height = self.inputs.get_scorch_height(units);
        let fli = self.inputs.get_fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond);
        let ws = self.inputs.get_mid_flame_wind_speed(SpeedUnits::MilesPerHour);
        let temp = self.inputs.get_air_temperature(TemperatureUnits::Fahrenheit);
        let switch = self.inputs.get_flame_length_or_scorch_height_switch();

        if scorch_height != -1.0
            && switch == FlameLengthOrScorchHeightSwitch::ScorchHeight
        {
            scorch_height
        } else if fli != -1.0 && ws != -1.0 && temp != -1.0 {
            self.calculate_scorch_height(
                fli,
                FirelineIntensityUnits::BtusPerFootPerSecond,
                ws,
                SpeedUnits::MilesPerHour,
                temp,
                TemperatureUnits::Fahrenheit,
                LengthUnits::Feet,
            )
        } else if flame_length != -1.0 {
            Self::calc_scorch(flame_length)
        } else {
            -1.0
        }
    }

    // -----------------------------------------------------------------------
    // Main mortality calculation
    // -----------------------------------------------------------------------

    /// Calculate mortality probability for the configured species and inputs.
    ///
    /// Returns probability in the requested units (fraction or percent).
    /// Returns -1.0 if an error occurs.
    pub fn calculate_mortality(&mut self, probability_units: FractionUnits) -> f64 {
        self.initialize_outputs();

        let eq_num = self.inputs.get_crown_scorch_or_bole_char_equation_number();
        let cd_code = self.inputs.get_crown_damage_equation_code();

        if eq_num == -1 && cd_code == CrownDamageEquationCode::NotSet {
            self.probability_of_mortality = -1.0;
        }

        match self.inputs.get_equation_type() {
            EquationType::CrownScorch => {
                self.probability_of_mortality = self.calculate_mortality_crown_scorch();
                if self.probability_of_mortality < 0.0 {
                    self.probability_of_mortality = -1.0;
                }
            }
            EquationType::CrownDamage => {
                self.probability_of_mortality = self.post_fire_injury_calculation();
                if self.probability_of_mortality < 0.0 {
                    self.probability_of_mortality = -1.0;
                }
            }
            EquationType::BoleChar => {
                self.probability_of_mortality = self.bole_char_calculate();
                if self.probability_of_mortality < 0.0 {
                    self.probability_of_mortality = -1.0;
                }
            }
            _ => {}
        }

        probability_units.from_base(self.probability_of_mortality)
    }

    // -----------------------------------------------------------------------
    // Crown scorch mortality calculation
    // -----------------------------------------------------------------------

    fn calculate_mortality_crown_scorch(&mut self) -> f64 {
        let dbh = self.inputs.get_dbh(LengthUnits::Inches);
        let fl = self.get_flame_length(LengthUnits::Feet);
        let black_hills_flame_length = fl;
        let scorch_height = self.get_scorch_height_internal(LengthUnits::Feet);

        // Check for valid species
        if self
            .species_table
            .index_from_species_code_and_equation_type(
                self.inputs.get_species_code(),
                self.inputs.get_equation_type(),
            )
            == -1
        {
            return -1.0;
        }

        // Calculate bark thickness
        let bt = self.calculate_bark_thickness_internal();
        self.inputs.set_bark_thickness(bt, LengthUnits::Inches);

        let tree_height = self.inputs.get_tree_height(LengthUnits::Feet);
        let crown_ratio = self.inputs.get_crown_ratio(FractionUnits::Fraction);
        let hcr = tree_height * crown_ratio; // height of crown length
        self.tree_crown_length_scorched = scorch_height - (tree_height - hcr);

        if self.tree_crown_length_scorched <= 0.0 {
            self.tree_crown_length_scorched = 0.0;
        }
        if self.tree_crown_length_scorched > hcr {
            self.tree_crown_length_scorched = hcr;
        }

        // Calculate crown volume scorched and crown length scorched percent
        let csl; // crown length scorched percent
        if hcr > 0.0 {
            self.tree_crown_volume_scorched = self.tree_crown_length_scorched
                * (2.0 * hcr - self.tree_crown_length_scorched)
                / (hcr * hcr);
            csl = 100.0 * (self.tree_crown_length_scorched / hcr);
        } else {
            self.tree_crown_volume_scorched = 0.0;
            return -1.0;
        }

        let mort_eq = self.inputs.get_crown_scorch_or_bole_char_equation_number();
        let cvs_pct = self.tree_crown_volume_scorched * 100.0;
        let bark_thick = self.inputs.get_bark_thickness(LengthUnits::Inches);

        let p = match mort_eq {
            1 => self.crown_scorch_eq1(dbh, csl, cvs_pct, bark_thick, tree_height),
            3 => {
                let mut p = self.crown_scorch_eq1(dbh, csl, cvs_pct, bark_thick, tree_height);
                if p < 0.8 {
                    p = 0.8;
                }
                p
            }
            4 => {
                let ch = fl / 1.8;
                let p = if self.inputs.get_fire_severity() == FireSeverity::Low {
                    1.0 / (1.0 + ((0.251 * dbh * 2.54) - (0.07 * ch * 2.54 * 12.0) - 4.407).exp())
                } else {
                    1.0 / (1.0
                        + ((0.0858 * dbh * 2.54) - (0.118 * ch * 2.54 * 12.0) - 2.157).exp())
                };
                self.tree_crown_volume_scorched = -1.0;
                self.tree_crown_length_scorched = -1.0;
                p
            }
            5 => {
                if csl <= 0.0 {
                    return 0.0;
                }
                let dbh_cm = LengthUnits::Centimeters.from_base(dbh);
                let bt_prime = 0.435 + (0.031 * dbh_cm);
                let f = cvs_pct / 10.0;
                let bt_prime =
                    0.169 + (5.136 * bt_prime) + (14.492 * bt_prime * bt_prime) - (0.348 * f * f);
                let mut p = 1.0 / (1.0 + bt_prime.exp());
                if p > 1.0 {
                    p = 1.0;
                }
                if p < 0.0 {
                    p = 0.0;
                }
                p
            }
            10 => {
                let p = Self::whitefir_prefire(csl);
                self.tree_crown_volume_scorched = -1.0;
                p
            }
            11 => {
                let p = Self::subalpine_fir_prefire(cvs_pct);
                self.tree_crown_length_scorched = -1.0;
                p
            }
            12 => {
                let p = Self::incense_cedar_prefire(csl);
                self.tree_crown_volume_scorched = -1.0;
                p
            }
            14 => {
                let p = Self::western_larch_prefire(cvs_pct, dbh);
                self.tree_crown_length_scorched = -1.0;
                p
            }
            15 => {
                let p = Self::engelmann_spruce_prefire(cvs_pct);
                self.tree_crown_length_scorched = -1.0;
                p
            }
            16 => {
                let p = Self::red_fir_prefire(csl);
                self.tree_crown_volume_scorched = -1.0;
                p
            }
            17 => {
                let p = Self::whitebark_pine_prefire(cvs_pct, dbh);
                self.tree_crown_length_scorched = -1.0;
                p
            }
            18 => {
                let p = Self::sugar_pine_prefire(csl);
                self.tree_crown_volume_scorched = -1.0;
                p
            }
            19 => {
                let p = Self::ponderosa_jeffrey_pine_prefire(cvs_pct);
                self.tree_crown_length_scorched = -1.0;
                p
            }
            20 => {
                let p = Self::douglas_fir_prefire(cvs_pct);
                self.tree_crown_length_scorched = -1.0;
                p
            }
            21 => {
                // Black Hills PiPo
                let cr = self.inputs.get_crown_ratio(FractionUnits::Fraction);
                let cbh = self.inputs.get_tree_height(LengthUnits::Feet)
                    - (self.inputs.get_tree_height(LengthUnits::Feet) * cr);
                let p = Self::eq21_black_hills_pipo(
                    self.inputs.get_tree_height(LengthUnits::Feet),
                    cbh,
                    self.inputs.get_dbh(LengthUnits::Feet),
                    scorch_height,
                    black_hills_flame_length,
                );
                self.tree_crown_length_scorched = -1.0;
                self.tree_crown_volume_scorched = -1.0;
                p
            }
            _ => {
                self.tree_crown_length_scorched = -1.0;
                self.tree_crown_volume_scorched = -1.0;
                return -1.0;
            }
        };

        self.probability_of_mortality = p;
        self.calculate_mortality_totals();
        p
    }

    /// Crown scorch equation 1 (also used by eq 3 with 0.8 floor).
    fn crown_scorch_eq1(
        &mut self,
        dbh: f64,
        csl: f64,
        cvs_pct: f64,
        bark_thick: f64,
        tree_height: f64,
    ) -> f64 {
        if dbh >= 1.0 {
            1.0 / (1.0
                + (-1.941 + (6.316 * (1.0 - (-bark_thick).exp()))
                    - 0.000535 * cvs_pct * cvs_pct)
                    .exp())
        } else if csl > 50.0 {
            1.0
        } else if tree_height < 3.0 {
            1.0
        } else {
            let bt = self.calculate_bark_thickness_internal();
            self.inputs.set_bark_thickness(bt, LengthUnits::Inches);
            let bark_thick = self.inputs.get_bark_thickness(LengthUnits::Inches);
            let p = 1.0
                / (1.0
                    + (-1.941 + (6.316 * (1.0 - (-bark_thick).exp()))
                        - 0.000535 * cvs_pct * cvs_pct)
                        .exp());
            p + (1.0 - p) * (1.0 - ((tree_height - 3.0) / (((1.0 / dbh) * tree_height) - 3.0)))
        }
    }

    // -----------------------------------------------------------------------
    // Pre-fire (crown scorch) species-specific equations
    // -----------------------------------------------------------------------

    fn whitefir_prefire(cs: f64) -> f64 {
        1.0 / (1.0
            + (-3.5083 + cs * 0.0956 - cs.powi(2) * 0.00184 + cs.powi(3) * 0.000017).exp())
    }

    fn subalpine_fir_prefire(cs: f64) -> f64 {
        1.0 / (1.0
            + (-1.6950 + cs * 0.2071 - cs.powi(2) * 0.0047 + cs.powi(3) * 0.000035).exp())
    }

    fn incense_cedar_prefire(cs: f64) -> f64 {
        1.0 / (1.0 + (-4.2466 + cs.powi(3) * 0.000007172).exp())
    }

    fn western_larch_prefire(cs: f64, dbh: f64) -> f64 {
        1.0 / (1.0 + (-1.6594 + cs * 0.0327 - dbh * 0.1241).exp())
    }

    fn engelmann_spruce_prefire(cs: f64) -> f64 {
        1.0 / (1.0 + (-(0.0845 + cs * 0.0445)).exp())
    }

    fn red_fir_prefire(cs: f64) -> f64 {
        1.0 / (1.0 + (-2.3085 + cs.powi(3) * 0.000004059).exp())
    }

    fn whitebark_pine_prefire(cs: f64, dbh: f64) -> f64 {
        1.0 / (1.0
            + (-0.3268 + cs * 0.1387 - cs.powi(2) * 0.0033 + cs.powi(3) * 0.000025
                - dbh * 0.0676)
                .exp())
    }

    fn sugar_pine_prefire(cs: f64) -> f64 {
        1.0 / (1.0 + (-2.0588 + cs.powi(2) * 0.000814).exp())
    }

    fn ponderosa_jeffrey_pine_prefire(cs: f64) -> f64 {
        1.0 / (1.0 + (-2.7103 + cs.powi(3) * 0.000004093).exp())
    }

    fn douglas_fir_prefire(cs: f64) -> f64 {
        1.0 / (1.0
            + (-2.0346 + cs * 0.0906 - cs.powi(2) * 0.0022 + cs.powi(3) * 0.000019).exp())
    }

    // -----------------------------------------------------------------------
    // Black Hills PiPo (Eq 21)
    // -----------------------------------------------------------------------

    fn eq21_black_hills_pipo(
        tree_height: f64,
        crown_base_height: f64,
        dbh: f64,
        scorch: f64,
        flame_length: f64,
    ) -> f64 {
        let tree_ht_m = LengthUnits::Meters.from_base(tree_height);
        let flame_m = LengthUnits::Meters.from_base(flame_length);
        let dbh_cm = LengthUnits::Centimeters.from_base(dbh);
        let scorch_m = LengthUnits::Meters.from_base(scorch);
        let cbh_m = LengthUnits::Meters.from_base(crown_base_height);

        // Crown length scorched percent
        let cls = if scorch_m > cbh_m {
            ((scorch_m - cbh_m) / (tree_ht_m - cbh_m)) * 100.0
        } else {
            0.0
        };

        if tree_ht_m < 1.37 {
            // Seedling
            let g = (-(2.714 + (4.08 * flame_m) + (-3.63 * tree_ht_m))).exp();
            1.0 / (1.0 + g)
        } else if dbh_cm < 10.2 {
            // Sapling
            let g = (-(-0.7661 + (2.7981 * flame_m) + (-1.2487 * tree_ht_m))).exp();
            1.0 / (1.0 + g)
        } else {
            // Other trees
            let g = (-(1.104 + (dbh_cm * -0.156) + (0.013 * cls)
                + (0.001 * dbh_cm * cls)))
                .exp();
            1.0 / (1.0 + g)
        }
    }

    // -----------------------------------------------------------------------
    // Post-fire injury (Crown Damage) calculation
    // -----------------------------------------------------------------------

    fn post_fire_injury_calculation(&mut self) -> f64 {
        let p = match self.inputs.get_crown_damage_equation_code() {
            CrownDamageEquationCode::WhiteFir => self.eq_white_fir_wf(),
            CrownDamageEquationCode::SubalpineFir => self.eq_subalpine_fir_sf(),
            CrownDamageEquationCode::IncenseCedar => self.eq_incense_cedar_ic(),
            CrownDamageEquationCode::WesternLarch => self.eq_western_larch_wl(),
            CrownDamageEquationCode::WhitebarkPine => self.eq_whitebark_pine_wp(),
            CrownDamageEquationCode::EngelmannSpruce => self.eq_engelmann_spruce_es(),
            CrownDamageEquationCode::SugarPine => self.eq_sugar_pine_sp(),
            CrownDamageEquationCode::RedFir => self.eq_red_fir_rf(),
            CrownDamageEquationCode::PonderosaPine => self.eq_ponderosa_pine_pp(),
            CrownDamageEquationCode::PonderosaKill => self.eq_ponderosa_kill_pk(),
            CrownDamageEquationCode::DouglasFir => self.eq_douglas_fir_df(),
            _ => return -1.0,
        };

        self.probability_of_mortality = p;
        self.calculate_mortality_totals();
        p
    }

    // -----------------------------------------------------------------------
    // Crown Damage equations (post-fire)
    // -----------------------------------------------------------------------

    fn eq_white_fir_wf(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        let dbh = self.inputs.get_dbh(LengthUnits::Inches);
        let beetle = if self.inputs.get_beetle_damage() == BeetleDamage::Yes {
            1.0
        } else {
            -1.0
        };
        1.0 / (1.0
            + (-(-3.5964 + cs.powi(3) * 0.00000628 + ckr * 0.3019 + dbh * 0.0483
                + beetle * 0.5209))
                .exp())
    }

    fn eq_subalpine_fir_sf(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        1.0 / (1.0 + (-(-2.6036 + cs.powi(3) * 0.000004587 + ckr * 1.3554)).exp())
    }

    fn eq_incense_cedar_ic(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        1.0 / (1.0 + (-(-5.6465 + cs.powi(3) * 0.000007274 + ckr * 0.5428)).exp())
    }

    fn eq_western_larch_wl(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        1.0 / (1.0 + (-(-3.8458 + cs.powi(2) * 0.0004 + ckr * 0.6266)).exp())
    }

    fn eq_whitebark_pine_wp(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        let dbh = self.inputs.get_dbh(LengthUnits::Inches);
        1.0 / (1.0
            + (-(-1.4059 + cs.powi(3) * 0.000004459 + (ckr * ckr) * 0.2843 + dbh * -0.1232))
                .exp())
    }

    fn eq_engelmann_spruce_es(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        1.0 / (1.0 + (-(-2.9791 + cs * 0.0405 + ckr * 1.1596)).exp())
    }

    fn eq_sugar_pine_sp(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        let beetle = if self.inputs.get_beetle_damage() == BeetleDamage::Yes {
            1.0
        } else {
            -1.0
        };
        1.0 / (1.0
            + (-(-2.7598 + (cs * cs) * 0.000642 + ckr.powi(3) * 0.0386 + beetle * 0.8485)).exp())
    }

    fn eq_red_fir_rf(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        1.0 / (1.0 + (-(-4.7515 + cs.powi(3) * 0.000005989 + ckr * 1.0668)).exp())
    }

    fn eq_ponderosa_pine_pp(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        let beetle = if self.inputs.get_beetle_damage() == BeetleDamage::Yes {
            1.0
        } else {
            0.0
        };
        let f = -4.1914 + cs.powi(2) * 0.000376 + ckr * 0.5130 + beetle * 1.5873;
        1.0 / (1.0 + (-f).exp())
    }

    fn eq_ponderosa_kill_pk(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        let beetle = if self.inputs.get_beetle_damage() == BeetleDamage::Yes {
            1.0
        } else {
            0.0
        };
        let f = -3.5729 + cs.powi(2) * 0.000567 + ckr * 0.4573 + beetle * 1.6075;
        1.0 / (1.0 + (-f).exp())
    }

    fn eq_douglas_fir_df(&self) -> f64 {
        let cs = self.inputs.get_crown_damage();
        let ckr = self.inputs.get_cambium_kill_rating();
        let dbh = self.inputs.get_dbh(LengthUnits::Inches);
        let beetle = if self.inputs.get_beetle_damage() == BeetleDamage::Yes {
            1.0
        } else {
            0.0
        };
        let f = -1.8912 + cs * 0.07 - cs.powi(2) * 0.0019 + cs.powi(3) * 0.000018
            + ckr * 0.5840
            - dbh * 0.0788
            - beetle * 0.7959
            + dbh * beetle * 0.1251;
        1.0 / (1.0 + (-f).exp())
    }

    // -----------------------------------------------------------------------
    // Bole Char mortality calculation
    // -----------------------------------------------------------------------

    fn bole_char_calculate(&mut self) -> f64 {
        let eq_num = self.inputs.get_crown_scorch_or_bole_char_equation_number();

        // Find coefficients in bole char table
        let entry = BOLE_CHAR_TABLE.iter().find(|r| {
            if r.equation_number == -1 {
                return false; // sentinel
            }
            r.equation_number == eq_num
        });

        let entry = match entry {
            Some(e) => e,
            None => return -1.0,
        };

        let b1 = entry.b1;
        let b2 = entry.b2;
        let b3 = entry.b3;

        let dbh_cm = LengthUnits::Centimeters.from_base(
            self.inputs.get_dbh(LengthUnits::Feet),
        );
        let bch_m = LengthUnits::Meters.from_base(
            self.inputs.get_bole_char_height(LengthUnits::Feet),
        );

        let f = 1.0 / (1.0 + (-1.0 * (b1 + b2 * dbh_cm + b3 * bch_m)).exp());

        self.calculate_mortality_totals();
        f
    }

    // -----------------------------------------------------------------------
    // Mortality totals (stand-level accumulation)
    // -----------------------------------------------------------------------

    fn initialize_outputs(&mut self) {
        self.total_prefire_trees = 0.0;
        self.probability_of_mortality = 0.0;
        self.killed_trees = 0.0;
        self.basal_area_prefire = 0.0;
        self.basal_area_killed = 0.0;
        self.basal_area_postfire = 0.0;
        self.prefire_canopy_cover = 0.0;
        self.postfire_canopy_cover = 0.0;

        self.average_mortality = 0.0;
        self.average_mortality_greater_than_4_dbh = 0.0;
        self.total_killed = 0.0;
        self.average_dbh_killed = 0.0;
        self.total_basal_area_prefire = 0.0;
        self.total_basal_area_killed = 0.0;
        self.total_basal_area_postfire = 0.0;
        self.total_prefire_canopy_cover = 0.0;
        self.total_postfire_canopy_cover = 0.0;

        self.global_total_probability_of_mortality = 0.0;
        self.global_divisor_for_avg_total_probability = 0.0;
        self.global_total_mort_greater_than_4_dbh = 0.0;
        self.global_divisor_for_avg_total_mort_greater_than_4_dbh = 0.0;
        self.global_killed_trees_times_dbh = 0.0;
        self.global_total_coverage_prefire_live = 0.0;
        self.global_total_cover_postfire_live = 0.0;
    }

    fn calculate_mortality_totals(&mut self) {
        let density = self.inputs.get_tree_density_per_unit_area(AreaUnits::Acres);
        let dbh = self.inputs.get_dbh(LengthUnits::Inches);

        self.killed_trees = self.probability_of_mortality * density;
        self.killed_trees += 0.5;
        self.killed_trees = (self.killed_trees as i32) as f64;
        if self.killed_trees > density {
            self.killed_trees = density;
        }

        // Individual basal area
        self.basal_area_prefire = Self::basal_area(dbh, density);
        self.basal_area_killed = Self::basal_area(dbh, self.killed_trees);
        self.basal_area_postfire = self.basal_area_prefire - self.basal_area_killed;

        // Convert square inches to square feet
        self.basal_area_prefire /= 144.0;
        self.basal_area_killed /= 144.0;
        self.basal_area_postfire /= 144.0;

        // Basal area totals
        self.total_basal_area_prefire += self.basal_area_prefire;
        self.total_basal_area_killed += self.basal_area_killed;
        self.total_basal_area_postfire += self.basal_area_postfire;

        // Canopy cover
        let cc = self.calculate_crown_cover();
        self.prefire_canopy_cover = cc * density;
        self.postfire_canopy_cover = self.prefire_canopy_cover - (cc * self.killed_trees);

        self.global_total_coverage_prefire_live += self.prefire_canopy_cover;
        self.global_total_cover_postfire_live += self.postfire_canopy_cover;

        self.prefire_canopy_cover = Self::cc_overlap(self.prefire_canopy_cover);
        self.postfire_canopy_cover = Self::cc_overlap(self.postfire_canopy_cover);

        // Canopy cover totals
        self.total_prefire_canopy_cover =
            Self::cc_overlap(self.global_total_coverage_prefire_live);
        self.total_postfire_canopy_cover =
            Self::cc_overlap(self.global_total_cover_postfire_live);
        self.total_prefire_canopy_cover =
            (self.total_prefire_canopy_cover + 0.5) as i32 as f64;
        self.total_postfire_canopy_cover =
            (self.total_postfire_canopy_cover + 0.5) as i32 as f64;

        if self.total_prefire_canopy_cover < 1.0 {
            self.total_prefire_canopy_cover = 1.0;
        }
        if self.total_postfire_canopy_cover < 1.0 {
            self.total_postfire_canopy_cover = 1.0;
        }

        // Stand accumulation
        self.total_prefire_trees += density;

        self.global_total_probability_of_mortality += self.probability_of_mortality;
        self.global_divisor_for_avg_total_probability += 1.0;
        self.average_mortality = (self.global_total_probability_of_mortality
            / self.global_divisor_for_avg_total_probability
            + 0.5) as i32 as f64;

        if self.inputs.get_dbh(LengthUnits::Feet) >= 4.0 {
            self.global_total_mort_greater_than_4_dbh += self.probability_of_mortality;
            self.global_divisor_for_avg_total_mort_greater_than_4_dbh += 1.0;
            self.average_mortality_greater_than_4_dbh = (self
                .global_total_mort_greater_than_4_dbh
                / self.global_divisor_for_avg_total_mort_greater_than_4_dbh
                + 0.5) as i32 as f64;
        }

        self.total_killed += self.killed_trees;
        self.global_killed_trees_times_dbh += dbh * self.killed_trees;
        if self.total_killed != 0.0 {
            self.average_dbh_killed = self.global_killed_trees_times_dbh / self.total_killed;
        }
    }

    fn basal_area(dbh: f64, count: f64) -> f64 {
        let r = dbh / 2.0;
        let f = std::f64::consts::PI * r * r;
        f * count
    }

    fn cc_overlap(sq_ft_cov: f64) -> f64 {
        let x = sq_ft_cov / 43560.0;
        100.0 * (1.0 - (-x).exp())
    }

    fn calculate_crown_cover(&self) -> f64 {
        let tree_height = self.inputs.get_tree_height(LengthUnits::Feet);
        let dbh = self.inputs.get_dbh(LengthUnits::Inches);

        if tree_height <= 0.0 || dbh <= 0.0 {
            return 0.0;
        }

        let species_index = self
            .species_table
            .index_from_species_code_and_equation_type(
                self.inputs.get_species_code(),
                self.inputs.get_equation_type(),
            );
        if species_index < 0 {
            return 0.0;
        }

        let canopy_idx =
            self.species_table.records[species_index as usize].crown_coefficient_code as usize;

        let crown_diameter = if let Some(coeff) = self.canopy_coefficient_table.get_by_index(canopy_idx) {
            if tree_height <= 4.5 {
                coeff.coefficient_r * dbh
            } else {
                dbh.powf(coeff.coefficient_b) * coeff.coefficient_a
            }
        } else {
            return 0.0;
        };

        let r = crown_diameter / 2.0;
        std::f64::consts::PI * r * r
    }

    // -----------------------------------------------------------------------
    // Output getters
    // -----------------------------------------------------------------------

    pub fn get_probability_of_mortality(&self, units: FractionUnits) -> f64 {
        units.from_base(self.probability_of_mortality)
    }

    pub fn get_tree_crown_length_scorched(&self, units: LengthUnits) -> f64 {
        units.from_base(self.tree_crown_length_scorched)
    }

    pub fn get_tree_crown_volume_scorched(&self, units: FractionUnits) -> f64 {
        units.from_base(self.tree_crown_volume_scorched)
    }

    pub fn get_killed_trees(&self) -> f64 {
        self.killed_trees
    }

    pub fn get_total_prefire_trees(&self) -> f64 {
        self.total_prefire_trees
    }

    pub fn get_basal_area_prefire(&self) -> f64 {
        self.basal_area_prefire
    }

    pub fn get_basal_area_killed(&self) -> f64 {
        self.basal_area_killed
    }

    pub fn get_basal_area_postfire(&self) -> f64 {
        self.basal_area_postfire
    }

    pub fn get_prefire_canopy_cover(&self) -> f64 {
        self.prefire_canopy_cover
    }

    pub fn get_postfire_canopy_cover(&self) -> f64 {
        self.postfire_canopy_cover
    }

    /// Access the underlying species table.
    pub fn species_table(&self) -> &SpeciesMasterTable {
        &self.species_table
    }

    /// Access the underlying equation required field table.
    pub fn equation_required_field_table(&self) -> &EquationRequiredFieldTable {
        &self.equation_required_field_table
    }
}

impl Default for MortalityCalculator {
    fn default() -> Self {
        Self::new(SpeciesMasterTable::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scorch_height_test1() {
        // Test from C++: 80°F, 5mph, 50 Btu/ft/s → 7.617325 ft
        let calc = MortalityCalculator::default();
        let sh = calc.calculate_scorch_height(
            50.0,
            FirelineIntensityUnits::BtusPerFootPerSecond,
            5.0,
            SpeedUnits::MilesPerHour,
            80.0,
            TemperatureUnits::Fahrenheit,
            LengthUnits::Feet,
        );
        assert!(
            (sh - 7.617325).abs() < 1e-4,
            "scorch height: expected 7.617325, got {sh}"
        );
    }

    #[test]
    fn scorch_height_test2() {
        // Test from C++: 70°F, 300 ft/min, 55 Btu/ft/s → 9.923720 ft
        let calc = MortalityCalculator::default();
        let sh = calc.calculate_scorch_height(
            55.0,
            FirelineIntensityUnits::BtusPerFootPerSecond,
            300.0,
            SpeedUnits::FeetPerMinute,
            70.0,
            TemperatureUnits::Fahrenheit,
            LengthUnits::Feet,
        );
        assert!(
            (sh - 9.923720).abs() < 1e-4,
            "scorch height: expected 9.923720, got {sh}"
        );
    }

    #[test]
    fn bark_thickness_basic() {
        let calc = MortalityCalculator::default();
        // ABCO (white fir) has bark_equation_number=27 → factor 0.048
        // DBH 10" → 0.48"
        let bt = calc.calculate_bark_thickness(10.0, "ABCO");
        assert!(bt > 0.0, "bark thickness should be > 0, got {bt}");
        assert!(
            (bt - 0.48).abs() < 1e-6,
            "expected 0.48, got {bt}"
        );
    }

    #[test]
    fn scorch_height_zero_intensity() {
        let calc = MortalityCalculator::default();
        let sh = calc.calculate_scorch_height(
            0.0,
            FirelineIntensityUnits::BtusPerFootPerSecond,
            5.0,
            SpeedUnits::MilesPerHour,
            80.0,
            TemperatureUnits::Fahrenheit,
            LengthUnits::Feet,
        );
        assert!((sh - 0.0).abs() < 1e-10, "expected 0.0, got {sh}");
    }

    #[test]
    fn crown_scorch_basic() {
        // Set up a crown scorch calculation with ABCO (white fir, eq 10)
        let mut calc = MortalityCalculator::default();
        calc.set_species_code("ABCO");
        calc.set_equation_type(EquationType::CrownScorch);
        calc.inputs.set_dbh(10.0, LengthUnits::Inches);
        calc.inputs.set_tree_height(50.0, LengthUnits::Feet);
        calc.inputs.set_crown_ratio(0.5, FractionUnits::Fraction);
        calc.inputs.set_flame_length(5.0, LengthUnits::Feet);
        calc.inputs
            .set_flame_length_or_scorch_height_switch(FlameLengthOrScorchHeightSwitch::FlameLength);
        calc.inputs
            .set_tree_density_per_unit_area(100.0, AreaUnits::Acres);
        calc.inputs.set_fire_severity(FireSeverity::Empty);

        let p = calc.calculate_mortality(FractionUnits::Fraction);
        // Should return a valid probability (not -1)
        assert!(
            p >= 0.0 && p <= 1.0,
            "expected valid probability, got {p}"
        );
    }
}
