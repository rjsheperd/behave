//! Spot fire distance calculations.
//!
//! Calculates spotting distance from burning piles, surface fires,
//! torching trees, and active crown fires for both flat and mountainous
//! terrain.
//!
//! C++ source: spot.h / spot.cpp

use std::f64::consts::PI;

use behave_crown::firebrand::CrownFirebrandProcessor;
use firelab_base::{
    FirelineIntensityUnits, LengthUnits, SpeedUnits, TimeUnits, UnitConversion,
};

use crate::inputs::{
    SpotDownWindCanopyMode, SpotFireLocation, SpotInputs, SpotTreeSpecies,
};

/// Species-specific flame height parameters [a, b] where:
/// flame_height = a * DBH^b * torchingTrees^0.4
const SPECIES_FLAME_HEIGHT_PARAMS: [[f64; 2]; 14] = [
    [15.7, 0.451],  //  0 Engelmann spruce
    [15.7, 0.451],  //  1 Douglas-fir
    [15.7, 0.451],  //  2 subalpine fir
    [15.7, 0.451],  //  3 western hemlock
    [12.9, 0.453],  //  4 ponderosa pine
    [12.9, 0.453],  //  5 lodgepole pine
    [12.9, 0.453],  //  6 western white pine
    [16.5, 0.515],  //  7 grand fir
    [16.5, 0.515],  //  8 balsam fir
    [2.71, 1.000],  //  9 slash pine
    [2.71, 1.000],  // 10 longleaf pine
    [2.71, 1.000],  // 11 pond pine
    [2.71, 1.000],  // 12 shortleaf pine
    [2.71, 1.000],  // 13 loblolly pine
];

/// Species-specific flame duration parameters [a, b] where:
/// flame_duration = a * DBH^b * torchingTrees^-0.2
const SPECIES_FLAME_DURATION_PARAMS: [[f64; 2]; 14] = [
    [12.6, -0.256],  //  0 Engelmann spruce
    [10.7, -0.278],  //  1 Douglas-fir
    [10.7, -0.278],  //  2 subalpine fir
    [6.30, -0.249],  //  3 western hemlock
    [12.6, -0.256],  //  4 ponderosa pine
    [12.6, -0.256],  //  5 lodgepole pine
    [10.7, -0.278],  //  6 western white pine
    [10.7, -0.278],  //  7 grand fir
    [10.7, -0.278],  //  8 balsam fir
    [11.9, -0.389],  //  9 slash pine
    [11.9, -0.389],  // 10 longleaf pine
    [7.91, -0.344],  // 11 pond pine
    [7.91, -0.344],  // 12 shortleaf pine
    [13.5, -0.544],  // 13 loblolly pine
];

/// Firebrand height factors indexed by flame ratio/duration category.
const FIREBRAND_HEIGHT_FACTORS: [[f64; 2]; 4] = [
    [4.24, 0.332],
    [3.64, 0.391],
    [2.78, 0.418],
    [4.70, 0.000],
];

/// Spot fire distance calculator for four source types:
/// burning piles, surface fires, torching trees, and active crown fires.
///
/// C++ class: `Spot`
#[derive(Debug, Clone)]
pub struct Spot {
    spot_inputs: SpotInputs,

    // Intermediate outputs
    cover_height_used_for_surface_fire: f64,
    cover_height_used_for_burning_pile: f64,
    cover_height_used_for_torching_trees: f64,
    flame_height_for_torching_trees: f64,
    flame_ratio: f64,
    firebrand_drift: f64,
    flame_duration: f64,
    firebrand_height_from_burning_pile: f64,
    firebrand_height_from_surface_fire: f64,
    firebrand_height_from_torching_trees: f64,

    // Flat terrain distances (stored in ft, base unit)
    flat_distance_from_burning_pile: f64,
    flat_distance_from_surface_fire: f64,
    flat_distance_from_torching_trees: f64,
    flat_distance_from_active_crown: f64,

    // Mountain terrain distances (stored in ft, base unit)
    mountain_distance_from_burning_pile: f64,
    mountain_distance_from_surface_fire: f64,
    mountain_distance_from_torching_trees: f64,
    mountain_distance_from_active_crown: f64,
}

impl Spot {
    pub fn new() -> Self {
        Self {
            spot_inputs: SpotInputs::new(),
            cover_height_used_for_surface_fire: 0.0,
            cover_height_used_for_burning_pile: 0.0,
            cover_height_used_for_torching_trees: 0.0,
            flame_height_for_torching_trees: 0.0,
            flame_ratio: 0.0,
            firebrand_drift: 0.0,
            flame_duration: 0.0,
            firebrand_height_from_burning_pile: 0.0,
            firebrand_height_from_surface_fire: 0.0,
            firebrand_height_from_torching_trees: 0.0,
            flat_distance_from_burning_pile: 0.0,
            flat_distance_from_surface_fire: 0.0,
            flat_distance_from_torching_trees: 0.0,
            flat_distance_from_active_crown: 0.0,
            mountain_distance_from_burning_pile: 0.0,
            mountain_distance_from_surface_fire: 0.0,
            mountain_distance_from_torching_trees: 0.0,
            mountain_distance_from_active_crown: 0.0,
        }
    }

