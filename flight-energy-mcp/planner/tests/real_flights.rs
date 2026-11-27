use drone_energy_planner::{
    FlightPhase, MeasuredPower, PowerProfile, Precipitation, ProfiledSegment,
    estimate_profiled_mission,
};

const CONFIGURATION: &str = "CMU Matrice 100 / R1 / payload 500 g / command 10 m/s at 25 m";
const WORKLOAD: &str = "2019 CMU instrumentation as recorded";
const PHASES: [FlightPhase; 3] = [
    FlightPhase::Takeoff,
    FlightPhase::Cruise,
    FlightPhase::Landing,
];
const PHASE_IDS: [&str; 3] = ["cmu-takeoff", "cmu-cruise", "cmu-landing"];

struct Observation<'a> {
    flight: u16,
    split: &'a str,
    phase: FlightPhase,
    duration_s: f64,
    observed_wh: f64,
    predicted_wh: Option<f64>,
    signed_error_wh: Option<f64>,
}

fn observations() -> Vec<Observation<'static>> {
    include_str!("../benchmark/matrice100-phases.csv")
        .lines()
        .skip(1)
        .map(|line| {
            let columns: [&str; 8] = line.split(',').collect::<Vec<_>>().try_into().unwrap();
            let [flight, split, phase, start, end, observed, predicted, error] = columns;
            let phase = match phase {
                "takeoff" => FlightPhase::Takeoff,
                "cruise" => FlightPhase::Cruise,
                "landing" => FlightPhase::Landing,
                _ => panic!("unexpected phase in benchmark: {phase}"),
            };
            Observation {
                flight: flight.parse().unwrap(),
                split,
                phase,
                duration_s: end.parse::<f64>().unwrap() - start.parse::<f64>().unwrap(),
                observed_wh: observed.parse().unwrap(),
                predicted_wh: (!predicted.is_empty()).then(|| predicted.parse().unwrap()),
                signed_error_wh: (!error.is_empty()).then(|| error.parse().unwrap()),
            }
        })
        .collect()
}

fn assert_wh_close(actual: f64, expected: f64) {
    // The reduced CSV reports Wh to nine decimal places.
    assert!(
        (actual - expected).abs() < 1e-8,
        "actual {actual}, expected {expected}"
    );
}

#[test]
fn phase_profiles_predict_two_complete_held_out_matrice_flights() {
    let data = observations();
    assert_eq!(data.len(), 15);
    let profiles = PHASES
        .iter()
        .zip(PHASE_IDS)
        .map(|(&phase, id)| {
            let fit: Vec<_> = data
                .iter()
                .filter(|r| r.split == "calibration" && r.phase == phase)
                .collect();
            assert_eq!(
                fit.iter().map(|r| r.flight).collect::<Vec<_>>(),
                [167, 178, 253]
            );
            let measured_wh: f64 = fit.iter().map(|r| r.observed_wh).sum();
            let duration_s: f64 = fit.iter().map(|r| r.duration_s).sum();
            PowerProfile {
                id,
                source: "CMU DOI 10.1184/R1/12683453.v1 (flights 167, 178, 253)",
                phase,
                configuration: CONFIGURATION,
                workload: WORKLOAD,
                precipitation: Precipitation::Unreported,
                ranges: &[],
                power: MeasuredPower::WholePack {
                    watts: measured_wh * 3600.0 / duration_s,
                },
            }
        })
        .collect::<Vec<_>>();

    for (flight, expected_mission_error_wh) in [(276, 0.782855564), (277, 0.597759456)] {
        let holdout: Vec<_> = data
            .iter()
            .filter(|r| r.split == "held_out" && r.flight == flight)
            .collect();
        assert_eq!(holdout.iter().map(|r| r.phase).collect::<Vec<_>>(), PHASES);
        let segments: Vec<_> = holdout
            .iter()
            .map(|r| ProfiledSegment {
                phase: r.phase,
                duration_s: r.duration_s,
                configuration: CONFIGURATION,
                workload: WORKLOAD,
                precipitation: Precipitation::Unreported,
                conditions: &[],
            })
            .collect();

        // Comparison concerns demand only. Usable capacity and reserve were not measured.
        let result = estimate_profiled_mission(0.0, 0.0, &segments, &profiles).unwrap();
        assert_eq!(result.segments.len(), holdout.len());
        for (actual, reference) in result.segments.iter().zip(&holdout) {
            assert_wh_close(actual.required_wh, reference.predicted_wh.unwrap());
            assert_wh_close(
                actual.required_wh - reference.observed_wh,
                reference.signed_error_wh.unwrap(),
            );
        }
        let measured_wh: f64 = holdout.iter().map(|r| r.observed_wh).sum();
        assert_wh_close(
            result.totals.required_wh - measured_wh,
            expected_mission_error_wh,
        );
    }
}
