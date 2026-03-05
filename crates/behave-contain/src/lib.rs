//! Fire containment simulation module.
//!
//! Implements the Fried & Fried wildfire containment model.
//!
//! C++ sources: Contain.h, ContainSim.h, ContainForce.h, ContainResource.h,
//! ContainForceAdapter.h, ContainAdapter.h

#![allow(dead_code)]

pub mod adapter;
pub mod algorithm;
pub mod force;
pub mod force_adapter;
pub mod resource;
pub mod sim;
