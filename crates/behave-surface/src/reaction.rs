//! Reaction intensity calculations (Rothermel 1972, equation 27).
//!
//! C++ source: surfaceFireReactionIntensity.h / surfaceFireReactionIntensity.cpp

use firelab_base::{FuelConstants, FuelLifeState, HeatSourceAndReactionIntensityUnits};

use crate::fuelbed::FuelbedIntermediates;

const MAX_LIFE: usize = FuelConstants::MAX_LIFE_STATES;
const DEAD: usize = FuelLifeState::Dead as usize;
const LIVE: usize = FuelLifeState::Live as usize;

/// Computes the Rothermel reaction intensity for a fuelbed.
///
/// C++ class: `SurfaceFireReactionIntensity`
#[derive(Debug, Clone)]
pub struct ReactionIntensity {
    eta_m: [f64; MAX_LIFE],
    eta_s: [f64; MAX_LIFE],
    reaction_intensity_for_life_state: [f64; MAX_LIFE],
    reaction_intensity: f64,
}

impl ReactionIntensity {
    pub fn new() -> Self {
        Self {
            eta_m: [0.0; MAX_LIFE],
            eta_s: [0.0; MAX_LIFE],
            reaction_intensity_for_life_state: [0.0; MAX_LIFE],
            reaction_intensity: 0.0,
        }
    }

    /// Calculate reaction intensity from fuelbed intermediates.
    ///
    /// Returns total reaction intensity in Btu/ft²/min.
    ///
    /// C++ method: `calculateReactionIntensity`
    pub fn calculate_reaction_intensity(&mut self, fuelbed: &FuelbedIntermediates) -> f64 {
        self.reaction_intensity = 0.0;

        let sigma = fuelbed.sigma();
        let relative_packing_ratio = fuelbed.relative_packing_ratio();

        // Alternate "A" variable, Albini 1976, p. 88
        let aa = 133.0 / sigma.powf(0.7913);

        // Optimum reaction velocity, Rothermel 1972
        let sigma_1_5 = sigma.powf(1.5);
        let gamma_max = sigma_1_5 / (495.0 + 0.0594 * sigma_1_5);
        let gamma =
            gamma_max * relative_packing_ratio.powf(aa) * (aa * (1.0 - relative_packing_ratio)).exp();

        let weighted_fuel_load = [
            fuelbed.weighted_fuel_load_by_life_state(FuelLifeState::Dead),
            fuelbed.weighted_fuel_load_by_life_state(FuelLifeState::Live),
        ];
        let weighted_heat = [
            fuelbed.weighted_heat_by_life_state(FuelLifeState::Dead),
            fuelbed.weighted_heat_by_life_state(FuelLifeState::Live),
        ];

        self.calculate_eta_m(fuelbed);
        self.calculate_eta_s(fuelbed);

        for i in 0..MAX_LIFE {
            self.reaction_intensity_for_life_state[i] =
                gamma * weighted_fuel_load[i] * weighted_heat[i] * self.eta_m[i] * self.eta_s[i];
        }

        self.reaction_intensity = self.reaction_intensity_for_life_state[DEAD]
            + self.reaction_intensity_for_life_state[LIVE];

        self.reaction_intensity
    }

    /// Get total reaction intensity.
    ///
    /// NOTE: C++ `getReactionIntensity()` accepts units parameter but ignores it,
    /// always returning the base-unit value (Btu/ft²/min). We preserve this behavior.
    pub fn reaction_intensity(
        &self,
        _units: HeatSourceAndReactionIntensityUnits,
    ) -> f64 {
        self.reaction_intensity
    }

    /// Get total reaction intensity in base units (Btu/ft²/min).
    pub fn reaction_intensity_base(&self) -> f64 {
        self.reaction_intensity
    }

    pub fn reaction_intensity_for_life_state(&self, life_state: FuelLifeState) -> f64 {
        self.reaction_intensity_for_life_state[life_state as usize]
    }

    // --- Internal ---

    /// Moisture damping coefficient, Rothermel 1972.
    fn calculate_eta_m(&mut self, fuelbed: &FuelbedIntermediates) {
        let weighted_moisture = [
            fuelbed.weighted_moisture_by_life_state(FuelLifeState::Dead),
            fuelbed.weighted_moisture_by_life_state(FuelLifeState::Live),
        ];
        let moisture_of_extinction = [
            fuelbed.moisture_of_extinction_by_life_state(FuelLifeState::Dead),
            fuelbed.moisture_of_extinction_by_life_state(FuelLifeState::Live),
        ];

        for i in 0..MAX_LIFE {
            let mut relative_moisture = 0.0;
            if moisture_of_extinction[i] > 0.0 {
                relative_moisture = weighted_moisture[i] / moisture_of_extinction[i];
            }
            if weighted_moisture[i] >= moisture_of_extinction[i] || relative_moisture > 1.0 {
                self.eta_m[i] = 0.0;
            } else {
                let rm = relative_moisture;
                self.eta_m[i] = 1.0 - 2.59 * rm + 5.11 * rm * rm - 3.52 * rm * rm * rm;
            }
        }
    }

