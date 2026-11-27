pub mod micro_ev_chargers;
pub mod micro_operator;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize};
#[cfg(feature = "server")]
use std::error::Error;
#[cfg(feature = "server")]
use surrealdb::{engine::remote::ws::Client, Surreal};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatData {
    pub chargers_count: usize,
    pub energy_system_size: f64,
    pub battery_size: f64,
    pub duration_energy_needed: f32,
    pub total_energy_usage: f32,
    pub total_btm_generation: f32,
    pub max_storage: f32,
    pub average_usage: f64,
    pub days: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GlobalHorizIrr {
    #[serde(deserialize_with = "deserialize_datetime")]
    index: DateTime<FixedOffset>,
    #[serde(deserialize_with = "deserialize_datetime")]
    date: DateTime<FixedOffset>,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    kt: f64,
    fd: f64,
    g0: f64,
    d0: f64,
    b0: f64,
    b_inc: f64,
    d_inc: f64,
    r_inc: f64,
    g_inc: f64,
}

fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let date_fmt = "%Y-%m-%d %H:%M:%S%:z"; // Ensure this format matches your input data
    DateTime::parse_from_str(&s, date_fmt).map_err(serde::de::Error::custom)
}

#[cfg(feature = "server")]
#[derive(Clone, Debug)]
pub struct SolarParams {
    pub irradiance: polars::prelude::DataFrame, // in W/m^2
                                                // Additional parameters as needed
}

#[cfg(feature = "server")]
pub async fn setup_surrealdb(db: &Surreal<Client>) -> Result<(), Box<dyn Error>> {
    // Create namespace and database
    let surr_sql = r#"
        DEFINE NAMESPACE microgrid;
        DEFINE DATABASE batteries;
        DEFINE DATABASE simulations;
        DEFINE TABLE if not EXISTS ghi;
        DEFINE TABLE if not EXISTS sufficiency;
        DEFINE TABLE if not EXISTS ev_pv_ldes;
        DEFINE TABLE if not EXISTS pv_ldes_operator;
        "#;

    db.query(surr_sql).await?;

    Ok(())
}
