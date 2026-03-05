//! Spot fire input parameters.
//!
//! C++ source: spotInputs.h / spotInputs.cpp

use firelab_base::{
    FirelineIntensityUnits, LengthUnits, SpeedUnits, UnitConversion,
};

/// Tree species affecting torching firebrand loft.
///
/// C++ source: `SpotTreeSpecies` in spotInputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotTreeSpecies {
    EngelmannSpruce = 0,
    DouglasFir = 1,
    SubalpineFir = 2,
    WesternHemlock = 3,
    PonderosaPine = 4,
    LodgepolePine = 5,
    WesternWhitePine = 6,
    GrandFir = 7,
    BalsamFir = 8,
    SlashPine = 9,
    LongleafPine = 10,
    PondPine = 11,
    ShortleafPine = 12,
    LoblollyPine = 13,
}

/// Location of the spot fire ignition relative to terrain.
///
/// C++ source: `SpotFireLocation` in spotInputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotFireLocation {
    MidslopeWindward = 0,
    ValleyBottom = 1,
    MidslopeLeeward = 2,
    RidgeTop = 3,
}

/// Downwind canopy condition for spot distance calculation.
///
/// C++ source: `SpotDownWindCanopyMode` in spotInputs.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotDownWindCanopyMode {
    Closed = 0,
    Open = 1,
}

/// All user-facing inputs for a spot fire calculation.
///
/// All values stored in base units (ft for lengths, ft/min for speed,
/// Btu/ft/s for fireline intensity).
///
/// C++ class: `SpotInputs`
#[derive(Debug, Clone)]
pub struct SpotInputs {
    burning_pile_flame_height: f64,    // ft
    surface_flame_length: f64,         // ft
    active_crown_flame_length: f64,    // ft
    crown_fireline_intensity: f64,     // Btu/ft/s
    dbh: f64,                          // ft (stored base)
    downwind_cover_height: f64,        // ft
    downwind_canopy_mode: SpotDownWindCanopyMode,
    location: SpotFireLocation,
    ridge_to_valley_distance: f64,     // ft
    ridge_to_valley_elevation: f64,    // ft
    wind_speed_at_twenty_feet: f64,    // ft/min
    torching_trees: i32,
    tree_height: f64,                  // ft
    tree_species: SpotTreeSpecies,
}

impl SpotInputs {
    pub fn new() -> Self {
        Self {
            burning_pile_flame_height: 0.0,
            surface_flame_length: 0.0,
            active_crown_flame_length: 0.0,
            crown_fireline_intensity: 0.0,
            dbh: 0.0,
            downwind_cover_height: 0.0,
            downwind_canopy_mode: SpotDownWindCanopyMode::Closed,
            location: SpotFireLocation::MidslopeWindward,
            ridge_to_valley_distance: 0.0,
            ridge_to_valley_elevation: 0.0,
            wind_speed_at_twenty_feet: 0.0,
            torching_trees: 0,
            tree_height: 0.0,
            tree_species: SpotTreeSpecies::EngelmannSpruce,
        }
    }

    pub fn initialize_members(&mut self) {
        self.burning_pile_flame_height = 0.0;
        self.surface_flame_length = 0.0;
        self.active_crown_flame_length = 0.0;
        self.crown_fireline_intensity = 0.0;
        self.dbh = 0.0;
        self.downwind_cover_height = 0.0;
        self.downwind_canopy_mode = SpotDownWindCanopyMode::Closed;
        self.location = SpotFireLocation::MidslopeWindward;
        self.ridge_to_valley_distance = 0.0;
        self.ridge_to_valley_elevation = 0.0;
        self.wind_speed_at_twenty_feet = 0.0;
        self.torching_trees = 0;
        self.tree_height = 0.0;
    }

    // --- Setters ---

    pub fn set_burning_pile_flame_height(&mut self, height: f64, units: LengthUnits) {
        self.burning_pile_flame_height = units.to_base(height);
    }

    pub fn set_dbh(&mut self, dbh: f64, units: LengthUnits) {
        self.dbh = units.to_base(dbh);
    }

    pub fn set_downwind_cover_height(&mut self, height: f64, units: LengthUnits) {
        self.downwind_cover_height = units.to_base(height);
    }

    pub fn set_downwind_canopy_mode(&mut self, mode: SpotDownWindCanopyMode) {
        self.downwind_canopy_mode = mode;
    }

