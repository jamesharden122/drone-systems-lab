//! Phase-specific estimates from observed mean electrical power.
use super::{
    EnergyEstimate, EstimateError, MissionSegment, estimate_mission_energy, valid_input,
    validate_budget,
};
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightPhase {
    Takeoff,
    Climb,
    Hover,
    Cruise,
    Descent,
    Landing,
}

/// Unreported matches only other explicitly unreported observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Precipitation {
    Dry,
    Rain,
    Snow,
    Freezing,
    Unreported,
}

/// Numeric axes of a calibration. CG is relative to the configured body origin
/// (forward, right, down). Vertical speed is positive upwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Factor {
    MassKg,
    CgForwardM,
    CgRightM,
    CgDownM,
    AirDensityKgM3,
    AmbientTemperatureC,
    PackTemperatureC,
    HeightAglM,
    VerticalSpeedMS,
    AirspeedMS,
    GustSpeedMS,
    PrecipitationRateMmH,
    MeanRotorRpm,
    RmsRotorAccelerationRadS2,
    RmsAttitudeErrorRad,
}

impl Factor {
    fn accepts(self, value: f64) -> bool {
        value.is_finite()
            && match self {
                Self::MassKg | Self::AirDensityKgM3 => value > 0.0,
                Self::AmbientTemperatureC | Self::PackTemperatureC => value > -273.15,
                Self::CgForwardM | Self::CgRightM | Self::CgDownM | Self::VerticalSpeedMS => true,
                _ => value >= 0.0,
            }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConditionValue {
    pub factor: Factor,
    pub value: f64,
}

/// Inclusive measured bounds in the units of the factor.
#[derive(Debug, Clone, Copy)]
pub struct ConditionRange {
    pub factor: Factor,
    pub min: f64,
    pub max: f64,
}

/// Meter location relative to the battery whose budget is being evaluated.
#[derive(Debug, Clone, Copy)]
pub enum ElectricalLoad {
    PackSide { watts: f64 },
    Downstream { watts: f64, efficiency: f64 },
    OtherBattery,
}

impl ElectricalLoad {
    fn pack_watts(self) -> Result<f64, &'static str> {
        match self {
            Self::OtherBattery => Ok(0.0),
            Self::PackSide { watts } if valid_input(watts) => Ok(watts),
            Self::Downstream { watts, efficiency } if valid_input(watts) => {
                if !(0.0 < efficiency && efficiency <= 1.0) {
                    return Err("converter efficiency must be in (0, 1]");
                }
                Ok(watts / efficiency)
            }
            _ => Err("load watts must be finite and nonnegative"),
        }
    }
}

/// Use one total pack-terminal measurement or three disjoint electrical branches.
#[derive(Debug, Clone, Copy)]
pub enum MeasuredPower {
    WholePack {
        watts: f64,
    },
    Split {
        propulsion: ElectricalLoad,
        compute: ElectricalLoad,
        auxiliary: ElectricalLoad,
    },
}

impl MeasuredPower {
    fn pack_watts(self) -> Result<f64, &'static str> {
        let watts = match self {
            Self::WholePack { watts } => watts,
            Self::Split {
                propulsion,
                compute,
                auxiliary,
            } => propulsion.pack_watts()? + compute.pack_watts()? + auxiliary.pack_watts()?,
        };
        if valid_input(watts) {
            Ok(watts)
        } else {
            Err("total pack watts must be finite and nonnegative")
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProfiledSegment<'a> {
    pub phase: FlightPhase,
    pub duration_s: f64,
    pub configuration: &'a str,
    pub workload: &'a str,
    pub precipitation: Precipitation,
    pub conditions: &'a [ConditionValue],
}

#[derive(Debug, Clone, Copy)]
pub struct PowerProfile<'a> {
    pub id: &'a str,
    pub source: &'a str,
    pub phase: FlightPhase,
    pub configuration: &'a str,
    pub workload: &'a str,
    pub precipitation: Precipitation,
    pub ranges: &'a [ConditionRange],
    pub power: MeasuredPower,
}

#[derive(Debug, PartialEq)]
pub struct SegmentEstimate<'a> {
    pub phase: FlightPhase,
    pub duration_s: f64,
    pub profile_id: &'a str,
    pub source: &'a str,
    pub pack_power_w: f64,
    pub required_wh: f64,
}

