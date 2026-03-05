//! Surface fire spread module (Rothermel model).
//!
//! C++ sources: surface.h, surfaceInputs.h, surfaceFuelbedIntermediates.h,
//! surfaceFireReactionIntensity.h, surfaceFire.h, surfaceTwoFuelModels.h,
//! fuelModels.h, moistureScenarios.h, chaparralFuel.h, palmettoGallberry.h,
//! westernAspen.h, windAdjustmentFactor.h, windSpeedUtility.h

#![allow(dead_code)]

pub mod chaparral;
pub mod facade;
pub mod fire;
pub mod fire_size;
pub mod fuelbed;
pub mod fuel_models;
pub mod inputs;
pub mod moisture;
pub mod palmetto_gallberry;
pub mod reaction;
pub mod two_fuel_models;
pub mod western_aspen;
pub mod wind;
