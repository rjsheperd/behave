//! Moisture scenario definitions.
//!
//! 16 built-in moisture scenarios combining 4 dead fuel moisture levels
//! (D1=Very Low, D2=Low, D3=Moderate, D4=High) with 4 live fuel curing
//! levels (L1=Fully Cured, L2=2/3 Cured, L3=1/3 Cured, L4=Fully Green).
//!
//! All moisture values stored internally as fractions (0–1).
//!
//! C++ source: moistureScenarios.h / moistureScenarios.cpp

use firelab_base::{FractionUnits, UnitConversion};

/// A named set of moisture values across all size classes.
///
/// C++ struct: `MoistureScenarioRecord`
#[derive(Debug, Clone)]
pub struct MoistureScenarioRecord {
    pub name: String,
    pub description: String,
    pub moisture_one_hour: f64,
    pub moisture_ten_hour: f64,
    pub moisture_hundred_hour: f64,
    pub moisture_live_herbaceous: f64,
    pub moisture_live_woody: f64,
}

/// Catalog of built-in moisture scenarios.
///
/// C++ class: `MoistureScenarios`
#[derive(Debug, Clone)]
pub struct MoistureScenarios {
    scenarios: Vec<MoistureScenarioRecord>,
}

impl MoistureScenarios {
    pub fn new() -> Self {
        let mut ms = Self { scenarios: Vec::new() };
        ms.populate();
        ms
    }

    pub fn num_scenarios(&self) -> usize {
        self.scenarios.len()
    }

    /// Find a scenario index by name (case-insensitive). Returns -1 if not found (matching C++).
    pub fn index_by_name(&self, name: &str) -> i32 {
        let upper = name.to_uppercase();
        for (i, s) in self.scenarios.iter().enumerate() {
            if s.name.to_uppercase() == upper {
                return i as i32;
            }
        }
        -1
    }

    pub fn is_defined_by_name(&self, name: &str) -> bool {
        let idx = self.index_by_name(name);
        idx >= 0 && (idx as usize) < self.scenarios.len()
    }

    pub fn is_defined_by_index(&self, index: i32) -> bool {
        !self.scenarios.is_empty() && index >= 0 && (index as usize) < self.scenarios.len()
    }

    // --- Getters by name ---

    pub fn description_by_name(&self, name: &str) -> String {
        let idx = self.index_by_name(name);
        if idx >= 0 && (idx as usize) < self.scenarios.len() {
            self.scenarios[idx as usize].description.clone()
        } else {
            format!("Error: Scenario {} is not defined", name)
        }
    }