    pub fn set_surface_flame_length(&mut self, length: f64, units: LengthUnits) {
        self.surface_flame_length = units.to_base(length);
    }

    pub fn set_active_crown_flame_length(&mut self, length: f64, units: LengthUnits) {
        self.active_crown_flame_length = units.to_base(length);
    }

    pub fn set_crown_fireline_intensity(&mut self, intensity: f64, units: FirelineIntensityUnits) {
        self.crown_fireline_intensity = units.to_base(intensity);
    }

    pub fn set_location(&mut self, location: SpotFireLocation) {
        self.location = location;
    }

    pub fn set_ridge_to_valley_distance(&mut self, distance: f64, units: LengthUnits) {
        self.ridge_to_valley_distance = units.to_base(distance);
    }

    pub fn set_ridge_to_valley_elevation(&mut self, elevation: f64, units: LengthUnits) {
        self.ridge_to_valley_elevation = units.to_base(elevation);
    }

    pub fn set_torching_trees(&mut self, count: i32) {
        self.torching_trees = count;
    }

    pub fn set_tree_height(&mut self, height: f64, units: LengthUnits) {
        self.tree_height = units.to_base(height);
    }

    pub fn set_tree_species(&mut self, species: SpotTreeSpecies) {
        self.tree_species = species;
    }

    pub fn set_wind_speed_at_twenty_feet(&mut self, speed: f64, units: SpeedUnits) {
        self.wind_speed_at_twenty_feet = units.to_base(speed);
    }

    // --- Bulk update setters ---

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
        self.initialize_members();
        self.set_location(location);
        self.set_ridge_to_valley_distance(ridge_to_valley_distance, distance_units);
        self.set_ridge_to_valley_elevation(ridge_to_valley_elevation, elevation_units);
        self.set_downwind_cover_height(downwind_cover_height, cover_height_units);
        self.set_downwind_canopy_mode(downwind_canopy_mode);
        self.set_wind_speed_at_twenty_feet(wind_speed_at_twenty_feet, wind_speed_units);
        self.set_burning_pile_flame_height(burning_pile_flame_height, flame_height_units);
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
        self.initialize_members();
        self.set_location(location);
        self.set_ridge_to_valley_distance(ridge_to_valley_distance, distance_units);
        self.set_ridge_to_valley_elevation(ridge_to_valley_elevation, elevation_units);
        self.set_downwind_cover_height(downwind_cover_height, cover_height_units);
        self.set_downwind_canopy_mode(downwind_canopy_mode);
        self.set_wind_speed_at_twenty_feet(wind_speed_at_twenty_feet, wind_speed_units);
        self.set_surface_flame_length(surface_flame_length, flame_length_units);
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
        self.initialize_members();
        self.set_location(location);
        self.set_ridge_to_valley_distance(ridge_to_valley_distance, distance_units);
        self.set_ridge_to_valley_elevation(ridge_to_valley_elevation, elevation_units);
        self.set_downwind_cover_height(downwind_cover_height, cover_height_units);
        self.set_downwind_canopy_mode(downwind_canopy_mode);
        self.set_torching_trees(torching_trees);
        self.set_dbh(dbh, dbh_units);
        self.set_tree_height(tree_height, tree_height_units);
        self.set_tree_species(tree_species);
        self.set_wind_speed_at_twenty_feet(wind_speed_at_twenty_feet, wind_speed_units);
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
        self.initialize_members();
        self.set_location(location);
        self.set_active_crown_flame_length(active_crown_flame_length, flame_length_units);
        self.set_ridge_to_valley_distance(ridge_to_valley_distance, distance_units);
        self.set_ridge_to_valley_elevation(ridge_to_valley_elevation, elevation_units);
        self.set_tree_height(tree_height, tree_height_units);
        self.set_downwind_canopy_mode(downwind_canopy_mode);
        self.set_wind_speed_at_twenty_feet(wind_speed_at_twenty_feet, wind_speed_units);
    }

    // --- Getters ---

    pub fn burning_pile_flame_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.burning_pile_flame_height)
    }

    pub fn dbh(&self, units: LengthUnits) -> f64 {
        units.from_base(self.dbh)
    }

    pub fn downwind_cover_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.downwind_cover_height)
    }

    pub fn downwind_canopy_mode(&self) -> SpotDownWindCanopyMode {
        self.downwind_canopy_mode
    }

    pub fn surface_flame_length(&self, units: LengthUnits) -> f64 {
        units.from_base(self.surface_flame_length)
    }

    pub fn active_crown_flame_length(&self, units: LengthUnits) -> f64 {
        units.from_base(self.active_crown_flame_length)
    }

    pub fn crown_fireline_intensity(&self, units: FirelineIntensityUnits) -> f64 {
        units.from_base(self.crown_fireline_intensity)
    }

    pub fn location(&self) -> SpotFireLocation {
        self.location
    }

    pub fn ridge_to_valley_distance(&self, units: LengthUnits) -> f64 {
        units.from_base(self.ridge_to_valley_distance)
    }

    pub fn ridge_to_valley_elevation(&self, units: LengthUnits) -> f64 {
        units.from_base(self.ridge_to_valley_elevation)
    }

    pub fn torching_trees(&self) -> i32 {
        self.torching_trees
    }

    pub fn tree_height(&self, units: LengthUnits) -> f64 {
        units.from_base(self.tree_height)
    }

    pub fn tree_species(&self) -> SpotTreeSpecies {
        self.tree_species
    }

    pub fn wind_speed_at_twenty_feet(&self, units: SpeedUnits) -> f64 {
        units.from_base(self.wind_speed_at_twenty_feet)
    }
}

