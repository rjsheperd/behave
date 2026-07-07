//! Golden-value parity tests against the C++ reference suite.
//!
//! Every expected value below is copied verbatim from
//! `src/testBehave/testBehave.cpp`. The C++ suite runs all test functions in
//! sequence against ONE shared `BehaveRun`, and some tests depend on state
//! left behind by earlier ones — so this file replicates the exact same
//! call sequence, in the same order, against one shared `BehaveRun`.
//!
//! Checks are collected (not asserted immediately) so a single divergence
//! reports alongside the rest, mirroring the C++ TestInfo pass/fail counter.

use behave_run::BehaveRun;

use behave_contain::algorithm::{ContainStatus, ContainTactic};
use behave_ignite::inputs::{IgnitionFuelBedType, LightningCharge};
use behave_spot::inputs::{SpotDownWindCanopyMode, SpotFireLocation, SpotTreeSpecies};
use behave_surface::chaparral::{ChaparralFuelLoadInputMode, ChaparralFuelType};
use behave_surface::fuel_models::FuelModels;
use behave_surface::inputs::SurfaceFireSpreadDirectionMode;
use behave_surface::moisture::MoistureScenarios;
use behave_surface::two_fuel_models::TwoFuelModelsMethod;
use behave_surface::western_aspen::AspenFireSeverity;
use behave_mortality::species::SpeciesMasterTable;
use firelab_base::*;

const TOL: f64 = 1e-6; // matches C++ error_tolerance

/// Mirror of C++ roundToSixDecimalPlaces (format to 6 places, re-parse).
fn round6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap()
}

#[derive(Default)]
struct TestInfo {
    passed: usize,
    failures: Vec<String>,
}

impl TestInfo {
    fn check(&mut self, name: &str, observed: f64, expected: f64, tol: f64) {
        if (observed - expected).abs() < tol {
            self.passed += 1;
        } else {
            self.failures.push(format!(
                "{name}: observed {observed} differs from expected {expected} by more than {tol}"
            ));
        }
    }

    fn check_bool(&mut self, name: &str, observed: bool, expected: bool) {
        self.check(name, observed as u8 as f64, expected as u8 as f64, TOL);
    }
}

// ---------------------------------------------------------------------------
// Scenario helpers (mirrors of the C++ set*Scenario functions)
// ---------------------------------------------------------------------------

/// C++ setSurfaceInputsForGS4LowMoistureScenario
fn set_surface_inputs_for_gs4_low_moisture(run: &mut BehaveRun) {
    run.surface.update_surface_inputs(
        124, // GS4
        6.0, 7.0, 8.0, 60.0, 90.0, FractionUnits::Percent,
        5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot,
        0.0, WindAndSpreadOrientationMode::RelativeToNorth,
        30.0, SlopeUnits::Percent, 0.0,
        50.0, FractionUnits::Percent,
        30.0, LengthUnits::Feet,
        0.50, FractionUnits::Fraction,
    );
}

/// C++ setSurfaceInputsForTwoFuelModelsLowMoistureScenario
fn set_surface_inputs_for_two_fuel_models_low_moisture(run: &mut BehaveRun) {
    run.surface.inputs_mut().update_surface_inputs_for_two_fuel_models(
        1, 124,
        6.0, 7.0, 8.0, 60.0, 90.0, FractionUnits::Percent,
        5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot,
        0.0, WindAndSpreadOrientationMode::RelativeToNorth,
        0.0, FractionUnits::Percent, TwoFuelModelsMethod::TwoDimensional,
        30.0, SlopeUnits::Percent, 0.0,
        50.0, FractionUnits::Percent,
        30.0, LengthUnits::Feet,
        0.50, FractionUnits::Fraction,
    );
}

/// C++ setCrownInputsLowMoistureScenario
fn set_crown_inputs_low_moisture(run: &mut BehaveRun) {
    run.crown.update_crown_inputs(
        124,
        6.0, 7.0, 8.0, 60.0, 90.0, 120.0, FractionUnits::Percent,
        5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot,
        0.0, WindAndSpreadOrientationMode::RelativeToNorth,
        30.0, SlopeUnits::Percent, 0.0,
        50.0, FractionUnits::Percent,
        30.0, 6.0, LengthUnits::Feet,
        0.50, FractionUnits::Fraction,
        0.03, DensityUnits::PoundsPerCubicFoot,
    );
}

// ---------------------------------------------------------------------------
// Test sections (one per C++ test function, executed in main()'s order)
// ---------------------------------------------------------------------------