#[derive(Debug, PartialEq)]
pub struct ProfiledEstimate<'a> {
    pub totals: EnergyEstimate,
    pub segments: Vec<SegmentEstimate<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageGap {
    Configuration,
    Phase,
    Workload,
    Precipitation,
    MissingFactor(Factor),
    ExtraFactor(Factor),
    OutsideRange(Factor),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProfileError {
    InvalidProfile {
        profile_index: usize,
        reason: &'static str,
    },
    InvalidConditions {
        segment_index: usize,
        reason: &'static str,
    },
    UnsupportedConditions {
        segment_index: usize,
        gap: CoverageGap,
    },
    AmbiguousProfile {
        segment_index: usize,
    },
    Energy(EstimateError),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfile {
                profile_index,
                reason,
            } => write!(f, "profiles[{profile_index}]: {reason}"),
            Self::InvalidConditions {
                segment_index,
                reason,
            } => write!(f, "segments[{segment_index}]: {reason}"),
            Self::UnsupportedConditions { segment_index, gap } => write!(
                f,
                "segments[{segment_index}]: no matching profile ({gap:?})"
            ),
            Self::AmbiguousProfile { segment_index } => {
                write!(f, "segments[{segment_index}]: multiple matching profiles")
            }
            Self::Energy(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ProfileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Energy(error) => Some(error),
            _ => None,
        }
    }
}

impl From<EstimateError> for ProfileError {
    fn from(error: EstimateError) -> Self {
        Self::Energy(error)
    }
}

