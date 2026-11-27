use crate::components::energy_components::general_fun::PowerComponent;
use chrono::{DateTime, Utc};
use std::collections::{HashMap,BTreeMap,BTreeSet};

#[cfg(feature = "server")]
use nalgebra::base::DVector;


#[derive(Clone, Debug)]
pub struct BatteryPowerComponent {
    pub storage: Option<Vec<(DateTime<Utc>, f32)>>,
    pub input_power_btm_ts: Option<Vec<(DateTime<Utc>, f32)>>,
    pub input_power_ftm_ts: Option<Vec<(DateTime<Utc>, f32)>>,
    pub output_power_btm_ts: Option<Vec<(DateTime<Utc>, f32)>>,
    pub output_power_ftm_ts: Option<Vec<(DateTime<Utc>, f32)>>,
    pub neg_stat_ts: Option<Vec<(DateTime<Utc>, bool)>>,
    pub charge_pct: Option<Vec<(DateTime<Utc>, f32)>>,
}

impl BatteryPowerComponent {
    pub fn merge_battery_power_component(
        components: Vec<BatteryPowerComponent>,
    ) -> BatteryPowerComponent {
        // Initialize merged fields
        let mut storage: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut input_power_btm_w_ts: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut input_power_ftm_w_ts: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut output_power_btm_w_ts: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut output_power_ftm_w_ts: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut neg_stat_ts: Vec<(DateTime<Utc>, bool)> = Vec::new();
        let mut charge_pct: Vec<(DateTime<Utc>, f32)> = Vec::new();

        // Helper closure to merge optional fields
        let merge_optional = |field: Option<Vec<(DateTime<Utc>, f32)>>,
                              merged: &mut Vec<(DateTime<Utc>, f32)>| {
            if let Some(data) = field {
                for (dt, value) in data {
                    // Merge values (sum, average, or any other logic can be applied here)
                    if let Some(pos) = merged
                        .iter_mut()
                        .find(|(existing_dt, _)| existing_dt == &dt)
                    {
                        pos.1 += value; // Sum the values for the same DateTime
                    } else {
                        merged.push((dt, value));
                    }
                }
            }
        };

        // Process each component
        for component in components {
            merge_optional(component.storage, &mut storage);
            merge_optional(component.input_power_btm_ts, &mut input_power_btm_w_ts);
            merge_optional(component.input_power_ftm_ts, &mut input_power_ftm_w_ts);
            merge_optional(component.output_power_btm_ts, &mut output_power_btm_w_ts);
            merge_optional(component.output_power_ftm_ts, &mut output_power_ftm_w_ts);
            // Merge neg_stat_ts (combine booleans)
            if let Some(data) = component.neg_stat_ts {
                for (dt, value) in data {
                    neg_stat_ts.push((dt, value));
                }
            }

            // Merge charge_pct (average the charge percentage)
            if let Some(data) = component.charge_pct {
                for (dt, value) in data {
                    if let Some(pos) = charge_pct
                        .iter_mut()
                        .find(|(existing_dt, _)| existing_dt == &dt)
                    {
                        pos.1 = (pos.1 + value) / 2.0; // Average the charge percentages
                    } else {
                        charge_pct.push((dt, value));
                    }
                }
            }
        }

        // Return the merged BatteryPowerComponent
         BatteryPowerComponent {
            storage: if storage.is_empty() { None } else { Some(storage) },
            input_power_btm_ts: if input_power_btm_w_ts.is_empty() { None } else { Some(input_power_btm_w_ts) },
            input_power_ftm_ts: if input_power_ftm_w_ts.is_empty() { None } else { Some(input_power_ftm_w_ts) },
            output_power_btm_ts: if output_power_btm_w_ts.is_empty() { None } else { Some(output_power_btm_w_ts) },
            output_power_ftm_ts: if output_power_ftm_w_ts.is_empty() { None } else { Some(output_power_ftm_w_ts) },
            neg_stat_ts: if neg_stat_ts.is_empty() { None } else { Some(neg_stat_ts) },
            charge_pct: if charge_pct.is_empty() { None } else { Some(charge_pct) },
        }
    }
}

#[derive(Clone, Debug)]
pub struct BatteryStorage {
    pub capacity: f32,
    pub watt_hours: f32,
    pub depth_of_discharge: f32,
    pub battery_system_voltage: f32,
    pub efficiency_charge: f32,
    pub efficiency_discharge: f32,
    pub battery_state: BatteryPowerComponent,
}