fn test_surface_single_fuel_model(t: &mut TestInfo, run: &mut BehaveRun) {
    set_surface_inputs_for_gs4_low_moisture(run);

    let mph = SpeedUnits::MilesPerHour;
    let twenty = WindHeightInputMode::TwentyFoot;

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_slope(30.0, SlopeUnits::Degrees);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_speed(5.0, mph, twenty);
    run.surface.set_wind_direction(45.0);
    run.surface.set_aspect(95.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: north mode, 45deg wind, 95deg aspect, 5mph 20ft wind, 30deg slope",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        19.677584, TOL,
    );

    let pct = FractionUnits::Percent;
    t.check(
        "surface: live moisture of extinction",
        round6(run.surface.live_fuel_moisture_of_extinction(pct)),
        137.968551, TOL,
    );
    t.check(
        "surface: characteristic live moisture",
        round6(run.surface.characteristic_moisture_by_life_state(FuelLifeState::Live, pct)),
        85.874007, TOL,
    );
    t.check(
        "surface: characteristic dead moisture",
        round6(run.surface.characteristic_moisture_by_life_state(FuelLifeState::Dead, pct)),
        6.005463, TOL,
    );
    t.check(
        "surface: characteristic SAVR",
        round6(run.surface.characteristic_savr(SurfaceAreaToVolumeUnits::SquareFeetOverCubicFeet)),
        1631.128734, TOL,
    );
    t.check(
        "surface: heat source (45deg wind, 95deg aspect, 30deg slope)",
        round6(run.surface.heat_source(HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerMinute)),
        5177.248579, TOL,
    );

    run.surface.set_wind_direction(0.0);
    run.surface.set_aspect(0.0);
    run.surface.set_slope(0.0, SlopeUnits::Degrees);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: heat source (0deg wind, 0deg aspect, 0deg slope)",
        round6(run.surface.heat_source(HeatSourceAndReactionIntensityUnits::BtusPerSquareFootPerMinute)),
        1164.267376, TOL,
    );

    run.surface.set_fuel_model_number(124);
    run.surface.set_wind_speed(5.0, mph, twenty);
    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.set_wind_direction(0.0);
    run.surface.set_slope(30.0, SlopeUnits::Percent);
    run.surface.set_aspect(0.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: upslope mode, 5mph 20ft upslope wind",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        8.876216, TOL,
    );
    t.check(
        "surface: backing spread rate",
        run.surface.backing_spread_rate(SpeedUnits::ChainsPerHour),
        2.91614659, TOL,
    );
    t.check(
        "surface: flanking spread rate",
        run.surface.flanking_spread_rate(SpeedUnits::ChainsPerHour),
        5.08766627, TOL,
    );
    t.check(
        "surface: spread distance (2 hours)",
        run.surface.spread_distance(LengthUnits::Chains, 2.0, TimeUnits::Hours),
        17.7524327, TOL,
    );
    t.check(
        "surface: backing spread distance (2 hours)",
        run.surface.backing_spread_distance(LengthUnits::Chains, 2.0, TimeUnits::Hours),
        5.8322932, TOL,
    );
    t.check(
        "surface: flanking spread distance (2 hours)",
        run.surface.flanking_spread_distance(LengthUnits::Chains, 2.0, TimeUnits::Hours),
        10.17533253, TOL,
    );

    // Moisture scenario input mode
    run.surface.set_moisture_input_mode(MoistureInputMode::MoistureScenario);
    run.surface.set_current_moisture_scenario_by_name("D1L1");
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: moisture scenario D1L1",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        15.023945, TOL,
    );

    run.surface.set_current_moisture_scenario_by_name("d2L3"); // case-insensitive
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: moisture scenario d2L3 (case-insensitive)",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        1.978840, TOL,
    );

    // Aggregate live and dead moisture input mode
    run.surface.set_moisture_input_mode(MoistureInputMode::AllAggregate);
    run.surface.set_moisture_dead_aggregate(3.0, pct);
    run.surface.set_moisture_live_aggregate(80.0, pct);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: aggregate live and dead moisture input mode",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        8.589431, TOL,
    );

    // Moisture-class-needed checks under AllAggregate.
    // NOTE: the C++ suite compares observed to itself here (testBehave.cpp:379
    // et seq. pass `observed` for both arguments), so these were never real
    // assertions in C++. We assert the declared expected values.
    let needed =
        |run: &BehaveRun, c: MoistureClassInput| run.surface.inputs().is_moisture_class_input_needed(c);
    t.check_bool("moisture needed AllAggregate: 1-hr", needed(run, MoistureClassInput::OneHour), false);
    t.check_bool("moisture needed AllAggregate: 10-hr", needed(run, MoistureClassInput::TenHour), false);
    t.check_bool("moisture needed AllAggregate: 100-hr", needed(run, MoistureClassInput::HundredHour), false);
    t.check_bool("moisture needed AllAggregate: live herb", needed(run, MoistureClassInput::LiveHerbaceous), false);
    t.check_bool("moisture needed AllAggregate: live woody", needed(run, MoistureClassInput::LiveWoody), false);
    t.check_bool("moisture needed AllAggregate: dead aggregate", needed(run, MoistureClassInput::DeadAggregate), true);
    t.check_bool("moisture needed AllAggregate: live aggregate", needed(run, MoistureClassInput::LiveAggregate), true);

    run.surface.set_moisture_input_mode(MoistureInputMode::DeadAggregateAndLiveSizeClass);
    t.check_bool("moisture needed DeadAgg+LiveSize: 1-hr", needed(run, MoistureClassInput::OneHour), false);
    t.check_bool("moisture needed DeadAgg+LiveSize: 10-hr", needed(run, MoistureClassInput::TenHour), false);
    t.check_bool("moisture needed DeadAgg+LiveSize: 100-hr", needed(run, MoistureClassInput::HundredHour), false);
    t.check_bool("moisture needed DeadAgg+LiveSize: live herb", needed(run, MoistureClassInput::LiveHerbaceous), true);
    t.check_bool("moisture needed DeadAgg+LiveSize: live woody", needed(run, MoistureClassInput::LiveWoody), true);
    t.check_bool("moisture needed DeadAgg+LiveSize: dead aggregate", needed(run, MoistureClassInput::DeadAggregate), true);
    t.check_bool("moisture needed DeadAgg+LiveSize: live aggregate", needed(run, MoistureClassInput::LiveAggregate), false);

    // Aggregate dead + live size class spread rate
    set_surface_inputs_for_gs4_low_moisture(run);
    run.surface.set_moisture_input_mode(MoistureInputMode::DeadAggregateAndLiveSizeClass);
    run.surface.set_moisture_dead_aggregate(3.0, pct);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: dead aggregate + live size class moisture mode",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        9.752679, TOL,
    );

    run.surface.set_moisture_input_mode(MoistureInputMode::BySizeClass);
    set_surface_inputs_for_gs4_low_moisture(run);

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.set_wind_speed(5.0, mph, twenty);
    run.surface.set_wind_direction(90.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: upslope mode, cross-slope wind (90deg)",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        7.091665, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_direction(0.0);
    run.surface.set_aspect(0.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: north mode, north wind, zero aspect",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        8.876216, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_aspect(215.0);
    run.surface.set_wind_direction(45.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: north mode, 45deg wind, 215deg aspect",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        4.113265, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_aspect(5.0);
    run.surface.set_wind_direction(45.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: north mode, 45deg wind, 5deg aspect",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        8.503960, TOL,
    );

    run.surface.set_fuel_model_number(4);
    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_aspect(0.0);
    run.surface.set_wind_direction(90.0);
    run.surface.set_wind_speed(5.0, mph, twenty);
    run.surface.set_slope(30.0, SlopeUnits::Degrees);
    run.surface.set_canopy_cover(40.0, pct);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: FM4, 90deg wind, 0deg aspect, 40% canopy",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        46.631688, TOL,
    );

    run.surface.set_fuel_model_number(91);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "surface: non-burnable fuel (FM91)",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        0.0, TOL,
    );
}

