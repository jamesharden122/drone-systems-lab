use super::{DefinitionsOverview, ResultsDefinitionsTableEv, TabKind};
use crate::components::energy_components::ev_chargers::Charger;
use crate::components::output_data_structs::struct_tables::*;
use crate::components::simulation::pv_charger_battery::setup_and_run_ev_charger_simulation;
use chrono::DateTime;
use dioxus::prelude::*;
use std::collections::HashMap;
// Helper for converting HTML datetime-local to RFC3339

#[component]
pub fn PVChargerBatteryForm() -> Element {
    let mut charger = use_signal(|| {
        Charger::new(
            280000.0, // maximum_power_w
            150.0,    // output_voltage_min_vdc
            1000.0,   // output_voltage_max_vdc
            500.0,    // max_output_current_a (peak current)
            400.0,    // input_voltage_vac
            60,       // input_frequency_hz
            361.0,    // fla_a (for 240kW config)
            421,      // breaker_rating_a (for 280kW config)
            320.0,    // rated_power_kva
            0.99,     // power_factor_at_full_load
            0.97,     // efficiency_at_nominal_power
        )
    });
    let mut charger_str = use_signal(|| {
        (
            "280000.0".to_string(), // maximum_power_w
            "150.0".to_string(),    // output_voltage_min_vdc
            "1000.0".to_string(),   // output_voltage_max_vdc
            "500.0".to_string(),    // max_output_current_a (peak current)
            "400.0".to_string(),    // input_voltage_vac
            "60".to_string(),       // input_frequency_hz
            "361.0".to_string(),    // fla_a (for 240kW config)
            "421".to_string(),      // breaker_rating_a (for 280kW config)
            "320.0".to_string(),    // rated_power_kva
            "0.99".to_string(),     // power_factor_at_full_load
            "0.97".to_string(),     // efficiency_at_nominal_power
        )
    });
    let mut pv_params = use_signal(|| (2000.0_f32, 550.0_f32));
    let mut pv_params_str = use_signal(|| ("2000.0".to_string(), "550.0".to_string()));
    let mut ev_params = use_signal(|| 5_usize);
    let mut ev_params_str = use_signal(|| "5".to_string());
    let mut battery_params = use_signal(|| (10000.0_f32, 4.0_f32));
    let mut battery_params_str = use_signal(|| ("500000.0".to_string(), "4.0".to_string()));
    let mut lamb_vec = use_signal(|| {
        (
            vec![
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 45.0, 15.0, 31.0, 30.0, 15.0, 15.0, 1.0, 1.0,
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            ],
            vec![
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 45.0, 1.0, 1.0, 30.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            ],
        )
    });
    let mut lamb_vec_str = use_signal(|| {
        (
            vec![
                "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "45.0", "15.0", "31.0", "30.0",
                "15.0", "15.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0",
                "1.0", "1.0",
            ]
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>(),
            vec![
                "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "45.0", "1.0", "1.0", "30.0",
                "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0", "1.0",
                "1.0",
            ]
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>(),
        )
    });
    let mut start_date = use_signal(|| "2025-01-01T00:00".to_string());
    let mut end_date = use_signal(|| "2025-06-20T00:00".to_string());
    let mut location = use_signal(|| "Adelanto".to_string());
    let mut result_out: Signal<Vec<HashMap<String, String>>> = use_signal(Vec::new);
    let mut tab = use_signal(|| TabKind::Results);
    // Then in your use_effect:
    rsx! {
        style { {include_str!("../css_files/pv_charger_batt.css")} }
        div { class: "prog-description",
            p{
                "Analyzes the resource demand and adequacy of a charging station and associated energy system."
            }
        }
        form {
            prevent_default: "onsubmit",
            onsubmit: move |evt| {
                // Dates must be converted to DateTime<Utc> (do this server-side if easier)
                spawn(async move {
                    tracing::debug!("james harden best sg ever 3");
                    let date_fmt = "%Y-%m-%dT%H:%M:%S%z";
                     if let Ok(mut new_data) = setup_and_run_ev_charger_simulation(
                                charger().clone(),
                                DateTime::parse_from_str((start_date().clone() + ":00+0000").as_str(), date_fmt)
                                    .unwrap()
                                    .into(),
                                DateTime::parse_from_str((end_date().clone() + ":00+0000").as_str(), date_fmt)
                                    .unwrap()
                                    .into(),
                                vec![battery_params()],
                                vec![pv_params()],
                                vec![ev_params()],
                                vec![lamb_vec()],
                            )
                            .await
                     {
                         result_out.write().append(&mut new_data);
                     }
                });
                tracing::debug!("james harden best sg ever 2");
            },
            h2 {style: "text-align:center; font-size: 2.0rem;", "PV-Charger-Battery Simulation" }
            div{ class: "top-row",
                div {
                    class: "dates-hyper",
                    div {
                        h3 {"Dates"}
                        label { "Start Date" }
                        input {
                            r#type: "datetime-local",
                            value: "{start_date().as_str()}",
                            oninput: move |e| {
                                if let Ok(temp_val) = e.value().parse::<String>(){
                                    start_date.set(temp_val)
                                }
                            }
                        }
                        label { "End Date" }
                        input {
                            r#type: "datetime-local",
                            value: "{end_date.read().as_str()}",
                            oninput: move |e| {
                                if let Ok(temp_val) = e.value().parse::<String>(){
                                    end_date.set(temp_val)
                                }
                            }
                        }
                    }
                    div {
                        h3 { "Location" }
                        select {
                            value: "{location()}",
                            oninput: move |e| location.set(e.value().clone()),
                            option { value: "Adelanto", "Adelanto" }
                            option { value: "Norwalk", "Norwalk" }
                            option { value: "Perris", "Perris" }
                            option { value: "Santa Fe", "Santa Fe" }
                            option { value: "La Puenta", "La Puenta" }
                        // Add more as needed
                        }
                    }

                }
                div{
                    class: "pv-bess-params",
                    div {
                        h3 { "PV Parameters" }
                        label{"Panel Count"}
                        input {
                            r#type: "number",
                            step: "any",
                            value: "{pv_params_str().0}",
                            oninput: move |e| {
                                let val = e.value();
                                pv_params_str.write().0 = val.clone();
                                if let Ok(temp_val) = val.parse(){
                                    pv_params.write().0 = temp_val;
                                }
                            }
                        }
                        label {"Watts"}
                        input {
                            r#type: "number",
                            step: "any",
                            value: "{pv_params_str().1}",
                            oninput: move |e| {
                                let val = e.value();
                                pv_params_str.write().1 = val.clone();
                                if let Ok(temp_val) = val.parse(){
                                    pv_params.write().1 = temp_val;
                                }
                            }
                        }
                    }
                    div {
                        h3 { "Battery" }
                        label { "Capacity" }
                        input {
                            r#type: "number",
                            step: "any",
                            value: "{battery_params_str.read().0}",
                            oninput: move |e| {
                                let val = e.value();
                                battery_params_str.write().0 = val.clone();
                                if let Ok(temp_val) = val.parse(){
                                    battery_params.write().0 =  temp_val;
                                }
                            }
                        }
                        label { "Watt-Hours" }
                        input {
                            r#type: "number",
                            step: "any",
                            value: "{battery_params_str.read().1}",
                            oninput: move |e| {
                                let val = e.value();
                                battery_params_str.write().1 = val.clone();
                                if let Ok(temp_val) = val.parse(){
                                    battery_params.write().1 =  temp_val;
                                }
                            }
                        }
                    }
                     div {
                        h3 { "Charging Station" }
                        label { "Charger Count" }
                        input {
                            r#type: "number",
                            step: "any",
                            value: "{ev_params_str()}",
                            oninput: move |e| {
                                let val = e.value();
                                *ev_params_str.write() = val.clone();
                                if let Ok(temp_val) = val.parse::<usize>(){
                                    ev_params.set(temp_val);
                                }
                            }
                        }
                     }
                }
            div{ class: "ev-inputs",
            h3 { "EV Charger" }
                div {
                    label {"Maximum Output Power(w): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().0}",
                        oninput: move |e| {
                            let val = e.value();
                            charger_str.write().0 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().maximum_power_w = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Output Voltage Min (VDC): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().1}",
                        oninput: move |e| {
                            let val = e.value();
                            charger_str.write().1 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().output_voltage_min_vdc = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Output Voltage Max (VDC): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().2}",
                        oninput: move |e| {
                            let val = e.value();
                            charger_str.write().2 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().output_voltage_max_vdc = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Max Output Current (A): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().3}",
                        oninput: move |e| {
                            let val = e.value();
                            charger_str.write().3 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().max_output_current_a = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Input Voltage (VAC): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().4}",
                        oninput: move |e|{
                            let val = e.value();
                            charger_str.write().4 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().input_voltage_vac = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Input Frequency (Hz): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().5}",
                        oninput: move |e|{
                            let val = e.value();
                            charger_str.write().5 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().input_frequency_hz = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Full Load Amps(A): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().6}",
                        oninput: move |e|{
                            let val = e.value();
                            charger_str.write().6 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().fla_a = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Breaker Rating(A): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().7}",
                        oninput: move |e|{
                            let val = e.value();
                            charger_str.write().7 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().breaker_rating_a = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Rated Power(kVA): "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().8}",
                        oninput: move |e|{
                            let val = e.value();
                            charger_str.write().8 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().rated_power_kva = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Power factor at full load: "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().9}",
                        oninput: move |e|{
                            let val = e.value();
                            charger_str.write().9 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().power_factor_at_full_load = temp_val
                            }
                        }
                    }
                }
                div {
                    label {"Efficiency at Nominal Power: "}
                    input {
                        r#type: "number",
                        step: "any",
                        value: "{charger_str().10}",
                        oninput: move |e|{
                            let val = e.value();
                            charger_str.write().10 = val.clone();
                            if let Ok(temp_val) = val.parse() {
                                charger.write().efficiency_at_nominal_power = temp_val
                            }
                        }
                    }
                }
            }
            }
            div {
                class: "lambda-divs",
                h3 { "Average Minutes of Charging (weekday/weekend)" }
                div {
                    p{ "Weekday:"}
                    div {
                        for j in 0..24 {
                            div{
                                label {"hour {j}"}
                                input {
                                    style: "width: 3em;",
                                    r#type: "number",
                                    step: "any",
                                    min: "1.0",
                                    max: "60.0",
                                    value: "{lamb_vec_str.read().0[j]}",
                                    oninput: move |e| {
                                        let val = e.value();
                                        lamb_vec_str.write().0[j] = val.clone();
                                        if let Ok(temp_val) = val.parse(){
                                            lamb_vec.write().0[j] = temp_val;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    p { "Weekend:" }
                    div {
                        for j in 0..24 {
                            div {
                                label {"hour {j}"}
                                input {
                                    style: "width: 3em;",
                                    r#type: "number",
                                    step: "any",
                                    min: "1.0",
                                    max: "60.0",
                                    value: "{lamb_vec_str.read().1[j]}",
                                    oninput: move |e| {
                                        let val = e.value();
                                        lamb_vec_str.write().0[j] = val.clone();
                                        if let Ok(temp_val) = val.parse(){
                                            lamb_vec.write().1[j] = temp_val;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button { class: "button", r#type: "submit", "Run Simulation" }
        }
        div{
            MapTable {
                    data: result_out(),
                    layout: TableLayout::Row,
            }
        }
        div { class: "definition-section",
            // Button Row
            div { class: "flex gap-4 mb-6",
                button {
                    class: "button",
                    onclick: move |_| tab.set(TabKind::Results),
                    "Results Definitions"
                }
                button {
                    class: "button",
                    onclick: move |_| tab.set(TabKind::Inputs),
                    "Input Definitions"
                }
            }
            // Tab Content
            div { class: "w-full",
                match tab() {
                    TabKind::Results => rsx!( ResultsDefinitionsTableEv {} ),
                    TabKind::Inputs => rsx!( DefinitionsOverview {} ),
                }
            }
        }
    }
}
