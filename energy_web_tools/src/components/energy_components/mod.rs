pub mod batteries;
pub mod ev_chargers;
pub mod operator;
pub mod photovoltaic;

pub mod general_fun {
    use anyhow;
    use chrono::{DateTime, Utc};
    #[cfg(feature = "server")]
    use polars::frame::DataFrame;
    #[cfg(feature = "server")]
    use polars::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct EnergyConsumer {
        pub demand: PowerComponent,
        pub time_cycles: f32,
        pub total_input: f32,
        pub total_output: f32,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct PowerComponent {
        pub input_power_w_ts: Option<Vec<(DateTime<Utc>, f32)>>,
        pub output_power_w_ts: Option<Vec<(DateTime<Utc>, f32)>>,
    }

    #[cfg(feature = "server")]
    impl PowerComponent {
        fn new() -> Self {
            Self {
                input_power_w_ts: None,
                output_power_w_ts: None,
            }
        }

        pub async fn new_ts(
            input_power_w_ts: Option<Vec<(DateTime<Utc>, f32)>>,
            output_power_w_ts: Option<Vec<(DateTime<Utc>, f32)>>,
        ) -> Self {
            Self {
                input_power_w_ts,
                output_power_w_ts,
            }
        }

        pub async fn power(volts: f32, amps: f32, power_factor: f32) -> Result<f32, anyhow::Error> {
            let const_val: f32 = 3.0;
            Ok(const_val.sqrt() * volts * amps * power_factor)
        }

        pub async fn merge_power_components(
            components: Vec<PowerComponent>,
            time_cycle: f32,
        ) -> EnergyConsumer {
            let mut colnames_in = components[0]
                .clone()
                .input_power_w_ts
                .unwrap()
                .iter()
                .map(|(dt, _val)| *dt)
                .collect::<Vec<DateTime<Utc>>>();
            let mut colnames_out = components[0]
                .clone()
                .output_power_w_ts
                .unwrap()
                .iter()
                .map(|(dt, _val)| *dt)
                .collect::<Vec<DateTime<Utc>>>();
            let mut input_power_ts: Vec<Column> = Vec::new();
            let mut output_power_ts: Vec<Column> = Vec::new();
            for (i, component) in components.iter().enumerate() {
                input_power_ts.push(Column::new(
                    format!("component-{}", i).as_str().into(),
                    component
                        .clone()
                        .input_power_w_ts
                        .unwrap()
                        .iter()
                        .map(|(_dt, val)| *val)
                        .collect::<Vec<f32>>(),
                ));
                output_power_ts.push(Column::new(
                    format!("component-{}", i).as_str().into(),
                    component
                        .clone()
                        .output_power_w_ts
                        .unwrap()
                        .iter()
                        .map(|(_dt, val)| *val)
                        .collect::<Vec<f32>>(),
                ));
            }
            let df_input: DataFrame = DataFrame::new(input_power_ts).unwrap();
            let df_output: DataFrame = DataFrame::new(output_power_ts).unwrap();
            let rows_in = df_input.height();
            let cols_in = df_input.width();
            let rows_out = df_output.height();
            let cols_out = df_output.width();
            println!("Input Rows: {}", rows_in);
            println!("Input Cols: {}", cols_in);
            println!("Output Rows: {}", rows_out);
            println!("Output Cols: {}", cols_out);
            let mut sums_output: Vec<(DateTime<Utc>, f32)> = Vec::new();
            let mut sums_input: Vec<(DateTime<Utc>, f32)> = Vec::new();
            let total_input = 0.5;
            let total_ouput = 0.5;
            if df_input.clone().size() != 0 {
                let df_input = df_input.clone().transpose(
                    None,
                    Some(either::Either::Right(
                        colnames_in
                            .iter_mut()
                            .map(|nm| nm.to_string())
                            .collect::<Vec<String>>(),
                    )),
                );
                let mut i = 0;
                loop {
                    if i >= rows_out {
                        break;
                    }
                    let sum = df_input.clone().unwrap().get_columns()[i]
                        .sum_reduce()
                        .unwrap()
                        .value()
                        .clone();
                    sums_input.push((colnames_in[i], sum.try_extract::<f32>().unwrap()));
                    i += 1;
                }
            }
            if df_output.clone().size() != 0 {
                let df_output = df_output.clone().transpose(
                    None,
                    Some(either::Either::Right(
                        colnames_out
                            .iter_mut()
                            .map(|nm| nm.to_string())
                            .collect::<Vec<String>>(),
                    )),
                );
                let mut i = 0;
                loop {
                    if i >= rows_out {
                        break;
                    }
                    //println!("{:?}",i);
                    let sum = df_output.clone().unwrap().get_columns()[i]
                        .sum_reduce()
                        .unwrap()
                        .value()
                        .clone();
                    //println!("{:?}",sum);
                    sums_output.push((colnames_out[i], sum.try_extract::<f32>().unwrap()));
                    i += 1;
                }
            }
            //println!("df output");
            //println!("{:?}", df_output);
            //println!("df input");
            //println!("{:?}", df_input);
            if sums_input.is_empty() {
                let _ = sums_output
                    .clone()
                    .into_iter()
                    .map(|(date, _)| (date, 0.0))
                    .collect::<Vec<(DateTime<Utc>, f32)>>();
            }

            if sums_output.is_empty() {
                let _ = sums_input
                    .clone()
                    .into_iter()
                    .map(|(date, _)| (date, 0.0))
                    .collect::<Vec<(DateTime<Utc>, f32)>>();
            }
            //println!("{:?}", sums_input);
            //println!("{:?}", sums_output);
            EnergyConsumer {
                demand: PowerComponent {
                    input_power_w_ts: Some(sums_input),
                    output_power_w_ts: Some(sums_output),
                },
                time_cycles: time_cycle,
                total_input,
                total_output: total_ouput,
            }
        }
    }
}