fn test_chaparral(t: &mut TestInfo, run: &mut BehaveRun) {
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.set_wind_direction(0.0);

    let inputs = run.surface.inputs_mut();
    inputs.set_is_using_chaparral(true);
    inputs.set_chaparral_fuel_bed_depth(1.0, LengthUnits::Feet);
    inputs.set_chaparral_fuel_type(ChaparralFuelType::NotSet);
    inputs.set_chaparral_fuel_load_input_mode(ChaparralFuelLoadInputMode::DirectFuelLoad);
    inputs.set_chaparral_fuel_dead_load_fraction(0.25);
    inputs.set_chaparral_total_fuel_load(0.333, LoadingUnits::PoundsPerSquareFoot);
    run.surface.set_wind_speed(3.0, SpeedUnits::MilesPerHour, WindHeightInputMode::DirectMidflame);
    run.surface.set_slope(0.0, SlopeUnits::Percent);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "chaparral: direct load, 1ft depth, 25% dead, 3mph, 0% slope",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        1.792546, TOL,
    );

    let inputs = run.surface.inputs_mut();
    inputs.set_is_using_chaparral(true);
    inputs.set_chaparral_fuel_bed_depth(2.0, LengthUnits::Feet);
    inputs.set_chaparral_fuel_type(ChaparralFuelType::Chamise);
    inputs.set_chaparral_fuel_load_input_mode(ChaparralFuelLoadInputMode::FuelLoadFromDepthAndChaparralType);
    inputs.set_chaparral_fuel_dead_load_fraction(0.33);
    run.surface.set_wind_height_input_mode(WindHeightInputMode::DirectMidflame);
    run.surface.set_wind_speed(4.0, SpeedUnits::MilesPerHour, WindHeightInputMode::DirectMidflame);
    run.surface.set_slope(10.0, SlopeUnits::Percent);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "chaparral: chamise from depth, 2ft, 33% dead, 4mph, 10% slope",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        6.108717, TOL,
    );

    let inputs = run.surface.inputs_mut();
    inputs.set_is_using_chaparral(true);
    inputs.set_chaparral_fuel_bed_depth(3.0, LengthUnits::Feet);
    inputs.set_chaparral_fuel_type(ChaparralFuelType::MixedBrush);
    inputs.set_chaparral_fuel_load_input_mode(ChaparralFuelLoadInputMode::FuelLoadFromDepthAndChaparralType);
    inputs.set_chaparral_fuel_dead_load_fraction(0.50);
    run.surface.set_wind_height_input_mode(WindHeightInputMode::DirectMidflame);
    run.surface.set_wind_speed(5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::DirectMidflame);
    run.surface.set_slope(20.0, SlopeUnits::Percent);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "chaparral: mixed brush from depth, 3ft, 50% dead, 5mph, 20% slope",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        13.945025, TOL,
    );

    run.surface.inputs_mut().set_is_using_chaparral(false);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
}

fn test_calculate_scorch_height(t: &mut TestInfo, run: &mut BehaveRun) {
    let observed = run.mortality.calculate_scorch_height(
        50.0, FirelineIntensityUnits::BtusPerFootPerSecond,
        5.0, SpeedUnits::MilesPerHour,
        80.0, TemperatureUnits::Fahrenheit,
        LengthUnits::Feet,
    );
    t.check("scorch height: 80F, 5mph, 50 Btu/ft/s", observed, 7.617325, TOL);

    let observed = run.mortality.calculate_scorch_height(
        55.0, FirelineIntensityUnits::BtusPerFootPerSecond,
        300.0, SpeedUnits::FeetPerMinute,
        70.0, TemperatureUnits::Fahrenheit,
        LengthUnits::Feet,
    );
    t.check("scorch height: 70F, 300 ft/min, 55 Btu/ft/s", observed, 9.923720, TOL);
}

fn test_palmetto_gallberry(t: &mut TestInfo, run: &mut BehaveRun) {
    run.surface.inputs_mut().set_is_using_palmetto_gallberry(true);
    run.surface.inputs_mut().update_surface_inputs_for_palmetto_gallberry(
        6.0, 7.0, 8.0, 60.0, 90.0, FractionUnits::Percent,
        5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot,
        0.0, WindAndSpreadOrientationMode::RelativeToUpslope,
        10.0, // age of rough (years)
        4.0,  // height of understory (ft)
        50.0, // palmetto coverage (%)
        50.0, BasalAreaUnits::SquareFeetPerAcre, // overstory basal area
        30.0, SlopeUnits::Percent, 0.0,
        50.0, FractionUnits::Percent,
        30.0, LengthUnits::Feet,
        0.50, FractionUnits::Fraction,
    );
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "palmetto-gallberry: spread rate",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        12.521131, TOL,
    );
    run.surface.inputs_mut().set_is_using_palmetto_gallberry(false);
}

fn test_western_aspen(t: &mut TestInfo, run: &mut BehaveRun) {
    run.surface.inputs_mut().set_is_using_western_aspen(true);
    run.surface.inputs_mut().update_surface_inputs_for_western_aspen(
        3, 50.0, FractionUnits::Percent, AspenFireSeverity::Low,
        10.0, LengthUnits::Inches,
        6.0, 7.0, 8.0, 60.0, 90.0, FractionUnits::Percent,
        5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot,
        0.0, WindAndSpreadOrientationMode::RelativeToUpslope,
        30.0, SlopeUnits::Percent, 0.0,
        50.0, FractionUnits::Percent,
        30.0, LengthUnits::Feet,
        0.50, FractionUnits::Fraction,
    );
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "western aspen: spread rate",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        0.847629, TOL,
    );

    t.check(
        "western aspen: mortality",
        round6(run.surface.get_aspen_mortality(FractionUnits::Fraction)),
        0.267093, TOL,
    );

    run.surface.inputs_mut().set_is_using_western_aspen(false);
}

fn test_length_to_width_ratio(t: &mut TestInfo, run: &mut BehaveRun) {
    set_surface_inputs_for_gs4_low_moisture(run);

    let mph = SpeedUnits::MilesPerHour;
    let twenty = WindHeightInputMode::TwentyFoot;

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_slope(0.0, SlopeUnits::Degrees);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_speed(0.0, mph, twenty);
    run.surface.set_wind_direction(0.0);
    run.surface.set_aspect(0.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "L/W: no wind, no slope",
        round6(run.surface.fire_length_to_width_ratio()),
        1.0, TOL,
    );

    let midflame = WindHeightInputMode::DirectMidflame;
    run.surface.set_wind_height_input_mode(midflame);
    run.surface.set_slope(0.0, SlopeUnits::Degrees);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_speed(5.0, mph, midflame);
    run.surface.set_wind_direction(0.0);
    run.surface.set_aspect(0.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "L/W: 5mph midflame, no slope",
        round6(run.surface.fire_length_to_width_ratio()),
        1.590064, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_slope(30.0, SlopeUnits::Degrees);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_speed(5.0, mph, twenty);
    run.surface.set_wind_direction(45.0);
    run.surface.set_aspect(95.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "L/W: 5mph 20ft, 45deg wind, 95deg aspect, 30deg slope",
        round6(run.surface.fire_length_to_width_ratio()),
        1.375624, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_slope(30.0, SlopeUnits::Degrees);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_speed(15.0, mph, twenty);
    run.surface.set_wind_direction(45.0);
    run.surface.set_aspect(95.0);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "L/W: 15mph 20ft, 45deg wind, 95deg aspect, 30deg slope",
        round6(run.surface.fire_length_to_width_ratio()),
        1.519936, TOL,
    );

    // Crown L/W. NOTE: the C++ suite reports the (stale) *surface* L/W values
    // for these three checks (testBehave.cpp:761/769/777 pass
    // observedLengthToWidthRatio, not observedCrownLengthToWidthRatio), so
    // the crown expectations were never actually asserted in C++. We assert
    // the declared expected values, which follow Rothermel: 1 + 0.125 * U20mph.
    run.crown.set_wind_speed(0.0, mph, twenty);
    set_crown_inputs_low_moisture(run); // resets wind to 5 mph
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown L/W: 5mph (scenario default)",
        round6(run.crown.get_crown_fire_length_to_width_ratio()),
        1.625, TOL,
    );

    set_crown_inputs_low_moisture(run);
    run.crown.set_wind_speed(10.0, mph, twenty);
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown L/W: 10mph",
        round6(run.crown.get_crown_fire_length_to_width_ratio()),
        2.25, TOL,
    );

    set_crown_inputs_low_moisture(run);
    run.crown.set_wind_speed(15.0, mph, twenty);
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown L/W: 15mph",
        round6(run.crown.get_crown_fire_length_to_width_ratio()),
        2.875, TOL,
    );
}

