//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals. In this template, we define a Hero
//! component and an Echo component for fullstack apps to be used in our app.

pub mod energy_components;
pub mod output_data_structs;
pub mod pv_battery_operator_sim;
pub mod pv_charger_battery_sim;
pub mod simulation;
pub mod site_spec;
pub mod surreal_data_structs;
pub mod time_processes;

use dioxus::prelude::*;

#[component]
pub fn DefinitionsOverview() -> Element {
    let mut explanation = use_signal(|| "".to_string());

    rsx! {
        style { {include_str!("../css_files/definitions.css")} }

        div { class: "definitions-overview",

            // Program description input
            div { class: "program-description",
                h1 {"Definitions: for the values used in the simulation"}
            }

            // Photovoltaic Table
            h3 { "Photovoltaic Panels" }
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Type" }
                        th { "Description" }
                    }
                }
                tbody {
                    tr { td { "Panel Count" } td { "int" } td { "The number of photovoltaic panels in the system." } }
                    tr { td { "Watts" } td { "float" } td { "Rated power output per panel (W)." } }
                    tr { td { "panel width" } td { "float" } td { "Width of the panel (m)." } }
                    tr { td { "panel length" } td { "float" } td { "Length of the panel (m)." } }
                    tr { td { "Irradiance" } td { "float" } td { "Solar irradiance (W/m²)." } }
                    tr { td { "V_mpp" } td { "float" } td { "Voltage at maximum power (V)." } }
                    tr { td { "I_mpp" } td { "float" } td { "Current at maximum power (A)." } }
                    tr { td { "V_oc" } td { "float" } td { "Open-circuit voltage (V)." } }
                    tr { td { "I_sc" } td { "float" } td { "Short-circuit current (A)." } }
                }
            }

            // EV Chargers Table
            h3 { "EV Chargers" }
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Type" }
                        th { "Description" }
                    }
                }
                tbody {
                    tr { td { "maximum_power_w" } td { "f32" } td { "Maximum output power (W)." } }
                    tr { td { "output_voltage_min_vdc" } td { "f32" } td { "Minimum output voltage (VDC)." } }
                    tr { td { "output_voltage_max_vdc" } td { "f32" } td { "Maximum output voltage (VDC)." } }
                    tr { td { "max_output_current_a" } td { "f32" } td { "Maximum (peak) output current (A)." } }
                    tr { td { "input_voltage_vac" } td { "f32" } td { "Input voltage required (VAC)." } }
                    tr { td { "input_frequency_hz" } td { "u16" } td { "Input frequency required (Hz)." } }
                    tr { td { "fla_a" } td { "f32" } td { "Full Load Amps (A)." } }
                    tr { td { "breaker_rating_a" } td { "u16" } td { "Breaker size (A)." } }
                    tr { td { "rated_power_kva" } td { "f32" } td { "Rated output power (kVA)." } }
                    tr { td { "power_factor_at_full_load" } td { "f32" } td { "Power factor at full load." } }
                    tr { td { "efficiency_at_nominal_power" } td { "f32" } td { "Efficiency at nominal (rated) power." } }
                }
            }

            // Battery Storage Table
            h3 { "Batteries - BatteryStorage" }
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Type" }
                        th { "Description" }
                    }
                }
                tbody {
                    tr { td { "capacity" } td { "f32" } td { "Nominal capacity of the battery (kWh)." } }
                    tr { td { "watt_hours" } td { "f32" } td { "Total energy in watt-hours." } }
                    tr { td { "depth_of_discharge" } td { "f32" } td { "Max depth of discharge (fraction of capacity)." } }
                    tr { td { "battery_system_voltage" } td { "f32" } td { "Nominal system voltage." } }
                    tr { td { "efficiency" } td { "f32" } td { "Round-trip efficiency (0–1)." } }
                    tr { td { "battery_state" } td { "BatteryPowerComponent" } td { "Tracks battery time-series data (SOC, power, etc.)." } }
                }
            }
             h3 { "Inverter" }
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Type" }
                        th { "Description" }
                    }
                }
                tbody {
                    tr { td { "AC Phase Config" } td { "String" } td { "AC phase configuration (e.g., single-phase, three-phase)." } }
                    tr { td { "Nominal AC Power" } td { "f32" } td { "Nominal AC power rating (kVA)." } }
                    tr { td { "DC Voltage Range" } td { "(f32, f32)" } td { "Minimum and maximum DC operating voltage (VDC)." } }
                    tr { td { "DC MPPT Range" } td { "(f32, f32)" } td { "Minimum and maximum MPPT (Maximum Power Point Tracking) voltage (VDC)." } }
                    tr { td { "Max DC Current" } td { "f32" } td { "Maximum DC input current (A)." } }
                    tr { td { "Max Efficiency" } td { "f32" } td { "Maximum inverter efficiency (%)." } }
                    tr { td { "AC Frequency Range" } td { "((f32, f32), (f32, f32))" } td { "Two operating AC frequency ranges (Hz)." } }
                    tr { td { "AC Voltage Range" } td { "(f32, f32)" } td { "Nominal AC voltage range (V)." } }
                    tr { td { "Power Factor Range" } td { "(f32, f32, f32)" } td { "Power factor range (leading, unity, lagging)." } }
                    tr { td { "Dimensions" } td { "(f32, f32, f32)" } td { "Physical dimensions (length, width, height in meters)." } }
                    tr { td { "Weight" } td { "f32" } td { "Inverter weight (kg)." } }
                    tr { td { "Ambient Temperature Range" } td { "(f32, f32)" } td { "Operating temperature range (°C)." } }
                    tr { td { "Altitude Limit" } td { "f32" } td { "Maximum operating altitude (m)." } }
                    tr { td { "Max Short Circuit Current from PV" } td { "f32" } td { "Maximum short-circuit current from PV (A)." } }
                }
            }

            // Operator Table
            h3 { "Operator" }
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Type" }
                        th { "Description" }
                    }
                }
                tbody {
                    tr { td { "date_to_index" } td { "map" } td { "Maps timestamps to matrix indices." } }
                    tr { td { "daily_rates" } td { "matrix" } td { "Daily electricity rates (cost per watt over time)." } }
                    tr { td { "ppw" } td { "matrix" } td { "Per-panel watt cost or solar generation cost." } }
                    tr { td { "discharge_rates" } td { "map" } td { "Battery discharge profiles (matrices), keyed by ID." } }
                    tr { td { "batteries" } td { "map" } td { "Collection of BatteryStorage instances by key." } }
                    tr { td { "solar" } td { "map" } td { "Collection of PV system configurations by key." } }
                    tr { td { "keys" } td { "string list" } td { "List of identifiers for battery and solar maps." } }
                }
            }
        }
    }
}

