#[cfg(feature = "server")]
use crate::components::energy_components::{
    batteries::BatteryStorage, operator::Operator, photovoltaic::pv_base_system::PvSystem,
};
use crate::components::output_data_structs::*;
use crate::components::surreal_data_structs::micro_operator::*;
//use crate::create_stat_csv;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use nalgebra::base::DMatrix;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "server")]
use surrealdb::{
    engine::{
        any::Any,
        local::{Db, Mem},
        remote::ws::Client,
    },
    opt::auth::Root,
    Surreal,
};
#[cfg(feature = "server")]
use tokio::task::JoinHandle;

#[server]
pub async fn setup_and_run_operator_simulation(
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    battery_param_vec: Vec<Vec<(f32, f32, f32, f32, f32,f32)>>,
    pv_param_vec: Vec<Vec<(f32, f32, f32, f32, f32, f32)>>,
    btm_rate_param: Option<f64>,
    btm_ppw_param: Option<f64>,
    rates_param: Vec<(f64, f64)>,
    keys: Vec<String>,
    sim_type: usize,
) -> Result<(Vec<HashMap<String, String>>, Vec<HashMap<String, String>>), ServerFnError> {

    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to start SurrealDB in memory");
    db.use_ns("charging-station").await?;
    db.use_db("batteries").await?;

    let _: Option<PvLdesOperator> = db
        .create((
            "pv_ldes_operator",
            format!("{}_{}", "san-benito", uuid::Uuid::new_v4()),
        ))
        .content(PvLdesOperator {
            date_time: surrealdb::sql::Datetime::from(Utc::now()),
            set: "test".to_string(),
            storage: 0.0,
            input_btm_power: 0.0,
            output_btm_power: 0.0,
            input_ftm_power: 0.0,
            output_ftm_power: 0.0,
            sale_rate: 0.0,
            ppkwh: 0.0,
            discharge_ratio: 0.0,
            revenue: 0.0,
            expense: 0.0,
        })
        .await?;
    println!("got here");
    let mut tasks: Vec<JoinHandle<Result<Vec<PvLdesOperator>, dioxus::prelude::ServerFnError>>> =
        Vec::new();
    let i = 1;
    let mut solar: Vec<Vec<PvSystem>> = Vec::new();
    let mut batteries: Vec<Vec<BatteryStorage>> = Vec::new();
    for (batt_set, solar_set) in battery_param_vec.iter().zip(pv_param_vec) {
        let mut solar_vec: Vec<PvSystem> = Vec::new();
        let mut batteries_vec: Vec<BatteryStorage> = Vec::new();
        for pv in solar_set.iter() {
            solar_vec.push(PvSystem::new(
                pv.0,
                pv.1,
                pv.2,
                pv.3,
                pv.4,
                "data/full_irradiance_data.csv",
            ));
        }
        for battery in batt_set.iter() {
            batteries_vec.push(BatteryStorage::new(
                battery.0, battery.1, battery.2, battery.3, battery.4, battery.5,
            ));
        }
        batteries.push(batteries_vec);
        solar.push(solar_vec);
    }
    let iou = Operator::new(start_date, end_date, keys, solar, batteries);
    let db_clone = Arc::new(db.clone());
    // Create an asynchronous task for each simulation
    let task = tokio::task::spawn(async move {
        //let _ = create_stat_csv("specification_neg_stat.csv");
        let result = run_simulation(
            iou.unwrap(),
            "hello",
            db_clone,
            btm_rate_param,
            btm_ppw_param,
            rates_param,
            start_date,
            end_date,
            sim_type,
        )
        .await;
        if let Err(e) = result.clone() {
            eprintln!("Simulation failed: {:?}", e);
        } else {
            println!("Simulation {} completed successfully", i);
        }
        result
    });
    tasks.push(task);
    // Await all tasks to complete
    let mut out_vec: Vec<HashMap<String, String>> = Vec::new();
    let mut stat_vec: Vec<HashMap<String, String>> = Vec::new();

    for task in tasks {
        let temp_val = &task.await?.unwrap();
        let stat_val = gen_stat_vals(temp_val.clone()).await.unwrap();
        stat_vec.push(struct_to_map(serde_json::to_value(stat_val)?, Vec::new()).await?);
        for op_struct in temp_val {
            let vals =
                struct_to_map(serde_json::to_value(op_struct)?, vec!["set".to_string()]).await?;
            out_vec.push(vals);
        }
    }
    Ok((out_vec, stat_vec))
}

#[cfg(feature = "server")]
pub async fn run_simulation(
    mut inv_ops_util: Operator, // Use Arc<Mutex> for shared, thread-safe state
    _file_path: &str,
    datab: Arc<Surreal<Db>>,
    btm_rate_param: Option<f64>,
    btm_ppw_param: Option<f64>,
    ftm_rates_param: Vec<(f64, f64)>,
    date1: DateTime<Utc>,
    date2: DateTime<Utc>,
    sim_type: usize,
) -> Result<Vec<PvLdesOperator>, ServerFnError> {
    let opt_map: (
        &HashMap<chrono::DateTime<Utc>, (usize, usize)>,
        HashMap<String, (BatteryStorage, f64)>,
        &DMatrix<f64>,//ftm_rate
        DMatrix<f64>,//btm_rate
        &DMatrix<f64>,//ftm_ppw
        DMatrix<f64>,//btm_ppw
    ) = inv_ops_util
        .estimate_revenue(btm_rate_param,btm_ppw_param,ftm_rates_param, date1, date2, sim_type)
        .await
        .unwrap();
    let mut out_list: Vec<PvLdesOperator> = Vec::new();
    for (key, (mut batt, num)) in opt_map.1 {
        // Call your async function, awaiting its result
        out_list.append(
            &mut battery_pvldesops_storage_to_db(
                opt_map.0.clone(),
                &mut batt,
                opt_map.2.clone(),//ftm_rate
                opt_map.3.clone(),//btm_rate
                opt_map.4.clone(),//ftm_ppw
                opt_map.5.clone(),//btm_ppw
                &datab,
                &key,
            )
            .await?,
        );
        // Print the profit
        println!("The profit for {} is: {}", key, num);
    }
    Ok(out_list)
}