fn test_elliptical_dimensions(t: &mut TestInfo, run: &mut BehaveRun) {
    let elapsed = 3.5869124; // minutes; chosen so semimajor axis == 1.0

    set_surface_inputs_for_gs4_low_moisture(run);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    set_surface_inputs_for_gs4_low_moisture(run);
    run.surface.set_slope(0.0, SlopeUnits::Degrees);
    run.surface.set_wind_speed(5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::DirectMidflame);
    run.surface.do_surface_run_in_direction_of_max_spread();

    let chains = LengthUnits::Chains;
    t.check(
        "ellipse: dimension a",
        round6(run.surface.elliptical_a(chains, elapsed, TimeUnits::Minutes)),
        0.628905, TOL,
    );
    t.check(
        "ellipse: dimension b",
        round6(run.surface.elliptical_b(chains, elapsed, TimeUnits::Minutes)),
        1.0, TOL,
    );
    t.check(
        "ellipse: dimension c",
        round6(run.surface.elliptical_c(chains, elapsed, TimeUnits::Minutes)),
        0.777482, TOL,
    );
    t.check(
        "ellipse: heading-to-backing ratio",
        round6(run.surface.heading_to_backing_ratio()),
        7.988029, TOL,
    );
    t.check(
        "ellipse: area in acres after 1 hour",
        round6(run.surface.fire_area(AreaUnits::Acres, 1.0, TimeUnits::Hours)),
        55.283555, TOL,
    );
    t.check(
        "ellipse: area in km^2 after 1 hour",
        round6(run.surface.fire_area(AreaUnits::SquareKilometers, 1.0, TimeUnits::Hours)),
        0.223725, TOL,
    );
    t.check(
        "ellipse: perimeter in chains after 1 hour",
        round6(run.surface.fire_perimeter(chains, 1.0, TimeUnits::Hours)),
        86.71476, TOL,
    );
}

fn test_direction_of_interest(t: &mut TestInfo, run: &mut BehaveRun) {
    set_surface_inputs_for_gs4_low_moisture(run);

    let twenty = WindHeightInputMode::TwentyFoot;
    let from_perimeter = SurfaceFireSpreadDirectionMode::FromPerimeter;

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.set_wind_direction(0.0);
    run.surface.do_surface_run_in_direction_of_interest(90.0, from_perimeter);
    t.check(
        "DOI: perimeter mode, 90deg from upslope",
        round6(run.surface.spread_rate_in_direction_of_interest(SpeedUnits::FeetPerMinute)),
        5.596433, TOL,
    );
    t.check(
        "DOI: perimeter mode, 90deg from upslope, flame length",
        round6(run.surface.flame_length_in_direction_of_interest(LengthUnits::Feet)),
        6.598148, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.set_wind_direction(290.0);
    run.surface.do_surface_run_in_direction_of_interest(160.0, from_perimeter);
    t.check(
        "DOI: 160deg from upslope, 290deg wind",
        round6(run.surface.spread_rate_in_direction_of_interest(SpeedUnits::ChainsPerHour)),
        2.766387, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.set_wind_direction(215.0);
    run.surface.do_surface_run_in_direction_of_interest(215.0, from_perimeter);
    t.check(
        "DOI: 215deg from upslope, 215deg wind",
        round6(run.surface.spread_rate_in_direction_of_interest(SpeedUnits::ChainsPerHour)),
        2.818063, TOL,
    );

    let from_ignition = SurfaceFireSpreadDirectionMode::FromIgnitionPoint;

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_direction(280.0);
    run.surface.set_aspect(135.0);
    run.surface.do_surface_run_in_direction_of_interest(30.0, from_ignition);
    t.check(
        "DOI: ignition mode, 30deg from north, 280deg wind, 135deg aspect",
        round6(run.surface.spread_rate_in_direction_of_interest(SpeedUnits::ChainsPerHour)),
        4.180938, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_direction(0.0);
    run.surface.set_aspect(45.0);
    run.surface.do_surface_run_in_direction_of_interest(90.0, from_ignition);
    t.check(
        "DOI: ignition mode, 90deg from north, north wind, 45deg aspect",
        round6(run.surface.spread_rate_in_direction_of_interest(SpeedUnits::ChainsPerHour)),
        3.438243, TOL,
    );

    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.set_wind_direction(280.0);
    run.surface.set_aspect(263.0);
    run.surface.do_surface_run_in_direction_of_interest(285.0, from_ignition);
    t.check(
        "DOI: ignition mode, 285deg from north, 280deg wind, 263deg aspect",
        round6(run.surface.spread_rate_in_direction_of_interest(SpeedUnits::ChainsPerHour)),
        2.944975, TOL,
    );
}

fn test_fireline_intensity(t: &mut TestInfo, run: &mut BehaveRun) {
    set_surface_inputs_for_gs4_low_moisture(run);
    run.surface.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.do_surface_run_in_direction_of_max_spread();
    t.check(
        "fireline intensity: Btu/ft/s",
        run.surface.fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond),
        598.339039, TOL,
    );
    t.check(
        "fireline intensity: kW/m",
        run.surface.fireline_intensity(FirelineIntensityUnits::KilowattsPerMeter),
        2072.730450, TOL,
    );
}

fn test_two_fuel_models(t: &mut TestInfo, run: &mut BehaveRun) {
    run.surface.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
    set_surface_inputs_for_two_fuel_models_low_moisture(run);

    let expected = [
        (0.0, 8.876216),
        (10.0, 10.470801),
        (20.0, 12.189713),
        (30.0, 13.958900),
        (40.0, 15.706408),
        (50.0, 17.362382),
        (60.0, 18.859066),
        (70.0, 20.130802),
        (80.0, 21.114030),
        (90.0, 21.747289),
        (100.0, 21.971217),
    ];
    for (coverage, expected_rate) in expected {
        run.surface
            .inputs_mut()
            .set_two_fuel_models_first_fuel_model_coverage(coverage, FractionUnits::Percent);
        run.surface.do_surface_run_in_direction_of_max_spread();
        t.check(
            &format!("two fuel models: first model coverage {coverage}%"),
            round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
            expected_rate, TOL,
        );
    }
}

fn test_crown_module_rothermel(t: &mut TestInfo, run: &mut BehaveRun) {
    let mph = SpeedUnits::MilesPerHour;
    let twenty = WindHeightInputMode::TwentyFoot;

    run.crown.set_wind_height_input_mode(twenty);

    set_crown_inputs_low_moisture(run);
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown Rothermel: spread rate",
        round6(run.crown.get_crown_fire_spread_rate(SpeedUnits::ChainsPerHour)),
        10.259921, TOL,
    );
    t.check(
        "crown Rothermel: length-to-width ratio",
        round6(run.crown.get_crown_fire_length_to_width_ratio()),
        1.625, TOL,
    );
    t.check(
        "crown Rothermel: area",
        round6(run.crown.get_crown_fire_area(AreaUnits::Acres, 1.0, TimeUnits::Hours)),
        5.087736, TOL,
    );
    // NOTE: C++ reports the (stale) area value for the perimeter check
    // (testBehave.cpp:1104), so the perimeter expectation was never actually
    // asserted in C++. We assert the declared expected value here.
    t.check(
        "crown Rothermel: perimeter",
        round6(run.crown.get_crown_fire_perimeter(LengthUnits::Chains, 1.0, TimeUnits::Hours)),
        26.033937, TOL,
    );
    t.check(
        "crown Rothermel: flame length",
        round6(run.crown.get_crown_flame_length(LengthUnits::Feet)),
        29.320557, TOL,
    );
    t.check(
        "crown Rothermel: fireline intensity",
        round6(run.crown.get_crown_fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond)),
        1775.061222, TOL,
    );

    set_crown_inputs_low_moisture(run);
    run.crown.set_moisture_one_hour(20.0, FractionUnits::Percent);
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown Rothermel: fire type Surface",
        run.crown.get_fire_type() as i32 as f64,
        FireType::Surface as i32 as f64, TOL,
    );

    set_crown_inputs_low_moisture(run);
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown Rothermel: fire type Torching",
        run.crown.get_fire_type() as i32 as f64,
        FireType::Torching as i32 as f64, TOL,
    );

    set_crown_inputs_low_moisture(run);
    run.crown.set_wind_speed(10.0, mph, twenty);
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown Rothermel: fire type Crowning",
        run.crown.get_fire_type() as i32 as f64,
        FireType::Crowning as i32 as f64, TOL,
    );

    set_crown_inputs_low_moisture(run);
    run.crown.set_canopy_height(60.0, LengthUnits::Feet);
    run.crown.set_canopy_base_height(30.0, LengthUnits::Feet);
    run.crown.set_canopy_bulk_density(0.06, DensityUnits::PoundsPerCubicFoot);
    run.crown.set_wind_speed(5.0, mph, twenty);
    run.crown.do_crown_run_rothermel();
    t.check(
        "crown Rothermel: fire type ConditionalCrownFire",
        run.crown.get_fire_type() as i32 as f64,
        FireType::ConditionalCrownFire as i32 as f64, TOL,
    );
}