#[component]
pub fn ResultsDefinitionsTableEv() -> Element {
    rsx! {
        style { {include_str!("../css_files/definitions.css")} }
        div { class: "definitions-overview",
        table { class: "definition-table",
            thead {
                tr {
                    th { "Field" }
                    th { "Description" }
                }
            }
            tbody {
                tr { td { "`chargers_count`" } td { "Number of EV chargers included in the system." } }
                tr { td { "`energy_system_size`" } td { "Total installed DC solar system size, in watts (number of panels × panel wattage)." } }
                tr { td { "`battery_size`" } td { "Total installed battery capacity, in watt-hours (capacity × rated watt-hours per battery unit)." } }
                tr { td { "`duration_energy_needed`" } td { "Fraction of total time intervals where the battery was depleted (storage ≤ 0), indicating undersupply risk." } }
                tr { td { "`total_energy_usage`" } td { "Total battery energy discharged to serve loads (sum of all BTM + FTM outputs, in watt-hours or kWh)." } }
                tr { td { "`total_btm_generation`" } td { "Total energy charged into the battery from behind-the-meter sources (typically grid or solar, in watt-hours or kWh)." } }
                tr { td { "`max_storage`" } td { "Maximum battery state-of-charge (SOC) observed during the simulation (in watt-hours or kWh)." } }
                tr { td { "`average_usage`" } td { "Average site energy usage rate over the simulation (typically in watts or kW, depending on your lambda units)." } }
                tr { td { "`days`" } td { "Number of unique days represented in the simulation data." } }
            }
        }
    }
    }
}

