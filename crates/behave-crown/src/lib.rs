//! Crown fire behavior module.
//!
//! Implements both Rothermel (1991) and Scott & Reinhardt (2001) crown fire
//! methods. The Crown struct owns two Surface instances internally.
//!
//! C++ sources: crown.h, crownInputs.h, CrownFirebrandProcessor.h

#![allow(dead_code)]

pub mod fire;
pub mod firebrand;
pub mod inputs;