fn test_crown_module_scott_and_reinhardt(t: &mut TestInfo, run: &mut BehaveRun) {
    let fpm = SpeedUnits::FeetPerMinute;
    let twenty = WindHeightInputMode::TwentyFoot;

    run.crown.set_wind_adjustment_factor_calculation_method(
        WindAdjustmentFactorCalculationMethod::UserInput,
    );
    run.crown.set_user_provided_wind_adjustment_factor(0.4);
    run.crown.update_crown_inputs(
        10,
        8.0, 9.0, 10.0, 0.0, 117.0, 100.0, FractionUnits::Percent,
        2187.226624, fpm, twenty,
        0.0, WindAndSpreadOrientationMode::RelativeToUpslope,
        20.0, SlopeUnits::Percent, 0.0,
        50.0, FractionUnits::Percent,
        38.104626, 2.952756, LengthUnits::Feet,
        0.50, FractionUnits::Fraction,
        0.01311, DensityUnits::PoundsPerCubicFoot,
    );
    run.crown.do_crown_run_scott_and_reinhardt();
    t.check(
        "crown S&R: final spread rate",
        round6(run.crown.get_final_spread_rate(fpm)),
        65.221842, TOL,
    );
    t.check(
        "crown S&R: final flame length",
        round6(run.crown.get_final_flame_length(LengthUnits::Feet)),
        60.744542, TOL,
    );
    t.check(
        "crown S&R: final fireline intensity",
        round6(run.crown.get_final_fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond)),
        5293.170672, TOL,
    );
    t.check(
        "crown S&R: critical open wind speed",
        run.crown.get_critical_open_wind_speed(fpm),
        1717.916785, TOL,
    );
    t.check(
        "crown S&R: fire type Crowning",
        run.crown.get_fire_type() as i32 as f64,
        FireType::Crowning as i32 as f64, TOL,
    );

    run.crown.set_moisture_input_mode(MoistureInputMode::AllAggregate);
    run.crown.set_moisture_dead_aggregate(9.0, FractionUnits::Percent);
    run.crown.set_moisture_live_aggregate(100.0, FractionUnits::Percent);
    run.crown.set_wind_adjustment_factor_calculation_method(
        WindAdjustmentFactorCalculationMethod::UseCrownRatio,
    );
    run.crown.set_wind_speed(2187.2266239, fpm, twenty);
    run.crown.do_crown_run_scott_and_reinhardt();
    t.check(
        "crown S&R: spread rate with aggregate moisture",
        round6(run.crown.get_final_spread_rate(SpeedUnits::ChainsPerHour)),
        64.016394, TOL,
    );

    run.crown.set_moisture_input_mode(MoistureInputMode::MoistureScenario);
    run.crown.set_current_moisture_scenario_by_name("D3L2");
    run.crown.do_crown_run_scott_and_reinhardt();
    t.check(
        "crown S&R: spread rate with moisture scenario D3L2",
        round6(run.crown.get_final_spread_rate(SpeedUnits::ChainsPerHour)),
        68.334996, TOL,
    );

    // Torching-fire case (FM5)
    run.crown.set_moisture_input_mode(MoistureInputMode::BySizeClass);
    run.crown.set_wind_adjustment_factor_calculation_method(
        WindAdjustmentFactorCalculationMethod::UserInput,
    );
    run.crown.set_user_provided_wind_adjustment_factor(0.15);
    run.crown.update_crown_inputs(
        5,
        5.0, 6.0, 8.0, 0.0, 117.0, 100.0, FractionUnits::Percent,
        24.854848, SpeedUnits::MilesPerHour, twenty,
        0.0, WindAndSpreadOrientationMode::RelativeToUpslope,
        20.0, SlopeUnits::Percent, 0.0,
        50.0, FractionUnits::Percent,
        71.631562, 4.92126, LengthUnits::Feet,
        0.50, FractionUnits::Fraction,
        0.003746, DensityUnits::PoundsPerCubicFoot,
    );
    run.crown.do_crown_run_scott_and_reinhardt();
    t.check(
        "crown S&R torching: final spread rate",
        round6(run.crown.get_final_spread_rate(fpm)),
        29.475388, TOL,
    );
    t.check(
        "crown S&R torching: final flame length",
        round6(run.crown.get_final_flame_length(LengthUnits::Feet)),
        12.759447, TOL,
    );
    t.check(
        "crown S&R torching: final fireline intensity",
        round6(run.crown.get_final_fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond)),
        509.568753, TOL,
    );
    t.check(
        "crown S&R torching: critical open wind speed",
        run.crown.get_critical_open_wind_speed(fpm),
        3874.421988, TOL,
    );
    t.check(
        "crown S&R torching: fire type Torching",
        run.crown.get_fire_type() as i32 as f64,
        FireType::Torching as i32 as f64, TOL,
    );

    run.surface.set_wind_adjustment_factor_calculation_method(
        WindAdjustmentFactorCalculationMethod::UseCrownRatio,
    );
}

