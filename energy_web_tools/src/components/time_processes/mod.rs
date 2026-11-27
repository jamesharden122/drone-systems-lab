pub mod sample_rates;

use chrono::prelude::*;
#[cfg(feature = "server")]
use chrono::{Date, DateTime, NaiveDate, TimeDelta, Utc};
use dioxus::prelude::*;
use dioxus::prelude::*;
#[cfg(feature = "server")]
use nalgebra::base::DMatrix;
#[cfg(feature = "server")]
use rand::prelude::*;
#[cfg(feature = "server")]
use rand_distr::{Distribution, Normal, Poisson};
use std::collections::{HashMap, HashSet};
#[cfg(feature = "server")]
use tokio::task::spawn_blocking;

#[cfg(feature = "server")]
fn to_utc_date_time<Tz: TimeZone>(dt: DateTime<Tz>) -> DateTime<Utc> {
    dt.with_timezone(&Utc)
}

/// Creates normal distributions for the simulation
#[cfg(feature = "server")]
fn create_normal_distributions(
    delta_t: f64,
    params: Vec<(f64, f64)>,
) -> Result<Vec<Normal<f64>>, ServerFnError> {
    let step_size = delta_t.sqrt();
    //Returns vector of normal distributions based on step size and length of parameter vector
    let distributions: Vec<Normal<f64>> = params
        .iter()
        .map(|(mu, sigma)| {
            Normal::new(mu * delta_t, sigma * step_size).expect("Invalid parameters")
        })
        .collect();

    Ok(distributions)
}

/// Simulates energy rates asynchronously using the given normal distributions
#[cfg(feature = "server")]
pub async fn simulate_energy_rates_from_normals(
    params: Vec<(f64, f64)>,
    delta_time: f64,
    mut initial_rate: Option<f64>,
    iid: bool,
    days_map: HashMap<DateTime<Utc>, (usize, usize)>,
) -> Result<HashMap<DateTime<Utc>, f64>, ServerFnError> {
    let normal_distributions: Vec<Normal<f64>> = create_normal_distributions(delta_time, params)?;
    println!("{:?}", normal_distributions.clone());
    let mut rng = thread_rng();
    let mut rates: HashMap<DateTime<Utc>, f64> = HashMap::new();
    let mut days: Vec<_> = days_map
        .keys()
        .cloned()
        .map(|dt| dt.date_naive())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    days.sort();

    //loop for updating rates matrix
    for day in days {
        let random_increments: Vec<f64> = normal_distributions
            .iter()
            .map(|dist| dist.sample(&mut rng))
            .collect::<Vec<f64>>();
        if iid == true {
            for (ind, inc) in random_increments.iter().enumerate() {
                let naive_datetime =
                    day.and_time(NaiveTime::from_hms(ind.try_into().unwrap(), 0, 0));
                let datetime_utc = DateTime::<Utc>::from_utc(naive_datetime, Utc);
                rates.insert(datetime_utc, (*inc).max(0.0));
                //println!("{},{}", initial_rate, inc);
            }
        } else {
            for (ind, inc) in random_increments.iter().enumerate() {
                let naive_datetime =
                    day.and_time(NaiveTime::from_hms(ind.try_into().unwrap(), 0, 0));
                let datetime_utc = DateTime::<Utc>::from_utc(naive_datetime, Utc);
                rates.insert(datetime_utc, initial_rate.unwrap_or(0.0));
                //println!("{},{}", initial_rate, inc);
                initial_rate = Some(initial_rate.unwrap_or(0.0) + inc);
            }
        }
    }
    Ok(rates)
}

#[cfg(feature = "server")]
fn create_poisson_distributions(lambdas: Vec<f64>) -> Result<Vec<Poisson<f64>>, ServerFnError> {
    Ok(lambdas
        .into_iter()
        .map(|lambda| Poisson::new(lambda).expect("Invalid lambda"))
        .collect())
}

