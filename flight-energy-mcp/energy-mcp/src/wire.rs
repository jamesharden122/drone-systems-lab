//! JSON types live at the MCP boundary. Remote derives describe the planner's
//! enums to Serde without making the numerical library depend on serialization.
use drone_energy_planner::{
    ConditionRange, ConditionValue, ElectricalLoad, EnergyEstimate, Factor, FlightPhase,
    MeasuredPower, PowerProfile, Precipitation, ProfileError, ProfiledSegment,
    estimate_profiled_mission,
};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(remote = "FlightPhase", rename_all = "snake_case")]
enum Phase {
    Takeoff,
    Climb,
    Hover,
    Cruise,
    Descent,
    Landing,
}

#[derive(Deserialize, JsonSchema)]
#[serde(remote = "Precipitation", rename_all = "snake_case")]
enum Weather {
    Dry,
    Rain,
    Snow,
    Freezing,
    Unreported,
}

#[derive(Deserialize, JsonSchema)]
#[serde(remote = "Factor", rename_all = "snake_case")]
enum Axis {
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

#[derive(Deserialize, JsonSchema)]
#[serde(
    remote = "ElectricalLoad",
    tag = "kind",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum Load {
    PackSide { watts: f64 },
    Downstream { watts: f64, efficiency: f64 },
    OtherBattery {},
}

#[derive(Deserialize, JsonSchema)]
#[serde(
    remote = "MeasuredPower",
    tag = "kind",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum Power {
    WholePack {
        watts: f64,
    },
    Split {
        #[serde(with = "Load")]
        propulsion: ElectricalLoad,
        #[serde(with = "Load")]
        compute: ElectricalLoad,
        #[serde(with = "Load")]
        auxiliary: ElectricalLoad,
    },
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Observation {
    #[serde(with = "Axis")]
    factor: Factor,
    value: f64,
}

impl From<Observation> for ConditionValue {
    fn from(row: Observation) -> Self {
        Self {
            factor: row.factor,
            value: row.value,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Range {
    #[serde(with = "Axis")]
    factor: Factor,
    min: f64,
    max: f64,
}

impl From<Range> for ConditionRange {
    fn from(row: Range) -> Self {
        Self {
            factor: row.factor,
            min: row.min,
            max: row.max,
        }
    }
}

// Store the owned arrays in planner format so the estimate can borrow them.
fn read_rows<'de, D, Wire, Row>(deserializer: D) -> Result<Vec<Row>, D::Error>
where
    D: Deserializer<'de>,
    Wire: Deserialize<'de>,
    Row: From<Wire>,
{
    Ok(Vec::<Wire>::deserialize(deserializer)?
        .into_iter()
        .map(Row::from)
        .collect())
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Segment {
    #[serde(with = "Phase")]
    phase: FlightPhase,
    duration_s: f64,
    configuration: String,
    workload: String,
    #[serde(with = "Weather")]
    precipitation: Precipitation,
    #[serde(deserialize_with = "read_rows::<_, Observation, _>")]
    #[schemars(with = "Vec<Observation>")]
    conditions: Vec<ConditionValue>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Profile {
    id: String,
    /// Evidence for the mean watts (measurement run, calibration table or synthetic fixture).
    source: String,
    #[serde(with = "Phase")]
    phase: FlightPhase,
    configuration: String,
    workload: String,
    #[serde(with = "Weather")]
    precipitation: Precipitation,
    #[serde(deserialize_with = "read_rows::<_, Range, _>")]
    #[schemars(with = "Vec<Range>")]
    ranges: Vec<ConditionRange>,
    #[serde(with = "Power")]
    power: MeasuredPower,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EstimateRequest {
    /// Usable battery energy in Wh before deducting the reserve.
    remaining_usable_wh: f64,
    reserve_wh: f64,
    segments: Vec<Segment>,
    profiles: Vec<Profile>,
}

#[derive(Serialize, JsonSchema)]
#[serde(remote = "EnergyEstimate")]
struct Totals {
    required_wh: f64,
    projected_remaining_wh: f64,
    reserve_margin_wh: f64,
    meets_reserve: bool,
}

#[derive(Serialize, JsonSchema)]
struct Evidence<'a> {
    #[serde(with = "Phase")]
    phase: FlightPhase,
    duration_s: f64,
    profile_id: &'a str,
    source: &'a str,
    pack_power_w: f64,
    required_wh: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct EstimateResponse<'a> {
    #[serde(with = "Totals")]
    totals: EnergyEstimate,
    segments: Vec<Evidence<'a>>,
}

impl EstimateRequest {
    pub fn evaluate(&self) -> Result<EstimateResponse<'_>, ProfileError> {
        let segments = self
            .segments
            .iter()
            .map(|s| ProfiledSegment {
                phase: s.phase,
                duration_s: s.duration_s,
                configuration: &s.configuration,
                workload: &s.workload,
                precipitation: s.precipitation,
                conditions: &s.conditions,
            })
            .collect::<Vec<_>>();
        let profiles = self
            .profiles
            .iter()
            .map(|p| PowerProfile {
                id: &p.id,
                source: &p.source,
                phase: p.phase,
                configuration: &p.configuration,
                workload: &p.workload,
                precipitation: p.precipitation,
                ranges: &p.ranges,
                power: p.power,
            })
            .collect::<Vec<_>>();
        let estimate = estimate_profiled_mission(
            self.remaining_usable_wh,
            self.reserve_wh,
            &segments,
            &profiles,
        )?;
        let evidence = estimate
            .segments
            .into_iter()
            .map(|s| Evidence {
                phase: s.phase,
                duration_s: s.duration_s,
                profile_id: s.profile_id,
                source: s.source,
                pack_power_w: s.pack_power_w,
                required_wh: s.required_wh,
            })
            .collect();
        Ok(EstimateResponse {
            totals: estimate.totals,
            segments: evidence,
        })
    }
}
