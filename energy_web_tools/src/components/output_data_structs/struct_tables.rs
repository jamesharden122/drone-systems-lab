#[cfg(feature = "server")]
use crate::components::energy_components::photovoltaic::pv_base_system::PvSystem;
use crate::components::energy_components::{
    batteries::BatteryStorage, ev_chargers::Charger, general_fun::PowerComponent,
};
use crate::components::output_data_structs::*;
//use crate::components::output_data_structs::{create_stat_csv, gen_stat};
use crate::components::surreal_data_structs::*;

use dioxus::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Props, PartialEq, Clone)]
pub struct MapTableProps {
    data: Vec<HashMap<String, String>>,
    #[props(default = TableLayout::Column)]
    layout: TableLayout,
}

#[derive(PartialEq, Clone, Copy)]
pub enum TableLayout {
    Row,
    Column,
}

fn format_value(val: &str) -> String {
    // Try parsing as f64 and round, else return original value
    if let Ok(num) = val.parse::<f64>() {
        format!("{:.4}", num) // 4 decimal places
    } else {
        val.to_string()
    }
}

#[component]
pub fn MapTable(props: MapTableProps) -> Element {
    rsx! {
        style { {include_str!("../../css_files/gen_struct_table.css")} }

        for map in props.data.iter() {
            match props.layout {
                TableLayout::Column => {
                    let mut sorted_keys: Vec<_> = map.keys().cloned().collect();
                    sorted_keys.sort();

                    rsx! {
                        table { class: "data-ai-table",
                            tbody {
                                for key in sorted_keys.iter() {
                                    tr { class: "data-ai-row",
                                        td { class: "data-ai-key", "{key}" }
                                        td { class: "data-ai-value", "{format_value(map.get(key).unwrap())}" }
                                    }
                                }
                            }
                        }
                    }
                },
                TableLayout::Row => {
                    let mut sorted_keys: Vec<_> = map.keys().cloned().collect();
                    sorted_keys.sort();
                    let values: Vec<_> = sorted_keys.iter()
                        .map(|k| format_value(map.get(k).unwrap()))
                        .collect();

                    rsx! {
                        table { class: "data-ai-table",
                            tr { class: "data-ai-header-row",
                                for key in &sorted_keys {
                                    th { class: "data-ai-header", "{key}" }
                                }
                            }
                            tr { class: "data-ai-row",
                                for val in &values {
                                    td { class: "data-ai-value", "{val}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MapTableCondense(props: MapTableProps) -> Element {
    rsx! {
        style { {include_str!("../../css_files/gen_struct_table.css")} }
        div { class: "table-container",
            match props.layout {
                TableLayout::Column => {
                    if let Some(first_map) = props.data.first() {
                        let mut sorted_keys: Vec<_> = first_map.keys().cloned().collect();
                        sorted_keys.sort();
                        rsx! {
                            table { class: "data-ai-table",
                                tr { class: "data-ai-header-row",
                                    th { class: "data-ai-header" }
                                    for (i, _) in props.data.iter().enumerate() {
                                        th { class: "data-ai-header", "Map {i+1}" }
                                    }
                                }
                                for key in &sorted_keys {
                                    tr { class: "data-ai-row",
                                        th { class: "data-ai-key", "{key}" }
                                        for map in props.data.iter() {
                                            td { class: "data-ai-value", "{format_value(map.get(key).unwrap_or(&\"\".to_string()))}" }
                                        }
                                    }
                                }
                            }
                        }
                    } else { rsx! {} }
                },
                TableLayout::Row => {
                    if let Some(first_map) = props.data.first() {
                        let mut sorted_keys: Vec<_> = first_map.keys().cloned().collect();
                        sorted_keys.sort();

                        rsx! {
                            table { class: "data-ai-table",
                                tr { class: "data-ai-header-row",
                                    for key in &sorted_keys {
                                        th { class: "data-ai-header", "{key}" }
                                    }
                                }
                                for map in props.data.iter() {
                                    tr { class: "data-ai-row",
                                        for key in &sorted_keys {
                                            td { class: "data-ai-value", "{format_value(map.get(key).unwrap_or(&\"\".to_string()))}" }
                                        }
                                    }
                                }
                            }
                        }
                    } else { rsx! {} }
                }
            }
        }
    }
}
