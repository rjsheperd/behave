//! Public-facing containment adapter (top-level API).
//!
//! C++ source: ContainAdapter.h / ContainAdapter.cpp
//!
//! Provides a high-level interface for configuring and running containment
//! simulations with unit conversions.

use firelab_base::{AreaUnits, FireSize, LengthUnits, SpeedUnits, TimeUnits, UnitConversion};

use crate::algorithm::{ContainStatus, ContainTactic};
use crate::force::ContainForce;
use crate::force_adapter::ContainForceAdapter;
use crate::resource::{ContainFlank, ContainResource};
use crate::sim::ContainSim;

/// High-level containment interface wrapping the Sem simulation.
#[derive(Debug)]
pub struct ContainAdapter {
    // Contain inputs
    report_size: f64,  // acres
    report_rate: f64,  // ch/h
    diurnal_ros: [f64; 24],
    fire_start_time: i32,
    lw_ratio: f64,
    force: ContainForceAdapter,
    tactic: ContainTactic,
    attack_distance: f64, // chains
    retry: bool,
    min_steps: i32,
    max_steps: i32,
    max_fire_size: i32,
    max_fire_time: i32,

    // Contain outputs (stored in base units: ft, ft², min)
    final_cost: f64,
    final_fire_line_length: f64,  // ft (base)
    perimeter_at_initial_attack: f64, // ft (base)
    perimeter_at_containment: f64,    // ft (base)
    fire_size_at_initial_attack: f64, // ft² (base)
    final_fire_size: f64,             // ft² (base)
    final_containment_area: f64,      // ft² (base)
    final_time: f64,                  // min (base)
    final_production_rate: f64,       // ft/min (base)
    containment_status: ContainStatus,
}

impl ContainAdapter {
    pub fn new() -> Self {
        Self {
            report_size: 0.0,
            report_rate: 0.0,
            diurnal_ros: [0.0; 24],
            fire_start_time: 0,
            lw_ratio: 1.0,
            force: ContainForceAdapter::new(),
            tactic: ContainTactic::HeadAttack,
            attack_distance: 0.0,
            retry: true,
            min_steps: 250,
            max_steps: 1000,
            max_fire_size: 1000,
            max_fire_time: 1080,
            final_cost: 0.0,
            final_fire_line_length: 0.0,
            perimeter_at_initial_attack: 0.0,
            perimeter_at_containment: 0.0,
            fire_size_at_initial_attack: 0.0,
            final_fire_size: 0.0,
            final_containment_area: 0.0,
            final_time: 0.0,
            final_production_rate: 0.0,
            containment_status: ContainStatus::Unreported,
        }
    }

    pub fn add_resource(
        &mut self,
        arrival: f64,
        duration: f64,
        time_units: TimeUnits,
        production_rate: f64,
        production_rate_units: SpeedUnits,
        description: &str,
        base_cost: f64,
        hour_cost: f64,
    ) {
        // Convert production rate to ch/h
        let production_rate_ch_h = if production_rate_units == SpeedUnits::ChainsPerHour {
            production_rate
        } else {
            let ft_per_min = production_rate_units.to_base(production_rate);
            SpeedUnits::ChainsPerHour.from_base(ft_per_min)
        };

        // Convert times to minutes (base)
        let duration_min = time_units.to_base(duration);
        let arrival_min = time_units.to_base(arrival);

        let resource = ContainResource::new(
            arrival_min,
            production_rate_ch_h,
            duration_min,
            ContainFlank::LeftFlank, // Only left flank used, mirrored
            description,
            base_cost,
            hour_cost,
        );
        self.force.add_resource(resource);
    }

    pub fn remove_all_resources(&mut self) {
        self.force.remove_all_resources();
    }

    pub fn set_report_size(&mut self, report_size: f64, area_units: AreaUnits) {
        let sq_ft = area_units.to_base(report_size);
        self.report_size = AreaUnits::Acres.from_base(sq_ft); // Contain expects acres
    }

    pub fn set_report_rate(&mut self, report_rate: f64, speed_units: SpeedUnits) {
        let ft_per_min = speed_units.to_base(report_rate);
        self.report_rate = SpeedUnits::ChainsPerHour.from_base(ft_per_min);
    }

    pub fn set_fire_start_time(&mut self, fire_start_time: i32) {
        self.fire_start_time = fire_start_time;
    }