    /// Mineral (silica) damping coefficient, Rothermel 1972.
    fn calculate_eta_s(&mut self, fuelbed: &FuelbedIntermediates) {
        let weighted_silica = [
            fuelbed.weighted_silica_by_life_state(FuelLifeState::Dead),
            fuelbed.weighted_silica_by_life_state(FuelLifeState::Live),
        ];

        for i in 0..MAX_LIFE {
            let denominator = weighted_silica[i].powf(0.19);
            if denominator < 1e-6 {
                self.eta_s[i] = 0.0;
            } else {
                self.eta_s[i] = 0.174 / denominator;
            }
            if self.eta_s[i] > 1.0 {
                self.eta_s[i] = 1.0;
            }
        }
    }
}

impl Default for ReactionIntensity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fuel_models::FuelModels;
    use crate::fuelbed::FuelbedIntermediates;
    use crate::inputs::SurfaceInputs;
    use firelab_base::{
        FractionUnits, LengthUnits, SlopeUnits, SpeedUnits,
        WindAndSpreadOrientationMode, WindHeightInputMode,
    };

    fn make_inputs_and_fuelbed(
        fuel_model: i32,
        m1h: f64,
        m10h: f64,
        m100h: f64,
        m_herb: f64,
        m_woody: f64,
    ) -> (FuelModels, SurfaceInputs, FuelbedIntermediates) {
        let fm = FuelModels::new();
        let mut inputs = SurfaceInputs::new();
        inputs.update_surface_inputs(
            fuel_model,
            m1h, m10h, m100h, m_herb, m_woody,
            FractionUnits::Fraction,
            0.0, SpeedUnits::FeetPerMinute,
            WindHeightInputMode::DirectMidflame,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            0.0, SlopeUnits::Degrees, 0.0,
            0.0, FractionUnits::Fraction,
            0.0, LengthUnits::Feet,
            0.0, FractionUnits::Fraction,
        );
        let mut fb = FuelbedIntermediates::new();
        fb.calculate_fuelbed_intermediates(fuel_model, &fm, &inputs);
        (fm, inputs, fb)
    }

    #[test]
    fn fm1_reaction_intensity_positive() {
        let (_fm, _inputs, fb) = make_inputs_and_fuelbed(1, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut ri = ReactionIntensity::new();
        let result = ri.calculate_reaction_intensity(&fb);
        assert!(result > 0.0, "reaction_intensity={}", result);
    }

    #[test]
    fn fm1_dead_component_dominates() {
        let (_fm, _inputs, fb) = make_inputs_and_fuelbed(1, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut ri = ReactionIntensity::new();
        ri.calculate_reaction_intensity(&fb);

        // FM1 is short grass, dead fuel only — dead component should be the total
        let dead_ri = ri.reaction_intensity_for_life_state(FuelLifeState::Dead);
        let live_ri = ri.reaction_intensity_for_life_state(FuelLifeState::Live);
        assert!(dead_ri > 0.0);
        // FM1 has no live fuel, so live RI should be 0
        assert!(
            live_ri.abs() < 1e-10,
            "FM1 live RI should be ~0, got {}",
            live_ri
        );
    }

    #[test]
    fn fm10_has_both_components() {
        let (_fm, _inputs, fb) = make_inputs_and_fuelbed(10, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut ri = ReactionIntensity::new();
        ri.calculate_reaction_intensity(&fb);

        let dead_ri = ri.reaction_intensity_for_life_state(FuelLifeState::Dead);
        let live_ri = ri.reaction_intensity_for_life_state(FuelLifeState::Live);
        // FM10 has both dead and live (woody) fuels
        assert!(dead_ri > 0.0, "dead_ri={}", dead_ri);
        assert!(live_ri > 0.0, "live_ri={}", live_ri);
    }

    #[test]
    fn higher_moisture_reduces_intensity() {
        let (_fm1, _inp1, fb_dry) = make_inputs_and_fuelbed(1, 0.03, 0.04, 0.05, 0.60, 1.50);
        let (_fm2, _inp2, fb_wet) = make_inputs_and_fuelbed(1, 0.10, 0.12, 0.14, 0.60, 1.50);

        let mut ri_dry = ReactionIntensity::new();
        let mut ri_wet = ReactionIntensity::new();
        let dry_val = ri_dry.calculate_reaction_intensity(&fb_dry);
        let wet_val = ri_wet.calculate_reaction_intensity(&fb_wet);

        assert!(
            dry_val > wet_val,
            "Drier fuel should have higher RI: dry={}, wet={}",
            dry_val,
            wet_val
        );
    }

    #[test]
    fn reaction_intensity_getter_ignores_units() {
        // C++ getReactionIntensity ignores its units parameter
        let (_fm, _inputs, fb) = make_inputs_and_fuelbed(1, 0.06, 0.07, 0.08, 0.60, 1.50);
        let mut ri = ReactionIntensity::new();
        ri.calculate_reaction_intensity(&fb);

        let base = ri.reaction_intensity_base();
        let with_units =
            ri.reaction_intensity(HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerMinute);
        assert!((base - with_units).abs() < 1e-10);
    }
}