#[component]
pub fn ResultsDefinitionsTableOperator() -> Element {
    rsx! {
        style { {include_str!("../css_files/definitions.css")} }
        div { class: "definitions-overview",
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Description" }
                    }
                }
                tbody {
                    tr { td { "`average_discharge_rate`" }
                        td { "The mean rate at which stored energy is released (discharged) from the battery or storage system over the analysis period. Typically expressed as a fraction of total capacity per time unit (e.g., per hour or per day)." } }
                    tr { td { "`average_ppw`" }
                        td { "The average profit per watt or per unit of energy transferred, generated, or sold during the period. This could be calculated as (total profit) divided by (total energy output/input), and may also represent profitability efficiency." } }
                    tr { td { "`total_expenses`" }
                        td { "The sum of all costs incurred during the analyzed period. This can include operational expenses, maintenance, energy procurement, and any other direct or indirect costs associated with running the system." } }
                    tr { td { "`total_input_btm_power`" }
                        td { "The total amount of energy (in kWh, Wh, or other units) received or stored from behind-the-meter (BTM) sources. BTM typically refers to energy generated or managed on the customer’s side of the utility meter (e.g., rooftop solar, on-site generation)." } }
                    tr { td { "`total_input_ftm_power`" }
                        td { "The total amount of energy (in kWh, Wh, etc.) received or imported from front-of-the-meter (FTM) sources, which refers to utility-scale or grid-supplied power." } }
                    tr { td { "`total_output_btm_power`" }
                        td { "The total amount of energy delivered from the system to behind-the-meter (BTM) loads or uses, such as on-site consumption or local microgrid supply." } }
                    tr { td { "`total_output_ftm_power`" }
                        td { "The total energy exported or delivered from the system to front-of-the-meter (FTM) destinations, typically meaning energy sold or delivered to the utility grid or wholesale market." } }
                    tr { td { "`total_profit`" }
                        td { "The net profit earned during the period, calculated as total revenue minus total expenses." } }
                    tr { td { "`total_revenue`" }
                        td { "The total income generated from energy sales, grid services, or other sources during the period, before expenses are deducted." } }
                }
            }
        }
    }
}

#[component]
pub fn ShapesResultsTable() -> Element {
    rsx! {
    style { {include_str!("../css_files/definitions.css")} }
        // Main Results Table
        div {class: "definitions-overview",
            h3 { class: "text-lg font-semibold mb-2", "Rectangle Results Table" }
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Definition" }
                    }
                }
                tbody {
                    tr { td { "Row Width" }
                         td { "The physical width of each MMS (mounting structure) row in yards, including all panels in that row plus gaps between modules." }
                    }
                    tr { td { "Column Length" }
                         td { "The length of each MMS row/table in yards (often just the length of a panel, or a small vertical grouping if stacking vertically)." }
                    }
                    tr { td { "Tables in a Row" }
                         td { "Number of distinct MMS mounting tables in one horizontal row. Each table is a repeated unit of panels mounted side-by-side." }
                    }
                    tr { td { "Row Count" }
                         td { "The number of horizontal rows of tables placed along the rectangle's “short” direction, filling the available site area." }
                    }
                    tr { td { "Total Panels" }
                         td { "Total number of PV modules in the rectangular array: Tables in a Row × Row Count × String Range Max (panels per string)" }
                    }
                    tr { td { "Total Watts" }
                         td { "Total DC power capacity (watts) for the rectangle layout: Total Panels × Module Wattage" }
                    }
                }
            }
        }

        // Section Details
        div {class: "definitions-overview",
            h3 { class: "text-md font-semibold mb-2", "Rectangle “Section Details”" }
            table { class: "definition-table",
                thead {
                    tr {
                        th { "Field" }
                        th { "Definition" }
                    }
                }
                tbody {
                    tr { td { "Section Length" }
                         td { "The horizontal (long) dimension of one MMS table group (in yards), including all included panels and gaps." }
                    }
                    tr { td { "Row Length" }
                         td { "The vertical (short) dimension of one MMS table group (in yards)." }
                    }
                    tr { td { "Distance Between Rows" }
                         td { "The calculated minimum spacing between each row of tables, based on sun/shadow geometry and buffer (in yards)." }
                    }
                    tr { td { "Distance Between Tables" }
                         td { "The spacing between adjacent mounting tables in the same row (in yards)." }
                    }
                    tr { td { "String Range Max" }
                         td { "Maximum number of panels per string as allowed by voltage constraints (from nms_max)." }
                    }
                    tr { td { "Number of Strings Per Inverter" }
                         td { "Maximum number of panel strings that can be connected to one inverter (from num_string_per_inv_max)." }
                    }
                    tr { td { "Distance Between Modules" }
                         td { "The physical spacing (gap) between adjacent PV modules within each MMS table, as specified by the designer (in yards)." }
                    }
                }
            }
        }
    }
}

// Enum for tab state
#[derive(Clone, Copy, PartialEq, Eq)]
enum TabKind {
    Results,
    Inputs,
}