    pub fn one_hour_by_name(&self, name: &str, units: FractionUnits) -> f64 {
        let idx = self.index_by_name(name);
        if idx >= 0 && (idx as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[idx as usize].moisture_one_hour)
        } else {
            -1.0
        }
    }

    pub fn ten_hour_by_name(&self, name: &str, units: FractionUnits) -> f64 {
        let idx = self.index_by_name(name);
        if idx >= 0 && (idx as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[idx as usize].moisture_ten_hour)
        } else {
            -1.0
        }
    }

    pub fn hundred_hour_by_name(&self, name: &str, units: FractionUnits) -> f64 {
        let idx = self.index_by_name(name);
        if idx >= 0 && (idx as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[idx as usize].moisture_hundred_hour)
        } else {
            -1.0
        }
    }

    pub fn live_herbaceous_by_name(&self, name: &str, units: FractionUnits) -> f64 {
        let idx = self.index_by_name(name);
        if idx >= 0 && (idx as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[idx as usize].moisture_live_herbaceous)
        } else {
            -1.0
        }
    }

    pub fn live_woody_by_name(&self, name: &str, units: FractionUnits) -> f64 {
        let idx = self.index_by_name(name);
        if idx >= 0 && (idx as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[idx as usize].moisture_live_woody)
        } else {
            -1.0
        }
    }

    // --- Getters by index ---

    pub fn name_by_index(&self, index: i32) -> String {
        if index >= 0 && (index as usize) < self.scenarios.len() {
            self.scenarios[index as usize].name.clone()
        } else {
            format!("Error: Scenario with vector index {} is not defined", index)
        }
    }

    pub fn description_by_index(&self, index: i32) -> String {
        if index >= 0 && (index as usize) < self.scenarios.len() {
            self.scenarios[index as usize].description.clone()
        } else {
            format!("Error: Scenario with vector index {} is not defined", index)
        }
    }

    pub fn one_hour_by_index(&self, index: i32, units: FractionUnits) -> f64 {
        if index >= 0 && (index as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[index as usize].moisture_one_hour)
        } else {
            -1.0
        }
    }

    pub fn ten_hour_by_index(&self, index: i32, units: FractionUnits) -> f64 {
        if index >= 0 && (index as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[index as usize].moisture_ten_hour)
        } else {
            -1.0
        }
    }

    pub fn hundred_hour_by_index(&self, index: i32, units: FractionUnits) -> f64 {
        if index >= 0 && (index as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[index as usize].moisture_hundred_hour)
        } else {
            -1.0
        }
    }

    pub fn live_herbaceous_by_index(&self, index: i32, units: FractionUnits) -> f64 {
        if index >= 0 && (index as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[index as usize].moisture_live_herbaceous)
        } else {
            -1.0
        }
    }

    pub fn live_woody_by_index(&self, index: i32, units: FractionUnits) -> f64 {
        if index >= 0 && (index as usize) < self.scenarios.len() {
            units.from_base(self.scenarios[index as usize].moisture_live_woody)
        } else {
            -1.0
        }
    }

    // --- Internal ---

    fn add_record(
        &mut self,
        name: &str,
        description: &str,
        one_hour: f64,
        ten_hour: f64,
        hundred_hour: f64,
        live_herbaceous: f64,
        live_woody: f64,
    ) {
        self.scenarios.push(MoistureScenarioRecord {
            name: name.to_string(),
            description: description.to_string(),
            moisture_one_hour: one_hour,
            moisture_ten_hour: ten_hour,
            moisture_hundred_hour: hundred_hour,
            moisture_live_herbaceous: live_herbaceous,
            moisture_live_woody: live_woody,
        });
    }

    fn populate(&mut self) {
        // Dead fuel moisture levels (fraction)
        let very_low_dead = [0.03, 0.04, 0.05];
        let low_dead = [0.06, 0.07, 0.08];
        let moderate_dead = [0.09, 0.10, 0.11];
        let high_dead = [0.12, 0.13, 0.14];

        // Live fuel curing levels [herbaceous, woody] (fraction)
        let fully_cured_herb = [0.30, 0.60];
        let two_thirds_cured_herb = [0.60, 0.90];
        let one_third_cured_herb = [0.90, 1.20];
        let fully_green_herb = [1.20, 1.50];

        // D1: Very low dead
        self.add_record("D1L1", "D1L1 - Very low dead, fully cured herb (3,4,5,30,60)",
            very_low_dead[0], very_low_dead[1], very_low_dead[2],
            fully_cured_herb[0], fully_cured_herb[1]);
        self.add_record("D1L2", "D1L2 - Very low dead, 2/3 cured herb (3,4,5,60,90)",
            very_low_dead[0], very_low_dead[1], very_low_dead[2],
            two_thirds_cured_herb[0], two_thirds_cured_herb[1]);
        self.add_record("D1L3", "D1L3 - Very low dead, 1/3 cured herb (3,4,5,90,120)",
            very_low_dead[0], very_low_dead[1], very_low_dead[2],
            one_third_cured_herb[0], one_third_cured_herb[1]);
        self.add_record("D1L4", "D1L4 - Very low dead, fully green herb (3,4,5,120,150)",
            very_low_dead[0], very_low_dead[1], very_low_dead[2],
            fully_green_herb[0], fully_green_herb[1]);

        // D2: Low dead
        self.add_record("D2L1", "D2L1 - Low dead, fully cured herb (6,7,8,30,60)",
            low_dead[0], low_dead[1], low_dead[2],
            fully_cured_herb[0], fully_cured_herb[1]);
        self.add_record("D2L2", "D2L2 - Low dead, 2/3 cured herb (6,7,8,60,90)",
            low_dead[0], low_dead[1], low_dead[2],
            two_thirds_cured_herb[0], two_thirds_cured_herb[1]);
        self.add_record("D2L3", "D2L3 - Low dead, 1/3 cured herb (6,7,8,90,120)",
            low_dead[0], low_dead[1], low_dead[2],
            one_third_cured_herb[0], one_third_cured_herb[1]);
        self.add_record("D2L4", "D2L4 - Low dead, fully green herb (6,7,8,120,150)",
            low_dead[0], low_dead[1], low_dead[2],
            fully_green_herb[0], fully_green_herb[1]);

        // D3: Moderate dead
        self.add_record("D3L1", "D3L1 - Moderate dead, fully cured herb (9,10,11,30,60)",
            moderate_dead[0], moderate_dead[1], moderate_dead[2],
            fully_cured_herb[0], fully_cured_herb[1]);
        self.add_record("D3L2", "D3L2 - Moderate dead, 2/3 cured herb (9,10,11,60,90)",
            moderate_dead[0], moderate_dead[1], moderate_dead[2],
            two_thirds_cured_herb[0], two_thirds_cured_herb[1]);
        self.add_record("D3L3", "D3L3 - Moderate dead, 1/3 cured herb (9,10,11,90,120)",
            moderate_dead[0], moderate_dead[1], moderate_dead[2],
            one_third_cured_herb[0], one_third_cured_herb[1]);
        self.add_record("D3L4", "D3L4 - Moderate dead, fully green herb (9,10,11,120,150)",
            moderate_dead[0], moderate_dead[1], moderate_dead[2],
            fully_green_herb[0], fully_green_herb[1]);

        // D4: High dead
        // NOTE: C++ descriptions say "D3L1" etc. for D4 scenarios (copy-paste bug).
        // We preserve those descriptions for parity.
        self.add_record("D4L1", "D3L1 - High dead, fully cured herb (12,13,14,30,60)",
            high_dead[0], high_dead[1], high_dead[2],
            fully_cured_herb[0], fully_cured_herb[1]);
        self.add_record("D4L2", "D3L2 - High dead, 2/3 cured herb (12,13,14,60,90)",
            high_dead[0], high_dead[1], high_dead[2],
            two_thirds_cured_herb[0], two_thirds_cured_herb[1]);
        self.add_record("D4L3", "D3L3 - High dead, 1/3 cured herb (12,13,14,90,120)",
            high_dead[0], high_dead[1], high_dead[2],
            one_third_cured_herb[0], one_third_cured_herb[1]);
        self.add_record("D4L4", "D3L4 - High dead, fully green herb (12,13,14,120,150)",
            high_dead[0], high_dead[1], high_dead[2],
            fully_green_herb[0], fully_green_herb[1]);
    }
}

