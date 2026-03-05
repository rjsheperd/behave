//! Tree mortality calculations.
//!
//! C++ sources: mortality.h, mortality_inputs.h, mortality_equation_table.h,
//! species_master_table.h, canopy_coefficient_table.h

#![allow(dead_code)]

pub mod calculator;
pub mod canopy;
pub mod equations;
pub mod inputs;
pub mod safety;
pub mod separation;
pub mod species;
