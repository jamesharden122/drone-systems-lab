use super::{DefinitionsOverview, ResultsDefinitionsTableOperator, TabKind};
use crate::components::energy_components::ev_chargers::Charger;
use crate::components::output_data_structs::{charts::*, struct_tables::*, RoundToPlaces};
use crate::components::simulation::pv_battery_sales::setup_and_run_operator_simulation;
use crate::components::time_processes::sample_rates;
use chrono::DateTime;
use dioxus::prelude::*;
use std::collections::HashMap;
// Helper for converting HTML datetime-local to RFC3339

#[component]
pub fn OperatorForm() -> Element {
    // num_panels, panel_watts panel_cost, transformer_efficiency, panel_width, panel_length, params
    let mut pv_params: Signal<Vec<(f32, f32, f32, f32, f32, f32)>> = use_signal(Vec::new);
    let mut temp_pv = use_signal(|| (4788.0_f32, 590.0_f32, 0.3_f32, 0.0_f32, 0.0_f32, 0.0_f32));
    let mut temp_pv_str = use_signal(|| {
        (
            String::from("4788.0"),
            String::from("590.0"),
            String::from("0.3"),
            String::from("f32"),
            String::from("0.0"),
            String::from("0.0"),
        )
    });
    // capacity, watt_hours, depth_of_discharge, battery_system_voltage, efficiency
    let mut battery_params: Signal<Vec<(f32, f32, f32, f32, f32, f32)>> = use_signal(Vec::new);
    let mut temp_batt =
        use_signal(|| (400000.0_f32, 4.0_f32, 0.3_f32, 0.0_f32, 0.98_f32, 0.98_f32));
    let mut temp_batt_str = use_signal(|| {
        (
            String::from("400000.0"),
            String::from("4.0"),
            String::from("0.3"),
            String::from("0.0"),
            String::from("0.98"),
            String::from("0.98"),
        )
    });
    let mut rates_vec = use_signal(|| sample_rates::caiso_rates().unwrap());
    let mut rates_vec_str = use_signal(|| sample_rates::caiso_rates_string().unwrap());
    let mut sim_type = use_signal(|| 0_usize); // default to 'no'
    let mut ppw = use_signal(|| 0.0_f64);
    let mut ppw_str = use_signal(|| String::from("0.0"));
    let keys = use_signal(|| vec!["set1".to_string()]);
    let mut start_date = use_signal(|| "2025-01-15T00:00".to_string());
    let mut end_date = use_signal(|| "2025-01-20T00:00".to_string());
    let mut location = use_signal(|| "Adelanto".to_string());
    let mut result_out: Signal<Vec<HashMap<String, String>>> = use_signal(Vec::new);
    let mut result_stat_out: Signal<Vec<HashMap<String, String>>> = use_signal(Vec::new);
    let mut tab = use_signal(|| TabKind::Results);
    // Then in your use_effect:
    rsx! {
        style { {include_str!("../css_files/pv_batt_operator.css")} }
        div {class: "prog-description",
            p{
                "Allows for specification of a battery, solar system, and rates of acquiring and selling energy.
                 Then optimizes a discharge schedule and charging schedule to maximize the profit of the battery
                  and give an idea of the profitability of operating the system in the provided scenario."
            }
        }
        div { class: "button-container",
             button {
                  class: "button",
                  onclick: move |_| sim_type.set(1),
                  "No Generation"
             }
             button {
                  class: "button",
                  onclick: move |_| sim_type.set(0),
                  "BTM-Generation"
             }
        }
        form {
            prevent_default: "onsubmit",
            onsubmit: move |_evt| {
                pv_params.set(vec![temp_pv()]);
                battery_params.set(vec![temp_batt()]);
                // Dates must be converted to DateTime<Utc> (do this server-side if easier)
                tracing::debug!("{:?}",rates_vec);
                spawn(async move {
                    tracing::debug!("james harden best sg ever 3");
                    let date_fmt = "%Y-%m-%dT%H:%M:%S%z";
                     if let Ok(mut new_data) = setup_and_run_operator_simulation(
                                DateTime::parse_from_str((start_date().clone() + ":00+0000").as_str(), date_fmt)
                                    .unwrap()
                                    .into(),
                                DateTime::parse_from_str((end_date().clone() + ":00+0000").as_str(), date_fmt)
                                    .unwrap()
                                    .into(),
                                    vec![battery_params()],
                                    vec![pv_params()],
                                    Some(ppw()),
                                    Some(ppw()),
                                    rates_vec(),
                                keys().clone(),
                                sim_type(),
                            )
                            .await
                     {
                         tracing::debug!("Writing to result out");
                         result_out.set(new_data.0);
                         result_stat_out.write().append(&mut new_data.1);
                     }
                });
                tracing::debug!("james harden best sg ever 2");
            },
            h2 {style: "text-align:center; font-size: 2.0rem;", "Battery Simulation With Behind-the-Meter(BTM) Generation" }
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
                        h3 { "Market" }
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
                    div {
                        h3 { "Location" }
                        select {
                            oninput: move |e| {
                                let val = e.value();
                                match val.as_str() {
                                    "caiso" => {
                                        rates_vec.set(sample_rates::caiso_rates().unwrap());
                                        rates_vec_str.set(sample_rates::caiso_rates_string().unwrap());
                                    }
                                    "pjm" => {
                                        rates_vec.set(sample_rates::pjm_rates().unwrap());
                                        rates_vec_str.set(sample_rates::pjm_rates_string().unwrap());
                                    }
                                    "ercot" => {
                                        rates_vec.set(sample_rates::ercot_rates().unwrap());
                                        rates_vec_str.set(sample_rates::ercot_rates_string().unwrap());
                                    }
                                    "nyiso" => {
                                        rates_vec.set(sample_rates::nyiso_rates().unwrap());
                                        rates_vec_str.set(sample_rates::nyiso_rates_string().unwrap());
                                    }
                                    _ => {},
                                }
                            },
                            option { value: "caiso", "CAISO" }
                            option { value: "pjm", "PJM" }
                            option { value: "ercot", "ERCOT" }
                            option { value: "nyiso", "NYISO" }
                            // Add more as needed
                        }
                    }
                }
                if *sim_type.read() == 0 {
                    div{ class: "pv-bess-params",
                        div {
                            h3 { "Solar Panels" }
                            label{"Panel Count:"}
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_pv_str().0}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_pv_str.write().0 =val.clone();
                                    if let Ok(temp_val) = e.value().parse(){
                                        temp_pv.write().0 = temp_val;
                                    }
                                }
                            }
                            label {"Watts:"}
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_pv_str().1}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_pv_str.write().1 =val.clone();
                                    if let Ok(temp_val) = e.value().parse(){
                                        temp_pv.write().1 = temp_val;
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "batt-inputs",
                        h3 { "Batteries" }
                        div{
                            label { "Capacity:" }
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_batt_str().0}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_batt_str.write().0 =val.clone();
                                    if let Ok(temp_val) = val.parse(){
                                        temp_batt.write().0 =  temp_val;
                                    }
                                }
                            }
                        }
                        div {
                            label { "Watt-Hours:" }
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_batt_str().1}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_batt_str.write().1 =val.clone();
                                    if let Ok(temp_val) = val.parse(){
                                        temp_batt.write().1 =  temp_val;
                                    }
                                }
                            }
                        }
                        div{
                            label { "Depth of Discharge:" }
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_batt_str().2}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_batt_str.write().2 =val.clone();
                                    if let Ok(temp_val) = val.parse(){
                                        temp_batt.write().2 =  temp_val;
                                    }
                                }
                            }
                        }
                        div {
                            label { "Voltage:" }
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_batt_str().3}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_batt_str.write().3 =val.clone();
                                    if let Ok(temp_val) = val.parse(){
                                        temp_batt.write().3 =  temp_val;
                                    }
                                }
                            }
                        }
                        div {
                            label { "Charge Efficiency:" }
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_batt_str().4}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_batt_str.write().4 =val.clone();
                                    if let Ok(temp_val) = val.parse(){
                                        temp_batt.write().4 =  temp_val;
                                    }
                                }
                            }
                        }
                        div {
                            label { "Discharge Efficiency:" }
                            input {
                                r#type: "number",
                                step: "any",
                                value: "{temp_batt_str().5}",
                                oninput: move |e| {
                                    let val = e.value();
                                    temp_batt_str.write().5 =val.clone();
                                    if let Ok(temp_val) = val.parse(){
                                        temp_batt.write().5 =  temp_val;
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "lambda-divs",
                    div {
                    h3 { "ISO Market - Energy Rates ($/kWh)" }
                    div {
                        for j in 0..24 {
                            div{
                                label {"hour {j}"}
                                input {
                                    r#type: "number",
                                    step: "any",
                                    value: "{rates_vec_str()[j].0}",
                                    oninput: move |e| {
                                        let val = e.value();
                                        rates_vec_str.write()[j].0 =val.clone();
                                        if let Ok(temp_val) = e.value().parse::<f64>(){
                                            rates_vec.write()[j].0 = temp_val;
                                        }
                                    }
                                }
                                input {
                                    r#type: "number",
                                    step: "any",
                                    value: "{rates_vec_str()[j].1}",
                                    oninput: move |e| {
                                        let val = e.value();
                                        rates_vec_str.write()[j].1 =val.clone();
                                        if let Ok(temp_val) = e.value().parse::<f64>(){
                                            rates_vec.write()[j].1 = temp_val;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if *sim_type.read() == 0 {
                        h3 {  "Behind-the-Meter (BTM) Generation Cost ($/kWh)" }
                        div {
                         class: "fixed-cost-div",
                            /*div {
                                label { "Fix Cost of Generation per kW to constant?" }
                                div {
                                    input {
                                        r#type: "radio",
                                        id: "yes",
                                        name: "fix_cost",
                                        value: "yes",
                                        checked: "{is_fixed()}",
                                        onchange: move |_| is_fixed.set(true),
                                    }
                                    label { r#for: "yes", "Yes" }
                                }
                                div {
                                    input {
                                        r#type: "radio",
                                        id: "no",
                                        name: "fix_cost",
                                        value: "no",
                                        checked: "{is_fixed()}",
                                        onchange: move |_| is_fixed.set(false),
                                    }
                                    label { r#for: "no", "No" }
                                }
                            }*/
                            div {
                                label {"Fixed Rate Cost of Generation Per Kwh"}
                                input {
                                    r#type: "number",
                                    step: "any",
                                    value: "{ppw_str()}",
                                    oninput: move |e| {
                                        let val = e.value();
                                        ppw_str.set(val.clone());
                                        if let Ok(temp_val) = val.parse::<f64>(){
                                            ppw.set(temp_val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button {class: "button", r#type: "submit", "Run Simulation" }
        }
        div{
            MapTable {
                    data: result_stat_out(),
                    layout: TableLayout::Row,
            }
        }
        div{
            style: "
            width: 100%;display: flex;  
            align-items: center;
            justify-content: center;
            margin-bottom: 5vh;
                   ",
            ChartFromMapTable {
                    data: result_out(),
                    metric: "revenue",
            }
        }


        div{
            MapTableCondense {
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
                    TabKind::Results => rsx!( ResultsDefinitionsTableOperator {} ),
                    TabKind::Inputs => rsx!( DefinitionsOverview {} ),
                }
            }
        }
    }
}
