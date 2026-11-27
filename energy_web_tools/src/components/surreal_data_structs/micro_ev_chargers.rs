use super::StatData;
use crate::components::energy_components::batteries::BatteryStorage;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use surrealdb::{
    engine::{any::Any, local::Db},
    opt::auth::Root,
    sql::Datetime,
    Surreal,
};
#[cfg(feature = "server")]
use tokio::task;
#[cfg(feature = "server")]
use uuid;

#[cfg(feature = "server")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvPvLdes {
    pub date_time: Datetime,
    pub storage: f32,
    pub input_btm_power: f32,
    pub output_btm_power: f32,
    pub input_ftm_power: f32,
    pub output_ftm_power: f32,
    pub negative_net_storage: bool,
}


#[cfg(feature = "server")]
pub async fn db_update_stats(
    db: &Surreal<Db>,
    data: Vec<EvPvLdes>,
    site: &str,
) -> Result<(), surrealdb::Error> {
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
pub async fn db_evpvldes_update_tms(
    db: &Surreal<Db>,
    data: Vec<EvPvLdes>,
    site: &str,
) -> Result<(), surrealdb::Error> {
    let mut handles = vec![];

    // Spawn each task concurrently using Tokio
    for record in data {
        let db = db.clone(); // Clone the db reference for each spawned task
        let site = site.to_string(); // Clone site string for use in the async block
        let handle = task::spawn(async move {
            let result: std::result::Result<std::option::Option<EvPvLdes>, surrealdb::Error> = db
                .create(("ev_pv_ldes", format!("{}_{}", site, uuid::Uuid::new_v4())))
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
pub async fn battery_storage_to_db_evpvldes(
    batt_system: &mut BatteryStorage,
    db: &Surreal<Db>,
    site: &str,
) -> Result<Vec<EvPvLdes>, surrealdb::Error> {
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

    // Build BTreeMaps to align by datetime
    let storage_map: BTreeMap<_, _> = storage_iter.collect();
    let input_btm_map: BTreeMap<_, _> = input_btm_iter.collect();
    let output_btm_map: BTreeMap<_, _> = output_btm_iter.collect();
    let input_ftm_map: BTreeMap<_, _> = input_ftm_iter.collect();
    let output_ftm_map: BTreeMap<_, _> = output_ftm_iter.collect();
    let neg_stat_map: BTreeMap<_, _> = neg_stat_iter.collect();

    let mut all_dates: Vec<_> = storage_map.keys().cloned().collect();
    all_dates.sort();

    let data: Vec<EvPvLdes> = all_dates
        .iter()
        .map(|dt| {
            EvPvLdes {
                date_time: surrealdb::sql::Datetime::from(*dt),
                storage: *storage_map.get(dt).unwrap_or(&0.0),
                input_btm_power: *input_btm_map.get(dt).unwrap_or(&0.0),
                output_btm_power: *output_btm_map.get(dt).unwrap_or(&0.0),
                input_ftm_power: *input_ftm_map.get(dt).unwrap_or(&0.0),
                output_ftm_power: *output_ftm_map.get(dt).unwrap_or(&0.0),
                negative_net_storage: *neg_stat_map.get(dt).unwrap_or(&false),
            }
        })
        .collect();

    db_evpvldes_update_tms(db, data.clone(), site).await?;
    Ok(data)
}
