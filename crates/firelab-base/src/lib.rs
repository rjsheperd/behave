//! Base types for the BehavePlus Rust port.
//!
//! Provides unit conversion enums, cross-crate domain enums, error types,
//! and the FireSize geometry module.
//!
//! C++ sources: behaveUnits.h, surfaceInputEnums.h (cross-crate subset), fireSize.h

#![allow(dead_code)]

pub mod enums;
pub mod error;
pub mod fire_size;
pub mod units;

pub use enums::*;
pub use error::*;
pub use fire_size::*;
pub use units::*;