    pub fn set_lw_ratio(&mut self, lw_ratio: f64) {
        self.lw_ratio = lw_ratio;
    }

    pub fn set_tactic(&mut self, tactic: ContainTactic) {
        self.tactic = tactic;
    }

    pub fn set_attack_distance(&mut self, attack_distance: f64, length_units: LengthUnits) {
        let ft = length_units.to_base(attack_distance);
        self.attack_distance = LengthUnits::Chains.from_base(ft); // Contain expects chains
    }

    pub fn set_retry(&mut self, retry: bool) {
        self.retry = retry;
    }

    pub fn set_min_steps(&mut self, min_steps: i32) {
        self.min_steps = min_steps;
    }

    pub fn set_max_steps(&mut self, max_steps: i32) {
        self.max_steps = max_steps;
    }

    pub fn set_max_fire_size(&mut self, max_fire_size: i32) {
        self.max_fire_size = max_fire_size;
    }

    pub fn set_max_fire_time(&mut self, max_fire_time: i32) {
        self.max_fire_time = max_fire_time;
    }

    pub fn do_contain_run(&mut self) {
        let mut report_rate = self.report_rate;
        if report_rate < 0.00001 {
            report_rate = 0.00001;
        }

        if self.force.resources().is_empty() || self.report_size == 0.0 {
            return;
        }

        // Fill diurnal ROS with report rate
        for i in 0..24 {
            self.diurnal_ros[i] = report_rate;
        }

        // Build the Sem-style ContainForce from adapter resources
        let mut old_force = ContainForce::new();
        for r in self.force.resources() {
            old_force.add_resource(r.clone());
        }

        let mut sim = ContainSim::new(
            self.report_size,
            report_rate,
            &self.diurnal_ros,
            self.fire_start_time,
            self.lw_ratio,
            old_force,
            self.tactic,
            self.attack_distance,
            self.retry,
            self.min_steps,
            self.max_steps,
            self.max_fire_size,
            self.max_fire_time,
        );

        // Run simulation
        sim.run();

        // Store results — convert from Contain units to base units
        self.final_cost = sim.final_fire_cost();
        self.final_fire_line_length =
            LengthUnits::Chains.to_base(sim.final_fire_line()); // ch → ft
        self.perimeter_at_containment =
            LengthUnits::Chains.to_base(sim.final_fire_perimeter()); // ch → ft
        self.final_fire_size =
            AreaUnits::Acres.to_base(sim.final_fire_size()); // ac → ft²
        self.final_containment_area =
            AreaUnits::Acres.to_base(sim.final_fire_sweep()); // ac → ft²
        self.final_time =
            TimeUnits::Minutes.to_base(sim.final_fire_time()); // min → min (base is min)
        self.containment_status = sim.status();
        if self.final_time > 0.0 {
            self.final_production_rate = self.final_fire_line_length / self.final_time;
        }

        // Calculate perimeter and size at initial attack using fire geometry
        self.calculate_initial_attack_geometry(report_rate);
    }

    fn calculate_initial_attack_geometry(&mut self, report_rate: f64) {
        // Effective windspeed back-computed from the L/W ratio (lw ≈ 1 + U/4),
        // fed through the FireSize module to build the fire ellipse; the
        // elapsed time at which that ellipse reaches the reported size, plus
        // the first resource arrival, gives the moment of initial attack.
        let effective_windspeed = 4.0 * (self.lw_ratio - 1.0); // mph
        let mut size = FireSize::new();
        size.calculate_fire_basic_dimensions(
            false,
            effective_windspeed,
            SpeedUnits::MilesPerHour,
            report_rate,
            SpeedUnits::ChainsPerHour,
        );

        // Base elliptical dimensions (per minute of growth, in ft)
        let elliptical_a = size.elliptical_a(LengthUnits::Feet, 1.0, TimeUnits::Minutes);
        let elliptical_b = size.elliptical_b(LengthUnits::Feet, 1.0, TimeUnits::Minutes);

        let report_size_sq_ft = AreaUnits::Acres.to_base(self.report_size);
        self.perimeter_at_initial_attack = 0.0;
        self.fire_size_at_initial_attack = 0.0;
        let denominator = std::f64::consts::PI * elliptical_a * elliptical_b;

        let first_arrival_time = self.force.first_arrival(ContainFlank::LeftFlank).max(0.0);

        if denominator > 1.0e-07 {
            // s = sqrt(A / (pi*a*b)), assuming constant rate of growth
            let initial_elapsed = (report_size_sq_ft / denominator).sqrt();
            let total_elapsed = initial_elapsed + first_arrival_time;
            self.perimeter_at_initial_attack =
                size.fire_perimeter(false, LengthUnits::Feet, total_elapsed, TimeUnits::Minutes);
            self.fire_size_at_initial_attack =
                size.fire_area(false, AreaUnits::SquareFeet, total_elapsed, TimeUnits::Minutes);
        }
    }

