use drone_energy_planner::{
    ConditionRange, ConditionValue, ElectricalLoad, Factor, FlightPhase, MeasuredPower,
    PowerProfile, Precipitation, ProfileError, ProfiledSegment, estimate_profiled_mission,
};

const CONDITIONS: [ConditionValue; 2] = [
    ConditionValue {
        factor: Factor::AirDensityKgM3,
        value: 1.2,
    },
    ConditionValue {
        factor: Factor::CgForwardM,
        value: 0.0,
    },
];
const COVERAGE: [ConditionRange; 2] = [
    ConditionRange {
        factor: Factor::CgForwardM,
        min: -0.01,
        max: 0.01,
    },
    ConditionRange {
        factor: Factor::AirDensityKgM3,
        min: 1.17,
        max: 1.23,
    },
];

fn segment() -> ProfiledSegment<'static> {
    ProfiledSegment {
        phase: FlightPhase::Hover,
        duration_s: 360.0,
        configuration: "test aircraft",
        workload: "perception",
        precipitation: Precipitation::Dry,
        conditions: &CONDITIONS,
    }
}

fn profile() -> PowerProfile<'static> {
    PowerProfile {
        id: "hover pack meter",
        source: "synthetic test",
        phase: FlightPhase::Hover,
        configuration: "test aircraft",
        workload: "perception",
        precipitation: Precipitation::Dry,
        ranges: &COVERAGE,
        power: MeasuredPower::WholePack { watts: 220.0 },
    }
}

fn branches(compute: ElectricalLoad) -> MeasuredPower {
    MeasuredPower::Split {
        propulsion: ElectricalLoad::PackSide { watts: 200.0 },
        compute,
        auxiliary: ElectricalLoad::PackSide { watts: 10.0 },
    }
}

#[test]
fn selects_each_phase_and_reports_the_budget_and_evidence() {
    let climb = ProfiledSegment {
        phase: FlightPhase::Climb,
        duration_s: 72.0,
        ..segment()
    };
    let profiles = [
        profile(),
        PowerProfile {
            phase: FlightPhase::Climb,
            id: "climb pack meter",
            power: MeasuredPower::WholePack { watts: 350.0 },
            ..profile()
        },
    ];
    let mission = [climb, segment()];
    let estimate = estimate_profiled_mission(35.0, 10.0, &mission, &profiles).unwrap();
    assert_eq!(estimate.segments.len(), 2);
    let climb = &estimate.segments[0];
    assert_eq!(climb.phase, FlightPhase::Climb);
    assert_eq!(climb.profile_id, "climb pack meter");
    assert_eq!(climb.pack_power_w, 350.0);
    assert_eq!(climb.required_wh, 7.0);
    let hover = &estimate.segments[1];
    assert_eq!(hover.phase, FlightPhase::Hover);
    assert_eq!(hover.profile_id, "hover pack meter");
    assert_eq!(hover.pack_power_w, 220.0);
    assert_eq!(hover.required_wh, 22.0);
    assert_eq!(estimate.totals.required_wh, 29.0);
    assert_eq!(estimate.totals.projected_remaining_wh, 6.0);
    assert_eq!(estimate.totals.reserve_margin_wh, -4.0);
    assert!(!estimate.totals.meets_reserve);
}

#[test]
fn assigns_each_electrical_load_to_the_planned_pack_once() {
    let mission = [segment()];
    let evaluate = |power| {
        estimate_profiled_mission(40.0, 10.0, &mission, &[PowerProfile { power, ..profile() }])
            .unwrap()
            .totals
    };
    let total_pack = evaluate(MeasuredPower::WholePack { watts: 220.0 });
    let regulated = evaluate(branches(ElectricalLoad::Downstream {
        watts: 9.0,
        efficiency: 0.9,
    }));
    let metered_upstream = evaluate(branches(ElectricalLoad::PackSide { watts: 10.0 }));
    let separate_battery = evaluate(branches(ElectricalLoad::OtherBattery));
    assert_eq!(total_pack.required_wh, 22.0);
    assert_eq!(regulated, total_pack);
    assert_eq!(metered_upstream, total_pack);
    assert_eq!(total_pack.required_wh - separate_battery.required_wh, 1.0);
}

