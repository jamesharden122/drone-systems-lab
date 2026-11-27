//! Worked example with invented phase measurements for an illustrative airframe.
//! Run with `cargo run --locked --offline --example six_phase`.

use drone_energy_planner::{
    ConditionRange, ConditionValue, ElectricalLoad, Factor, FlightPhase, MeasuredPower,
    PowerProfile, Precipitation, ProfiledSegment, estimate_profiled_mission,
};

const AIRFRAME: &str = "synthetic-quad-A";
const WORKLOAD: &str = "camera logging";
const NOMINAL_DENSITY: [ConditionValue; 1] = [ConditionValue {
    factor: Factor::AirDensityKgM3,
    value: 1.20,
}];
const NOMINAL_RANGE: [ConditionRange; 1] = [ConditionRange {
    factor: Factor::AirDensityKgM3,
    min: 1.18,
    max: 1.22,
}];
const THIN_RANGE: [ConditionRange; 1] = [ConditionRange {
    factor: Factor::AirDensityKgM3,
    min: 1.02,
    max: 1.06,
}];

// Phase, profile ID, planned seconds, synthetic mean pack watts.
const FLIGHT: [(FlightPhase, &str, f64, f64); 6] = [
    (FlightPhase::Takeoff, "takeoff", 12.0, 450.0),
    (FlightPhase::Climb, "climb", 48.0, 375.0),
    (FlightPhase::Hover, "hover", 360.0, 220.0),
    (FlightPhase::Cruise, "cruise", 180.0, 280.0),
    (FlightPhase::Descent, "descent", 48.0, 225.0),
    (FlightPhase::Landing, "landing", 12.0, 150.0),
];

fn segment(
    phase: FlightPhase,
    duration_s: f64,
    conditions: &[ConditionValue],
) -> ProfiledSegment<'_> {
    ProfiledSegment {
        phase,
        duration_s,
        configuration: AIRFRAME,
        workload: WORKLOAD,
        precipitation: Precipitation::Dry,
        conditions,
    }
}

fn profile(
    phase: FlightPhase,
    id: &'static str,
    ranges: &'static [ConditionRange],
    power: MeasuredPower,
) -> PowerProfile<'static> {
    PowerProfile {
        id,
        source: "illustrative measurements for six_phase",
        phase,
        configuration: AIRFRAME,
        workload: WORKLOAD,
        precipitation: Precipitation::Dry,
        ranges,
        power,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mission = FLIGHT.map(|(phase, _, seconds, _)| segment(phase, seconds, &NOMINAL_DENSITY));
    let mut profiles = Vec::from(FLIGHT.map(|(phase, id, _, watts)| {
        profile(
            phase,
            id,
            &NOMINAL_RANGE,
            MeasuredPower::WholePack { watts },
        )
    }));
    profiles.push(profile(
        FlightPhase::Hover,
        "hover-thinner-air",
        &THIN_RANGE,
        MeasuredPower::WholePack { watts: 230.0 },
    ));

    let usable_wh = 65.0;
    let reserve_wh = 15.0;
    let estimate = estimate_profiled_mission(usable_wh, reserve_wh, &mission, &profiles)?;
    println!("Synthetic six-phase fixture");
    println!(
        "{:<18} {:>12} {:>13}",
        "Profile", "Mean pack W", "Demand Wh"
    );
    for result in &estimate.segments {
        println!(
            "{:<18} {:>12.1} {:>13.2}",
            result.profile_id, result.pack_power_w, result.required_wh
        );
    }
    println!(
        "Total {:.2} Wh, reserve margin {:+.2} Wh",
        estimate.totals.required_wh, estimate.totals.reserve_margin_wh
    );
    assert_eq!(estimate.totals.required_wh, 46.0);

    // Change only density during the six-minute hover. Each operating point
    // has its own synthetic profile with the same configuration and workload.
    let thin_air = [ConditionValue {
        factor: Factor::AirDensityKgM3,
        value: 1.04,
    }];
    let mut comparison = mission;
    for leg in &mut comparison {
        if leg.phase == FlightPhase::Hover {
            leg.conditions = &thin_air;
        }
    }
    let thinner = estimate_profiled_mission(usable_wh, reserve_wh, &comparison, &profiles)?;
    let difference_wh = thinner.totals.required_wh - estimate.totals.required_wh;
    println!(
        "Lower-density hover changes mission demand by {difference_wh:+.2} Wh in this fixture."
    );
    assert_eq!(difference_wh, 1.0);

    // Reconstruct the nominal 220 W hover from three disjoint measurements.
    // 9 W downstream / 0.90 converter efficiency = 10 W at the battery.
    let split_hover = profile(
        FlightPhase::Hover,
        "hover-branch-meters",
        &NOMINAL_RANGE,
        MeasuredPower::Split {
            propulsion: ElectricalLoad::PackSide { watts: 200.0 },
            compute: ElectricalLoad::Downstream {
                watts: 9.0,
                efficiency: 0.90,
            },
            auxiliary: ElectricalLoad::PackSide { watts: 10.0 },
        },
    );
    let hover = [segment(FlightPhase::Hover, 360.0, &NOMINAL_DENSITY)];
    let branch_profiles = [split_hover];
    let branch_estimate =
        estimate_profiled_mission(usable_wh, reserve_wh, &hover, &branch_profiles)?;
    let pack_estimate = estimate_profiled_mission(usable_wh, reserve_wh, &hover, &profiles)?;
    assert_eq!(branch_estimate.totals, pack_estimate.totals);
    println!(
        "Both meter layouts report {:.2} Wh for the nominal hover, including 1.00 Wh for compute.",
        branch_estimate.totals.required_wh
    );
    Ok(())
}

#[test]
fn synthetic_mission_and_matched_comparisons() {
    main().unwrap();
}