#[cfg(feature = "server")]
pub async fn sample_from_poisson_distributions(
    poisson_distributions: Vec<Poisson<f64>>,
    time_steps: i32,
) -> Result<Vec<f64>, ServerFnError> {
    // Use `spawn_blocking` to move this CPU-heavy task to a separate thread
    spawn_blocking(move || {
        let mut rng = thread_rng();
        let mut pois_full_sample: Vec<f64> = Vec::new();
        let mut count = 1;
        while count <= time_steps {
            pois_full_sample.append(
                &mut poisson_distributions
                    .iter()
                    .map(|dist| dist.sample(&mut rng) / 60.0) // Sample from each distribution
                    .collect::<Vec<f64>>(),
            );
            count += 1;
        }
        Ok(pois_full_sample)
    })
    .await?
}

#[cfg(feature = "server")]
pub async fn modulated_markov_poison_process(
    d1: DateTime<Utc>,
    d2: DateTime<Utc>,
    mut lambda: Vec<f64>,
) -> Result<
    (
        nalgebra::base::DMatrix<f64>,
        nalgebra::base::DMatrix<f64>,
        nalgebra::base::DMatrix<DateTime<Utc>>,
    ),
    ServerFnError,
> {
    println!(
        "Lambda rate is: {}",
        lambda.clone().into_iter().sum::<f64>()
    );

    // Create time steps and day hours using blocking tasks
    let (times, day_hours) = spawn_blocking(move || {
        let mut times: Vec<f64> = Vec::new();
        let mut day_hours = Vec::new();
        let mut curr_time = d1;

        while curr_time < d2 {
            day_hours.push(to_utc_date_time(curr_time));
            if curr_time <= (d1 + TimeDelta::hours(23)) {
                times.push(curr_time.hour() as f64);
            }
            curr_time += TimeDelta::hours(1);
        }
        (times, day_hours)
    })
    .await?;

    let time_steps = day_hours.len() as i32;

    // Create Poisson distributions
    let poisson_dist_vec: Vec<Poisson<f64>> = create_poisson_distributions(lambda.clone())?;

    // Sample from Poisson distributions asynchronously
    let poisson_sample =
        sample_from_poisson_distributions(poisson_dist_vec, time_steps / 24).await?;

    // Assemble matrices using blocking tasks
    let (parameter_matrix, data_matrix, days_matrix) = spawn_blocking(move || {
        let mut matrix = times;
        matrix.append(&mut lambda);

        let parameter_matrix: DMatrix<f64> = DMatrix::from_vec(24, 2, matrix);
        let data_matrix: DMatrix<f64> =
            DMatrix::from_vec(24, (time_steps / 24).try_into().unwrap(), poisson_sample);
        let days_matrix: DMatrix<DateTime<Utc>> =
            DMatrix::from_vec(24, (time_steps / 24).try_into().unwrap(), day_hours);
        (parameter_matrix, data_matrix, days_matrix)
    })
    .await?;

    Ok((data_matrix, parameter_matrix, days_matrix))
}

#[cfg(feature = "server")]
pub async fn daily_rates_process(
    date_map: HashMap<DateTime<Utc>, (usize, usize)>,
    initial_rate: Option<f64>,
    params: Vec<(f64, f64)>,
    iid: bool
) -> Result<nalgebra::base::DMatrix<f64>, ServerFnError> {
    let len_days = date_map
        .keys()
        .cloned()
        .map(|dt| dt.date_naive())
        .collect::<HashSet<NaiveDate>>()
        .len();
    println!("{:?}", len_days);
    let mut rates_matrix = DMatrix::<f64>::from_element(24, len_days, 0.0);
    println!("{:?}", rates_matrix.shape());
    let delta_time: f64 = 1.0; // 8760.0;
    let daily_rates = simulate_energy_rates_from_normals(
        params,
        delta_time,
        initial_rate,
        iid,
        date_map.clone(),
    )
    .await?;
    for (date, (cols, rows)) in date_map.iter() {
        println!(
            "Date: {}, Row: {}, Col: {},Rate: {}",
            date,
            rows,
            cols,
            daily_rates.get(date).unwrap_or(&0.0)
        );
        rates_matrix[(*rows, *cols)] = *daily_rates.get(date).unwrap();
    }

    Ok(rates_matrix)
}