    // --- Getters ---

    pub fn get_final_cost(&self) -> f64 {
        self.final_cost
    }

    pub fn get_final_fire_line_length(&self, length_units: LengthUnits) -> f64 {
        length_units.from_base(self.final_fire_line_length)
    }

    pub fn get_perimeter_at_initial_attack(&self, length_units: LengthUnits) -> f64 {
        length_units.from_base(self.perimeter_at_initial_attack)
    }

    pub fn get_perimeter_at_containment(&self, length_units: LengthUnits) -> f64 {
        length_units.from_base(self.perimeter_at_containment)
    }

    pub fn get_fire_size_at_initial_attack(&self, area_units: AreaUnits) -> f64 {
        area_units.from_base(self.fire_size_at_initial_attack)
    }

    pub fn get_final_fire_size(&self, area_units: AreaUnits) -> f64 {
        area_units.from_base(self.final_fire_size)
    }

    pub fn get_final_containment_area(&self, area_units: AreaUnits) -> f64 {
        area_units.from_base(self.final_containment_area)
    }

    pub fn get_final_time_since_report(&self, time_units: TimeUnits) -> f64 {
        time_units.from_base(self.final_time)
    }

    pub fn get_final_production_rate(&self, speed_units: SpeedUnits) -> f64 {
        speed_units.from_base(self.final_production_rate)
    }

    pub fn get_containment_status(&self) -> ContainStatus {
        self.containment_status
    }
}

impl Default for ContainAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_adapter() -> ContainAdapter {
        let mut adapter = ContainAdapter::new();
        adapter.set_attack_distance(0.0, LengthUnits::Chains);
        adapter.set_lw_ratio(3.0);
        adapter.set_report_rate(5.0, SpeedUnits::ChainsPerHour);
        adapter.set_report_size(1.0, AreaUnits::Acres);
        adapter.set_tactic(ContainTactic::HeadAttack);
        adapter.add_resource(
            2.0, 8.0, TimeUnits::Hours,
            20.0, SpeedUnits::ChainsPerHour,
            "test", 0.0, 0.0,
        );
        adapter.do_contain_run();
        adapter
    }

    #[test]
    fn containment_status() {
        let adapter = make_test_adapter();
        assert_eq!(adapter.get_containment_status(), ContainStatus::Contained);
    }

    #[test]
    fn final_fire_line_length() {
        let adapter = make_test_adapter();
        let line = adapter.get_final_fire_line_length(LengthUnits::Chains);
        assert!(
            (line - 39.539849615).abs() < 1e-5,
            "final fire line: expected 39.539849615, got {line}"
        );
    }

    #[test]
    fn final_time_since_report() {
        let adapter = make_test_adapter();
        let time = adapter.get_final_time_since_report(TimeUnits::Minutes);
        assert!(
            (time - 238.75).abs() < 1e-6,
            "final time: expected 238.75, got {time}"
        );
    }

    #[test]
    fn final_fire_size() {
        let adapter = make_test_adapter();
        let size = adapter.get_final_fire_size(AreaUnits::Acres);
        assert!(
            (size - 9.42749714).abs() < 1e-5,
            "final fire size: expected 9.42749714, got {size}"
        );
    }

    #[test]
    fn final_containment_area() {
        let adapter = make_test_adapter();
        let area = adapter.get_final_containment_area(AreaUnits::Acres);
        assert!(
            (area - 9.42749714).abs() < 1e-5,
            "final containment area: expected 9.42749714, got {area}"
        );
    }

    #[test]
    fn perimeter_at_containment() {
        let adapter = make_test_adapter();
        let perim = adapter.get_perimeter_at_containment(LengthUnits::Chains);
        assert!(
            (perim - 39.539849615).abs() < 1e-5,
            "perimeter at containment: expected 39.539849615, got {perim}"
        );
    }
}