fn test_spot_module(t: &mut TestInfo, run: &mut BehaveRun) {
    set_surface_inputs_for_gs4_low_moisture(run);
    run.surface.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
    run.surface.set_user_provided_wind_adjustment_factor(1.0);
    run.surface.set_wind_adjustment_factor_calculation_method(
        WindAdjustmentFactorCalculationMethod::UserInput,
    );
    run.surface.do_surface_run_in_direction_of_max_spread();
    let flame_length = run.surface.flame_length_output(LengthUnits::Feet);

    let location = SpotFireLocation::RidgeTop;
    let miles = LengthUnits::Miles;
    let feet = LengthUnits::Feet;
    let mph = SpeedUnits::MilesPerHour;

    run.spot.update_spot_inputs_for_burning_pile(
        location, 1.0, miles, 2000.0, feet, 30.0, feet,
        SpotDownWindCanopyMode::Closed, 5.0, feet, 5.0, mph,
    );
    run.spot.calculate_spotting_distance_from_burning_pile();
    t.check(
        "spot: mountain distance from burning pile, closed canopy",
        round6(run.spot.get_max_mountainous_terrain_spotting_distance_from_burning_pile(miles)),
        0.021330, TOL,
    );
    t.check(
        "spot: flat distance from burning pile, closed canopy",
        round6(run.spot.get_max_flat_terrain_spotting_distance_from_burning_pile(miles)),
        0.017067, TOL,
    );

    run.spot.update_spot_inputs_for_burning_pile(
        location, 1.0, miles, 2000.0, feet, 30.0, feet,
        SpotDownWindCanopyMode::Open, 5.0, feet, 5.0, mph,
    );
    run.spot.calculate_spotting_distance_from_burning_pile();
    t.check(
        "spot: mountain distance from burning pile, open canopy",
        round6(run.spot.get_max_mountainous_terrain_spotting_distance_from_burning_pile(miles)),
        0.030863, TOL,
    );
    t.check(
        "spot: flat distance from burning pile, open canopy",
        round6(run.spot.get_max_flat_terrain_spotting_distance_from_burning_pile(miles)),
        0.024700, TOL,
    );

    run.spot.update_spot_inputs_for_surface_fire(
        location, 1.0, miles, 2000.0, feet, 30.0, feet,
        SpotDownWindCanopyMode::Closed, 5.0, mph, flame_length, feet,
    );
    run.spot.calculate_spotting_distance_from_surface_fire();
    t.check(
        "spot: mountain distance from surface fire, closed canopy",
        round6(run.spot.get_max_mountainous_terrain_spotting_distance_from_surface_fire(miles)),
        0.267467, TOL,
    );
    t.check(
        "spot: flat distance from surface fire, closed canopy",
        round6(run.spot.get_max_flat_terrain_spotting_distance_from_surface_fire(miles)),
        0.22005, TOL,
    );

    run.spot.update_spot_inputs_for_torching_trees(
        location, 1.0, miles, 2000.0, feet, 30.0, feet,
        SpotDownWindCanopyMode::Closed,
        15, 20.0, LengthUnits::Inches, 30.0, feet,
        SpotTreeSpecies::EngelmannSpruce, 5.0, mph,
    );
    run.spot.calculate_spotting_distance_from_torching_trees();
    t.check(
        "spot: mountain distance from torching trees, closed canopy",
        round6(run.spot.get_max_mountainous_terrain_spotting_distance_from_torching_trees(miles)),
        0.222396, TOL,
    );
    t.check(
        "spot: flat distance from torching trees, closed canopy",
        round6(run.spot.get_max_flat_terrain_spotting_distance_from_torching_trees(miles)),
        0.181449, TOL,
    );

    run.spot.update_spot_inputs_for_active_crown_fire(
        location, 1.0, miles, 2000.0, feet, 30.0, feet,
        SpotDownWindCanopyMode::Closed, 5.0, mph, 20.0, feet,
    );
    run.spot.calculate_spotting_distance_from_active_crown();
    t.check(
        "spot: mountain distance from active crown fire, closed canopy",
        round6(run.spot.get_max_mountainous_terrain_spotting_distance_from_active_crown(miles)),
        0.400473, TOL,
    );
}

fn test_speed_unit_conversion(t: &mut TestInfo, run: &mut BehaveRun) {
    let mph = SpeedUnits::MilesPerHour;
    let twenty = WindHeightInputMode::TwentyFoot;

    run.surface.set_wind_adjustment_factor_calculation_method(
        WindAdjustmentFactorCalculationMethod::UseCrownRatio,
    );
    set_surface_inputs_for_gs4_low_moisture(run);

    run.surface.set_fuel_model_number(124);
    run.surface.set_wind_speed(5.0, mph, twenty);
    run.surface.set_wind_height_input_mode(twenty);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToUpslope);
    run.surface.set_wind_direction(0.0);
    run.surface.set_slope(30.0, SlopeUnits::Percent);
    run.surface.set_aspect(0.0);
    run.surface.do_surface_run_in_direction_of_max_spread();

    let expected = [
        (SpeedUnits::ChainsPerHour, 8.876216, "ch/h"),
        (SpeedUnits::FeetPerMinute, 9.763838, "ft/min"),
        (SpeedUnits::KilometersPerHour, 0.178561, "km/h"),
        (SpeedUnits::MetersPerMinute, 2.976018, "m/min"),
        (SpeedUnits::MetersPerSecond, 0.049600, "m/s"),
        (SpeedUnits::MilesPerHour, 0.110953, "mi/h"),
    ];
    for (units, expected_rate, label) in expected {
        t.check(
            &format!("speed units: spread rate in {label}"),
            round6(run.surface.spread_rate(units)),
            expected_rate, TOL,
        );
    }
}

