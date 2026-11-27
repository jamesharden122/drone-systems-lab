use crate::components::energy_components::batteries::{BatteryPowerComponent, BatteryStorage};
use crate::components::energy_components::general_fun::PowerComponent;
#[cfg(feature = "server")]
use crate::components::energy_components::photovoltaic::pv_base_system::PvSystem;
#[cfg(feature = "server")]
use crate::components::time_processes::daily_rates_process;
#[cfg(feature = "server")]
use anyhow::Result;
use chrono::{prelude::*, DateTime, Utc};
use dioxus::prelude::*;
#[cfg(feature = "server")]
use good_lp::{
    constraint, default_solver, variables, Constraint, ProblemVariables, Solution, SolverModel,
    Variable, *,
};
#[cfg(feature = "server")]
use nalgebra::base::DMatrix;
use std::collections::HashMap;

#[cfg(feature = "server")]
#[derive(Clone, Debug)]
pub struct Operator {
    pub date_to_index: HashMap<DateTime<Utc>, (usize, usize)>,
    pub daily_rates: DMatrix<f64>, // Time and price per watt
    pub ppw: DMatrix<f64>,         // Time and cost of solar generation
    pub discharge_rates: HashMap<String, DMatrix<f64>>, // Time and battery discharge amount
    pub batteries: HashMap<String, Vec<BatteryStorage>>, // Grouped by key
    pub solar: HashMap<String, Vec<PvSystem>>, // Grouped by key//pub discount_rates: Option<HashMap<String, Vec<PvSystem>>>,
    pub keys: Vec<String>,                     // Unified keys for both maps
}

#[cfg(feature = "server")]
impl Operator {
    pub fn new(
        d1: DateTime<Utc>,
        d2: DateTime<Utc>,
        keys: Vec<String>,
        solar: Vec<Vec<PvSystem>>,
        batteries: Vec<Vec<BatteryStorage>>,
    ) -> Result<Self, ServerFnError> {
        let date_fmt = "%Y-%m-%d %H:%M:%S%:z";
        let mut daily_rates_vec: Vec<f64> = Vec::new();
        let mut ppw_vec: Vec<f64> = Vec::new();
        let mut discharge_rates_vec: Vec<f64> = Vec::new();
        let mut date_to_index: HashMap<DateTime<Utc>, (usize, usize)> = HashMap::new();
        let mut batteries_map: HashMap<String, Vec<BatteryStorage>> = HashMap::new();
        let mut solar_map: HashMap<String, Vec<PvSystem>> = HashMap::new();
        let mut discharge_rates: HashMap<String, DMatrix<f64>> = HashMap::new();
        let mut curr_time = d1;
        let mut ind = 0;
        loop {
            let curr_day = curr_time.day();
            if curr_time == d2 {
                break;
            }
            date_to_index.insert(
                curr_time.into(),
                (ind, curr_time.hour().try_into().unwrap()),
            );
            discharge_rates_vec
                .push(Self::discharge_rate_fun(curr_time.with_timezone(&Utc), 0.2)?.1);
            ppw_vec.push(Self::ppw_fun(curr_time.with_timezone(&Utc))?.1);
            daily_rates_vec.push(Self::daily_rates_fun(curr_time.with_timezone(&Utc))?.1);
            curr_time += chrono::Duration::hours(1);
            if curr_time.day() != curr_day {
                ind += 1;
            }
            //println!("Date: {},Day: {},Hour: {}", curr_time,ind,curr_time.hour());
        }

        for ((solar_system, battery), key) in solar.iter().zip(&batteries).zip(&keys) {
            solar_map.insert(key.to_string(), solar_system.clone());
            batteries_map.insert(key.to_string(), battery.clone());
            discharge_rates.insert(
                key.to_string(),
                DMatrix::from_vec(discharge_rates_vec.len() / 24, 24, daily_rates_vec.clone()),
            );
        }
        let ppw: DMatrix<f64> = DMatrix::from_vec(ppw_vec.len() / 24, 24, ppw_vec).transpose();
        let daily_rates: DMatrix<f64> =
            DMatrix::from_vec(daily_rates_vec.len() / 24, 24, discharge_rates_vec).transpose();
        // Ensure unified keys for solar and batteries
        Ok(Self {
            date_to_index,
            daily_rates,
            ppw,
            discharge_rates,
            batteries: batteries_map,
            solar: solar_map,
            keys,
        })
    }

