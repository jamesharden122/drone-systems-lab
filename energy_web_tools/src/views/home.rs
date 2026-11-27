use crate::components::{
    pv_battery_operator_sim::OperatorForm, pv_charger_battery_sim::PVChargerBatteryForm,
    site_spec::SiteSpec,
};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`

#[component]
pub fn Home() -> Element {
    let mut selected_tool = use_signal(|| 0);

    rsx! {
        style { {include_str!("../css_files/web_tools.css")} }
        div { class: "web-tools-root",
            div { class: "web-tools-hero",
                h1 { "Dr. William Francis Web Apps" }
                p { "Accelerating infrastructure with AI, simulation, and forecasting." }
            }

            div { class: "web-tools-selector",
                button { class: "selector-button", onclick: move |_| selected_tool.set(1), "📐 Site Specification" }
                button { class: "selector-button", onclick: move |_| selected_tool.set(2), "⚡ EV + Solar + BESS Simulation" }
                button { class: "selector-button", onclick: move |_| selected_tool.set(3), "📊 Forecasting Platform" }
            }

            div { class: "web-tools-panel",
                match selected_tool() {
                    1 => rsx! { SiteSpec {} },
                    2 => rsx! { PVChargerBatteryForm {} },
                    3 => rsx! { OperatorForm {} },
                    _ => rsx! {
                        p { class: "tool-placeholder", "Select a tool to begin." }
                    }
                }
            }
        }
    }
}
