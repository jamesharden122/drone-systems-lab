use crate::components::energy_components::general_fun::PowerComponent;
use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)] // Ensure Charger can be cloned/copied
pub struct Charger {
    pub maximum_power_w: f32,
    pub output_voltage_min_vdc: f32,
    pub output_voltage_max_vdc: f32,
    pub max_output_current_a: f32,
    pub input_voltage_vac: f32,
    pub input_frequency_hz: u16,
    pub fla_a: f32,
    pub breaker_rating_a: u16,
    pub rated_power_kva: f32,
    pub power_factor_at_full_load: f32,
    pub efficiency_at_nominal_power: f32,
    pub power: PowerComponent,
}

impl Charger {
    pub fn new(
        maximum_power_w: f32,
        output_voltage_min_vdc: f32,
        output_voltage_max_vdc: f32,
        max_output_current_a: f32,
        input_voltage_vac: f32,
        input_frequency_hz: u16,
        fla_a: f32,
        breaker_rating_a: u16,
        rated_power_kva: f32,
        power_factor_at_full_load: f32,
        efficiency_at_nominal_power: f32,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            maximum_power_w,
            output_voltage_min_vdc,
            output_voltage_max_vdc,
            max_output_current_a,
            input_voltage_vac,
            input_frequency_hz,
            fla_a,
            breaker_rating_a,
            rated_power_kva,
            power_factor_at_full_load,
            efficiency_at_nominal_power,
            power: PowerComponent {
                input_power_w_ts: Some(vec![(now, 0.0)]),
                output_power_w_ts: Some(vec![(now, 0.0)]),
            },
        }
    }

    // Example method to add input power with timestamp
    #[cfg(feature = "server")]
    pub async fn add_input_power_ts(
        mut charger: Charger,
        d1: DateTime<Utc>,
        d2: DateTime<Utc>,
    ) -> Result<Charger, ServerFnError> {
        let input_power_w = PowerComponent::power(
            charger.input_voltage_vac,
            charger.fla_a,
            charger.power_factor_at_full_load,
        )
        .await
        .unwrap();
        //println!("{:?}",output_power_w);
        //instantiate dates and energy inputs and outputs
        let mut curr_time = d1;
        let mut input_power_w_ts: Vec<(DateTime<Utc>, f32)> = Vec::new();
        let mut output_power_w_ts: Vec<(DateTime<Utc>, f32)> = Vec::new();

        loop {
            if curr_time == d2 {
                break;
            }
            input_power_w_ts.push((curr_time, input_power_w));
            output_power_w_ts.push((
                curr_time,
                input_power_w * charger.efficiency_at_nominal_power,
            ));
            curr_time += TimeDelta::try_hours(1).unwrap();
        }
        charger.power.input_power_w_ts = Some(input_power_w_ts);
        charger.power.output_power_w_ts = Some(output_power_w_ts);
        Ok(charger)
    }
}
