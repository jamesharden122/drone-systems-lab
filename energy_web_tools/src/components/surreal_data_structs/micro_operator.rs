use super::StatData;
use crate::components::energy_components::batteries::BatteryStorage;
use crate::components::output_data_structs::RoundToPlaces;
#[cfg(feature = "server")]
use chrono::NaiveDateTime;
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use chrono::{DateTime, FixedOffset};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use nalgebra::base::DMatrix;
use serde::{Deserialize, Deserializer, Serialize};
#[cfg(feature = "server")]
use std::{collections::HashMap, error::Error};
#[cfg(feature = "server")]
use surrealdb::{
    engine::remote::ws::Client,
    engine::{any::Any, local::Db},
    sql::Datetime,
    Surreal,
};

#[cfg(feature = "server")]
use tokio::task;
#[cfg(feature = "server")]
use uuid;

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PvLdesStats {
    pub total_revenue: f32,
    pub total_expenses: f32,
    pub total_profit: f32,
    pub total_input_btm_power: f32,
    pub total_output_btm_power: f32,
    pub total_input_ftm_power: f32,
    pub total_output_ftm_power: f32,
    pub average_ppkwh: f32,
    pub average_discharge_rate: f32,
}

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PvLdesOperator {
    pub date_time: Datetime,
    pub set: String,
    pub storage: f32,
    pub input_btm_power: f32,
    pub output_btm_power: f32,
    pub input_ftm_power: f32,
    pub output_ftm_power: f32,
    pub sale_rate: f32,
    pub ppkwh: f32,
    pub discharge_ratio: f32,
    pub revenue: f32,
    pub expense: f32,
}

#[cfg(feature = "server")]
pub async fn db_pvldesops_update_stats(
    db: &Surreal<Db>,
    data: Vec<PvLdesOperator>,
    site: &str,
) -> Result<(), ServerFnError<surrealdb::Error>> {
    // Write stat_data to SurrealDB
    for record in data {
        let _temp: Option<StatData> = db
            .create((
                "sufficiency",
                format!("{}_{}", site, uuid::Uuid::new_v4().to_string()),
            ))
            .content(record)
            .await?;
    }
    Ok(())
}

#[cfg(feature = "server")]
pub async fn db_pvldesops_update_ts(
    db: &Surreal<Db>,
    data: Vec<PvLdesOperator>,
    site: &str,
) -> Result<(), ServerFnError<surrealdb::Error>> {
    let mut handles = vec![];
    // Spawn each task concurrently using Tokio
    for record in data {
        let db = db.clone(); // Clone the db reference for each spawned task
        let site = site.to_string(); // Clone site string for use in the async block
        let handle = task::spawn(async move {
            let result: std::result::Result<std::option::Option<PvLdesOperator>, surrealdb::Error> =
                db.create((
                    "pv_ldes_operator",
                    format!("{}_{}", site, uuid::Uuid::new_v4()),
                ))
                .content(record)
                .await;

            match result {
                Ok(_temp) => (), //println!("{:?}", temp),
                Err(err) => eprintln!("Error creating record: {:?}", err),
            }
        });
        handles.push(handle);
    }

    // Await all tasks to complete
    for handle in handles {
        handle.await.unwrap(); // Double `?` to handle both join error and Result error
    }

    Ok(())
}