#[cfg(feature = "server")]
impl BatteryStorage {
    // General constructor
    pub fn new(
        capacity: f32,
        watt_hours: f32,
        depth_of_discharge: f32,
        battery_system_voltage: f32,
        efficiency_charge: f32,
        efficiency_discharge: f32,
    ) -> BatteryStorage {
        BatteryStorage {
            capacity,
            watt_hours,
            depth_of_discharge,
            battery_system_voltage,
            efficiency_charge,
            efficiency_discharge,
            battery_state: BatteryPowerComponent {
                storage: None,
                input_power_btm_ts: None,
                input_power_ftm_ts: None,
                output_power_btm_ts: None,
                output_power_ftm_ts: None,
                neg_stat_ts: None,
                charge_pct: None,
            },
        }
    }
    pub fn merge_battery_storage(
        batt_sett: Vec<BatteryStorage>,
    ) -> BatteryStorage {
        let mut capacity_vec: Vec<f32> = Vec::new();
        let mut watt_hours_vec: Vec<f32> = Vec::new();
        let mut battery_system_voltage_vec: Vec<f32> = Vec::new();
        let mut efficiency_charge_vec: Vec<f32> = Vec::new();
        let mut efficiency_discharge_vec: Vec<f32> = Vec::new();
        let mut depth_of_discharge_vec: Vec<f32> = Vec::new();
        let mut battery_state_vec: Vec<BatteryPowerComponent> = Vec::new();
        let count = batt_sett.len();
        for batt in batt_sett.iter() {
            capacity_vec.push(batt.capacity);
            watt_hours_vec.push(batt.watt_hours);
            battery_system_voltage_vec.push(batt.battery_system_voltage);
            efficiency_charge_vec.push(batt.efficiency_charge);
            efficiency_discharge_vec.push(batt.efficiency_discharge);
            battery_state_vec.push(batt.battery_state.clone());
            depth_of_discharge_vec.push(batt.depth_of_discharge)
        }
        let capacity_dvec = DVector::from_vec(capacity_vec);
        let capacity_weights_dec = capacity_dvec.clone() / capacity_dvec.sum();
        println!("Capacity Weights {:?}", capacity_weights_dec);
        println!("Capacity {:?}", capacity_dvec);
        println!("Capacity sum {:?}", capacity_dvec.sum());
        let watt_hours_dvec = DVector::from_vec(watt_hours_vec);
        let battery_system_voltage_dvec = DVector::from_vec(battery_system_voltage_vec);
        let efficiency_charge_dvec = DVector::from_vec(efficiency_charge_vec);
        let efficiency_discharge_dvec = DVector::from_vec(efficiency_discharge_vec);
        let battery_state = BatteryPowerComponent::merge_battery_power_component(battery_state_vec);
        let depth_of_discharge_dvec = DVector::from_vec(depth_of_discharge_vec);
        BatteryStorage {
            capacity: capacity_dvec.sum(),
            watt_hours: watt_hours_dvec.dot(&capacity_weights_dec),
            depth_of_discharge: depth_of_discharge_dvec.dot(&capacity_weights_dec),
            battery_system_voltage: battery_system_voltage_dvec.mean(),
            efficiency_charge: efficiency_charge_dvec.dot(&capacity_weights_dec),
            efficiency_discharge: efficiency_discharge_dvec.dot(&capacity_weights_dec),
            battery_state,
        }
    }