    // ---------------------------------------------------------------
    // Static helper functions (matching C++ static/private methods)
    // ---------------------------------------------------------------

    /// Critical cover height: minimum cover height used in flat distance calc.
    fn calculate_spot_critical_cover_height(firebrand_height: f64, cover_height: f64) -> f64 {
        let critical_height = if firebrand_height < 1e-7 {
            0.0
        } else {
            2.2 * firebrand_height.powf(0.337) - 4.0
        };

        if cover_height > critical_height {
            cover_height
        } else {
            critical_height
        }
    }

    /// Flat terrain spotting distance (miles).
    /// All inputs in imperial: firebrand_height (ft), cover_height (ft),
    /// wind_speed (mi/h).
    fn spot_distance_flat_terrain(
        firebrand_height: f64,
        cover_height: f64,
        wind_speed_at_twenty_feet: f64,
    ) -> f64 {
        if cover_height > 1e-7 {
            0.000718 * wind_speed_at_twenty_feet * cover_height.sqrt()
                * (0.362
                    + (firebrand_height / cover_height).sqrt() / 2.0
                        * (firebrand_height / cover_height).ln())
        } else {
            0.0
        }
    }

    /// Mountain terrain spotting distance adjustment.
    /// flat_distance and ridge_to_valley_distance in miles,
    /// ridge_to_valley_elevation in feet.
    fn spot_distance_mountain_terrain(
        flat_distance: f64,
        location: SpotFireLocation,
        ridge_to_valley_distance: f64,
        ridge_to_valley_elevation: f64,
    ) -> f64 {
        let mut mountain_distance = flat_distance;
        if ridge_to_valley_elevation > 1e-7 && ridge_to_valley_distance > 1e-7 {
            let a1 = flat_distance / ridge_to_valley_distance;
            let b1 = ridge_to_valley_elevation / (10.0 * PI) / 1000.0;
            let loc = location as i32 as f64;
            let mut x = a1;
            for _ in 0..6 {
                x = a1
                    - b1 * ((PI * x - loc * PI / 2.0).cos()
                        - (loc * PI / 2.0).cos());
            }
            mountain_distance = x * ridge_to_valley_distance;
        }
        mountain_distance
    }

    /// Effective downwind cover height, halved if canopy mode is Open.
    fn calculate_downwind_canopy_cover_height(&self, units: LengthUnits) -> f64 {
        let height = self.spot_inputs.downwind_cover_height(units);
        if self.spot_inputs.downwind_canopy_mode() == SpotDownWindCanopyMode::Open {
            height * 0.5
        } else {
            height
        }
    }

    /// Effective tree height, halved if canopy mode is Open.
    fn calculate_tree_height(&self, units: LengthUnits) -> f64 {
        let height = self.spot_inputs.tree_height(units);
        if self.spot_inputs.downwind_canopy_mode() == SpotDownWindCanopyMode::Open {
            height * 0.5
        } else {
            height
        }
    }

    // ---------------------------------------------------------------
    // Main calculation methods
    // ---------------------------------------------------------------

    pub fn calculate_spotting_distance_from_burning_pile(&mut self) {
        let location = self.spot_inputs.location();
        let ridge_to_valley_distance = self.spot_inputs.ridge_to_valley_distance(LengthUnits::Miles);
        let ridge_to_valley_elevation = self.spot_inputs.ridge_to_valley_elevation(LengthUnits::Feet);
        let downwind_cover_height = self.calculate_downwind_canopy_cover_height(LengthUnits::Feet);
        let wind_speed = self.spot_inputs.wind_speed_at_twenty_feet(SpeedUnits::MilesPerHour);
        let pile_flame_height = self.spot_inputs.burning_pile_flame_height(LengthUnits::Feet);

        self.firebrand_height_from_burning_pile = 0.0;
        self.flat_distance_from_burning_pile = 0.0;
        self.mountain_distance_from_burning_pile = 0.0;

        if wind_speed > 1e-7 && pile_flame_height > 1e-7 {
            // Maximum firebrand height
            self.firebrand_height_from_burning_pile = 12.2 * pile_flame_height;

            // Cover height used in calculation
            self.cover_height_used_for_burning_pile = Self::calculate_spot_critical_cover_height(
                self.firebrand_height_from_burning_pile,
                downwind_cover_height,
            );

            if self.cover_height_used_for_burning_pile > 1e-7 {
                // Flat terrain spotting distance (miles)
                self.flat_distance_from_burning_pile = 0.000718 * wind_speed
                    * self.cover_height_used_for_burning_pile.sqrt()
                    * (0.362
                        + (self.firebrand_height_from_burning_pile
                            / self.cover_height_used_for_burning_pile)
                            .sqrt()
                            / 2.0
                            * (self.firebrand_height_from_burning_pile
                                / self.cover_height_used_for_burning_pile)
                                .ln());

                // Mountain terrain adjustment
                self.mountain_distance_from_burning_pile = Self::spot_distance_mountain_terrain(
                    self.flat_distance_from_burning_pile,
                    location,
                    ridge_to_valley_distance,
                    ridge_to_valley_elevation,
                );

                // Convert from miles to feet (base unit)
                self.flat_distance_from_burning_pile =
                    LengthUnits::Miles.to_base(self.flat_distance_from_burning_pile);
                self.mountain_distance_from_burning_pile =
                    LengthUnits::Miles.to_base(self.mountain_distance_from_burning_pile);
            }
        }
    }

