use drone_energy_planner::{
    EnergyEstimate, EstimateError, MissionSegment, SegmentField, estimate_mission_energy,
};

const fn segment(duration_s: f64, pack_power_w: f64) -> MissionSegment {
    MissionSegment {
        duration_s,
        pack_power_w,
    }
}

// 180 W for 10 min uses 30 Wh. Another 300 W for 2 min uses 10 Wh.
const MISSION: [MissionSegment; 2] = [segment(600.0, 180.0), segment(120.0, 300.0)];

#[test]
fn compares_mission_demand_to_both_budgets() {
    for (available, remaining, margin, meets_reserve) in [
        (65.0, 25.0, 5.0, true),
        (60.0, 20.0, 0.0, true),
        (55.0, 15.0, -5.0, false),
        (30.0, -10.0, -30.0, false),
    ] {
        assert_eq!(
            estimate_mission_energy(available, 20.0, &MISSION),
            Ok(EnergyEstimate {
                required_wh: 40.0,
                projected_remaining_wh: remaining,
                reserve_margin_wh: margin,
                meets_reserve,
            })
        );
    }
}

#[test]
fn zero_demand_preserves_the_budget_even_below_reserve() {
    let idle = [segment(0.0, 300.0), segment(90.0, 0.0)];
    for mission in [&[][..], &idle[..]] {
        for (available, margin, meets_reserve) in [(10.0, 5.0, true), (1.0, -4.0, false)] {
            assert_eq!(
                estimate_mission_energy(available, 5.0, mission),
                Ok(EnergyEstimate {
                    required_wh: 0.0,
                    projected_remaining_wh: available,
                    reserve_margin_wh: margin,
                    meets_reserve,
                })
            );
        }
    }
}

#[test]
fn rejects_invalid_budgets_and_identifies_the_bad_segment_field() {
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            estimate_mission_energy(invalid, 0.0, &[]),
            Err(EstimateError::InvalidRemainingUsableWh)
        );
        assert_eq!(
            estimate_mission_energy(0.0, invalid, &[]),
            Err(EstimateError::InvalidReserveWh)
        );
        for (bad_segment, field) in [
            (segment(invalid, 0.0), SegmentField::DurationS),
            (segment(0.0, invalid), SegmentField::PackPowerW),
        ] {
            assert_eq!(
                estimate_mission_energy(10.0, 0.0, &[MISSION[0], bad_segment]),
                Err(EstimateError::InvalidSegment { index: 1, field })
            );
        }
    }
}

#[test]
fn reports_overflow_in_segment_sum_or_margin() {
    let huge = segment(3600.0, f64::MAX);
    for (reserve, segments) in [
        (0.0, &[segment(7200.0, f64::MAX)][..]),
        (0.0, &[huge, huge][..]),
        (f64::MAX, &[huge][..]),
    ] {
        assert_eq!(
            estimate_mission_energy(0.0, reserve, segments),
            Err(EstimateError::NonFiniteCalculation)
        );
    }
}

#[test]
fn retains_representable_demand_at_extreme_scales() {
    let estimate = estimate_mission_energy(f64::MAX, 0.0, &[segment(3600.0, f64::MAX)]).unwrap();
    assert_eq!(estimate.required_wh, f64::MAX);
    assert!(estimate.meets_reserve);

    // Scaling the smallest positive duration to hours first would erase this demand.
    let duration_s = f64::from_bits(1);
    let estimate = estimate_mission_energy(0.0, 0.0, &[segment(duration_s, f64::MAX)]).unwrap();
    assert!(estimate.required_wh > 0.0);
    assert!(!estimate.meets_reserve);
}