#[cfg(feature = "server")]
pub async fn battery_pvldesops_storage_to_db(
    dates_to_index: HashMap<DateTime<Utc>, (usize, usize)>,
    batt_system: &mut BatteryStorage,
    ftm_rate: DMatrix<f64>,
    btm_rate: DMatrix<f64>,
    ftm_ppw: DMatrix<f64>,
    btm_ppw: DMatrix<f64>,
    db: &Surreal<Db>,
    site: &str,
) -> Result<Vec<PvLdesOperator>, ServerFnError<surrealdb::Error>> {
    use std::collections::BTreeMap;

    let storage_iter = batt_system
        .clone()
        .battery_state
        .storage
        .unwrap_or_default()
        .into_iter();
    let input_btm_iter = batt_system
        .clone()
        .battery_state
        .input_power_btm_ts
        .unwrap_or_default()
        .into_iter();
    let output_btm_iter = batt_system
        .clone()
        .battery_state
        .output_power_btm_ts
        .unwrap_or_default()
        .into_iter();
    let input_ftm_iter = batt_system
        .clone()
        .battery_state
        .input_power_ftm_ts
        .unwrap_or_default()
        .into_iter();
    let output_ftm_iter = batt_system
        .clone()
        .battery_state
        .output_power_ftm_ts
        .unwrap_or_default()
        .into_iter();
    let neg_stat_iter = batt_system
        .clone()
        .battery_state
        .neg_stat_ts
        .unwrap_or_default()
        .into_iter();

    let storage_map: BTreeMap<_, _> = storage_iter.collect();
    let input_btm_map: BTreeMap<_, _> = input_btm_iter.collect();
    let input_ftm_map: BTreeMap<_, _> = input_ftm_iter.collect();
    let output_btm_map: BTreeMap<_, _> = output_btm_iter.collect();
    let output_ftm_map: BTreeMap<_, _> = output_ftm_iter.collect();

    let mut all_dates = storage_map.keys().cloned().collect::<Vec<_>>();
    all_dates.sort();

    let data: Vec<PvLdesOperator> = all_dates
        .iter()
        .filter_map(|dt| {
            let s_val = storage_map.get(dt)?;
            let i_btm = *input_btm_map.get(dt).unwrap_or(&0.0);
            let o_btm = *output_btm_map.get(dt).unwrap_or(&0.0);
            let i_ftm = *input_ftm_map.get(dt).unwrap_or(&0.0);
            let o_ftm = *output_ftm_map.get(dt).unwrap_or(&0.0);

            let idx = *dates_to_index.get(dt)?;

            let sale_rate_ftm = ftm_rate[(idx.1, idx.0)] as f32;
            let sale_rate_btm = btm_rate[(idx.1, idx.0)] as f32;
            let ppw_ftm = ftm_ppw[(idx.1, idx.0)] as f32;
            let ppw_btm = btm_ppw[(idx.1, idx.0)] as f32;

            let output_sum = o_btm + o_ftm;
            let input_sum = i_btm + i_ftm;

            let sale_rate = if output_sum > 0.0 {
                (sale_rate_btm * o_btm + sale_rate_ftm * o_ftm) / output_sum
            } else {
                sale_rate_ftm
            };
            let ppw = if input_sum > 0.0 {
                (ppw_btm * i_btm + ppw_ftm * i_ftm) / input_sum
            } else {
                ppw_btm
            };

            Some(PvLdesOperator {
                date_time: surrealdb::sql::Datetime::from(*dt),
                set: site.to_string(),
                storage: *s_val,
                input_btm_power: i_btm,
                output_btm_power: o_btm,
                input_ftm_power: i_ftm,
                output_ftm_power: o_ftm,
                sale_rate,
                ppkwh: ppw,
                discharge_ratio: if *s_val > 0.0 { (o_btm + o_ftm) / *s_val } else { 0.0 },
                revenue: (sale_rate_ftm / 1000.0) * o_ftm + (sale_rate_btm / 1000.0) * o_btm,
                expense: (ppw_ftm / 1000.0) * i_ftm + (ppw_btm / 1000.0) * i_btm,
            })
        })
        .collect();

    db_pvldesops_update_ts(db, data.clone(), site).await?;
    Ok(data)
}



#[cfg(feature = "server")]
pub async fn gen_stat_vals(ts_vals: Vec<PvLdesOperator>) -> Option<PvLdesStats> {
    let mut total_revenue = 0.0;
    let mut total_expenses = 0.0;
    let mut total_profit = 0.0;
    let mut total_input_btm_power = 0.0;
    let mut total_output_btm_power = 0.0;
    let mut total_input_ftm_power = 0.0;
    let mut total_output_ftm_power = 0.0;
    let mut total_ppkwh = 0.0;
    let mut total_discharge_rate = 0.0;

    let count = ts_vals.len() as f32;

    for val in ts_vals.iter() {
        total_revenue += val.revenue;
        total_expenses += val.expense;
        total_profit += val.revenue - val.expense;
        total_input_btm_power += val.input_btm_power;
        total_output_btm_power += val.output_btm_power;
        total_input_ftm_power += val.input_ftm_power;
        total_output_ftm_power += val.output_ftm_power;
        total_ppkwh += val.ppkwh;
        total_discharge_rate += val.discharge_ratio;
    }

    Some(PvLdesStats {
        total_revenue,
        total_expenses,
        total_profit,
        total_input_btm_power,
        total_output_btm_power,
        total_input_ftm_power,
        total_output_ftm_power,
        average_ppkwh: if count > 0.0 { total_ppkwh / count } else { 0.0 },
        average_discharge_rate: if count > 0.0 {
            total_discharge_rate / count
        } else {
            0.0
        },
    })
}
