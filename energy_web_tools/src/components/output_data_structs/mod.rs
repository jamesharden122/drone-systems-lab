pub mod struct_tables;
pub mod charts;
use chrono::{DateTime, Utc};
#[cfg(feature = "server")]
use crate::components::energy_components::{
    batteries::BatteryStorage, ev_chargers::Charger, general_fun::PowerComponent,
    photovoltaic::pv_base_system::PvSystem,
};
use crate::components::surreal_data_structs::StatData;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;

pub type Lambdas = Vec<f64>;

pub trait RoundToPlaces {
    fn round_to_places(self, places: u32) -> Self;
}

impl RoundToPlaces for f64 {
    fn round_to_places(self, places: u32) -> Self {
        let factor = 10_f64.powi(places as i32);
        (self * factor).trunc() / factor
    }
}

impl RoundToPlaces for f32 {
    fn round_to_places(self, places: u32) -> Self {
        let factor = 10_f32.powi(places as i32);
        (self * factor).trunc() / factor
    }
}

#[server]
pub async fn struct_to_map(
    value: Value,
    ignore_fields: Vec<String>,
) -> Result<HashMap<String, String>, ServerFnError> {
    let map = match value {
        Value::Object(obj) => obj
            .into_iter()
            .filter(|(k, _)| !ignore_fields.contains(k))
            .map(|(k, v)| (k, v.to_string()))
            .collect::<HashMap<_, _>>(),
        _ => HashMap::new(),
    };
    Ok(map)
}
#[cfg(feature = "server")]
pub async fn gen_stat(
    batt_system: &mut BatteryStorage,
    pv_system: PvSystem,
    ev_chargers: Vec<&mut Charger>,
    lamb_vec: Vec<f64>,
    file_path: &str,
) -> std::result::Result<StatData, ServerFnError> {
    fn sum_bools(bools: &Vec<bool>) -> usize {
        bools.iter().map(|&b| b as usize).sum()
    }
    fn max_f32_in_vec(vec: &Vec<f32>) -> Option<f32> {
        vec.iter().cloned().fold(None, |max, val| match max {
            None => Some(val),
            Some(max_val) => Some(max_val.max(val)),
        })
    }

    let state = &batt_system.battery_state;

    // All time series may be Option -- sum BTM + FTM for input/output at each timestamp
    let storage_iter = state.storage.clone().unwrap_or_default().into_iter();

    let input_btm = state.input_power_btm_ts.clone().unwrap_or_default();
    let input_ftm = state.input_power_ftm_ts.clone().unwrap_or_default();
    let output_btm = state.output_power_btm_ts.clone().unwrap_or_default();
    let output_ftm = state.output_power_ftm_ts.clone().unwrap_or_default();

    // Merge all DateTimes
    let mut all_dates = HashSet::new();
    for (dt, _) in &input_btm { all_dates.insert(*dt); }
    for (dt, _) in &input_ftm { all_dates.insert(*dt); }
    for (dt, _) in &output_btm { all_dates.insert(*dt); }
    for (dt, _) in &output_ftm { all_dates.insert(*dt); }

    // BTreeMap to keep order
    use std::collections::BTreeMap;
    let input_btm_map: BTreeMap<_, _> = input_btm.into_iter().collect();
    let input_ftm_map: BTreeMap<_, _> = input_ftm.into_iter().collect();
    let output_btm_map: BTreeMap<_, _> = output_btm.into_iter().collect();
    let output_ftm_map: BTreeMap<_, _> = output_ftm.into_iter().collect();

    // Build input/output sum at each timestamp
    let mut combined_input: Vec<(DateTime<Utc>, f32)> = Vec::new();
    let mut combined_output: Vec<(DateTime<Utc>, f32)> = Vec::new();

    for dt in &all_dates {
        let inp = input_btm_map.get(dt).unwrap_or(&0.0) + input_ftm_map.get(dt).unwrap_or(&0.0);
        let out = output_btm_map.get(dt).unwrap_or(&0.0) + output_ftm_map.get(dt).unwrap_or(&0.0);
        combined_input.push((*dt, inp));
        combined_output.push((*dt, out));
    }

    // Align storage with input/output (assuming storage timestamps cover all others)
    let neg_stat_vec = state.neg_stat_ts.clone().unwrap_or_default();
    let mut neg_stat_map = BTreeMap::new();
    for (dt, val) in &neg_stat_vec { neg_stat_map.insert(*dt, *val); }

    // Zip for stats, filter by dates that are present in all
    let mut data_neg_stat = Vec::new();
    let mut data_storage = Vec::new();
    for (dt, val) in state.storage.clone().unwrap_or_default() {
        let neg = neg_stat_map.get(&dt).copied().unwrap_or(false);
        data_neg_stat.push(neg);
        data_storage.push(val);
    }

    let days_length = state
        .storage
        .as_ref()
        .map(|vec| {
            vec.iter()
                .map(|(dt, _)| dt.date_naive())
                .collect::<HashSet<_>>()
                .len()
        })
        .unwrap_or(0);
    Ok(StatData {
    chargers_count: ev_chargers.len(),
    energy_system_size: (pv_system.num_panels * pv_system.panel_watts) as f64,
    battery_size: (batt_system.capacity * batt_system.watt_hours) as f64,
    duration_energy_needed: (sum_bools(&data_neg_stat) as f32) / (data_neg_stat.len() as f32),
    total_energy_usage: output_btm_map.values().sum::<f32>() + output_ftm_map.values().sum::<f32>(),
    total_btm_generation: input_btm_map.values().sum::<f32>(),
    max_storage: max_f32_in_vec(&data_storage).unwrap_or(0.0),
    average_usage: lamb_vec.iter().sum::<f64>() / 60.0,
    days: days_length,
})
}
