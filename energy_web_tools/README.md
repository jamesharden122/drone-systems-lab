# Energy Web Tools

Energy Web Tools is a Dioxus (Rust) full‑stack app for simulating and visualizing distributed energy systems:

- Photovoltaic generation from irradiance time series (CSV via Polars).
- Battery Energy Storage Systems (BESS) with charge/discharge and state‑of‑charge tracking.
- EV charger fleets with stochastic demand (modulated Markov–Poisson process).
- Economic optimization of discharge/charge schedules using linear programming (good_lp).
- In‑memory SurrealDB for quick, ephemeral persistence during simulations.


The Energy Web Tools test is referenced on the [Tesero Sol renewable energy consulting page](https://tesorosol.com/renewable-energy-consulting).

## Quick Start

1) Install prerequisites

- Rust (stable): https://rustup.rs
- Dioxus CLI: `cargo install dioxus-cli --locked`

2) Place irradiance data

- Put your irradiance CSV at `data/full_irradiance_data.csv` (repo ignores `data/`).
- See “Irradiance CSV Schema” below for required columns and types.

3) Run the app (web)

- UI only: `dx serve --platform web`
- Full stack (server functions, Polars, SurrealDB):

  `dx serve --platform web --features server`

4) Optional platforms

- Desktop: `dx serve --platform desktop [--features server]`


## Project Structure

```
energy_web_tools/
├─ assets/                      # Static assets (icons, CSS)
├─ data/                        # Local data (ignored by Git)
│  └─ full_irradiance_data.csv  # Required irradiance file (you provide)
├─ src/
│  ├─ main.rs                   # App entry + routes
│  ├─ views/                    # Page-level components
│  │  ├─ home.rs                # Tool selector page
│  ├─ components/
│  │  ├─ energy_components/     # Core energy models
│  │  │  ├─ photovoltaic.rs     # PvSystem (Polars CSV → power)
│  │  │  ├─ batteries.rs        # BatteryStorage + state update
│  │  │  ├─ ev_chargers.rs      # EV Charger model
│  │  │  ├─ operator.rs         # Optimizer (good_lp)
│  │  ├─ output_data_structs/   # Tables & charts utilities
│  │  ├─ surreal_data_structs/  # SurrealDB schema + helpers
│  │  ├─ time_processes/        # Stochastic processes (rates, Poisson)
│  │  ├─ simulation/            # Server functions for simulations
│  │  ├─ pv_charger_battery_sim.rs  # EV+PV+BESS UI form
│  │  ├─ pv_battery_operator_sim.rs # Operator (profit) UI form
├─ Cargo.toml                   # Features: web, desktop, server
├─ Dioxus.toml                  # Dioxus CLI configuration
└─ .gitignore                   # Keeps pushes lightweight
```


## Features Overview

- EV + Solar + BESS Simulation (UI: “⚡ EV + Solar + BESS Simulation”)
  - EV demand via modulated Markov–Poisson process (weekday/weekend lambdas, per hour).
  - PV generation from `data/full_irradiance_data.csv`.
  - Battery storage response and summary stats.

- Operator Simulation (UI: “📊 Forecasting Platform”)
  - Optimizes BESS discharge/charge against hourly market prices.
  - Supports “No Generation” or “Behind‑the‑Meter Generation”.
  - Produces per‑timestamp results and aggregate statistics.


## Running Modes

- Web UI only (faster compile): `dx serve --platform web`
- Full‑stack (enables server functions with Polars, SurrealDB, Tokio, etc.):

  `dx serve --platform web --features server`

Notes

- The “server” feature gates all heavy/async work (CSV reading, DB, optimization).
- Desktop builds also work: `dx serve --platform desktop --features server`.


## Irradiance CSV Schema

File path: `data/full_irradiance_data.csv`

The CSV must have a header row and at least these columns (Polars dtypes shown in parentheses):

- index: Datetime(Milliseconds)
- date: Datetime(Milliseconds)
- year: Int32
- month: Int32
- day: Int32
- hour: Int32
- minute: Int32
- kt: Float32
- fd: Float32
- g0: Float32
- d0: Float32
- b0: Float32
- b_inc: Float32
- d_inc: Float32
- r_inc: Float32
- g_inc: Float32

Important

- Time window: PV generation is filtered with `date > start && date < end`.
- Power used: `g_inc` (global irradiance on the plane of array, W/m²) is converted to W based on panel count and watt rating.
- Minimum required columns for the code paths used are `date` and `g_inc` (others are kept for flexibility).


## UI Workflows

1) EV + Solar + BESS Simulation

- Configure EV charger model, PV array (panel count + watts), battery, and two 24‑value lambda vectors (weekday/weekend).
- Pick a simulation window; click “Run Simulation”.
- The app builds PV power from CSV, EV demand from Poisson, updates battery state, writes results to an in‑memory SurrealDB entry, and shows stats + a chart.

2) Operator Simulation

- Provide PV/BESS parameters, 24 hourly market rates (mean/std per hour), optional BTM generation cost, and a date range.
- Choose “No Generation” (grid only) or “BTM‑Generation”.
- The optimizer (good_lp) computes an hourly discharge/charge schedule that maximizes profit subject to SoC constraints, then renders results and aggregates.


## Server Functions (API)

These are enabled with the `server` feature and called from the UI components.

- setup_and_run_ev_charger_simulation(
  template_charger: `Charger`,
  start_date: `DateTime<Utc>`, end_date: `DateTime<Utc>`,
  battery_param_vec: `Vec<(f32, f32)>`,
  pv_param_vec: `Vec<(f32, f32)>`,
  ev_charger_param_vec: `Vec<usize>`,
  lamb_vec: `Vec<(Vec<f64>, Vec<f64>)>`,
) -> `Result<Vec<HashMap<String, String>>, ServerFnError>`

  - Builds one scenario per input combination, returns flattened maps suitable for tables/charts.

- setup_and_run_operator_simulation(
  start_date: `DateTime<Utc>`, end_date: `DateTime<Utc>`,
  battery_param_vec: `Vec<Vec<(f32, f32, f32, f32, f32, f32)>>`,
  pv_param_vec: `Vec<Vec<(f32, f32, f32, f32, f32, f32)>>`,
  btm_rate_param: `Option<f64>`, btm_ppw_param: `Option<f64>`,
  rates_param: `Vec<(f64, f64)>`,
  keys: `Vec<String>`,
  sim_type: `usize>`,
) -> `Result<(Vec<HashMap<String, String>>, Vec<HashMap<String, String>>), ServerFnError>`

  - Returns (per‑timestamp outputs, aggregate stats). `sim_type = 1` disables PV generation.


## Tips & Troubleshooting

- Ensure the irradiance CSV covers the date range you simulate; otherwise PV output will be empty.
- Always run from the repo root so the relative `data/full_irradiance_data.csv` path resolves.
- Compiles are faster without `--features server` (no Polars/DB/async). Enable it when you need simulations.
- All data under `data/` is ignored by Git to keep pushes small.


## Development

- Format: `cargo fmt`
- Lint: `cargo clippy --features server`
- Build (web): `dx build --platform web`

Key crates

- dioxus (fullstack UI), polars (CSV/time‑series), nalgebra (matrices), good_lp (LP), surrealdb (in‑memory), tokio (async), chrono (time), rand/rand_distr (stochastic), uuid.


## License

This repository does not currently include an explicit license. If you plan to publish or distribute, consider adding one (e.g., MIT/Apache‑2.0).