fn test_ignite_module(t: &mut TestInfo, run: &mut BehaveRun) {
    run.ignite.update_ignite_inputs(
        6.0, 8.0, FractionUnits::Percent,
        80.0, TemperatureUnits::Fahrenheit,
        50.0, FractionUnits::Percent,
        IgnitionFuelBedType::DouglasFirDuff,
        6.0, LengthUnits::Inches,
        LightningCharge::Unknown,
    );
    t.check(
        "ignite: firebrand probability, Douglas fir duff",
        run.ignite.calculate_firebrand_ignition_probability(FractionUnits::Fraction),
        0.54831705, TOL,
    );
    t.check(
        "ignite: lightning probability, Douglas fir duff",
        run.ignite.calculate_lightning_ignition_probability(FractionUnits::Fraction),
        0.39362018, TOL,
    );

    run.ignite.update_ignite_inputs(
        7.0, 9.0, FractionUnits::Percent,
        90.0, TemperatureUnits::Fahrenheit,
        25.0, FractionUnits::Percent,
        IgnitionFuelBedType::LodgepolePineDuff,
        8.0, LengthUnits::Inches,
        LightningCharge::Negative,
    );
    t.check(
        "ignite: firebrand probability, lodgepole pine duff (percent)",
        run.ignite.calculate_firebrand_ignition_probability(FractionUnits::Percent),
        50.717573, TOL,
    );
    t.check(
        "ignite: lightning probability, lodgepole pine duff (percent)",
        run.ignite.calculate_lightning_ignition_probability(FractionUnits::Percent),
        17.931991, TOL,
    );
}

fn test_safety_module(t: &mut TestInfo, run: &mut BehaveRun) {
    run.safety.update_safety_inputs(
        5.0, LengthUnits::Feet, 6, 1, 50.0, 300.0, AreaUnits::SquareFeet,
    );
    run.safety.calculate_safety_zone();
    t.check(
        "safety: separation distance",
        run.safety.get_separation_distance(LengthUnits::Feet),
        20.0, TOL,
    );
    t.check(
        "safety: safety zone area",
        run.safety.get_safety_zone_area(AreaUnits::Acres),
        0.082490356, TOL,
    );
    t.check(
        "safety: safety zone radius",
        run.safety.get_safety_zone_radius(LengthUnits::Feet),
        33.819766, TOL,
    );
}

fn test_contain_module(t: &mut TestInfo, run: &mut BehaveRun) {
    run.contain.set_attack_distance(0.0, LengthUnits::Chains);
    run.contain.set_lw_ratio(3.0);
    run.contain.set_report_rate(5.0, SpeedUnits::ChainsPerHour);
    run.contain.set_report_size(1.0, AreaUnits::Acres);
    run.contain.set_tactic(ContainTactic::HeadAttack);
    run.contain.add_resource(
        2.0, 8.0, TimeUnits::Hours, 20.0, SpeedUnits::ChainsPerHour, "test", 0.0, 0.0,
    );
    run.contain.do_contain_run();

    t.check(
        "contain: final fire line length",
        run.contain.get_final_fire_line_length(LengthUnits::Chains),
        39.539849615, TOL,
    );
    t.check(
        "contain: perimeter at initial attack",
        run.contain.get_perimeter_at_initial_attack(LengthUnits::Chains),
        37.51917991, TOL,
    );
    t.check(
        "contain: perimeter at containment",
        run.contain.get_perimeter_at_containment(LengthUnits::Chains),
        39.539849615, TOL,
    );
    t.check(
        "contain: fire size at initial attack",
        run.contain.get_fire_size_at_initial_attack(AreaUnits::Acres),
        8.954501709, TOL,
    );
    t.check(
        "contain: final fire size",
        run.contain.get_final_fire_size(AreaUnits::Acres),
        9.42749714, TOL,
    );
    t.check(
        "contain: final containment area",
        run.contain.get_final_containment_area(AreaUnits::Acres),
        9.42749714, TOL,
    );
    t.check(
        "contain: final time since report",
        run.contain.get_final_time_since_report(TimeUnits::Minutes),
        238.75, TOL,
    );
    t.check(
        "contain: containment status Contained",
        run.contain.get_containment_status() as i32 as f64,
        ContainStatus::Contained as i32 as f64, TOL,
    );
}

fn test_fine_dead_fuel_moisture_tool(t: &mut TestInfo, run: &mut BehaveRun) {
    let pct = FractionUnits::Percent;
    let tool = &mut run.fine_dead_fuel_moisture_tool;

    tool.calculate_by_index(0, 0, 0, 0, 0, 0, 0, 0);
    t.check("FDFM: reference moisture, all-zero indices", tool.get_reference_moisture(pct), 1.0, TOL);
    t.check("FDFM: correction moisture, all-zero indices", tool.get_correction_moisture(pct), 2.0, TOL);
    t.check("FDFM: fine dead fuel moisture, all-zero indices", tool.get_fine_dead_fuel_moisture(pct), 3.0, TOL);

    tool.calculate_by_index(1, 1, 1, 1, 1, 1, 1, 1);
    t.check("FDFM: reference moisture, all-one indices", tool.get_reference_moisture(pct), 2.0, TOL);
    t.check("FDFM: correction moisture, all-one indices", tool.get_correction_moisture(pct), 4.0, TOL);
    t.check("FDFM: fine dead fuel moisture, all-one indices", tool.get_fine_dead_fuel_moisture(pct), 6.0, TOL);

    let max = (
        tool.get_aspect_index_size() - 1,
        tool.get_dry_bulb_index_size() - 1,
        tool.get_elevation_index_size() - 1,
        tool.get_month_index_size() - 1,
        tool.get_rh_index_size() - 1,
        tool.get_shading_index_size() - 1,
        tool.get_slope_index_size() - 1,
        tool.get_time_of_day_index_size() - 1,
    );
    tool.calculate_by_index(max.0, max.1, max.2, max.3, max.4, max.5, max.6, max.7);
    t.check("FDFM: reference moisture, max indices", tool.get_reference_moisture(pct), 12.0, TOL);
    t.check("FDFM: correction moisture, max indices", tool.get_correction_moisture(pct), 6.0, TOL);
    t.check("FDFM: fine dead fuel moisture, max indices", tool.get_fine_dead_fuel_moisture(pct), 18.0, TOL);

    tool.calculate_by_index(
        max.0 + 1, max.1 + 1, max.2 + 1, max.3 + 1, max.4 + 1, max.5 + 1, max.6 + 1, max.7 + 1,
    );
    t.check("FDFM: reference moisture, out-of-bounds indices", tool.get_reference_moisture(pct), -1.0, TOL);
    t.check("FDFM: correction moisture, out-of-bounds indices", tool.get_correction_moisture(pct), -1.0, TOL);
    t.check("FDFM: fine dead fuel moisture, out-of-bounds indices", tool.get_fine_dead_fuel_moisture(pct), -1.0, TOL);
}

