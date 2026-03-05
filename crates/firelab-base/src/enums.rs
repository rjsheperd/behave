//! Cross-crate domain enums used by two or more BehavePlus crates.
//!
//! C++ sources: surfaceInputEnums.h, crown.h

/// How midflame wind speed is derived.
///
/// C++ source: `WindHeightInputMode` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindHeightInputMode {
    /// User enters midflame wind speed directly.
    DirectMidflame = 0,
    /// User enters 20-foot wind speed.
    TwentyFoot = 1,
    /// User enters 10-meter wind speed.
    TenMeter = 2,
}

/// Orientation reference for wind direction and spread angles.
///
/// C++ source: `WindAndSpreadOrientationMode` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindAndSpreadOrientationMode {
    /// Angles relative to upslope direction.
    RelativeToUpslope = 0,
    /// Angles relative to compass north.
    RelativeToNorth = 1,
}

/// How the wind adjustment factor is calculated.
///
/// C++ source: `WindAdjustmentFactorCalculationMethod` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindAdjustmentFactorCalculationMethod {
    /// User provides WAF directly.
    UserInput = 0,
    /// Calculate WAF using crown ratio.
    UseCrownRatio = 1,
    /// Calculate WAF without crown ratio.
    DontUseCrownRatio = 2,
}

/// How moisture values are provided to the model.
///
/// C++ source: `MoistureInputMode` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoistureInputMode {
    BySizeClass = 0,
    AllAggregate = 1,
    DeadAggregateAndLiveSizeClass = 2,
    LiveAggregateAndDeadSizeClass = 3,
    MoistureScenario = 4,
}

/// Individual moisture size-class identifiers.
///
/// C++ source: `MoistureClassInput` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoistureClassInput {
    OneHour = 0,
    TenHour = 1,
    HundredHour = 2,
    LiveHerbaceous = 3,
    LiveWoody = 4,
    DeadAggregate = 5,
    LiveAggregate = 6,
}

/// Dead vs. live fuel classification.
///
/// C++ source: `FuelLifeState` in surfaceInputEnums.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuelLifeState {
    Dead = 0,
    Live = 1,
}

/// Foundational fuel-model array-size constants.
///
/// C++ source: `FuelConstants` in surfaceInputEnums.h
pub struct FuelConstants;

impl FuelConstants {
    pub const MAX_LIFE_STATES: usize = 2;
    pub const MAX_LIVE_SIZE_CLASSES: usize = 5;
    pub const MAX_DEAD_SIZE_CLASSES: usize = 4;
    pub const MAX_PARTICLES: usize = 5;
    pub const MAX_SAVR_SIZE_CLASSES: usize = 5;
    pub const MAX_FUEL_MODELS: usize = 256;
}

/// Crown fire type classification.
///
/// C++ source: `FireType` in crown.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireType {
    /// Surface fire only.
    Surface = 0,
    /// Surface fire with torching.
    Torching = 1,
    /// Active crown fire possible if transition occurs.
    ConditionalCrownFire = 2,
    /// Active crown fire spreading through canopy.
    Crowning = 3,
}

#[cfg(test)]
mod tests {}
