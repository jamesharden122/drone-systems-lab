//! Estimate the energy needed by a candidate mission using a single battery budget.

use std::fmt;

mod profile;
pub use profile::*;

/// One interval of the mission, with average discharge power at the battery terminals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MissionSegment {
    pub duration_s: f64,
    pub pack_power_w: f64,
}

/// The demand and resulting battery budget, in watt-hours.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyEstimate {
    pub required_wh: f64,
    /// Available energy after the mission. A negative value measures unmet demand.
    pub projected_remaining_wh: f64,
    /// Projected remaining energy minus the configured reserve.
    pub reserve_margin_wh: f64,
    /// True when the unrounded reserve margin is zero or positive.
    pub meets_reserve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentField {
    DurationS,
    PackPowerW,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstimateError {
    InvalidRemainingUsableWh,
    InvalidReserveWh,
    /// The named field of the zero-based segment is negative or non-finite.
    InvalidSegment {
        index: usize,
        field: SegmentField,
    },
    NonFiniteCalculation,
}

impl fmt::Display for EstimateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRemainingUsableWh => {
                write!(f, "remaining_usable_wh must be finite and nonnegative")
            }
            Self::InvalidReserveWh => write!(f, "reserve_wh must be finite and nonnegative"),
            Self::InvalidSegment { index, field } => {
                let name = match field {
                    SegmentField::DurationS => "duration_s",
                    SegmentField::PackPowerW => "pack_power_w",
                };
                write!(f, "segments[{index}].{name} must be finite and nonnegative")
            }
            Self::NonFiniteCalculation => {
                write!(f, "mission energy calculation produced a non-finite result")
            }
        }
    }
}

impl std::error::Error for EstimateError {}

/// Evaluate one mission against the usable energy available **before** deducting the reserve.
///
/// Pack power is measured or estimated at the battery terminals, so no efficiency
/// correction is applied. An empty mission consumes zero watt-hours.
///
/// # Errors
///
/// Returns an [`EstimateError`] for a negative or non-finite input or a non-finite
/// arithmetic result. A reserve shortfall is a successful estimate with a negative margin.
///
/// # Examples
///
/// ```
/// use drone_energy_planner::{estimate_mission_energy, MissionSegment};
///
/// let mission = [
///     MissionSegment { duration_s: 600.0, pack_power_w: 180.0 },
///     MissionSegment { duration_s: 120.0, pack_power_w: 300.0 },
/// ];
/// let estimate = estimate_mission_energy(65.0, 20.0, &mission)?;
/// assert_eq!(estimate.required_wh, 40.0);
/// assert_eq!(estimate.projected_remaining_wh, 25.0);
/// assert_eq!(estimate.reserve_margin_wh, 5.0);
/// assert!(estimate.meets_reserve);
/// # Ok::<(), drone_energy_planner::EstimateError>(())
/// ```
pub fn estimate_mission_energy(
    remaining_usable_wh: f64,
    reserve_wh: f64,
    segments: &[MissionSegment],
) -> Result<EnergyEstimate, EstimateError> {
    validate_budget(remaining_usable_wh, reserve_wh)?;

    let mut required_wh = 0.0;
    for (index, segment) in segments.iter().enumerate() {
        required_wh += segment.demand_wh(index)?;
        if !required_wh.is_finite() {
            return Err(EstimateError::NonFiniteCalculation);
        }
    }

    let projected_remaining_wh = remaining_usable_wh - required_wh;
    let reserve_margin_wh = projected_remaining_wh - reserve_wh;
    if !projected_remaining_wh.is_finite() || !reserve_margin_wh.is_finite() {
        return Err(EstimateError::NonFiniteCalculation);
    }
    Ok(EnergyEstimate {
        required_wh,
        projected_remaining_wh,
        reserve_margin_wh,
        meets_reserve: reserve_margin_wh >= 0.0,
    })
}

fn valid_input(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn validate_budget(remaining_usable_wh: f64, reserve_wh: f64) -> Result<(), EstimateError> {
    if !valid_input(remaining_usable_wh) {
        return Err(EstimateError::InvalidRemainingUsableWh);
    }
    if !valid_input(reserve_wh) {
        return Err(EstimateError::InvalidReserveWh);
    }
    Ok(())
}

impl MissionSegment {
    fn demand_wh(self, index: usize) -> Result<f64, EstimateError> {
        let Self {
            duration_s,
            pack_power_w,
        } = self;
        if !valid_input(duration_s) {
            return Err(EstimateError::InvalidSegment {
                index,
                field: SegmentField::DurationS,
            });
        }
        if !valid_input(pack_power_w) {
            return Err(EstimateError::InvalidSegment {
                index,
                field: SegmentField::PackPowerW,
            });
        }
        // Scale long durations first to avoid watt-second overflow. Multiplying
        // first for subsecond intervals preserves very small positive durations.
        let wh = if duration_s >= 1.0 {
            pack_power_w * (duration_s / 3600.0)
        } else {
            (pack_power_w * duration_s) / 3600.0
        };
        if wh.is_finite() {
            Ok(wh)
        } else {
            Err(EstimateError::NonFiniteCalculation)
        }
    }
}