    pub fn calculate_spotting_distance_from_surface_fire(&mut self) {
        let location = self.spot_inputs.location();
        let ridge_to_valley_distance = self.spot_inputs.ridge_to_valley_distance(LengthUnits::Miles);
        let ridge_to_valley_elevation = self.spot_inputs.ridge_to_valley_elevation(LengthUnits::Feet);
        let downwind_cover_height = self.calculate_downwind_canopy_cover_height(LengthUnits::Feet);
        let wind_speed = self.spot_inputs.wind_speed_at_twenty_feet(SpeedUnits::MilesPerHour);
        let flame_length = self.spot_inputs.surface_flame_length(LengthUnits::Feet);

        self.firebrand_height_from_surface_fire = 0.0;
        self.flat_distance_from_surface_fire = 0.0;
        self.firebrand_drift = 0.0;

        if wind_speed > 1e-7 && flame_length > 1e-7 {
            // f is a function relating thermal energy to windspeed
            let f = 322.0 * (0.474 * wind_speed).powf(-1.01);

            // Byram's fireline intensity derived from flame length
            let byrams = (flame_length / 0.45).powf(1.0 / 0.46);

            // Initial firebrand height (ft)
            self.firebrand_height_from_surface_fire = if (f * byrams) < 1e-7 {
                0.0
            } else {
                1.055 * (f * byrams).sqrt()
            };

            // Cover height used in calculation
            self.cover_height_used_for_surface_fire = Self::calculate_spot_critical_cover_height(
                self.firebrand_height_from_surface_fire,
                downwind_cover_height,
            );

            if self.cover_height_used_for_surface_fire > 1e-7 {
                self.firebrand_drift =
                    0.000278 * wind_speed * self.firebrand_height_from_surface_fire.powf(0.643);

                self.flat_distance_from_surface_fire = Self::spot_distance_flat_terrain(
                    self.firebrand_height_from_surface_fire,
                    self.cover_height_used_for_surface_fire,
                    wind_speed,
                ) + self.firebrand_drift;

                self.mountain_distance_from_surface_fire = Self::spot_distance_mountain_terrain(
                    self.flat_distance_from_surface_fire,
                    location,
                    ridge_to_valley_distance,
                    ridge_to_valley_elevation,
                );

                // Convert from miles to feet (base unit)
                self.flat_distance_from_surface_fire =
                    LengthUnits::Miles.to_base(self.flat_distance_from_surface_fire);
                self.mountain_distance_from_surface_fire =
                    LengthUnits::Miles.to_base(self.mountain_distance_from_surface_fire);
            }
        }
    }