impl Default for SpotInputs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spot_inputs_default() {
        let si = SpotInputs::new();
        assert_eq!(si.burning_pile_flame_height(LengthUnits::Feet), 0.0);
        assert_eq!(si.wind_speed_at_twenty_feet(SpeedUnits::MilesPerHour), 0.0);
        assert_eq!(si.torching_trees(), 0);
        assert_eq!(si.location(), SpotFireLocation::MidslopeWindward);
        assert_eq!(si.downwind_canopy_mode(), SpotDownWindCanopyMode::Closed);
    }

    #[test]
    fn spot_inputs_set_and_get() {
        let mut si = SpotInputs::new();
        si.set_burning_pile_flame_height(5.0, LengthUnits::Feet);
        si.set_dbh(20.0, LengthUnits::Inches);
        si.set_tree_height(30.0, LengthUnits::Feet);
        si.set_wind_speed_at_twenty_feet(5.0, SpeedUnits::MilesPerHour);
        si.set_location(SpotFireLocation::RidgeTop);
        si.set_downwind_canopy_mode(SpotDownWindCanopyMode::Open);
        si.set_torching_trees(15);
        si.set_tree_species(SpotTreeSpecies::EngelmannSpruce);

        assert!((si.burning_pile_flame_height(LengthUnits::Feet) - 5.0).abs() < 1e-10);
        assert!((si.dbh(LengthUnits::Inches) - 20.0).abs() < 1e-10);
        assert!((si.tree_height(LengthUnits::Feet) - 30.0).abs() < 1e-10);
        assert!((si.wind_speed_at_twenty_feet(SpeedUnits::MilesPerHour) - 5.0).abs() < 0.01);
        assert_eq!(si.location(), SpotFireLocation::RidgeTop);
        assert_eq!(si.downwind_canopy_mode(), SpotDownWindCanopyMode::Open);
        assert_eq!(si.torching_trees(), 15);
        assert_eq!(si.tree_species(), SpotTreeSpecies::EngelmannSpruce);
    }

    #[test]
    fn spot_inputs_bulk_update_burning_pile() {
        let mut si = SpotInputs::new();
        si.update_spot_inputs_for_burning_pile(
            SpotFireLocation::RidgeTop,
            1.0, LengthUnits::Miles,
            2000.0, LengthUnits::Feet,
            30.0, LengthUnits::Feet,
            SpotDownWindCanopyMode::Closed,
            5.0, LengthUnits::Feet,
            5.0, SpeedUnits::MilesPerHour,
        );
        assert_eq!(si.location(), SpotFireLocation::RidgeTop);
        assert!((si.burning_pile_flame_height(LengthUnits::Feet) - 5.0).abs() < 1e-10);
        assert!((si.ridge_to_valley_distance(LengthUnits::Miles) - 1.0).abs() < 1e-10);
        assert!((si.ridge_to_valley_elevation(LengthUnits::Feet) - 2000.0).abs() < 1e-10);
        assert!((si.downwind_cover_height(LengthUnits::Feet) - 30.0).abs() < 1e-10);
    }
}