fn validate_profile(profile: &PowerProfile<'_>) -> Result<f64, &'static str> {
    if [
        profile.id,
        profile.source,
        profile.configuration,
        profile.workload,
    ]
    .iter()
    .any(|s| s.trim().is_empty())
    {
        return Err("id, source, configuration and workload must be nonempty");
    }
    for (i, range) in profile.ranges.iter().enumerate() {
        if !range.factor.accepts(range.min)
            || !range.factor.accepts(range.max)
            || range.min > range.max
        {
            return Err("invalid numeric calibration range");
        }
        if profile.ranges[..i].iter().any(|r| r.factor == range.factor) {
            return Err("duplicate numeric calibration factor");
        }
    }
    profile.power.pack_watts()
}

fn validate_segment(segment: &ProfiledSegment<'_>) -> Result<(), &'static str> {
    if !valid_input(segment.duration_s) {
        return Err("duration_s must be finite and nonnegative");
    }
    if segment.configuration.trim().is_empty() || segment.workload.trim().is_empty() {
        return Err("configuration and workload must be nonempty");
    }
    for (i, condition) in segment.conditions.iter().enumerate() {
        if !condition.factor.accepts(condition.value) {
            return Err("invalid numeric condition value");
        }
        if segment.conditions[..i]
            .iter()
            .any(|c| c.factor == condition.factor)
        {
            return Err("duplicate numeric condition factor");
        }
    }
    Ok(())
}

fn numeric_gap(segment: &ProfiledSegment<'_>, profile: &PowerProfile<'_>) -> Option<CoverageGap> {
    for range in profile.ranges {
        let Some(value) = segment.conditions.iter().find(|c| c.factor == range.factor) else {
            return Some(CoverageGap::MissingFactor(range.factor));
        };
        if !(range.min..=range.max).contains(&value.value) {
            return Some(CoverageGap::OutsideRange(range.factor));
        }
    }
    segment
        .conditions
        .iter()
        .find(|c| !profile.ranges.iter().any(|r| r.factor == c.factor))
        .map(|c| CoverageGap::ExtraFactor(c.factor))
}

fn select_profile(
    segment: &ProfiledSegment<'_>,
    segment_index: usize,
    profiles: &[PowerProfile<'_>],
) -> Result<usize, ProfileError> {
    let mut candidates = profiles
        .iter()
        .enumerate()
        .filter(|(_, p)| p.configuration == segment.configuration)
        .peekable();
    let unsupported = |gap| ProfileError::UnsupportedConditions { segment_index, gap };
    if candidates.peek().is_none() {
        return Err(unsupported(CoverageGap::Configuration));
    }
    let mut phase = candidates
        .filter(|(_, p)| p.phase == segment.phase)
        .peekable();
    if phase.peek().is_none() {
        return Err(unsupported(CoverageGap::Phase));
    }
    let mut workload = phase
        .filter(|(_, p)| p.workload == segment.workload)
        .peekable();
    if workload.peek().is_none() {
        return Err(unsupported(CoverageGap::Workload));
    }

    let mut found = None;
    let mut gap = CoverageGap::Precipitation;
    for (index, profile) in workload.filter(|(_, p)| p.precipitation == segment.precipitation) {
        if let Some(missing) = numeric_gap(segment, profile) {
            gap = missing;
        } else if found.replace(index).is_some() {
            return Err(ProfileError::AmbiguousProfile { segment_index });
        }
    }
    found.ok_or_else(|| unsupported(gap))
}

/// Select exactly one measured profile for each segment and evaluate the pack
/// budget. Profile IDs must be unique. Numeric axes must agree as sets, and
/// observations on a range endpoint are included. No interpolation is applied.
pub fn estimate_profiled_mission<'p>(
    remaining_usable_wh: f64,
    reserve_wh: f64,
    segments: &[ProfiledSegment<'_>],
    profiles: &[PowerProfile<'p>],
) -> Result<ProfiledEstimate<'p>, ProfileError> {
    validate_budget(remaining_usable_wh, reserve_wh)?;
    let mut ids = HashSet::new();
    let mut watts = Vec::with_capacity(profiles.len());
    for (profile_index, profile) in profiles.iter().enumerate() {
        if !ids.insert(profile.id) {
            return Err(ProfileError::InvalidProfile {
                profile_index,
                reason: "duplicate profile id",
            });
        }
        watts.push(
            validate_profile(profile).map_err(|reason| ProfileError::InvalidProfile {
                profile_index,
                reason,
            })?,
        );
    }
    let mut demands = Vec::with_capacity(segments.len());
    let mut estimates = Vec::with_capacity(segments.len());
    for (segment_index, segment) in segments.iter().enumerate() {
        validate_segment(segment).map_err(|reason| ProfileError::InvalidConditions {
            segment_index,
            reason,
        })?;
        let i = select_profile(segment, segment_index, profiles)?;
        let power_segment = MissionSegment {
            duration_s: segment.duration_s,
            pack_power_w: watts[i],
        };
        estimates.push(SegmentEstimate {
            phase: segment.phase,
            duration_s: segment.duration_s,
            profile_id: profiles[i].id,
            source: profiles[i].source,
            pack_power_w: watts[i],
            required_wh: power_segment.demand_wh(segment_index)?,
        });
        demands.push(power_segment);
    }
    let totals = estimate_mission_energy(remaining_usable_wh, reserve_wh, &demands)?;
    Ok(ProfiledEstimate {
        totals,
        segments: estimates,
    })
}

/// Dry-air density using absolute local pressure in Pa and ambient temperature in °C.
/// Returns None for nonphysical inputs or an unrepresentable result.
pub fn dry_air_density_kg_m3(pressure_pa: f64, temperature_c: f64) -> Option<f64> {
    if !pressure_pa.is_finite()
        || pressure_pa <= 0.0
        || !Factor::AmbientTemperatureC.accepts(temperature_c)
    {
        return None;
    }
    let density = (pressure_pa / 287.05) / (temperature_c + 273.15);
    (density.is_finite() && density > 0.0).then_some(density)
}

/// Ground and wind-to vectors in the same north/east frame, both in m/s.
pub fn horizontal_airspeed_m_s(ground_ne_m_s: [f64; 2], wind_to_ne_m_s: [f64; 2]) -> Option<f64> {
    let speed = (ground_ne_m_s[0] - wind_to_ne_m_s[0]).hypot(ground_ne_m_s[1] - wind_to_ne_m_s[1]);
    speed.is_finite().then_some(speed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weather_units_and_direction() {
        assert!((dry_air_density_kg_m3(101325.0, 15.0).unwrap() - 1.225).abs() < 0.001);
        assert_eq!(dry_air_density_kg_m3(101325.0, -273.15), None);
        assert_eq!(
            horizontal_airspeed_m_s([4.0, 3.0], [-4.0, -3.0]),
            Some(10.0)
        );
        assert_eq!(horizontal_airspeed_m_s([f64::NAN, 0.0], [0.0; 2]), None);
    }
}