fn test_slope_tool(t: &mut TestInfo, run: &mut BehaveRun) {
    let tool = &mut run.slope_tool;

    // Imperial: 1:1980, 3.6 in map distance, 50 ft contours, 4.1 contours.
    // BehavePlus 6 rounds these outputs to the nearest integer.
    tool.calculate_slope_from_map_measurements(1980, 3.6, LengthUnits::Inches, 50.0, 4.1, LengthUnits::Feet);
    t.check(
        "slope tool: degrees from map (imperial)",
        tool.get_slope_from_map_measurements(SlopeUnits::Degrees).round(),
        19.0, TOL,
    );
    t.check(
        "slope tool: percent from map (imperial)",
        tool.get_slope_from_map_measurements(SlopeUnits::Percent).round(),
        35.0, TOL,
    );
    t.check(
        "slope tool: elevation change (imperial)",
        tool.get_slope_elevation_change(LengthUnits::Feet).round(),
        205.0, TOL,
    );
    t.check(
        "slope tool: horizontal distance (imperial)",
        tool.get_slope_horizontal_distance(LengthUnits::Feet).round(),
        594.0, TOL,
    );

    // Metric: 1:3960, 3.0 cm map distance, 15 m contours, 5.5 contours.
    tool.calculate_slope_from_map_measurements(3960, 3.0, LengthUnits::Centimeters, 15.0, 5.5, LengthUnits::Meters);
    t.check(
        "slope tool: degrees from map (metric)",
        tool.get_slope_from_map_measurements(SlopeUnits::Degrees).round(),
        35.0, TOL,
    );
    t.check(
        "slope tool: percent from map (metric)",
        tool.get_slope_from_map_measurements(SlopeUnits::Percent).round(),
        69.0, TOL,
    );
    t.check(
        "slope tool: elevation change (metric)",
        tool.get_slope_elevation_change(LengthUnits::Meters).round(),
        82.0, TOL,
    );
    t.check(
        "slope tool: horizontal distance (metric)",
        tool.get_slope_horizontal_distance(LengthUnits::Meters).round(),
        119.0, TOL,
    );

    // Horizontal distances at 15-degree increments from upslope.
    // BehavePlus 6 rounds these to the nearest tenth.
    let expected = [2.9, 2.9, 2.9, 2.9, 3.0, 3.0, 3.0];
    tool.calculate_horizontal_distance(3.0, LengthUnits::Inches, 30.0, SlopeUnits::Percent);
    for (i, expected_distance) in expected.into_iter().enumerate() {
        let observed = (tool.get_horizontal_distance_at_index(i, LengthUnits::Feet) * 10.0).round() / 10.0;
        t.check(
            &format!("slope tool: horizontal distance at {}deg from upslope", 15 * i),
            observed, expected_distance, TOL,
        );
    }
}

fn test_vapor_pressure_deficit_calculator(t: &mut TestInfo, run: &mut BehaveRun) {
    // (temp F, RH %, expected VPD hPa); C++ tolerance is 1e-3 here.
    let scenarios = [
        (50.0, 100.0, 0.0),
        (50.0, 90.0, 1.22833),
        (50.0, 80.0, 2.45667),
        (50.0, 70.0, 3.685),
        (50.0, 60.0, 4.91334),
        (50.0, 50.0, 6.14167),
        (50.0, 40.0, 7.37001),
        (50.0, 30.0, 8.59834),
        (50.0, 20.0, 9.82667),
        (50.0, 10.0, 11.055),
    ];
    for (temp_f, rh, expected) in scenarios {
        run.vpd_calculator.set_temperature(temp_f, TemperatureUnits::Fahrenheit);
        run.vpd_calculator.set_relative_humidity(rh, FractionUnits::Percent);
        run.vpd_calculator.run_calculation();
        t.check(
            &format!("VPD: {rh}% RH at {temp_f}F"),
            run.vpd_calculator.get_vapor_pressure_deficit(PressureUnits::HectoPascal),
            expected, 1e-3,
        );
    }
}

fn test_simple_surface(t: &mut TestInfo, run: &mut BehaveRun) {
    run.surface.update_surface_inputs(
        124,
        6.0, 7.0, 8.0, 60.0, 90.0, FractionUnits::Percent,
        5.0, SpeedUnits::MilesPerHour, WindHeightInputMode::TwentyFoot,
        0.0, WindAndSpreadOrientationMode::RelativeToNorth,
        30.0, SlopeUnits::Percent, 0.0,
        0.0, FractionUnits::Percent,
        0.0, LengthUnits::Feet,
        0.0, FractionUnits::Fraction,
    );
    run.surface.set_user_provided_wind_adjustment_factor(1.0);
    run.surface.set_wind_adjustment_factor_calculation_method(
        WindAdjustmentFactorCalculationMethod::UserInput,
    );
    run.surface.set_wind_height_input_mode(WindHeightInputMode::TwentyFoot);
    run.surface.set_wind_and_spread_orientation_mode(WindAndSpreadOrientationMode::RelativeToNorth);
    run.surface.do_surface_run_in_direction_of_max_spread();

    t.check(
        "simple surface: rate of spread",
        round6(run.surface.spread_rate(SpeedUnits::ChainsPerHour)),
        34.011429, TOL,
    );
    t.check(
        "simple surface: flame length",
        round6(run.surface.flame_length_output(LengthUnits::Feet)),
        15.811421, TOL,
    );
    t.check(
        "simple surface: fireline intensity",
        round6(run.surface.fireline_intensity(FirelineIntensityUnits::BtusPerFootPerSecond)),
        2292.684759, TOL,
    );
}

// ---------------------------------------------------------------------------
// Driver: same order as testBehave.cpp main()
// ---------------------------------------------------------------------------

fn new_behave_run() -> BehaveRun {
    let mut run = BehaveRun::new(FuelModels::new(), SpeciesMasterTable::new());
    run.set_moisture_scenarios(MoistureScenarios::new());
    run
}

fn finish(t: TestInfo) {
    assert!(
        t.failures.is_empty(),
        "{} of {} parity checks failed:\n{}",
        t.failures.len(),
        t.failures.len() + t.passed,
        t.failures.join("\n"),
    );
}

#[test]
fn parity_with_cpp_test_behave() {
    let mut t = TestInfo::default();
    let mut run = new_behave_run();

    set_surface_inputs_for_gs4_low_moisture(&mut run);

    test_surface_single_fuel_model(&mut t, &mut run);
    test_chaparral(&mut t, &mut run);
    test_calculate_scorch_height(&mut t, &mut run);
    test_palmetto_gallberry(&mut t, &mut run);
    test_western_aspen(&mut t, &mut run);
    test_length_to_width_ratio(&mut t, &mut run);
    test_elliptical_dimensions(&mut t, &mut run);
    test_direction_of_interest(&mut t, &mut run);
    test_fireline_intensity(&mut t, &mut run);
    test_two_fuel_models(&mut t, &mut run);
    test_crown_module_rothermel(&mut t, &mut run);
    test_crown_module_scott_and_reinhardt(&mut t, &mut run);
    test_spot_module(&mut t, &mut run);
    test_speed_unit_conversion(&mut t, &mut run);
    test_ignite_module(&mut t, &mut run);
    test_safety_module(&mut t, &mut run);
    test_contain_module(&mut t, &mut run);
    // testMortalityModule is an empty stub in C++ — nothing to port.
    test_fine_dead_fuel_moisture_tool(&mut t, &mut run);
    test_slope_tool(&mut t, &mut run);
    test_vapor_pressure_deficit_calculator(&mut t, &mut run);
    test_simple_surface(&mut t, &mut run);

    finish(t);
}