    pub fn calculate_spotting_distance_from_torching_trees(&mut self) {
        let location = self.spot_inputs.location();
        let ridge_to_valley_distance = self.spot_inputs.ridge_to_valley_distance(LengthUnits::Miles);
        let ridge_to_valley_elevation = self.spot_inputs.ridge_to_valley_elevation(LengthUnits::Feet);
        let downwind_cover_height = self.calculate_downwind_canopy_cover_height(LengthUnits::Feet);
        let wind_speed = self.spot_inputs.wind_speed_at_twenty_feet(SpeedUnits::MilesPerHour);
        let torching_trees = self.spot_inputs.torching_trees();
        let dbh = self.spot_inputs.dbh(LengthUnits::Inches);
        let tree_height = self.spot_inputs.tree_height(LengthUnits::Feet);
        let species = self.spot_inputs.tree_species();
        let species_idx = species as usize;

        // Initialize return variables
        self.flame_ratio = 0.0;
        self.flame_height_for_torching_trees = 0.0;
        self.flame_duration = 0.0;
        self.firebrand_height_from_torching_trees = 0.0;
        self.flat_distance_from_torching_trees = 0.0;
        self.mountain_distance_from_torching_trees = 0.0;

        if wind_speed > 1e-7 && dbh > 1e-7 && torching_trees >= 1 {
            if species_idx < 14 {
                // Steady flame height (ft)
                self.flame_height_for_torching_trees =
                    SPECIES_FLAME_HEIGHT_PARAMS[species_idx][0]
                        * dbh.powf(SPECIES_FLAME_HEIGHT_PARAMS[species_idx][1])
                        * (torching_trees as f64).powf(0.4);

                self.flame_ratio = tree_height / self.flame_height_for_torching_trees;

                // Steady flame duration
                self.flame_duration =
                    SPECIES_FLAME_DURATION_PARAMS[species_idx][0]
                        * dbh.powf(SPECIES_FLAME_DURATION_PARAMS[species_idx][1])
                        * (torching_trees as f64).powf(-0.2);

                let i = if self.flame_ratio >= 1.0 {
                    0
                } else if self.flame_ratio >= 0.5 {
                    1
                } else if self.flame_duration < 3.5 {
                    2
                } else {
                    3
                };

                // Initial firebrand height (ft)
                self.firebrand_height_from_torching_trees =
                    FIREBRAND_HEIGHT_FACTORS[i][0]
                        * self.flame_duration.powf(FIREBRAND_HEIGHT_FACTORS[i][1])
                        * self.flame_height_for_torching_trees
                        + tree_height / 2.0;

                // Cover height used in calculation
                self.cover_height_used_for_torching_trees =
                    Self::calculate_spot_critical_cover_height(
                        self.firebrand_height_from_torching_trees,
                        downwind_cover_height,
                    );

                if self.cover_height_used_for_torching_trees > 1e-7 {
                    self.flat_distance_from_torching_trees = Self::spot_distance_flat_terrain(
                        self.firebrand_height_from_torching_trees,
                        self.cover_height_used_for_torching_trees,
                        wind_speed,
                    );

                    self.mountain_distance_from_torching_trees =
                        Self::spot_distance_mountain_terrain(
                            self.flat_distance_from_torching_trees,
                            location,
                            ridge_to_valley_distance,
                            ridge_to_valley_elevation,
                        );

                    // Convert from miles to feet (base unit)
                    self.flat_distance_from_torching_trees =
                        LengthUnits::Miles.to_base(self.flat_distance_from_torching_trees);
                    self.mountain_distance_from_torching_trees =
                        LengthUnits::Miles.to_base(self.mountain_distance_from_torching_trees);
                }
            }
        }
    }

    pub fn calculate_spotting_distance_from_active_crown(&mut self) {
        let location = self.spot_inputs.location();
        let ridge_to_valley_distance = self.spot_inputs.ridge_to_valley_distance(LengthUnits::Miles);
        let ridge_to_valley_elevation = self.spot_inputs.ridge_to_valley_elevation(LengthUnits::Feet);
        let tree_height = self.calculate_tree_height(LengthUnits::Meters);

        let mut fireline_intensity =
            self.spot_inputs.crown_fireline_intensity(FirelineIntensityUnits::KilowattsPerMeter);
        let flame_length = self.spot_inputs.active_crown_flame_length(LengthUnits::Meters);

        // Use Byram (1959) FLI approximation if FLI is null
        if fireline_intensity.abs() < 0.01 && flame_length > 0.0 {
            fireline_intensity = (flame_length / 0.0775).powf(1.0 / 0.46);
        }

        let wind_speed =
            self.spot_inputs.wind_speed_at_twenty_feet(SpeedUnits::KilometersPerHour);
        let wind_height = 6.096; // 20 ft in meters
        let ember_diam_mm = 0.5;

        let mut processor = CrownFirebrandProcessor::new_with_params(
            tree_height,
            fireline_intensity,
            wind_speed,
            wind_height,
            ember_diam_mm,
        );

        let flat_dist_m = processor.firebrand_distance();
        self.flat_distance_from_active_crown = LengthUnits::Meters.to_base(flat_dist_m);

        let flat_dist_mi =
            LengthUnits::Miles.from_base(self.flat_distance_from_active_crown);
        let mountain_dist_mi = Self::spot_distance_mountain_terrain(
            flat_dist_mi,
            location,
            ridge_to_valley_distance,
            ridge_to_valley_elevation,
        );
        self.mountain_distance_from_active_crown =
            LengthUnits::Miles.to_base(mountain_dist_mi);
    }

    // ---------------------------------------------------------------
    // Pass-through setters
    // ---------------------------------------------------------------

    pub fn set_burning_pile_flame_height(&mut self, height: f64, units: LengthUnits) {
        self.spot_inputs.set_burning_pile_flame_height(height, units);
    }

    pub fn set_dbh(&mut self, dbh: f64, units: LengthUnits) {
        self.spot_inputs.set_dbh(dbh, units);
    }

    pub fn set_downwind_cover_height(&mut self, height: f64, units: LengthUnits) {
        self.spot_inputs.set_downwind_cover_height(height, units);
    }

