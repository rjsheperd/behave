//! Canopy coefficient table for crown width calculations.
//!
//! Coefficients for tree crown widths based on data from R6 Permanent Plot
//! Grid Inventory. The index number is the FVS Species Index Number, used
//! as a lookup code in the species master table.
//!
//! For trees with height > 4.5 ft: crown_width = a * dbh^b
//! For trees with height <= 4.5 ft: crown_width = r * height
//!
//! C++ source: canopy_coefficient_table.h / canopy_coefficient_table.cpp

/// A single row in the canopy coefficient table.
///
/// C++ struct: `CanopyCoefficientTableRecord`
#[derive(Debug, Clone)]
pub struct CanopyCoefficientRecord {
    pub index_number: i32,
    pub crown_code: &'static str,
    pub coefficient_a: f64,
    pub coefficient_b: f64,
    pub coefficient_r: f64,
}

/// Lookup table for canopy damage coefficients, indexed by FVS species index.
///
/// C++ class: `CanopyCoefficientTable`
#[derive(Debug, Clone)]
pub struct CanopyCoefficientTable {
    records: Vec<CanopyCoefficientRecord>,
}

impl CanopyCoefficientTable {
    pub fn new() -> Self {
        Self {
            records: CANOPY_COEFFICIENTS.to_vec(),
        }
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Get coefficients by index number (FVS species index).
    /// Returns None if index is out of bounds or refers to a dummy record.
    pub fn get_by_index(&self, index: usize) -> Option<&CanopyCoefficientRecord> {
        self.records.get(index).filter(|r| r.index_number >= 0)
    }
}

impl Default for CanopyCoefficientTable {
    fn default() -> Self {
        Self::new()
    }
}

// Static table matching C++ initializer list exactly.
// Dummy records (index_number = -1) preserve C++ array indexing.
static CANOPY_COEFFICIENTS: &[CanopyCoefficientRecord] = &[
    //                        idx  code      a        b       r
    CanopyCoefficientRecord { index_number: -1, crown_code: "",   coefficient_a: 0.0,    coefficient_b: 0.0,    coefficient_r: 0.0 },    // 0: dummy
    CanopyCoefficientRecord { index_number:  1, crown_code: "SF", coefficient_a: 3.9723, coefficient_b: 0.5177, coefficient_r: 0.473 },  // 1
    CanopyCoefficientRecord { index_number:  2, crown_code: "WF", coefficient_a: 3.8166, coefficient_b: 0.5229, coefficient_r: 0.452 },  // 2
    CanopyCoefficientRecord { index_number:  3, crown_code: "GF", coefficient_a: 4.1870, coefficient_b: 0.5341, coefficient_r: 0.489 },  // 3
    CanopyCoefficientRecord { index_number:  4, crown_code: "AF", coefficient_a: 3.2348, coefficient_b: 0.5179, coefficient_r: 0.385 },  // 4
    CanopyCoefficientRecord { index_number:  5, crown_code: "RF", coefficient_a: 3.1146, coefficient_b: 0.5780, coefficient_r: 0.345 },  // 5
    CanopyCoefficientRecord { index_number: -1, crown_code: "",   coefficient_a: 0.0,    coefficient_b: 0.0,    coefficient_r: 0.0 },    // 6: dummy
    CanopyCoefficientRecord { index_number:  7, crown_code: "NF", coefficient_a: 3.0614, coefficient_b: 0.6276, coefficient_r: 0.320 },  // 7
    CanopyCoefficientRecord { index_number:  8, crown_code: "YC", coefficient_a: 3.5341, coefficient_b: 0.5374, coefficient_r: 0.331 },  // 8
    CanopyCoefficientRecord { index_number:  9, crown_code: "C",  coefficient_a: 4.0920, coefficient_b: 0.4912, coefficient_r: 0.412 },  // 9
    CanopyCoefficientRecord { index_number: 10, crown_code: "S",  coefficient_a: 3.6802, coefficient_b: 0.4940, coefficient_r: 0.412 },  // 10
    CanopyCoefficientRecord { index_number: 11, crown_code: "LP", coefficient_a: 2.4132, coefficient_b: 0.6403, coefficient_r: 0.298 },  // 11
    CanopyCoefficientRecord { index_number: 12, crown_code: "JP", coefficient_a: 3.2367, coefficient_b: 0.6247, coefficient_r: 0.406 },  // 12
    CanopyCoefficientRecord { index_number: 13, crown_code: "SP", coefficient_a: 3.0610, coefficient_b: 0.6201, coefficient_r: 0.385 },  // 13
    CanopyCoefficientRecord { index_number: 14, crown_code: "WP", coefficient_a: 3.4447, coefficient_b: 0.5185, coefficient_r: 0.476 },  // 14
    CanopyCoefficientRecord { index_number: 15, crown_code: "PP", coefficient_a: 2.8541, coefficient_b: 0.6400, coefficient_r: 0.407 },  // 15
    CanopyCoefficientRecord { index_number: 16, crown_code: "DF", coefficient_a: 4.4215, coefficient_b: 0.5329, coefficient_r: 0.517 },  // 16
    CanopyCoefficientRecord { index_number: 17, crown_code: "RW", coefficient_a: 4.4215, coefficient_b: 0.5329, coefficient_r: 0.517 },  // 17
    CanopyCoefficientRecord { index_number: 18, crown_code: "RC", coefficient_a: 6.2318, coefficient_b: 0.4259, coefficient_r: 0.698 },  // 18
    CanopyCoefficientRecord { index_number: 19, crown_code: "WH", coefficient_a: 5.4864, coefficient_b: 0.5144, coefficient_r: 0.533 },  // 19
    CanopyCoefficientRecord { index_number: 20, crown_code: "MH", coefficient_a: 2.9372, coefficient_b: 0.5878, coefficient_r: 0.253 },  // 20
    CanopyCoefficientRecord { index_number: 21, crown_code: "BM", coefficient_a: 7.5183, coefficient_b: 0.4461, coefficient_r: 0.815 },  // 21
    CanopyCoefficientRecord { index_number: 22, crown_code: "RA", coefficient_a: 7.0806, coefficient_b: 0.4771, coefficient_r: 0.730 },  // 22
    CanopyCoefficientRecord { index_number: 23, crown_code: "WA", coefficient_a: 7.0806, coefficient_b: 0.4771, coefficient_r: 0.730 },  // 23
    CanopyCoefficientRecord { index_number: 24, crown_code: "PB", coefficient_a: 5.8980, coefficient_b: 0.4841, coefficient_r: 0.601 },  // 24
    CanopyCoefficientRecord { index_number: 25, crown_code: "GC", coefficient_a: 2.4922, coefficient_b: 0.8544, coefficient_r: 0.140 },  // 25
    CanopyCoefficientRecord { index_number: 26, crown_code: "AS", coefficient_a: 4.0910, coefficient_b: 0.5907, coefficient_r: 0.351 },  // 26
    CanopyCoefficientRecord { index_number: 27, crown_code: "CW", coefficient_a: 7.5183, coefficient_b: 0.4461, coefficient_r: 0.815 },  // 27
    CanopyCoefficientRecord { index_number: 28, crown_code: "WO", coefficient_a: 2.4922, coefficient_b: 0.8544, coefficient_r: 0.140 },  // 28
    CanopyCoefficientRecord { index_number: 29, crown_code: "J",  coefficient_a: 4.5859, coefficient_b: 0.4841, coefficient_r: 0.468 },  // 29
    CanopyCoefficientRecord { index_number: 30, crown_code: "LL", coefficient_a: 2.1039, coefficient_b: 0.6758, coefficient_r: 0.207 },  // 30
    CanopyCoefficientRecord { index_number: 31, crown_code: "WB", coefficient_a: 2.1606, coefficient_b: 0.6897, coefficient_r: 0.255 },  // 31
    CanopyCoefficientRecord { index_number: 32, crown_code: "KP", coefficient_a: 2.1451, coefficient_b: 0.7132, coefficient_r: 0.248 },  // 32
    CanopyCoefficientRecord { index_number: 33, crown_code: "PY", coefficient_a: 4.5859, coefficient_b: 0.4841, coefficient_r: 0.468 },  // 33
    CanopyCoefficientRecord { index_number: 34, crown_code: "DG", coefficient_a: 2.4922, coefficient_b: 0.8544, coefficient_r: 0.140 },  // 34
    CanopyCoefficientRecord { index_number: 35, crown_code: "HT", coefficient_a: 4.5859, coefficient_b: 0.4841, coefficient_r: 0.468 },  // 35
    CanopyCoefficientRecord { index_number: 36, crown_code: "CH", coefficient_a: 4.5859, coefficient_b: 0.4841, coefficient_r: 0.468 },  // 36
    CanopyCoefficientRecord { index_number: 37, crown_code: "WI", coefficient_a: 4.5859, coefficient_b: 0.4841, coefficient_r: 0.468 },  // 37
    CanopyCoefficientRecord { index_number: -1, crown_code: "",   coefficient_a: 0.0,    coefficient_b: 0.0,    coefficient_r: 0.0 },    // 38: dummy
    CanopyCoefficientRecord { index_number: 39, crown_code: "",   coefficient_a: 4.4215, coefficient_b: 0.5329, coefficient_r: 0.517 },  // 39: Other
    CanopyCoefficientRecord { index_number: -1, crown_code: "",   coefficient_a: 0.0,    coefficient_b: 0.0,    coefficient_r: 0.0 },    // 40: dummy (sentinel)
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_size() {
        let t = CanopyCoefficientTable::new();
        assert_eq!(t.record_count(), 41);
    }

    #[test]
    fn dummy_records_return_none() {
        let t = CanopyCoefficientTable::new();
        assert!(t.get_by_index(0).is_none());
        assert!(t.get_by_index(6).is_none());
        assert!(t.get_by_index(38).is_none());
        assert!(t.get_by_index(40).is_none());
    }

    #[test]
    fn silver_fir_index_1() {
        let t = CanopyCoefficientTable::new();
        let r = t.get_by_index(1).unwrap();
        assert_eq!(r.crown_code, "SF");
        assert_eq!(r.coefficient_a, 3.9723);
        assert_eq!(r.coefficient_b, 0.5177);
        assert_eq!(r.coefficient_r, 0.473);
    }

    #[test]
    fn ponderosa_pine_index_15() {
        let t = CanopyCoefficientTable::new();
        let r = t.get_by_index(15).unwrap();
        assert_eq!(r.crown_code, "PP");
        assert_eq!(r.coefficient_a, 2.8541);
        assert_eq!(r.coefficient_b, 0.6400);
        assert_eq!(r.coefficient_r, 0.407);
    }

    #[test]
    fn other_index_39() {
        let t = CanopyCoefficientTable::new();
        let r = t.get_by_index(39).unwrap();
        assert_eq!(r.crown_code, "");
        assert_eq!(r.coefficient_a, 4.4215);
    }

    #[test]
    fn out_of_bounds() {
        let t = CanopyCoefficientTable::new();
        assert!(t.get_by_index(100).is_none());
    }
}
