#[cfg(feature = "server")]
pub mod pv_base_system {
    use crate::components::energy_components::general_fun::PowerComponent;
    use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
    use polars::prelude::*;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;

    #[derive(Clone, Debug)]
    pub struct SolarParams {
        pub irradiance: DataFrame,
    }

    #[derive(Clone, Debug)]
    pub struct PvSystem {
        pub num_panels: f32,
        pub panel_watts: f32,
        pub panel_cost: f32,
        pub transformer_efficiency: f32,
        pub panel_width: f32,
        pub panel_length: f32,
        pub params: SolarParams,
    }

    impl PvSystem {
        /// Calculates the solar generation based on irradiance and other factors.
        /// This method would encapsulate the logic for calculating solar power generation.        ///
        /// Creates a new instance of the system with given parameters.
        pub fn new(
            num_panels: f32,
            panel_watts: f32,
            transformer_efficiency: f32,
            panel_width: f32,
            panel_length: f32,
            params: &str,
        ) -> Self {
            let _delta_time = 1.0;
            let mut reader = PathBuf::new();
            let mut schema = Schema::with_capacity(9);
            reader.push(params);
            schema.with_column(
                "index".into(),
                DataType::Datetime(datatypes::TimeUnit::Milliseconds, None),
            );
            schema.with_column(
                "date".into(),
                DataType::Datetime(datatypes::TimeUnit::Milliseconds, None),
            );
            schema.with_column("year".into(), DataType::Int32);
            schema.with_column("month".into(), DataType::Int32);
            schema.with_column("day".into(), DataType::Int32);
            schema.with_column("hour".into(), DataType::Int32);
            schema.with_column("minute".into(), DataType::Int32);
            schema.with_column("kt".into(), DataType::Float32);
            schema.with_column("fd".into(), DataType::Float32);
            schema.with_column("g0".into(), DataType::Float32);
            schema.with_column("d0".into(), DataType::Float32);
            schema.with_column("b0".into(), DataType::Float32);
            schema.with_column("b_inc".into(), DataType::Float32);
            schema.with_column("d_inc".into(), DataType::Float32);
            schema.with_column("r_inc".into(), DataType::Float32);
            schema.with_column("g_inc".into(), DataType::Float32);

            // Read the CSV file into a DataFrame
        let irradiance_records = CsvReadOptions::default()
        .with_has_header(true)
        .with_schema(Some(schema.into()))
        .try_into_reader_with_file_path(Some(reader))
        .unwrap()
        .finish()
        .unwrap();
            //let irradiance_records = irradiance_records.rename("index", "date".into()).unwrap()
            // Print the DaStaFrame to check it's loaded correctly
            //println!("{:?}", irradiance_records);
            Self {
                num_panels,
                panel_watts,
                panel_cost: 0.0,
                transformer_efficiency,
                panel_width,
                panel_length,
                params: SolarParams {
                    irradiance: irradiance_records.to_owned(),
                },
            }
        }

        pub async fn into_power_component(
            self,
            date1: DateTime<Utc>,
            date2: DateTime<Utc>,
        ) -> PowerComponent {
            fn naive_to_utc(naive_datetime: NaiveDateTime) -> DateTime<Utc> {
                DateTime::from_naive_utc_and_offset(naive_datetime, Utc)
            }
            //println!("{:?}", self.params.irradiance.get_column_names());
            let ranged_df = self
                .params
                .irradiance
                .clone()
                .lazy()
                .filter(
                    col("date")
                        .gt(lit(date1.naive_utc()))
                        .and(col("date").lt(lit(date2.naive_utc()))),
                )
                .collect()
                .unwrap();
            println!("{:?}", ranged_df.clone());
            let date_column = ranged_df.column("date").unwrap();
            let mpp_column = ranged_df.column("g_inc").unwrap().f32().unwrap().to_vec();
            let input_power_w_ts: Vec<(DateTime<Utc>, f32)> = Vec::new();
            let output_power_w_ts: Vec<(DateTime<Utc>, f32)> = mpp_column
                .iter()
                .zip(date_column.datetime().unwrap().as_datetime_iter())
                .map(|(x, y)| {
                    (
                        naive_to_utc(y.unwrap()),
                        x.unwrap() / 1000.0 * self.num_panels * self.panel_watts,
                    )
                })
                .collect::<Vec<(DateTime<Utc>, f32)>>();
            PowerComponent {
                input_power_w_ts: Some(input_power_w_ts),
                output_power_w_ts: Some(output_power_w_ts),
            }
        }
    }
}