    pub fn set_downwind_canopy_mode(&mut self, mode: SpotDownWindCanopyMode) {
        self.spot_inputs.set_downwind_canopy_mode(mode);
    }

    pub fn set_flame_length(&mut self, length: f64, units: LengthUnits) {
        self.spot_inputs.set_surface_flame_length(length, units);
    }

    pub fn set_active_crown_flame_length(&mut self, length: f64, units: LengthUnits) {
        self.spot_inputs.set_active_crown_flame_length(length, units);
    }

    pub fn set_fireline_intensity(&mut self, intensity: f64, units: FirelineIntensityUnits) {
        self.spot_inputs.set_crown_fireline_intensity(intensity, units);
    }

    pub fn set_location(&mut self, location: SpotFireLocation) {
        self.spot_inputs.set_location(location);
    }

    pub fn set_ridge_to_valley_distance(&mut self, distance: f64, units: LengthUnits) {
        self.spot_inputs.set_ridge_to_valley_distance(distance, units);
    }

    pub fn set_ridge_to_valley_elevation(&mut self, elevation: f64, units: LengthUnits) {
        self.spot_inputs.set_ridge_to_valley_elevation(elevation, units);
    }

    pub fn set_torching_trees(&mut self, count: i32) {
        self.spot_inputs.set_torching_trees(count);
    }

    pub fn set_tree_height(&mut self, height: f64, units: LengthUnits) {
        self.spot_inputs.set_tree_height(height, units);
    }

    pub fn set_tree_species(&mut self, species: SpotTreeSpecies) {
        self.spot_inputs.set_tree_species(species);
    }

    pub fn set_wind_speed_at_twenty_feet(&mut self, speed: f64, units: SpeedUnits) {
        self.spot_inputs.set_wind_speed_at_twenty_feet(speed, units);
    }

    // Bulk update pass-throughs

    pub fn update_spot_inputs_for_burning_pile(
        &mut self,
        location: SpotFireLocation,
        ridge_to_valley_distance: f64, distance_units: LengthUnits,
        ridge_to_valley_elevation: f64, elevation_units: LengthUnits,
        downwind_cover_height: f64, cover_height_units: LengthUnits,
        downwind_canopy_mode: SpotDownWindCanopyMode,
        burning_pile_flame_height: f64, flame_height_units: LengthUnits,
        wind_speed_at_twenty_feet: f64, wind_speed_units: SpeedUnits,
    ) {
        self.spot_inputs.update_spot_inputs_for_burning_pile(
            location,
            ridge_to_valley_distance, distance_units,
            ridge_to_valley_elevation, elevation_units,
            downwind_cover_height, cover_height_units,
            downwind_canopy_mode,
            burning_pile_flame_height, flame_height_units,
            wind_speed_at_twenty_feet, wind_speed_units,
        );
    }

    pub fn update_spot_inputs_for_surface_fire(
        &mut self,
        location: SpotFireLocation,
        ridge_to_valley_distance: f64, distance_units: LengthUnits,
        ridge_to_valley_elevation: f64, elevation_units: LengthUnits,
        downwind_cover_height: f64, cover_height_units: LengthUnits,
        downwind_canopy_mode: SpotDownWindCanopyMode,
        wind_speed_at_twenty_feet: f64, wind_speed_units: SpeedUnits,
        surface_flame_length: f64, flame_length_units: LengthUnits,
    ) {
        self.spot_inputs.update_spot_inputs_for_surface_fire(
            location,
            ridge_to_valley_distance, distance_units,
            ridge_to_valley_elevation, elevation_units,
            downwind_cover_height, cover_height_units,
            downwind_canopy_mode,
            wind_speed_at_twenty_feet, wind_speed_units,
            surface_flame_length, flame_length_units,
        );
    }

    pub fn update_spot_inputs_for_torching_trees(
        &mut self,
        location: SpotFireLocation,
        ridge_to_valley_distance: f64, distance_units: LengthUnits,
        ridge_to_valley_elevation: f64, elevation_units: LengthUnits,
        downwind_cover_height: f64, cover_height_units: LengthUnits,
        downwind_canopy_mode: SpotDownWindCanopyMode,
        torching_trees: i32,
        dbh: f64, dbh_units: LengthUnits,
        tree_height: f64, tree_height_units: LengthUnits,
        tree_species: SpotTreeSpecies,
        wind_speed_at_twenty_feet: f64, wind_speed_units: SpeedUnits,
    ) {
        self.spot_inputs.update_spot_inputs_for_torching_trees(
            location,
            ridge_to_valley_distance, distance_units,
            ridge_to_valley_elevation, elevation_units,
            downwind_cover_height, cover_height_units,
            downwind_canopy_mode,
            torching_trees,
            dbh, dbh_units,
            tree_height, tree_height_units,
            tree_species,
            wind_speed_at_twenty_feet, wind_speed_units,
        );
    }