impl Default for MoistureScenarios {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenarios() -> MoistureScenarios {
        MoistureScenarios::new()
    }

    #[test]
    fn total_scenario_count() {
        assert_eq!(scenarios().num_scenarios(), 16);
    }

    #[test]
    fn d1l1_by_name() {
        let s = scenarios();
        assert!(s.is_defined_by_name("D1L1"));
        assert_eq!(s.one_hour_by_name("D1L1", FractionUnits::Fraction), 0.03);
        assert_eq!(s.ten_hour_by_name("D1L1", FractionUnits::Fraction), 0.04);
        assert_eq!(s.hundred_hour_by_name("D1L1", FractionUnits::Fraction), 0.05);
        assert_eq!(s.live_herbaceous_by_name("D1L1", FractionUnits::Fraction), 0.30);
        assert_eq!(s.live_woody_by_name("D1L1", FractionUnits::Fraction), 0.60);
    }

    #[test]
    fn d1l1_as_percent() {
        let s = scenarios();
        let pct = FractionUnits::Percent;
        assert!((s.one_hour_by_name("D1L1", pct) - 3.0).abs() < 1e-10);
        assert!((s.live_herbaceous_by_name("D1L1", pct) - 30.0).abs() < 1e-10);
    }

    #[test]
    fn d2l3_by_name() {
        let s = scenarios();
        assert_eq!(s.one_hour_by_name("D2L3", FractionUnits::Fraction), 0.06);
        assert_eq!(s.ten_hour_by_name("D2L3", FractionUnits::Fraction), 0.07);
        assert_eq!(s.hundred_hour_by_name("D2L3", FractionUnits::Fraction), 0.08);
        assert_eq!(s.live_herbaceous_by_name("D2L3", FractionUnits::Fraction), 0.90);
        assert_eq!(s.live_woody_by_name("D2L3", FractionUnits::Fraction), 1.20);
    }

    #[test]
    fn d4l4_by_name() {
        let s = scenarios();
        assert_eq!(s.one_hour_by_name("D4L4", FractionUnits::Fraction), 0.12);
        assert_eq!(s.live_woody_by_name("D4L4", FractionUnits::Fraction), 1.50);
    }

    #[test]
    fn case_insensitive_lookup() {
        let s = scenarios();
        assert_eq!(s.index_by_name("d1l1"), s.index_by_name("D1L1"));
        assert!(s.is_defined_by_name("d3l2"));
    }

    #[test]
    fn lookup_by_index() {
        let s = scenarios();
        assert!(s.is_defined_by_index(0));
        assert_eq!(s.name_by_index(0), "D1L1");
        assert_eq!(s.one_hour_by_index(0, FractionUnits::Fraction), 0.03);

        assert!(s.is_defined_by_index(15));
        assert_eq!(s.name_by_index(15), "D4L4");
    }

    #[test]
    fn undefined_name_returns_negative() {
        let s = scenarios();
        assert!(!s.is_defined_by_name("BOGUS"));
        assert_eq!(s.index_by_name("BOGUS"), -1);
        assert_eq!(s.one_hour_by_name("BOGUS", FractionUnits::Fraction), -1.0);
    }

    #[test]
    fn undefined_index_returns_negative() {
        let s = scenarios();
        assert!(!s.is_defined_by_index(-1));
        assert!(!s.is_defined_by_index(16));
        assert_eq!(s.one_hour_by_index(-1, FractionUnits::Fraction), -1.0);
        assert_eq!(s.one_hour_by_index(99, FractionUnits::Fraction), -1.0);
    }

    #[test]
    fn d4_descriptions_preserve_cpp_bug() {
        // C++ has "D3L1" in the description for D4L1 (copy-paste bug), preserved for parity
        let s = scenarios();
        let desc = s.description_by_name("D4L1");
        assert!(desc.starts_with("D3L1"), "got: {}", desc);
    }

    #[test]
    fn all_scenarios_sequential_indices() {
        let s = scenarios();
        let names = [
            "D1L1", "D1L2", "D1L3", "D1L4",
            "D2L1", "D2L2", "D2L3", "D2L4",
            "D3L1", "D3L2", "D3L3", "D3L4",
            "D4L1", "D4L2", "D4L3", "D4L4",
        ];
        for (i, expected_name) in names.iter().enumerate() {
            assert_eq!(s.name_by_index(i as i32), *expected_name);
        }
    }
}