#[test]
fn accepts_either_range_endpoint_and_reports_shared_boundaries_as_ambiguous() {
    for (cg_m, density_kg_m3) in [(-0.01, 1.17), (0.01, 1.23)] {
        let conditions = [
            ConditionValue {
                factor: Factor::AirDensityKgM3,
                value: density_kg_m3,
            },
            ConditionValue {
                factor: Factor::CgForwardM,
                value: cg_m,
            },
        ];
        let mission = [ProfiledSegment {
            conditions: &conditions,
            ..segment()
        }];
        assert!(estimate_profiled_mission(40.0, 10.0, &mission, &[profile()]).is_ok());
    }
    let adjacent_ranges = [
        ConditionRange {
            factor: Factor::CgForwardM,
            min: 0.01,
            max: 0.02,
        },
        COVERAGE[1],
    ];
    let boundary = [
        CONDITIONS[0],
        ConditionValue {
            factor: Factor::CgForwardM,
            value: 0.01,
        },
    ];
    let second_segment = ProfiledSegment {
        conditions: &boundary,
        ..segment()
    };
    let profiles = [
        profile(),
        PowerProfile {
            id: "adjacent CG range",
            ranges: &adjacent_ranges,
            ..profile()
        },
    ];
    assert!(matches!(
        estimate_profiled_mission(40.0, 10.0, &[segment(), second_segment], &profiles),
        Err(ProfileError::AmbiguousProfile {
            segment_index: 1,
            ..
        })
    ));
}

#[test]
fn unsupported_inputs_identify_the_affected_segment() {
    let outside_cg = [
        CONDITIONS[0],
        ConditionValue {
            factor: Factor::CgForwardM,
            value: -0.02,
        },
    ];
    let extra_condition = [
        CONDITIONS[0],
        CONDITIONS[1],
        ConditionValue {
            factor: Factor::GustSpeedMS,
            value: 8.0,
        },
    ];
    let cases = [
        ProfiledSegment {
            configuration: "different airframe",
            ..segment()
        },
        ProfiledSegment {
            workload: "different processor workload",
            ..segment()
        },
        ProfiledSegment {
            phase: FlightPhase::Landing,
            ..segment()
        },
        ProfiledSegment {
            precipitation: Precipitation::Rain,
            ..segment()
        },
        ProfiledSegment {
            precipitation: Precipitation::Unreported,
            ..segment()
        },
        ProfiledSegment {
            conditions: &CONDITIONS[..1],
            ..segment()
        },
        ProfiledSegment {
            conditions: &outside_cg,
            ..segment()
        },
        ProfiledSegment {
            conditions: &extra_condition,
            ..segment()
        },
    ];
    for second in cases {
        assert!(matches!(
            estimate_profiled_mission(40.0, 10.0, &[segment(), second], &[profile()]),
            Err(ProfileError::UnsupportedConditions {
                segment_index: 1,
                ..
            })
        ));
    }
}

#[test]
fn rejects_invalid_coverage_values_and_power_measurements() {
    for efficiency in [-0.01, 0.0, 1.01, f64::NAN, f64::INFINITY] {
        let power = branches(ElectricalLoad::Downstream {
            watts: 9.0,
            efficiency,
        });
        assert!(matches!(
            estimate_profiled_mission(
                40.0,
                10.0,
                &[segment()],
                &[PowerProfile { power, ..profile() }]
            ),
            Err(ProfileError::InvalidProfile {
                profile_index: 0,
                ..
            })
        ));
    }
    for watts in [-1.0, f64::INFINITY, f64::NAN] {
        let power = MeasuredPower::WholePack { watts };
        assert!(matches!(
            estimate_profiled_mission(
                40.0,
                10.0,
                &[segment()],
                &[PowerProfile { power, ..profile() }]
            ),
            Err(ProfileError::InvalidProfile {
                profile_index: 0,
                ..
            })
        ));
    }
    for (min, max) in [(0.02, 0.01), (f64::NAN, 0.01)] {
        let invalid = [
            ConditionRange {
                factor: Factor::CgForwardM,
                min,
                max,
            },
            COVERAGE[1],
        ];
        assert!(matches!(
            estimate_profiled_mission(
                40.0,
                10.0,
                &[segment()],
                &[PowerProfile {
                    ranges: &invalid,
                    ..profile()
                }]
            ),
            Err(ProfileError::InvalidProfile {
                profile_index: 0,
                ..
            })
        ));
    }
    let invalid_values = [
        ConditionValue {
            factor: Factor::AirDensityKgM3,
            value: f64::NAN,
        },
        CONDITIONS[1],
    ];
    let second = ProfiledSegment {
        conditions: &invalid_values,
        ..segment()
    };
    assert!(matches!(
        estimate_profiled_mission(40.0, 10.0, &[segment(), second], &[profile()]),
        Err(ProfileError::InvalidConditions {
            segment_index: 1,
            ..
        })
    ));
}