    pub async fn update_power_operator_component(
        mut self,
        ftm_generator: Option<Vec<(DateTime<Utc>, f32)>>,
        ftm_utility: Option<Vec<(DateTime<Utc>, f32)>>,
        btm_generator: Option<Vec<(DateTime<Utc>, f32)>>,
        btm_utility: Option<Vec<(DateTime<Utc>, f32)>>,
    ) -> BatteryStorage {
        // Unwrap to vectors (empty if None)
        let ftm_gen_vec = ftm_generator.unwrap_or_default();
        let ftm_util_vec = ftm_utility.unwrap_or_default();
        let btm_gen_vec = btm_generator.unwrap_or_default();
        let btm_util_vec = btm_utility.unwrap_or_default();

        // Collect all unique timestamps from all inputs
        use std::collections::BTreeSet;
        let mut all_timestamps = BTreeSet::new();
        for (vec) in [&ftm_gen_vec, &ftm_util_vec, &btm_gen_vec, &btm_util_vec] {
            for (dt, _) in vec.iter() {
                all_timestamps.insert(*dt);
            }
        }

        // Create fast lookups
        let ftm_gen_map: HashMap<DateTime<Utc>, f32> = ftm_gen_vec.iter().cloned().collect();
        let ftm_util_map: HashMap<DateTime<Utc>, f32> = ftm_util_vec.iter().cloned().collect();
        let btm_gen_map: HashMap<DateTime<Utc>, f32> = btm_gen_vec.iter().cloned().collect();
        let btm_util_map: HashMap<DateTime<Utc>, f32> = btm_util_vec.iter().cloned().collect();

        // Vectors for battery state tracking
        let mut storage: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut input_btm: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut input_ftm: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut output_btm: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut output_ftm: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut neg_dummy: Vec<(DateTime<Utc>, bool)> = Vec::new();
        let mut charge_pct: Vec<(DateTime<Utc>, f32)> = Vec::new();

        let mut last_storage = self.depth_of_discharge * self.capacity * self.watt_hours;

        for (idx, dt) in all_timestamps.iter().enumerate() {
            // Get each flow at this timestamp (default to 0.0 if missing)
            let ftm_in  = *ftm_gen_map.get(dt).unwrap_or(&0.0);
            let ftm_out = *ftm_util_map.get(dt).unwrap_or(&0.0);
            let btm_in  = *btm_gen_map.get(dt).unwrap_or(&0.0);
            let btm_out = *btm_util_map.get(dt).unwrap_or(&0.0);

            // Store for BTM/FTM input/output time series
            input_btm.push((*dt, btm_in));
            input_ftm.push((*dt, ftm_in));
            output_btm.push((*dt, btm_out));
            output_ftm.push((*dt, ftm_out));

            // Aggregate for main battery state calculation (if you want to split this, let me know)
            let total_in  = btm_in.min(self.capacity) * self.efficiency_charge
                          + ftm_in.min(self.capacity) * self.efficiency_charge;
            let total_out = btm_out.min(self.capacity) * (1.0 / self.efficiency_discharge)
                          + ftm_out.min(self.capacity) * (1.0 / self.efficiency_discharge);

            // Calculate new storage
            let mut temp_storage = last_storage + total_in - total_out;
            if temp_storage > self.capacity * self.watt_hours {
                temp_storage = self.capacity * self.watt_hours;
            }
            // Optional: Prevent negative storage
            if temp_storage < 0.0 {
                temp_storage = 0.0;
            }

            storage.push((*dt, temp_storage));
            neg_dummy.push((*dt, temp_storage < 0.0));
            charge_pct.push((*dt, temp_storage / (self.capacity * self.watt_hours)));

            last_storage = temp_storage;
        }

        // Store in the BatteryPowerComponent struct
        self.battery_state.storage = Some(storage);
        self.battery_state.input_power_btm_ts = Some(input_btm);
        self.battery_state.input_power_ftm_ts = Some(input_ftm);
        self.battery_state.output_power_btm_ts = Some(output_btm);
        self.battery_state.output_power_ftm_ts = Some(output_ftm);
        self.battery_state.neg_stat_ts = Some(neg_dummy);
        self.battery_state.charge_pct = Some(charge_pct);
        self
    }