    pub fn update_spot_inputs_for_active_crown_fire(
        &mut self,
        location: SpotFireLocation,
        ridge_to_valley_distance: f64, distance_units: LengthUnits,
        ridge_to_valley_elevation: f64, elevation_units: LengthUnits,
        tree_height: f64, tree_height_units: LengthUnits,
        downwind_canopy_mode: SpotDownWindCanopyMode,
        wind_speed_at_twenty_feet: f64, wind_speed_units: SpeedUnits,
        active_crown_flame_length: f64, flame_length_units: LengthUnits,
    ) {
        self.spot_inputs.update_spot_inputs_for_active_crown_fire(
            location,
            ridge_to_valley_distance, distance_units,
            ridge_to_valley_elevation, elevation_units,
            tree_height, tree_height_units,
            downwind_canopy_mode,
            wind_speed_at_twenty_feet, wind_speed_units,
            active_crown_flame_length, flame_length_units,
        );
    }

    // ---------------------------------------------------------------
    // Getters — input pass-throughs
    // ---------------------------------------------------------------

    pub fn get_burning_pile_flame_height(&self, units: LengthUnits) -> f64 {
        self.spot_inputs.burning_pile_flame_height(units)
    }

    pub fn get_dbh(&self, units: LengthUnits) -> f64 {
        self.spot_inputs.dbh(units)
    }

    pub fn get_downwind_cover_height(&self, units: LengthUnits) -> f64 {
        self.calculate_downwind_canopy_cover_height(units)
    }

    pub fn get_downwind_canopy_mode(&self) -> SpotDownWindCanopyMode {
        self.spot_inputs.downwind_canopy_mode()
    }

    pub fn get_surface_flame_length(&self, units: LengthUnits) -> f64 {
        self.spot_inputs.surface_flame_length(units)
    }

    pub fn get_active_crown_flame_length(&self, units: LengthUnits) -> f64 {
        self.spot_inputs.active_crown_flame_length(units)
    }

    pub fn get_crown_fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        self.spot_inputs.crown_fireline_intensity(units)
    }

    pub fn get_location(&self) -> SpotFireLocation {
        self.spot_inputs.location()
    }

    pub fn get_ridge_to_valley_distance(&self, units: LengthUnits) -> f64 {
        self.spot_inputs.ridge_to_valley_distance(units)
    }

    pub fn get_ridge_to_valley_elevation(&self, units: LengthUnits) -> f64 {
        self.spot_inputs.ridge_to_valley_elevation(units)
    }

    pub fn get_torching_trees(&self) -> i32 {
        self.spot_inputs.torching_trees()
    }

    pub fn get_tree_height(&self, units: LengthUnits) -> f64 {
        self.spot_inputs.tree_height(units)
    }

    pub fn get_tree_species(&self) -> SpotTreeSpecies {
        self.spot_inputs.tree_species()
    }

    pub fn get_wind_speed_at_twenty_feet(&self, units: SpeedUnits) -> f64 {
        self.spot_inputs.wind_speed_at_twenty_feet(units)
    }

    // ---------------------------------------------------------------
    // Getters — computed intermediates
    // ---------------------------------------------------------------

    pub fn get_cover_height_used_for_burning_pile(&self, units: LengthUnits) -> f64 {
        units.from_base(self.cover_height_used_for_burning_pile)
    }

    pub fn get_cover_height_used_for_surface_fire(&self, units: LengthUnits) -> f64 {
        units.from_base(self.cover_height_used_for_surface_fire)
    }

    pub fn get_cover_height_used_for_torching_trees(&self, units: LengthUnits) -> f64 {
        units.from_base(self.cover_height_used_for_torching_trees)
    }

    pub fn get_flame_height_for_torching_trees(&self, units: LengthUnits) -> f64 {
        units.from_base(self.flame_height_for_torching_trees)
    }

    pub fn get_flame_ratio_for_torching_trees(&self) -> f64 {
        self.flame_ratio
    }

    pub fn get_flame_duration_for_torching_trees(&self, units: TimeUnits) -> f64 {
        units.from_base(self.flame_duration)
    }

    pub fn get_max_firebrand_height_from_burning_pile(&self, units: LengthUnits) -> f64 {
        units.from_base(self.firebrand_height_from_burning_pile)
    }

    pub fn get_max_firebrand_height_from_surface_fire(&self, units: LengthUnits) -> f64 {
        units.from_base(self.firebrand_height_from_surface_fire)
    }

    pub fn get_max_firebrand_height_from_torching_trees(&self, units: LengthUnits) -> f64 {
        units.from_base(self.firebrand_height_from_torching_trees)
    }

    // ---------------------------------------------------------------
    // Getters — flat terrain distances
    // ---------------------------------------------------------------

    pub fn get_max_flat_terrain_spotting_distance_from_burning_pile(
        &self, units: LengthUnits,
    ) -> f64 {
        units.from_base(self.flat_distance_from_burning_pile)
    }

    pub fn get_max_flat_terrain_spotting_distance_from_surface_fire(
        &self, units: LengthUnits,
    ) -> f64 {
        units.from_base(self.flat_distance_from_surface_fire)
    }

    pub fn get_max_flat_terrain_spotting_distance_from_torching_trees(
        &self, units: LengthUnits,
    ) -> f64 {
        units.from_base(self.flat_distance_from_torching_trees)
    }

    // ---------------------------------------------------------------
    // Getters — mountain terrain distances
    // ---------------------------------------------------------------

    pub fn get_max_mountainous_terrain_spotting_distance_from_burning_pile(
        &self, units: LengthUnits,
    ) -> f64 {
        units.from_base(self.mountain_distance_from_burning_pile)
    }

    pub fn get_max_mountainous_terrain_spotting_distance_from_surface_fire(
        &self, units: LengthUnits,
    ) -> f64 {
        units.from_base(self.mountain_distance_from_surface_fire)
    }

    pub fn get_max_mountainous_terrain_spotting_distance_from_torching_trees(
        &self, units: LengthUnits,
    ) -> f64 {
        units.from_base(self.mountain_distance_from_torching_trees)
    }

    pub fn get_max_mountainous_terrain_spotting_distance_from_active_crown(
        &self, units: LengthUnits,
    ) -> f64 {
        units.from_base(self.mountain_distance_from_active_crown)
    }
}

