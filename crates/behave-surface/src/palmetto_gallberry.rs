//! Palmetto-Gallberry fuel model calculations.
//!
//! C++ source: palmettoGallberry.h / palmettoGallberry.cpp
//!
//! Southeastern US understory fuel model with regression equations
//! for fuel loads based on age, palmetto coverage, overstory basal area,
//! and understory height.

/// Palmetto-Gallberry fuel load calculator.
#[derive(Debug, Clone)]
pub struct PalmettoGallberry {
    moisture_of_extinction_dead: f64,
    heat_of_combustion_dead: f64,
    heat_of_combustion_live: f64,
    dead_fine_fuel_load: f64,
    dead_medium_fuel_load: f64,
    dead_foliage_fuel_load: f64,
    fuel_bed_depth: f64,
    litter_load: f64,
    live_fine_fuel_load: f64,
    live_medium_fuel_load: f64,
    live_foliage_load: f64,
}

impl PalmettoGallberry {
    pub fn new() -> Self {
        Self {
            moisture_of_extinction_dead: 0.40,
            heat_of_combustion_dead: 8300.0,
            heat_of_combustion_live: 8300.0,
            dead_fine_fuel_load: 0.0,
            dead_medium_fuel_load: 0.0,
            dead_foliage_fuel_load: 0.0,
            fuel_bed_depth: 0.0,
            litter_load: 0.0,
            live_fine_fuel_load: 0.0,
            live_medium_fuel_load: 0.0,
            live_foliage_load: 0.0,
        }
    }

    pub fn calculate_dead_fine_fuel_load(&mut self, age: f64, height: f64) -> f64 {
        self.dead_fine_fuel_load = (-0.00121 + 0.00379 * age.ln() + 0.00118 * height * height)
            .max(0.0);
        self.dead_fine_fuel_load
    }

    pub fn calculate_dead_medium_fuel_load(&mut self, age: f64, palmetto_cover: f64) -> f64 {
        let cover_pct = palmetto_cover * 100.0;
        self.dead_medium_fuel_load =
            (-0.00775 + 0.00021 * cover_pct + 0.00007 * age * age).max(0.0);
        self.dead_medium_fuel_load
    }

    pub fn calculate_dead_foliage_fuel_load(&mut self, age: f64, palmetto_cover: f64) -> f64 {
        let cover_pct = palmetto_cover * 100.0;
        self.dead_foliage_fuel_load = 0.00221 * age.powf(0.51263) * (0.02482 * cover_pct).exp();
        self.dead_foliage_fuel_load
    }

    pub fn calculate_litter_load(&mut self, age: f64, overstory_basal_area: f64) -> f64 {
        self.litter_load =
            (0.03632 + 0.0005336 * overstory_basal_area) * (1.0 - 0.25_f64.powf(age));
        self.litter_load
    }

    pub fn calculate_live_fine_fuel_load(&mut self, age: f64, height: f64) -> f64 {
        self.live_fine_fuel_load = 0.00546 + 0.00092 * age + 0.00212 * height * height;
        self.live_fine_fuel_load
    }

    pub fn calculate_live_medium_fuel_load(&mut self, age: f64, height: f64) -> f64 {
        self.live_medium_fuel_load =
            (-0.02128 + 0.00014 * age * age + 0.00314 * height * height).max(0.0);
        self.live_medium_fuel_load
    }

    pub fn calculate_live_foliage_load(
        &mut self,
        age: f64,
        palmetto_cover: f64,
        height: f64,
    ) -> f64 {
        let cover_pct = palmetto_cover * 100.0;
        self.live_foliage_load =
            (-0.0036 + 0.00253 * age + 0.00049 * cover_pct + 0.00282 * height * height).max(0.0);
        self.live_foliage_load
    }

    pub fn calculate_fuel_bed_depth(&mut self, height: f64) -> f64 {
        self.fuel_bed_depth = 2.0 * height / 3.0;
        self.fuel_bed_depth
    }

    // --- Getters ---

    pub fn get_moisture_of_extinction_dead(&self) -> f64 { self.moisture_of_extinction_dead }
    pub fn get_heat_of_combustion_dead(&self) -> f64 { self.heat_of_combustion_dead }
    pub fn get_heat_of_combustion_live(&self) -> f64 { self.heat_of_combustion_live }
    pub fn get_dead_fine_fuel_load(&self) -> f64 { self.dead_fine_fuel_load }
    pub fn get_dead_medium_fuel_load(&self) -> f64 { self.dead_medium_fuel_load }
    pub fn get_dead_foliage_fuel_load(&self) -> f64 { self.dead_foliage_fuel_load }
    pub fn get_fuel_bed_depth(&self) -> f64 { self.fuel_bed_depth }
    pub fn get_litter_load(&self) -> f64 { self.litter_load }
    pub fn get_live_fine_fuel_load(&self) -> f64 { self.live_fine_fuel_load }
    pub fn get_live_medium_fuel_load(&self) -> f64 { self.live_medium_fuel_load }
    pub fn get_live_foliage_load(&self) -> f64 { self.live_foliage_load }
}

impl Default for PalmettoGallberry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let pg = PalmettoGallberry::new();
        assert!((pg.get_moisture_of_extinction_dead() - 0.40).abs() < 1e-6);
        assert!((pg.get_heat_of_combustion_dead() - 8300.0).abs() < 1e-6);
        assert!((pg.get_heat_of_combustion_live() - 8300.0).abs() < 1e-6);
    }

    #[test]
    fn dead_fine_fuel_load() {
        let mut pg = PalmettoGallberry::new();
        let load = pg.calculate_dead_fine_fuel_load(10.0, 3.0);
        assert!(load > 0.0, "dead fine fuel load should be positive, got {load}");
    }

    #[test]
    fn fuel_bed_depth() {
        let mut pg = PalmettoGallberry::new();
        let depth = pg.calculate_fuel_bed_depth(6.0);
        assert!(
            (depth - 4.0).abs() < 1e-6,
            "expected 2/3 * 6 = 4, got {depth}"
        );
    }

    #[test]
    fn litter_load_increases_with_age() {
        let mut pg = PalmettoGallberry::new();
        let load5 = pg.calculate_litter_load(5.0, 50.0);
        let load20 = pg.calculate_litter_load(20.0, 50.0);
        assert!(
            load20 > load5,
            "litter should increase with age: {load5} vs {load20}"
        );
    }

    #[test]
    fn negative_loads_clamped() {
        let mut pg = PalmettoGallberry::new();
        // Very small age and height should not produce negative loads
        let load = pg.calculate_dead_fine_fuel_load(0.5, 0.1);
        assert!(load >= 0.0, "load should be >= 0, got {load}");
    }
}