    pub fn update_power_component(
        &mut self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<()> {
        todo!();
    }
    pub async fn estimate_revenue(
    &mut self,
    btm_rate_param: Option<f64>,
    btm_ppw_param: Option<f64>,
    ftm_rates_param: Vec<(f64, f64)>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    sim_type: usize,
) -> Result<
    (
        &HashMap<chrono::DateTime<Utc>, (usize, usize)>,
        HashMap<String, (BatteryStorage, f64)>,
        &DMatrix<f64>,//ftm_rate
        DMatrix<f64>,//btm_rate
        &DMatrix<f64>,//ftm_ppw
        DMatrix<f64>,//btm_ppw
    ),
    ServerFnError,
> {
    let dr = daily_rates_process(
            self.date_to_index.clone(),
            Some(0.3),
            ftm_rates_param.clone(),
            true,
        )
        .await?;
            // Optionally create btm_rate and btm_ppw matrices
    let (rows, cols) = self.daily_rates.shape();
    self.daily_rates = dr.clone();
    let btm_rate_mat = DMatrix::from_element(rows, cols, btm_rate_param.unwrap_or(0.0));
    let btm_ppw_mat = DMatrix::from_element(rows, cols, btm_ppw_param.unwrap_or(0.0));

    let mut batt_map: HashMap<String, (BatteryStorage, f64)> = HashMap::new();
    for key in &self.keys {
        println!("{:?}", key);
        let solar_vec: Vec<PvSystem> = self.solar.get(key).unwrap().clone();
        let batt_set_vec: Vec<BatteryStorage> = self.batteries.get(key).unwrap().clone();

        let mut solar_power_component_vec: Vec<PowerComponent> = Vec::new();
        for ts in solar_vec.iter() {
            solar_power_component_vec
                .push(ts.clone().into_power_component(start_date, end_date).await);
        }
        let solar_power_component =
            PowerComponent::merge_power_components(solar_power_component_vec, 1.0).await;
        let battery = BatteryStorage::merge_battery_storage(batt_set_vec);
        println!("battery: {:?}", battery);
        // Discharge simulation depending on type
        let (discharge_out, profit) = match sim_type {
            1 => {
                let (discharge, charge, profit) = Self::discharge_process_no_gen(
                    self.date_to_index.clone(),
                    self.daily_rates.clone(),
                    battery.clone(),
                ).await.unwrap();
                ((Some(discharge), Some(charge), None), profit)
            }
            _ => {
                let (discharge, charge_ftm, charge_btm, profit) = Self::discharge_process_gen_and_grid(
                    self.date_to_index.clone(),
                    self.daily_rates.clone(),
                    btm_ppw_mat.clone(),
                    battery.clone(),
                    solar_power_component.demand.clone(),
                ).await.unwrap();
                ((Some(discharge), Some(charge_ftm), Some(charge_btm)), profit)
            }
        };

        // Discharge-to-power conversion
        let (btm_comp, ftm_comp) = match sim_type {
            1 => {
                Self::discharge_rates_to_power_component(
                    self.date_to_index.clone(),
                    None,
                    None,
                    discharge_out.0.clone(),
                    discharge_out.1.clone(),
                )
                .await
                .unwrap()
            }
            _ => {
                Self::discharge_rates_to_power_component(
                    self.date_to_index.clone(),
                    None,
                    discharge_out.2.clone(),
                    discharge_out.0.clone(),
                    discharge_out.1.clone(),
                )
                .await
                .unwrap()
            }
        };

        println!("Power Component: (BTM: {:?}, FTM: {:?})", btm_comp, ftm_comp);

        // Compose battery state update with new per-type power signals
        batt_map.insert(
            key.to_string(),
            (
                battery
                    .update_power_operator_component(
                        ftm_comp.input_power_w_ts.clone(),
                        ftm_comp.output_power_w_ts.clone(),
                        btm_comp.input_power_w_ts.clone(),
                        btm_comp.output_power_w_ts.clone(),
                    )
                    .await,
                profit,
            ),
        );
    }
    Ok((&self.date_to_index, batt_map, &self.daily_rates, btm_rate_mat, &self.daily_rates, btm_ppw_mat))
}


    fn daily_rates_fun(curr_time: DateTime<Utc>) -> Result<(DateTime<Utc>, f64), ServerFnError> {
        let intercept = 0.5;
        let hour_coeff = 0.1;
        let hour_squared_coeff = 0.02;
        let weekday_coeff = 0.2;
        let month_coeff = 0.05;

        let hour = curr_time.hour() as f64;
        let hour_squared = hour.powi(2);
        let weekday = curr_time.weekday().num_days_from_monday() as f64;
        let month = curr_time.month() as f64;

        let daily_rate = intercept
            + hour_coeff * hour
            + hour_squared_coeff * hour_squared
            + weekday_coeff * weekday
            + month_coeff * month;

        Ok((curr_time, daily_rate))
    }

    fn ppw_fun(curr_time: DateTime<Utc>) -> Result<(DateTime<Utc>, f64), ServerFnError> {
        let ppw = match curr_time.month() {
            1..=3 => 0.08,
            4..=6 => 0.08,
            7..=9 => 0.08,
            10..=12 => 0.08,
            _ => unreachable!(),
        };
        Ok((curr_time, ppw))
    }

    fn discharge_rate_fun(
        curr_time: DateTime<Utc>,
        batt_charge_pct: f64,
    ) -> Result<(DateTime<Utc>, f64), ServerFnError> {
        let weekday_rates = vec![
            0.05, 0.04, 0.03, 0.02, 0.02, 0.03, 0.05, 0.08, 0.10, 0.12, 0.15, 0.18, 0.20, 0.22,
            0.25, 0.28, 0.18, 0.32, 0.35, 0.38, 0.40, 0.42, 0.45, 0.50,
        ];
        let weekend_rates = vec![
            0.04, 0.03, 0.02, 0.01, 0.01, 0.02, 0.04, 0.06, 0.08, 0.10, 0.12, 0.15, 0.18, 0.20,
            0.22, 0.24, 0.25, 0.28, 0.30, 0.32, 0.35, 0.38, 0.40, 0.45,
        ];

        let is_weekend = matches!(
            curr_time.weekday(),
            chrono::Weekday::Sat | chrono::Weekday::Sun
        );
        let hour = curr_time.hour() as usize;

        let discharge_rate: f32 = if is_weekend {
            weekend_rates[hour]
        } else {
            weekday_rates[hour]
        };

        // Adjust based on battery charge percentage
        let adjusted_rate: f32 = discharge_rate.min(batt_charge_pct as f32);
        Ok((curr_time, adjusted_rate.into()))
    }

    pub async fn discharge_process_no_gen(
        date_to_index: HashMap<DateTime<Utc>, (usize, usize)>,
        rates: DMatrix<f64>,
        battery: BatteryStorage,
    ) -> anyhow::Result<(DMatrix<f64>, DMatrix<f64>, f64), ServerFnError> {
        let (max_col, max_row) = date_to_index
            .values()
            .fold((0, 0), |acc, &(r, c)| (acc.0.max(r), acc.1.max(c)));
        let row_dim = max_row + 1;
        let col_dim = max_col + 1;
        // For safety, check that `rates` and `ppw` match these dimensions
        assert_eq!(rates.nrows(), row_dim);
        assert_eq!(rates.ncols(), col_dim);
        println!("row_dim: {}  col_dim: {}", row_dim, col_dim);
        // 3) Create battery_storage_matrix
        let mut battery_storage_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        if let Some(storage_ts) = &battery.battery_state.storage {
            for &(dt, val) in storage_ts {
                if let Some(&(c, r)) = date_to_index.get(&dt) {
                    battery_storage_matrix[(r, c)] = val as f64;
                }
            }
        }
        // -------------------------------------------------------------------
        // B) Build a linear program with good_lp to find optimal discharge
        // -------------------------------------------------------------------
        let total_elements = row_dim * col_dim;
        // Some definitions for SoC bounds
        let total_capacity: f64 = (battery.capacity * battery.watt_hours) as f64;
        let min_soc = (battery.depth_of_discharge as f64) * total_capacity;
        let max_soc = total_capacity;
        let initial_soc = min_soc;
        // 1) Create problem variables (D, C, S).
        let mut vars = ProblemVariables::new();
        let d_mat = DMatrix::from_vec(
            row_dim,
            col_dim,
            vars.add_vector(variable().min(0.0).max(battery.capacity), total_elements),
        );
        let c_mat = DMatrix::from_vec(
            row_dim,
            col_dim,
            vars.add_vector(variable().min(0.0).max(battery.capacity), total_elements),
        );
        let s_mat = DMatrix::from_vec(
            row_dim,
            col_dim,
            vars.add_vector(variable().min(min_soc).max(max_soc), total_elements),
        );
        // 2) We build the model in "minimize" form. We'll build the objective
        let mut objective: Vec<Expression> = Vec::new();
        for (_i, (_dt, (c, r))) in date_to_index.iter().enumerate() {
            let rate = rates[(*r, *c)];
            objective.push(-(d_mat[(*r, *c)] * rate / 1000 - c_mat[(*r, *c)] * rate / 1000));
        }
        let mut model = vars
            .minimise(objective.iter().sum::<Expression>())
            .using(default_solver);
        // 4) Constraints (example battery logic):
        // (a) S_{t+1} = S_{t} + eff_charge*C_t - (1/eff_discharge)*D_t
        let mut sorted_times: Vec<_> = date_to_index.iter().collect();
        sorted_times.sort_by_key(|&(dt, _)| *dt);
        if let Some(&(dt, (c0, r0))) = sorted_times.first() {
            println!("min date: {:?}", dt);
            model = model.with(constraint!(s_mat[(*r0, *c0)] == initial_soc));
        }
        for window in sorted_times.windows(2) {
            let (_, &(c1, r1)) = window[0];
            let (_, &(c2, r2)) = window[1];
            model = model.with(constraint!(
                s_mat[(r2, c2)]
                    == s_mat[(r1, c1)] + (battery.efficiency_charge as f64) * c_mat[(r1, c1)]
                        - (1.0 / (battery.efficiency_discharge as f64)) * d_mat[(r1, c1)]
            ));
            model = model.with(constraint!(d_mat[(r1, c1)] <= s_mat[(r1, c1)] - min_soc));
            model = model.with(constraint!(s_mat[(r2, c2)] >= min_soc));
            model = model.with(constraint!(s_mat[(r1, c1)] >= min_soc));
            model = model.with(constraint!(c_mat[(r1, c1)] <= battery.capacity));
        }
        if let Some(&(dt, (c0, r0))) = sorted_times.last() {
            println!("max date: {:?}", dt);
            model = model.with(constraint!(
                s_mat[(*r0, *c0)] + (battery.efficiency_charge as f64) * c_mat[(*r0, *c0)]
                    - (1.0 / (battery.efficiency_discharge as f64)) * d_mat[(*r0, *c0)]
                    >= min_soc
            ));
        }
        // 5) Solve
        let solution = model.solve().expect("Failed to solve LP");
        let objective_expr = objective.iter().sum::<Expression>();
        let minimized_val = solution.eval(&objective_expr);
        let profit = -minimized_val;
        // -------------------------------------------------------------------
        // C) Extract the solver results
        // -------------------------------------------------------------------
        // Create matrices to store the charge and discharge results
        let mut discharge_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        let mut charge_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        let mut storage_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        // Fill them with solved values
        for (_i, (_dt, (c, r))) in date_to_index.iter().enumerate() {
            discharge_matrix[(*r, *c)] = solution.value(d_mat[(*r, *c)]);
            charge_matrix[(*r, *c)] = solution.value(c_mat[(*r, *c)]);
            storage_matrix[(*r, *c)] = solution.value(s_mat[(*r, *c)]);
        }
        println!(
            "battery total size:{}",
            ((battery.capacity) * (battery.watt_hours))
        );
        println!("{:?}", discharge_matrix);
        println!("{:?}", charge_matrix);
        println!("{:?}", storage_matrix);
        // Return the solver's solution for now.
        Ok((discharge_matrix, charge_matrix, profit))
    }

    pub async fn discharge_process_gen_and_grid(
        date_to_index: HashMap<DateTime<Utc>, (usize, usize)>,
        rates: DMatrix<f64>,
        ppw_fixed: DMatrix<f64>,
        battery: BatteryStorage,
        generator_power_comp: PowerComponent,
    ) -> anyhow::Result<(DMatrix<f64>, DMatrix<f64>, DMatrix<f64>, f64), ServerFnError> {
        let (max_col, max_row) = date_to_index
            .values()
            .fold((0, 0), |acc, &(r, c)| (acc.0.max(r), acc.1.max(c)));
        let row_dim = max_row + 1;
        let col_dim = max_col + 1;
        // For safety, check that `rates` and `ppw` match these dimensions
        assert_eq!(rates.nrows(), row_dim);
        assert_eq!(rates.ncols(), col_dim);
        assert_eq!(ppw_fixed.nrows(), row_dim);
        assert_eq!(ppw_fixed.ncols(), col_dim);
        println!("row_dim: {}  col_dim: {}", row_dim, col_dim);
        // 2) Create generator_output_matrix
        let mut generator_output_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        let gen_output_ts = generator_power_comp.output_power_w_ts.unwrap_or_default();
        for (dt, val) in gen_output_ts {
            if let Some(&(c, r)) = date_to_index.get(&dt) {
                generator_output_matrix[(r, c)] = val as f64;
            }
        }

        // 3) Create battery_storage_matrix
        let mut battery_storage_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        if let Some(storage_ts) = &battery.battery_state.storage {
            for &(dt, val) in storage_ts {
                if let Some(&(c, r)) = date_to_index.get(&dt) {
                    battery_storage_matrix[(r, c)] = val as f64;
                }
            }
        }

        // -------------------------------------------------------------------
        // B) Build a linear program with good_lp to find optimal discharge
        // -------------------------------------------------------------------

        // Flatten row_dim x col_dim into T = row_dim * col_dim
        let total_elements = row_dim * col_dim;
        // Some definitions for SoC bounds
        let total_capacity: f64 = (battery.capacity * battery.watt_hours) as f64;
        let min_soc = (battery.depth_of_discharge as f64) * total_capacity; // e.g. must stay above DoD fraction
        let max_soc = total_capacity; // full capacity
        let initial_soc = min_soc; // or battery_state.storage[0], if defined
        println!("Min Charge: {:?}", min_soc);
        println!("Max Charge: {:?}", max_soc);
        // 1) Create problem variables (D, C, S).
        let mut vars = ProblemVariables::new();
        let d_mat = DMatrix::from_vec(
            row_dim,
            col_dim,
            vars.add_vector(variable().min(0.0).max(battery.capacity), total_elements),
        );
        let c_ftm_mat = DMatrix::from_vec(
            row_dim,
            col_dim,
            vars.add_vector(variable().min(0.0).max(battery.capacity), total_elements),
        );
        let c_btm_mat = DMatrix::from_vec(
            row_dim,
            col_dim,
            vars.add_vector(variable().min(0.0).max(battery.capacity), total_elements),
        );
        let s_mat = DMatrix::from_vec(
            row_dim,
            col_dim,
            vars.add_vector(variable().min(0.0).max(max_soc), total_elements),
        );
        // 2) We build the model in "minimize" form. We'll build the objective
        //    as -(d * rate) + (c * price) to emulate "maximize d*rate - c*price".
        //    // 3) Build the objective by iterating over all cells in row-major order
        let mut objective: Vec<Expression> = Vec::new();
        for (_i, (_dt, (c, r))) in date_to_index.iter().enumerate() {
            let rate = rates[(*r, *c)];
            let price_fixed = ppw_fixed[(*r, *c)];
            objective.push(
                -(d_mat[(*r, *c)] * rate / 1000
                    - c_btm_mat[(*r, *c)] * price_fixed / 1000
                    - c_ftm_mat[(*r, *c)] * rate / 1000),
            );
        }
        let mut model = vars
            .minimise(objective.iter().sum::<Expression>())
            .using(default_solver);

        // 4) Constraints (example battery logic):
        //
        // (a) S_{t+1} = S_{t} + eff_charge*C_t - (1/eff_discharge)*D_t
        // Because we have T = row_dim*col_dim, we need to define an ordering for timeslots.
        // For demonstration, we assume each cell is sequential in time. If you want
        // 2D "days x hours", you'd define your own adjacency for "t+1".
        //
        // We'll do a naive linear chain for i in [0..(T-2)].
        //s_mat: storage_matrix
        //c_mat: charge matrix
        //d_mat: is the discharge_matrix
        //minimum charge
        let mut sorted_times: Vec<_> = date_to_index.iter().collect();
        sorted_times.sort_by_key(|&(dt, _)| *dt);
        if let Some(&(dt, (c0, r0))) = sorted_times.first() {
            println!("min date: {:?}", dt);
            model = model.with(constraint!(s_mat[(*r0, *c0)] == initial_soc));
        }
        for window in sorted_times.windows(2) {
            let (_, &(c1, r1)) = window[0];
            let (_, &(c2, r2)) = window[1];
            model = model.with(constraint!(
                s_mat[(r2, c2)]
                    == s_mat[(r1, c1)] + (battery.efficiency_charge as f64) * c_btm_mat[(r1, c1)]
                        + (battery.efficiency_charge as f64) * c_ftm_mat[(r1, c1)]
                        - (1.0 / (battery.efficiency_discharge as f64)) * d_mat[(r1, c1)]
            ));
            model = model.with(constraint!(d_mat[(r1, c1)] <= s_mat[(r1, c1)] - min_soc));
            model = model.with(constraint!(s_mat[(r1, c1)] >= min_soc));
            model = model.with(constraint!(s_mat[(r2, c2)] >= min_soc));
            model = model.with(constraint!(
                c_btm_mat[(r1, c1)] >= 0.0
            ));
            model = model.with(constraint!(
                c_btm_mat[(r1, c1)] <= generator_output_matrix[(r1, c1)]
            ));
            model = model.with(constraint!(
                c_ftm_mat[(r1, c1)] >= 0.0
            ));
            model = model.with(constraint!(
                c_ftm_mat[(r1, c1)] + c_btm_mat[(r1, c1)] <= battery.capacity
            ));
        }
        if let Some(&(dt, (c0, r0))) = sorted_times.last() {
            println!("max date: {:?}", dt);
            model = model.with(constraint!(
                s_mat[(*r0, *c0)] + (battery.efficiency_charge as f64) * c_btm_mat[(*r0, *c0)]
                    + (battery.efficiency_charge as f64) * c_ftm_mat[(*r0, *c0)]
                    - (1.0 / (battery.efficiency_discharge as f64)) * d_mat[(*r0, *c0)]
                    >= min_soc
            ));
        }
        // 5) Solve
        let solution = model.solve().expect("Failed to solve LP");
        let objective_expr = objective.iter().sum::<Expression>();
        let minimized_val = solution.eval(&objective_expr);
        let profit = -minimized_val;
        // -------------------------------------------------------------------
        // C) Extract the solver resultinto a `battery_output_matrix`.
        //    This matrix has the same (row_dim x col_dim) shape and
        //    is stored according to your date_to_index mapping.
        // -------------------------------------------------------------------
        // Create matrices to store the charge and discharge results
        let mut discharge_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        let mut charge_ftm_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        let mut charge_btm_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        let mut storage_matrix = DMatrix::<f64>::zeros(row_dim, col_dim);
        // Fill them with solved values
        for (_i, (_dt, (c, r))) in date_to_index.iter().enumerate() {
            discharge_matrix[(*r, *c)] = solution.value(d_mat[(*r, *c)]);
            charge_ftm_matrix[(*r, *c)] = solution.value(c_ftm_mat[(*r, *c)]);
            charge_btm_matrix[(*r, *c)] = solution.value(c_btm_mat[(*r, *c)]);
            storage_matrix[(*r, *c)] = solution.value(s_mat[(*r, *c)]);
        }
        println!(
            "battery total size:{}",
            ((battery.capacity) * (battery.watt_hours))
        );
        println!("Discharge Matrix: {:?}", discharge_matrix);
        println!("Charge FTM Matrix: {:?}", charge_ftm_matrix);
        println!("Charge BTM Matrix: {:?}", charge_btm_matrix);
        println!("Storage Matrix: {:?}", storage_matrix);
        // Return the solver's solution for now.
        Ok((
            discharge_matrix,
            charge_ftm_matrix,
            charge_btm_matrix,
            profit,
        ))
    }

    pub async fn discharge_rates_to_power_component(
        date_to_index: HashMap<DateTime<Utc>, (usize, usize)>,
        discharge_btm: Option<DMatrix<f64>>,
        charge_btm: Option<DMatrix<f64>>,
        discharge_ftm: Option<DMatrix<f64>>,
        charge_ftm: Option<DMatrix<f64>>,
    ) -> Result<(PowerComponent, PowerComponent), anyhow::Error> {
        let mut output_ts_ftm = Vec::with_capacity(date_to_index.len());
        let mut input_ts_ftm = Vec::with_capacity(date_to_index.len());
        let mut output_ts_btm = Vec::with_capacity(date_to_index.len());
        let mut input_ts_btm = Vec::with_capacity(date_to_index.len());

        for (dt, (c, r)) in &date_to_index {
            // BTM
            let discharge_btm_val = discharge_btm
                .as_ref()
                .map(|m| m[(*r, *c)] as f32)
                .unwrap_or(0.0);
            let charge_btm_val = charge_btm
                .as_ref()
                .map(|m| m[(*r, *c)] as f32)
                .unwrap_or(0.0);
            output_ts_btm.push((*dt, discharge_btm_val));
            input_ts_btm.push((*dt, charge_btm_val));

            // FTM
            let discharge_ftm_val = discharge_ftm
                .as_ref()
                .map(|m| m[(*r, *c)] as f32)
                .unwrap_or(0.0);
            let charge_ftm_val = charge_ftm
                .as_ref()
                .map(|m| m[(*r, *c)] as f32)
                .unwrap_or(0.0);
            output_ts_ftm.push((*dt, discharge_ftm_val));
            input_ts_ftm.push((*dt, charge_ftm_val));
        }

        let btm_component = PowerComponent::new_ts(Some(input_ts_btm), Some(output_ts_btm)).await;
        let ftm_component = PowerComponent::new_ts(Some(input_ts_ftm), Some(output_ts_ftm)).await;

        Ok((btm_component, ftm_component))
    }
}
