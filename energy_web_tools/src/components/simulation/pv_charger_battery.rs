#[cfg(feature = "server")]
use crate::components::energy_components::photovoltaic::pv_base_system::PvSystem;
use crate::components::energy_components::{
    batteries::BatteryStorage, ev_chargers::Charger, general_fun::PowerComponent,
};
use crate::components::output_data_structs::*;
use crate::components::surreal_data_structs::*;
use crate::components::time_processes::*;
use chrono::{DateTime, Datelike, Utc, Weekday};
use dioxus::prelude::*;
use std::{collections::HashMap, sync::Arc};
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
use tokio::sync::Mutex;
#[cfg(feature = "server")]
use tokio::task::JoinHandle;

#[server]
pub async fn setup_and_run_ev_charger_simulation(
    mut template_charger: Charger,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    battery_param_vec: Vec<(f32, f32)>,
    pv_param_vec: Vec<(f32, f32)>,
    ev_charger_param_vec: Vec<usize>,
    lamb_vec: Vec<(Lambdas, Lambdas)>,
) -> Result<Vec<HashMap<String, String>>, ServerFnError> {
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to start SurrealDB in memory");
    db.use_ns("charging-station").await?;
    db.use_db("batteries").await?;
    // Setup SurrealDB entry
    let _temp: Option<micro_ev_chargers::EvPvLdes> = db
        .create((
            "ev_pv_ldes",
            format!("{}_{}", "san-benito", uuid::Uuid::new_v4()),
        ))
        .content(micro_ev_chargers::EvPvLdes {
            date_time: surrealdb::sql::Datetime::from(Utc::now()),
            storage: 5.0,
            input_ftm_power: 10.0,
            output_ftm_power: 15.0,
            input_btm_power: 10.0,
            output_btm_power: 15.0,
            negative_net_storage: false,
        })
        .await?;
    println!("got here");
    let template_charger = Charger::add_input_power_ts(template_charger, start_date, end_date)
        .await
        .unwrap();
    //let charging_station = vec![template_charger_180kw.clone(); 20];
    //let _ = create_stat_csv("specification_neg_stat.csv");
    let mut tasks: Vec<JoinHandle<Result<StatData, ServerFnError>>> = Vec::new();
    let mut i = 1;
    for battery in battery_param_vec.iter() {
        for pv in pv_param_vec.iter() {
            for ev_charger in ev_charger_param_vec.iter() {
                for (weekday_rates, weekend_rates) in lamb_vec.iter() {
                    let solar_system =
                        PvSystem::new(pv.0, pv.1, 0.33, 7.0, 7.0, "data/full_irradiance_data.csv");
                    let battery_storage =
                        BatteryStorage::new(battery.0, battery.1, 80.0, 48.0, 98.0, 0.98);
                    let charging_station = vec![template_charger.clone(); *ev_charger];
                    let weekday_rates_clone = weekday_rates.clone();
                    let weekend_rates_clone = weekend_rates.clone();
                    let db_clone = Arc::new(db.clone());
                    let temp_start_date = start_date;
                    let temp_end_date = end_date;
                    // Create an asynchronous task for each simulation
                    let task = tokio::task::spawn(async move {
                        //let _ = create_stat_csv("specification_neg_stat.csv");
                        let result = run_simulation(
                            charging_station,
                            solar_system,
                            Arc::new(Mutex::new(battery_storage)),
                            weekday_rates_clone.to_vec(),
                            weekend_rates_clone.to_vec(),
                            String::from(""),
                            temp_start_date,
                            temp_end_date,
                            db_clone,
                            String::from("norwalk-art-complex"),
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
                    i += 1;
                }
            }
        }
    }
    let mut out_vec: Vec<HashMap<String, String>> = Vec::new();
    // Await all tasks to complete
    for task in tasks {
        /*let mut struc_vec = task
        .await??
        .iter()
        .map(|x| serde_json::to_string(x).unwrap())
        .collect::<Vec<String>>();*/
        let vals = struct_to_map(serde_json::to_value(&task.await?.unwrap())?, Vec::new()).await?;
        out_vec.push(vals);
    }

    Ok(out_vec)
}

#[cfg(feature = "server")]
pub async fn run_simulation(
    mut charging_station: Vec<Charger>,
    base_photovoltaic: PvSystem,
    battery_storage: Arc<Mutex<BatteryStorage>>, // Use Arc<Mutex> for shared, thread-safe state
    lambda_weekdays: Vec<f64>,
    lambda_weekends: Vec<f64>,
    _file_path: String,
    date0: DateTime<Utc>,
    date1: DateTime<Utc>,
    datab: Arc<Surreal<Db>>,
    site: String,
) -> Result<StatData, ServerFnError> {
    // Simulate the process
    let df_wd =
        modulated_markov_poison_process(date0.into(), date1.into(), lambda_weekdays.clone())
            .await?;
    let df_we =
        modulated_markov_poison_process(date0.into(), date1.into(), lambda_weekends.clone())
            .await?;
    println!("modulated_markov_done");
    let time_matrix_wd = df_wd.0;
    let date_matrix_wd = df_wd.2;
    println!("{:?}", time_matrix_wd.shape());
    println!("{:?}", date_matrix_wd.shape());
    let time_matrix_we = df_we.0;
    let date_matrix_we = df_we.2;
    println!("{:?}", time_matrix_we.shape());
    println!("{:?}", date_matrix_we.shape());
    // Update charging station based on the time matrix
    let mut i = 0;
    //println!("{:?}", charging_station);
    let charging_station = charging_station
        .iter_mut()
        .map(|charger| {
            let mut j = 0;
            charger.power.input_power_w_ts = Some(
                charger
                    .clone()
                    .power
                    .input_power_w_ts
                    .unwrap()
                    .iter_mut()
                    .map(|(dt, pow)| {
                        let mut temp = date_matrix_wd[(i, j)];
                        let mut temp_tm = time_matrix_wd[(i, j)] as f32;
                        let mut power_output = (*pow) * (time_matrix_wd[(i, j)] as f32);
                        if dt.weekday() == Weekday::Sat || dt.weekday() == Weekday::Sun {
                            power_output = (*pow) * (time_matrix_we[(i, j)] as f32);
                            temp = date_matrix_we[(i, j)];
                            temp_tm = time_matrix_we[(i, j)] as f32;
                        }
                        if i % 23 == 0 && i != 0 {
                            j += 1;
                            i = 0;
                        } else {
                            i += 1;
                        }
                        match true {
                            _ if &temp == dt => (), //println!("{:?}", "This matched"),
                            _ => println!("Doesn't match"),
                        }
                        (*dt, power_output)
                    })
                    .collect::<Vec<(DateTime<Utc>, f32)>>(),
            );
            j = 0;
            charger.power.output_power_w_ts = Some(
                charger
                    .clone()
                    .power
                    .output_power_w_ts
                    .unwrap()
                    .iter_mut()
                    .map(|(dt, _pow)| (*dt, 0.0))
                    .collect::<Vec<(DateTime<Utc>, f32)>>(),
            );
            charger
        })
        .collect::<Vec<&mut Charger>>();
    // ... further result processing and outp
    // Processing the power components
    let power_component_vec: Vec<PowerComponent> = charging_station
        .iter()
        .map(|charger| charger.power.clone())
        .collect();
    //println!("{:?}", power_component_vec);
    let charging_station_comp =
        PowerComponent::merge_power_components(power_component_vec, 0.0).await;
    //println!("{:?}",charging_station_comp);
    let generator = base_photovoltaic
        .clone()
        .into_power_component(date0, date1)
        .await;
    //println!("{:?}",generator);
    // Update battery storage;
    let mut batt_stor = battery_storage
        .lock()
        .await
        .clone()
        .update_power_component( None, None, 
            Some(generator),
            Some(charging_station_comp.demand))
        .await;
    //println!("{:?}",batt_stor.battery_state);
    // Write the battery state to CSV and the database
    //sim_to_csv(&mut battery_storage.lock().await.clone());
    let sd = gen_stat(
        &mut batt_stor,
        base_photovoltaic,
        charging_station,
        lambda_weekends,
        "stat_df.csv",
    )
    .await?;
    let out_struc =
        micro_ev_chargers::battery_storage_to_db_evpvldes(&mut batt_stor, &datab, site.as_str())
            .await?;
    Ok(sd)
}