impl Default for Spot {
    fn default() -> Self {
        Self::new()
    }
}

/// Round to six decimal places (matching C++ test helper).
fn round_to_six(x: f64) -> f64 {
    (x * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Common test setup matching C++ `testSpotModule` parameters.
    fn make_test_spot() -> Spot {
        Spot::new()
    }

    // Test parameters from C++ testSpotModule:
    // location = RIDGE_TOP
    // ridgeToValleyDistance = 1.0 mi
    // ridgeToValleyElevation = 2000.0 ft
    // downwindCoverHeight = 30.0 ft
    // burningPileFlameHeight = 5.0 ft
    // windSpeedAtTwentyFeet = 5.0 mph
    // torchingTrees = 15
    // DBH = 20.0 in
    // treeHeight = 30.0 ft
    // treeSpecies = ENGELMANN_SPRUCE

    #[test]
    fn burning_pile_closed_canopy_mountain() {
        let mut spot = make_test_spot();
        spot.update_spot_inputs_for_burning_pile(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            5.0, LengthUnits::Feet,
            5.0, SpeedUnits::MilesPerHour,
        );
        spot.calculate_spotting_distance_from_burning_pile();

        let mountain = round_to_six(
            spot.get_max_mountainous_terrain_spotting_distance_from_burning_pile(LengthUnits::Miles),
        );
        assert!(
            (mountain - 0.021330).abs() < 1e-6,
            "burning pile mountain closed: expected 0.021330, got {mountain}"
        );
    }

    #[test]
    fn burning_pile_closed_canopy_flat() {
        let mut spot = make_test_spot();
        spot.update_spot_inputs_for_burning_pile(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            5.0, LengthUnits::Feet,
            5.0, SpeedUnits::MilesPerHour,
        );
        spot.calculate_spotting_distance_from_burning_pile();

        let flat = round_to_six(
            spot.get_max_flat_terrain_spotting_distance_from_burning_pile(LengthUnits::Miles),
        );
        assert!(
            (flat - 0.017067).abs() < 1e-6,
            "burning pile flat closed: expected 0.017067, got {flat}"
        );
    }

    #[test]
    fn burning_pile_open_canopy_mountain() {
        let mut spot = make_test_spot();
        spot.update_spot_inputs_for_burning_pile(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Open,
            5.0, LengthUnits::Feet,
            5.0, SpeedUnits::MilesPerHour,
        );
        spot.calculate_spotting_distance_from_burning_pile();

        let mountain = round_to_six(
            spot.get_max_mountainous_terrain_spotting_distance_from_burning_pile(LengthUnits::Miles),
        );
        assert!(
            (mountain - 0.030863).abs() < 1e-6,
            "burning pile mountain open: expected 0.030863, got {mountain}"
        );
    }

    #[test]
    fn burning_pile_open_canopy_flat() {
        let mut spot = make_test_spot();
        spot.update_spot_inputs_for_burning_pile(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Open,
            5.0, LengthUnits::Feet,
            5.0, SpeedUnits::MilesPerHour,
        );
        spot.calculate_spotting_distance_from_burning_pile();

        let flat = round_to_six(
            spot.get_max_flat_terrain_spotting_distance_from_burning_pile(LengthUnits::Miles),
        );
        assert!(
            (flat - 0.024700).abs() < 1e-6,
            "burning pile flat open: expected 0.024700, got {flat}"
        );
    }

    /// Compute the flame length from GS4 low moisture scenario with
    /// WAF=1.0 UserInput, matching the C++ spot test setup.
    fn gs4_flame_length() -> f64 {
        use behave_surface::facade::Surface;
        use behave_surface::fuel_models::FuelModels;
        use firelab_base::{
            FractionUnits, SlopeUnits, WindAndSpreadOrientationMode,
            WindHeightInputMode, WindAdjustmentFactorCalculationMethod,
        };

        let fm = FuelModels::new();
        let mut s = Surface::new(fm);
        s.update_surface_inputs(
            124, // GS4
            0.06, 0.07, 0.08, 0.60, 0.90,
            FractionUnits::Fraction,
            5.0, SpeedUnits::MilesPerHour,
            WindHeightInputMode::TwentyFoot,
            0.0,
            WindAndSpreadOrientationMode::RelativeToUpslope,
            30.0, SlopeUnits::Percent,
            0.0,
            0.50, FractionUnits::Fraction,
            30.0, LengthUnits::Feet,
            0.50, FractionUnits::Fraction,
        );
        // Override WAF to 1.0 UserInput (matching C++ spot test)
        s.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
        s.set_user_provided_wind_adjustment_factor(1.0);
        s.set_wind_adjustment_factor_calculation_method(
            WindAdjustmentFactorCalculationMethod::UserInput,
        );
        s.do_surface_run_in_direction_of_max_spread();
        s.flame_length_output(LengthUnits::Feet)
    }

    #[test]
    fn surface_fire_closed_canopy_mountain() {
        let flame_length = gs4_flame_length();
        let mut spot = make_test_spot();

        spot.update_spot_inputs_for_surface_fire(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            5.0, SpeedUnits::MilesPerHour,
            flame_length, LengthUnits::Feet,
        );
        spot.calculate_spotting_distance_from_surface_fire();

        let mountain = round_to_six(
            spot.get_max_mountainous_terrain_spotting_distance_from_surface_fire(LengthUnits::Miles),
        );
        assert!(
            (mountain - 0.267467).abs() < 1e-6,
            "surface fire mountain closed: expected 0.267467, got {mountain}"
        );
    }

    #[test]
    fn surface_fire_closed_canopy_flat() {
        let flame_length = gs4_flame_length();
        let mut spot = make_test_spot();

        spot.update_spot_inputs_for_surface_fire(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            5.0, SpeedUnits::MilesPerHour,
            flame_length, LengthUnits::Feet,
        );
        spot.calculate_spotting_distance_from_surface_fire();

        let flat = round_to_six(
            spot.get_max_flat_terrain_spotting_distance_from_surface_fire(LengthUnits::Miles),
        );
        assert!(
            (flat - 0.22005).abs() < 1e-5,
            "surface fire flat closed: expected 0.22005, got {flat}"
        );
    }

    #[test]
    fn torching_trees_closed_canopy_mountain() {
        let mut spot = make_test_spot();

        spot.update_spot_inputs_for_torching_trees(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            15,
            20.0, LengthUnits::Inches,
            30.0, LengthUnits::Feet,
            SpotTreeSpecies::EngelmannSpruce,
            5.0, SpeedUnits::MilesPerHour,
        );
        spot.calculate_spotting_distance_from_torching_trees();

        let mountain = round_to_six(
            spot.get_max_mountainous_terrain_spotting_distance_from_torching_trees(LengthUnits::Miles),
        );
        assert!(
            (mountain - 0.222396).abs() < 1e-6,
            "torching trees mountain closed: expected 0.222396, got {mountain}"
        );
    }

    #[test]
    fn torching_trees_closed_canopy_flat() {
        let mut spot = make_test_spot();

        spot.update_spot_inputs_for_torching_trees(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            15,
            20.0, LengthUnits::Inches,
            30.0, LengthUnits::Feet,
            SpotTreeSpecies::EngelmannSpruce,
            5.0, SpeedUnits::MilesPerHour,
        );
        spot.calculate_spotting_distance_from_torching_trees();

        let flat = round_to_six(
            spot.get_max_flat_terrain_spotting_distance_from_torching_trees(LengthUnits::Miles),
        );
        assert!(
            (flat - 0.181449).abs() < 1e-6,
            "torching trees flat closed: expected 0.181449, got {flat}"
        );
    }

    #[test]
    fn active_crown_mountain() {
        let mut spot = make_test_spot();

        spot.update_spot_inputs_for_active_crown_fire(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            5.0, SpeedUnits::MilesPerHour,
            20.0, LengthUnits::Feet,
        );
        spot.calculate_spotting_distance_from_active_crown();

        let mountain = round_to_six(
            spot.get_max_mountainous_terrain_spotting_distance_from_active_crown(LengthUnits::Miles),
        );
        assert!(
            (mountain - 0.400473).abs() < 1e-6,
            "active crown mountain: expected 0.400473, got {mountain}"
        );
    }
}