    #[cfg(feature = "server")]
    pub async fn update_power_component(
        mut self,
        ftm_generator: Option<PowerComponent>,
        ftm_utility: Option<PowerComponent>,
        btm_generator: Option<PowerComponent>,
        btm_utility: Option<PowerComponent>,
    ) -> BatteryStorage {
        use std::collections::BTreeMap;
        use chrono::{DateTime, Utc};

        // Unpack each time series or use empty vecs
        let ftm_gen_ts = ftm_generator
            .and_then(|p| p.output_power_w_ts)
            .unwrap_or_default();
        let ftm_util_ts = ftm_utility
            .and_then(|p| p.input_power_w_ts)
            .unwrap_or_default();
        let btm_gen_ts = btm_generator
            .and_then(|p| p.output_power_w_ts)
            .unwrap_or_default();
        let btm_util_ts = btm_utility
            .and_then(|p| p.input_power_w_ts)
            .unwrap_or_default();

        // Build BTreeMaps for all
        let ftm_gen_map: BTreeMap<_, _> = ftm_gen_ts.iter().copied().collect();
        let ftm_util_map: BTreeMap<_, _> = ftm_util_ts.iter().copied().collect();
        let btm_gen_map: BTreeMap<_, _> = btm_gen_ts.iter().copied().collect();
        let btm_util_map: BTreeMap<_, _> = btm_util_ts.iter().copied().collect();

        // Gather all unique DateTimes
        let mut all_datetimes = BTreeMap::new();
        for map in [&ftm_gen_map, &ftm_util_map, &btm_gen_map, &btm_util_map] {
            for (&dt, _) in map.iter() {
                all_datetimes.insert(dt, ());
            }
        }

        // Early return: if no data, clear outputs
        if all_datetimes.is_empty() {
            self.battery_state.storage = Some(vec![]);
            self.battery_state.input_power_btm_ts = Some(vec![]);
            self.battery_state.input_power_ftm_ts = Some(vec![]);
            self.battery_state.output_power_btm_ts = Some(vec![]);
            self.battery_state.output_power_ftm_ts = Some(vec![]);
            self.battery_state.neg_stat_ts = Some(vec![]);
            self.battery_state.charge_pct = Some(vec![]);
            return self;
        }

        // Compose (dt, ftm_in, btm_in, ftm_out, btm_out)
        let mut temp_vals = Vec::with_capacity(all_datetimes.len());
        for dt in all_datetimes.keys() {
            let ftm_in = ftm_gen_map.get(dt).copied().unwrap_or(0.0);
            let btm_in = btm_gen_map.get(dt).copied().unwrap_or(0.0);
            let ftm_out = ftm_util_map.get(dt).copied().unwrap_or(0.0);
            let btm_out = btm_util_map.get(dt).copied().unwrap_or(0.0);
            temp_vals.push((*dt, ftm_in, btm_in, ftm_out, btm_out));
        }

        let mut storage = Vec::new();
        let mut input_btm = Vec::new();
        let mut input_ftm = Vec::new();
        let mut output_btm = Vec::new();
        let mut output_ftm = Vec::new();
        let mut neg_dummy = Vec::new();
        let mut charge_pct = Vec::new();

        // Aggregate totals for SOC calculation
        let (first_dt, first_ftm_in, first_btm_in, first_ftm_out, first_btm_out) = temp_vals[0];
        let first_total_in = first_btm_in + first_ftm_in;
        let first_total_out = first_btm_out + first_ftm_out;

        let first_soc = self.depth_of_discharge * self.capacity * self.watt_hours
            + first_total_in * self.efficiency_charge
            - first_total_out * (1.0 / self.efficiency_discharge);

        storage.push((first_dt, first_soc));
        input_btm.push((first_dt, first_btm_in));
        input_ftm.push((first_dt, first_ftm_in));
        output_btm.push((first_dt, first_btm_out));
        output_ftm.push((first_dt, first_ftm_out));
        neg_dummy.push((first_dt, first_soc < 0.0));
        charge_pct.push((first_dt, first_soc / (self.capacity * self.watt_hours)));

        for idx in 1..temp_vals.len() {
            let (dt, ftm_in, btm_in, ftm_out, btm_out) = temp_vals[idx];
            let prev_soc = storage[idx - 1].1;
            let total_in = (btm_in + ftm_in).min(self.capacity) * self.efficiency_charge;
            let total_out = (btm_out + ftm_out).min(self.capacity) * (1.0 / self.efficiency_discharge);

            let mut temp_storage = prev_soc + total_in - total_out;
            temp_storage = temp_storage.clamp(0.0, self.capacity * self.watt_hours);

            storage.push((dt, temp_storage));
            input_btm.push((dt, btm_in));
            input_ftm.push((dt, ftm_in));
            output_btm.push((dt, btm_out));
            output_ftm.push((dt, ftm_out));
            let is_negative = temp_storage <= 0.0;
            neg_dummy.push((dt, is_negative));
            charge_pct.push((dt, temp_storage / (self.capacity * self.watt_hours)));

            println!(
                "Date {} ftm_in {} btm_in {} ftm_out {} btm_out {} storage {}",
                dt, ftm_in, btm_in, ftm_out, btm_out, temp_storage
            );
        }

        self.battery_state.storage = Some(storage);
        self.battery_state.input_power_btm_ts = Some(input_btm);
        self.battery_state.input_power_ftm_ts = Some(input_ftm);
        self.battery_state.output_power_btm_ts = Some(output_btm);
        self.battery_state.output_power_ftm_ts = Some(output_ftm);
        self.battery_state.neg_stat_ts = Some(neg_dummy);
        self.battery_state.charge_pct = Some(charge_pct);

        self
    }

}

// Constructor that calculates capacity
