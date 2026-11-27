use super::{DefinitionsOverview, ShapesResultsTable, TabKind};
use dioxus::prelude::*;
use std::f32::consts::PI;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MathError {
    #[error("Attempted division by zero")]
    DivisionByZero,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}

#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct CorrectionFactors {
    pub temperature: f32,
    pub irradiance: f32,
    pub over_current: f32,
    pub dc_to_ac: Option<f32>,
}
#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct PvPanel {
    pub watts: f32,
    pub dimensions: (f32, f32),
    pub irradiance: f32,
    pub voltage_mpp: f32,
    pub current_mpp: f32,
    pub voltage_oc: f32,
    pub current_sc: f32,
}
#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct Inverter {
    pub ac_phase_config: String,
    pub nominal_ac_power: f32,
    pub dc_voltage_range: (f32, f32),
    pub dc_mppt_range: (f32, f32),
    pub max_dc_current: f32,
    pub max_efficiency: f32,
    pub ac_frequency_range: ((f32, f32), (f32, f32)),
    pub ac_voltage_range: (f32, f32),
    pub power_factor_range: (f32, f32, f32),
    pub dimensions: (f32, f32, f32),
    pub weight: f32,
    pub ambient_temperature_range: (f32, f32),
    pub altitude_limit: f32,
    pub max_current_sc: f32,
}
#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct CorrectionFactorsFormProps {
    pub cf: Signal<CorrectionFactors>,
}

#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct PvPanelFormProps {
    pub pv: Signal<PvPanel>,
}

#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct InverterFormProps {
    pub iv: Signal<Inverter>,
}

#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct TriangleInput {
    adj: f32,
    opp: f32,
    hyp: f32,
    alpha_angle: f32,
    beta_angle: f32,
    phi_angle: f32,
}

#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct RectanleInput {
    long_side: f32,
    short_side: f32,
}

#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct TriangleInpProps {
    pub ti: Signal<TriangleInput>,
}

#[derive(Clone, Debug, Default, Props, PartialEq)]
pub struct RectangleInpProps {
    pub ri: Signal<RectanleInput>,
}

// Parent
#[component]
pub fn SiteSpec() -> Element {
    let mut shape: Signal<u32> = use_signal(|| 1);
    let correction_factors = use_signal(|| CorrectionFactors {
        temperature: 1.0,
        irradiance: 1.0,
        over_current: 1.0,
        dc_to_ac: None,
    });
    let pv_panel = use_signal(|| PvPanel {
        watts: 550.0,
        dimensions: (1134.0, 2278.0),
        irradiance: 1000.0,
        voltage_mpp: 190.4,
        current_mpp: 2.89,
        voltage_oc: 228.8,
        current_sc: 3.08,
    });
    let inverter = use_signal(|| Inverter {
        ac_phase_config: String::from("3-phase"),
        nominal_ac_power: 4095000.0,
        dc_voltage_range: (835.0, 1500.0),
        dc_mppt_range: (835.0, 1300.0),
        max_dc_current: 5000.0,
        max_efficiency: 99.45,
        ac_frequency_range: ((47.5, 53.0), (57.0, 63.0)),
        ac_voltage_range: (590.0, 610.0),
        power_factor_range: (0.0, 1.0, 0.0),
        dimensions: (4325.0 / 914.4, 2250.0 / 914.4, 1022.0 / 914.4),
        weight: 4045.0,
        ambient_temperature_range: (-20.0, 60.0),
        altitude_limit: 2000.0,
        max_current_sc: 9000.0,
    });
    let triangle_input = use_signal(TriangleInput::default);
    let rectangle_input = use_signal(RectanleInput::default);
    // State to display a result or error
    let mut result_message =
        use_signal(|| (((0.0_f32, 0.0_f32), (0_usize, 0_usize)), 0_usize, 0_usize));
    let mut dmin_buffer = use_signal(|| 3.5_f32);
    let dbm = use_signal(|| 0.4_f32); //distance_between_modubles
    let mut mms_dim = use_signal(|| (0.0_f32, 0.0_f32));
    let mut string_range = use_signal(|| (0.0_f32, 0.0_f32));
    let space_between_tables = use_signal(|| 3.0_f32);
    let mut num_strings_per_inverter = use_signal(|| 0.0_f32);
    // Example of how to use the data elsewhere
    let mut triangle_result = use_signal(|| {
        // (area_acres, total_panels, total_watts)
        (0.0_f32, 0_usize, 0.0_f32)
    });
    let mut tab = use_signal(|| TabKind::Results);
    //String inputs
    rsx! {
           style {{include_str!("../css_files/site_spec.css")}}
            div {class: "prog-description",
                p{
                    "Provides geometrical dimensions specification of a solar farm, based upon provided inputs."
                }
            }
           div { class: "form-container",
               CorrectionFactorsForm {
                   cf: correction_factors
               }
               PvPanelForm {
                   pv: pv_panel
               }

               InverterForm {
                   iv: inverter
               }
           }
           div { class: "button-container",
             button {
                  class: "button",
                  onclick: move |_| shape.set(1),
                  "Triangle"
             }
             button {
                  class: "button",
                  onclick: move |_| shape.set(2),
                  "Rectangle"
             }
           }
           match shape() {
              1 => rsx! {
                  div { class: "block_image",
                        h3 {"Triangle Reference"}
                        img {
                            src: "https://i.ibb.co/SD9y3vch/Screenshot-from-2025-06-17-13-10-08.png",
                            alt: "Group-1",
                            style: "border: 1.4em;  margin-left: 10%; margin-right: 10%; height: 22vh;",
                        }
                    }
                  div { class: "triangle-function",
                  div { class: "rect-form-button",
            TriangleInputForm { ti: triangle_input }
            button {
                class: "button",
                onclick: move |_| {
                    let ti = triangle_input.read();
                    let p = pv_panel.read();
                    let c = correction_factors.read();
                    let i = inverter.read();

                    // Compute string range for panels per MMS (optional, mimic rectangle logic)
                    *string_range.write() = (
                        nms_max(i.dc_voltage_range.1, p.voltage_mpp, c.clone()).unwrap(),
                        nms_min(i.dc_voltage_range.0, p.voltage_mpp, c.clone()).unwrap(),
                    );
                     *num_strings_per_inverter.write() = num_string_per_inv_max(
                        i.max_current_sc,
                        p.current_sc,
                        c.clone()
                    ).unwrap();

                    // Compute MMS section dimension
                    *mms_dim.write() = mms_section_dim(
                        mm_to_yards(p.dimensions.0),
                        mm_to_yards(p.dimensions.1),
                        string_range.read().0.floor() as usize,
                        1,
                        *dbm.read()
                    ).unwrap();

                    // Run the triangle layout calculation
                    let res = triangle_row_layout_with_mms(
                        ti.adj,
                        ti.opp,
                        ti.hyp,
                        ti.alpha_angle,
                        ti.beta_angle,
                        ti.phi_angle,
                        'c', // or choose the relevant base
                        mms_dim.read().1,
                        2.0 + *dmin_buffer.read(),
                        mms_dim.read().0,
                        string_range.read().0.floor() as usize,
                        p.watts,
                    );

                    match res {
                        Ok((area_acres, total_panels, total_watts)) => {
                            triangle_result.set((area_acres, total_panels, total_watts));
                        },
                        Err(e) => {
                            log::error!("Error: {:?}", e);
                        }
                    }
                },
                "Run Triangle"
             }
            }
            // Triangle results display
            div { class: "results-section",
                table { class: "results-table",
                    thead {
                        tr { th { "Spec" } th { "Value" } }
                    }
                    tbody {
                        tr { td { "Area (acres)" }   td { "{triangle_result.read().0:.4}" } }
                        tr { td { "Total Panels" }   td { "{triangle_result.read().1}" } }
                        tr { td { "Total Watts" }    td { "{triangle_result.read().2 as usize}" } }
                    }
                }
            }
        }
    },
              2 => rsx! {
                 div { class: "square-function",
                 div { class: "rect-form-button",
                    RectangleInputForm { ri: rectangle_input }
                    button {class: "button",
                    onclick: move |_| {
                       let r = rectangle_input.read();
                       let p = pv_panel.read();
                       let i = inverter.read();
                       let c = correction_factors.read();
                       *string_range.write() = (
                          nms_max(i.dc_voltage_range.1, p.voltage_mpp, c.clone()).unwrap(),
                          nms_min(i.dc_voltage_range.0, p.voltage_mpp, c.clone()).unwrap(),
                       );
                       tracing::debug!("String Range: {:?}", string_range());
                       *num_strings_per_inverter.write() = num_string_per_inv_max(
                          i.max_current_sc,
                          p.current_sc,
                          c.clone()
                       ).unwrap();
                       *mms_dim.write() = mms_section_dim(
                          mm_to_yards(p.dimensions.0),
                          mm_to_yards(p.dimensions.1),
                          string_range.read().0.floor() as usize,
                          1,
                          *dbm.read()
                       ).unwrap();
                       let unixtime = 1_718_993_600_000;
                       let lat = 34.408331;
                       let lon = -117.241227;
                       let pos = sun::pos(unixtime, lat, lon);
                       let tilt_angle = 25_f32.to_radians();
                       let az = pos.azimuth;
                       let alt = pos.altitude;
                       let lm = mm_to_yards(p.dimensions.0);
                       let h0 = 0.5;
                       let dmin = inter_row_distance(
                          tilt_angle,
                          alt as f32,
                          az as f32, lm, h0
                       ).unwrap();
                       *dmin_buffer.write() += dmin;
                       let rectangle_spec = run_rectangle(
                          r.long_side,
                          r.short_side,
                          *mms_dim.read(),
                          *dmin_buffer.read(),
                          p.watts,
                          string_range.read().0.floor(),
                          *space_between_tables.read(),
                       );
                       match rectangle_spec {
                          Ok(value) => {
                              let total_panels = value.1.0 * value.1.1 * (string_range.read().0.floor() as usize);
                              let total_watts = total_panels * (p.watts as usize);
                              result_message.set((value, total_panels, total_watts));
                          },
                          Err(e) => {
                              log::error!("Error: {:?}", e);
                          }
                       }
                    },
                    "Run Rectangle"
                 }
            }
           div { class: "results-section",
                 table { class: "results-table",
                     thead {
                         tr { th { "Spec" } th { "Value" } }
                     }
                     tbody {
                         tr { td { "Row Width" }      td { "{result_message.read().0.0.0} yds" } }
                         tr { td { "Column Length" }  td { "{result_message.read().0.0.1} yds" } }
                         tr { td {"Tables in a Row"} td { "{result_message.read().0.1.0}" } }
                         tr { td { "Row Count" }      td { "{result_message.read().0.1.1}" } }
                         tr { td { "Total Panels" }   td { "{result_message.read().1}" } }
                         tr { td { "Total Watts" }    td { "{result_message.read().2}" } }
                     }
                 }
                 div { class: "section-details",
                     p {
                         "Section Length: {mms_dim.read().0} yds"
                         br {}
                         "Row Length: {mms_dim.read().1} yds"
                         br {}
                         "Distance Between Rows: {dmin_buffer()} yds"
                         br {}
                         "Distance Between Tables: {space_between_tables()} yds"
                         br {}
                         "String Range Max: {string_range.read().0.floor()}"
                         br {}
                         "Number of Strings Per Inverter: {num_strings_per_inverter()}"
                         br {}
                         "Distance Between Modules: {dbm} yds"
                     }
                 }
             }
            },
        },


         _ => rsx! {
             div { class: "select-shape", "Select shape." }
         },
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
                    TabKind::Results => rsx!( ShapesResultsTable {} ),
                    TabKind::Inputs => rsx!( DefinitionsOverview {} ),
                }
            }
        }
         }
}

#[component]
pub fn RectangleInputForm(mut props: RectangleInpProps) -> Element {
    let ri = props.ri.read();
    rsx! {
        div { class: "function-form-section",
            h3 { "Rectangle Inputs" }
            label { "Long Side(yds)" }
            input {
                r#type: "number", step: "any",
                value: "{ri.long_side}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ri.write().long_side = val;
                    }
                }
            }
            label { "Short Side(yds)" }
            input {
                r#type: "number", step: "any",
                value: "{ri.short_side}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ri.write().short_side = val;
                    }
                }
            }
        }
    }
}

#[component]
pub fn TriangleInputForm(mut props: TriangleInpProps) -> Element {
    let ti = props.ti.read();

    rsx! {
        div { class: "function-form-section",
            h3 { "Triangle Inputs" }
            label { "Adjacent(yds)" }
            input {
                r#type: "number", step: "any",
                value: "{ti.adj}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ti.write().adj = val;
                    }
                }
            }
            label { "Opposite(yds) "}
            input {
                r#type: "number", step: "any",
                value: "{ti.opp}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ti.write().opp = val;
                    }
                }
            }
            label { "Hypotenuse(yds) " }
            input {
                r#type: "number", step: "any",
                value: "{ti.hyp}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ti.write().hyp = val;
                    }
                }
            }
            label { "Alpha Angle" }
            input {
                r#type: "number", step: "any",
                value: "{ti.alpha_angle}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ti.write().alpha_angle = val;
                    }
                }
            }
            label { "Beta Angle" }
            input {
                r#type: "number", step: "any",
                value: "{ti.beta_angle}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ti.write().beta_angle = val;
                    }
                }
            }
            label { "Phi Angle" }
            input {
                r#type: "number", step: "any",
                value: "{ti.phi_angle}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ti.write().phi_angle = val;
                    }
                }
            }
        }
    }
}

fn mm_to_yards(mm: f32) -> f32 {
    mm / 1000.0 * 1.09361
}

pub fn run_rectangle(
    long_side: f32,
    short_side: f32,
    mms_dim: (f32, f32),
    dmin: f32,
    panel_watts: f32,
    string_count: f32,
    space_between_tables: f32,
) -> Result<((f32, f32), (usize, usize)), MathError> {
    // --- Rectangular Sections ---
    println!("Running rectangle layout for section 1");
    let group = mms_section_group_dim(
        mms_dim.0,
        mms_dim.1,
        (long_side, short_side),
        space_between_tables,
        dmin,
        panel_watts,
        string_count,
    )?;
    Ok(group)
}

pub fn nms_max(v1: f32, v3: f32, cfact: CorrectionFactors) -> Result<f32, MathError> {
    if v3 == 0.0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(v1 / (v3 * cfact.temperature))
}

pub fn nms_min(v2: f32, v3: f32, cfact: CorrectionFactors) -> Result<f32, MathError> {
    if v3 == 0.0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(v2 / (v3 * cfact.temperature))
}

pub fn num_string_per_inv_max(
    sc_inv: f32,
    sc_pv: f32,
    cfact: CorrectionFactors,
) -> Result<f32, MathError> {
    if sc_pv == 0.0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(sc_inv / (sc_pv * cfact.over_current * cfact.irradiance))
}

pub fn num_inv(
    power_inv: f32,
    plant_capacity: f32,
    cfact: CorrectionFactors,
) -> Result<f32, MathError> {
    let efficiency = cfact.dc_to_ac.ok_or(MathError::InvalidInput(
        "DC-to-AC conversion factor missing".to_string(),
    ))?;
    Ok(plant_capacity / (power_inv * efficiency))
}

pub fn num_transformers(
    power_trf: f32,
    plant_capacity: f32,
    cfact: CorrectionFactors,
) -> Result<f32, MathError> {
    let efficiency = cfact.dc_to_ac.ok_or(MathError::InvalidInput(
        "DC-to-AC conversion factor missing".to_string(),
    ))?;
    Ok(plant_capacity / (power_trf * efficiency))
}

pub fn inter_row_distance(
    tilt_angle: f32,
    alt_angle: f32,
    azm_angle: f32,
    lm: f32,
    h0: f32,
) -> Result<f32, MathError> {
    println!("Tilt Angle: {:?}", tilt_angle);
    println!("Altitude: {:?}", alt_angle);
    println!("azm_angle: {:?}", azm_angle);
    let h = h0 + (lm * tilt_angle.sin());
    let l_sh = h / alt_angle.tan();
    let dist = l_sh * azm_angle.cos();
    Ok(dist.max(0.0))
}

pub fn mms_section_dim(
    module_wdth: f32,
    module_lgth: f32,
    num_panels_hrz: usize,
    num_panels_vrt: usize,
    dmm: f32,
) -> Result<(f32, f32), MathError> {
    let width_mms = (module_wdth * num_panels_hrz as f32) + dmm * (num_panels_hrz as f32 - 1.0);
    let length_mms = (module_lgth * num_panels_vrt as f32) + dmm * (num_panels_vrt as f32 - 1.0);
    Ok((width_mms, length_mms))
}

pub fn mms_section_group_dim(
    mms_sec_wdth: f32,
    mms_sec_lgth: f32,
    block: (f32, f32),
    dw: f32,
    dr: f32,
    _watts: f32,
    _num_panels: f32,
) -> Result<((f32, f32), (usize, usize)), MathError> {
    let num_sec_vrt = block.1 / (mms_sec_lgth + dr);
    let num_sec_hrz = block.0 / (mms_sec_wdth + dw);
    tracing::debug!("mms_sec_lgth {:?}", mms_sec_lgth);
    tracing::debug!("mms_sec_wdth {:?}", mms_sec_wdth);
    tracing::debug!("dr {:?}", dr);
    tracing::debug!("dw {:?}", dw);
    tracing::debug!("num_sec_vrt {:?}", num_sec_vrt);
    tracing::debug!("num_sec_hrz {:?}", num_sec_hrz);
    let width_mms_section = (mms_sec_wdth * num_sec_hrz.floor()) + dw * (num_sec_hrz.floor() - 1.0);
    let length_mms_section =
        (mms_sec_lgth * num_sec_vrt.floor()) + dr * (num_sec_vrt.floor() - 1.0);
    Ok((
        (width_mms_section, length_mms_section),
        (num_sec_hrz.floor() as usize, num_sec_vrt.floor() as usize),
    ))
}

pub fn run_triangle(
    adj: f32,
    opp: f32,
    hyp: f32,
    alpha_angle: f32,
    beta_angle: f32,
    phi_angle: f32,
    mms_dim: (f32, f32),
    dmin: f32,
    panel_watts: f32,
    string_count: usize,
) -> Result<(), MathError> {
    // --- Triangular Sections ---
    println!("Running triangle layout for section 1");
    let layout_1 = triangle_row_layout_with_mms(
        adj,
        opp,
        hyp,
        alpha_angle,
        beta_angle,
        phi_angle,
        'c',
        mms_dim.1,
        2.0 + dmin,
        mms_dim.0,
        string_count,
        panel_watts,
    )?;
    println!("Section 1 layout: {:?}", layout_1);
    println!("Running triangle layout for section 6");
    Ok(())
}

/// Converts degrees to radians
fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

pub fn triangle_area_acres(
    a: f32,
    b: f32,
    c: f32,
    alpha_deg: f32,
    beta_deg: f32,
    gamma_deg: f32,
) -> Result<f32, MathError> {
    // Check validity
    if [a, b, c].iter().any(|&s| s <= 0.0) || (alpha_deg + beta_deg + gamma_deg - 180.0).abs() > 0.5
    {
        return Err(MathError::InvalidInput("Invalid triangle".into()));
    }

    // Convert from yards to meters (1 yard = 0.9144 meters)
    let (a_m, b_m, c_m) = (a * 0.9144, b * 0.9144, c * 0.9144);

    // Use Heron's formula: s = (a+b+c)/2
    let s = (a_m + b_m + c_m) / 2.0;
    let area_m2 = (s * (s - a_m) * (s - b_m) * (s - c_m)).sqrt();

    // Convert square meters to acres (1 acre = 4046.85642 m²)
    let area_acres = area_m2 / 4046.85642;

    Ok(area_acres)
}
/// Computes the area of a triangle using two sides and the included angle.
/// # Parameters:
/// - `a`: One side length (m)
/// - `b`: Another side length (m)
/// - `angle_deg`: Included angle between `a` and `b` in degrees
/// # Returns:
/// - `Ok(f32)`: Area in square meters
fn area_from_two_sides_angle(a: f32, b: f32, angle_deg: f32) -> Result<f32, MathError> {
    if a <= 0.0 || b <= 0.0 {
        return Err(MathError::InvalidInput(
            "Side lengths must be positive".into(),
        ));
    }
    let angle_rad = deg_to_rad(angle_deg);
    if angle_rad <= 0.0 || angle_rad >= PI {
        return Err(MathError::InvalidInput(
            "Angle must be between 0° and 180°".into(),
        ));
    }
    Ok(0.5 * a * b * angle_rad.sin())
}

pub fn triangle_row_layout_with_mms(
    a: f32,
    b: f32,
    c: f32,
    alpha_deg: f32,
    beta_deg: f32,
    gamma_deg: f32,
    base_side: char,       // 'a', 'b', or 'c'
    mms_lgth: f32,         // vertical length of MMS section (m)
    inter_row_dist: f32,   // spacing between MMS rows (m)
    mms_width: f32,        // width of MMS section (m)
    panels_per_mms: usize, // number of panels in each MMS section
    watts_per_panel: f32,  // wattage of each panel
) -> Result<(f32, usize, f32), MathError> {
    if [a, b, c].iter().any(|&s| s <= 0.0) || (alpha_deg + beta_deg + gamma_deg - 180.0).abs() > 0.5
    {
        return Err(MathError::InvalidInput("Invalid triangle".into()));
    }

    // Determine base side and included angle
    let (base, adj, opp, angle_deg) = match base_side {
        'a' | 'A' => (a, b, c, alpha_deg),
        'b' | 'B' => (b, a, c, beta_deg),
        'c' | 'C' => (c, a, b, gamma_deg),
        _ => {
            return Err(MathError::InvalidInput(
                "Base must be 'a', 'b', or 'c'".into(),
            ))
        }
    };

    let angle_rad = deg_to_rad(angle_deg);
    let height = (adj * opp * angle_rad.sin()) / base;
    let row_pitch = mms_lgth + inter_row_dist;

    let mut layout = Vec::new();
    let mut h_i = row_pitch * 0.5;
    let mut i = 0;
    let mut total_mms_groups = 0;

    while h_i < height {
        let row_width = base * (1.0 - h_i / height).clamp(0.0, 1.0);
        let groups = (row_width / mms_width).floor() as usize;
        total_mms_groups += groups;
        layout.push((i, row_width, groups));
        i += 1;
        h_i += row_pitch;
    }

    let total_panels = total_mms_groups * panels_per_mms;
    let total_watts = total_panels as f32 * watts_per_panel;

    Ok((
        triangle_area_acres(a, b, c, alpha_deg, beta_deg, gamma_deg).unwrap(),
        total_panels,
        total_watts,
    ))
}

#[component]
pub fn CorrectionFactorsForm(mut props: CorrectionFactorsFormProps) -> Element {
    let mut cfact = use_signal(|| {
        (
            props.cf.read().temperature.to_string(),
            props.cf.read().irradiance.to_string(),
            props.cf.read().over_current.to_string(),
            props.cf.read().dc_to_ac.unwrap_or(0.96).to_string(),
        )
    });
    rsx! {
        div { class: "form-section",
            h3 { "Correction Factors" }
            label { "Temperature" }
            input {
                r#type: "number", step: "any",
                value: "{cfact().0}",

                oninput: move |e| {
                    cfact.write().0 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.cf.write().temperature = val;
                    }
                }
            }
            label { "Irradiance" }
            input {
                r#type: "number", step: "any",
                value: "{cfact().1}",
                oninput: move |e| {
                    cfact.write().1 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.cf.write().irradiance = val;
                    }
                }
            }
            label { "Over Current" }
            input {
                r#type: "number", step: "any",
                value: "{cfact().2}",
                oninput: move |e| {
                    cfact.write().2 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.cf.write().over_current = val;
                    }
                }
            }
            label { "DC to AC Efficiency" }
            input {
                r#type: "number", step: "any",
                value: "{cfact().3}",
                oninput: move |e| {
                    cfact.write().3 = e.value().parse().unwrap();
                    props.cf.write().dc_to_ac = e.value().parse().ok();
                }
            }
        }
    }
}

#[component]
pub fn PvPanelForm(mut props: PvPanelFormProps) -> Element {
    let mut panel = use_signal(|| {
        (
            props.pv.read().watts.to_string(),
            props.pv.read().dimensions.0.to_string(),
            props.pv.read().dimensions.1.to_string(),
            props.pv.read().irradiance.to_string(),
            props.pv.read().voltage_mpp.to_string(),
            props.pv.read().current_mpp.to_string(),
            props.pv.read().voltage_oc.to_string(),
            props.pv.read().current_sc.to_string(),
        )
    });
    rsx! {
        div { class: "form-section",
            h3 { "PV Panel" }
            label { "Watts" }
            input {
                r#type: "number", step: "any",
                value: "{panel().0}",
                oninput: move |e| {
                    panel.write().0 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                       props.pv.write().watts = val;
                    }
                }
            }
            label { "Width (mm)" }
            input {
                r#type: "number", step: "any",
                value: "{panel().1}",
                oninput: move |e| {
                    panel.write().1 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.pv.write().dimensions.0 = val;
                    }
                }
            }
            label { "Height (mm)" }
            input {
                r#type: "number", step: "any",
                value: "{panel().2}",
                oninput: move |e| {
                    panel.write().2 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.pv.write().dimensions.1 = val;
                    }
                }
            }
            label { "Irradiance (W/m²)" }
            input {
                r#type: "number", step: "any",
                value: "{panel().3}",
                oninput: move |e| {
                    panel.write().3 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.pv.write().irradiance = val;
                    }
                }
            }
            label { "V_mpp" }
            input {
                r#type: "number", step: "any",
                value: "{panel().4}",
                oninput: move |e| {
                    panel.write().4 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.pv.write().voltage_mpp = val;
                    }
                }
            }
            label { "I_mpp" }
            input {
                r#type: "number", step: "any",
                value: "{panel().5}",
                oninput: move |e| {
                    panel.write().5 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.pv.write().current_mpp = val;
                    }
                }
            }
            label { "V_oc" }
            input {
                r#type: "number", step: "any",
                value: "{panel().6}",
                oninput: move |e| {
                    panel.write().6 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.pv.write().voltage_oc = val;
                    }
                }
            }
            label { "I_sc" }
            input {
                r#type: "number", step: "any",
                value: "{panel().7}",
                oninput: move |e| {
                    panel.write().7 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.pv.write().current_sc = val;
                    }
                }
            }
        }
    }
}

#[component]
pub fn InverterForm(mut props: InverterFormProps) -> Element {
    let mut inv = use_signal(|| {
        (
            props.iv.read().nominal_ac_power.to_string(),
            props.iv.read().dc_voltage_range.0.to_string(),
            props.iv.read().dc_voltage_range.1.to_string(),
            props.iv.read().max_current_sc.to_string(),
        )
    });
    rsx! {
        div { class: "form-section",
            h3 { "Inverter" }
            label { "Nominal AC Power (kVA)" }
            input {
                r#type: "number", step: "any",
                value: "{inv().0}",
                oninput: move |e| {
                    inv.write().0 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.iv.write().nominal_ac_power = val;
                    }
                }
            }
            label { "DC Voltage Range (min)" }
            input {
                r#type: "number", step: "any",
                value: "{inv().1}",
                oninput: move |e| {
                    inv.write().1 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.iv.write().dc_voltage_range.0 = val;
                    }
                }
            }
            label { "DC Voltage Range (max)" }
            input {
                r#type: "number", step: "any",
                value: "{inv().2}",
                oninput: move |e| {
                    inv.write().2 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.iv.write().dc_voltage_range.1 = val;
                    }
                }
            }

            label { "Max Short-Circuit Current from PV (A)" }
            input {
                r#type: "number", step: "any",
                value: "{inv().3}",
                oninput: move |e| {
                    inv.write().3 = e.value().parse().unwrap();
                    if let Ok(val) = e.value().parse() {
                        props.iv.write().max_current_sc = val;
                    }
                }
            }
        }
    }
}

#[component]
pub fn InverterFormFull(mut props: Inverter) -> Element {
    rsx! {
        div { class: "form-section",
            h3 { "Inverter" }
            label { "AC Phase Config" }
            input {
                value: "{props.ac_phase_config}",
                oninput: move |e| {
                    props.ac_phase_config = e.value();
                }
            }
            label { "Nominal AC Power (kVA)" }
            input {
                r#type: "number", step: "any",
                value: "{props.nominal_ac_power}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.nominal_ac_power = val;
                    }
                }
            }
            label { "DC Voltage Range (min)" }
            input {
                r#type: "number", step: "any",
                value: "{props.dc_voltage_range.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.dc_voltage_range.0 = val;
                    }
                }
            }
            label { "DC Voltage Range (max)" }
            input {
                r#type: "number", step: "any",
                value: "{props.dc_voltage_range.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.dc_voltage_range.1 = val;
                    }
                }
            }
            label { "MPPT Range (min)" }
            input {
                r#type: "number", step: "any",
                value: "{props.dc_mppt_range.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.dc_mppt_range.0 = val;
                    }
                }
            }
            label { "MPPT Range (max)" }
            input {
                r#type: "number", step: "any",
                value: "{props.dc_mppt_range.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.dc_mppt_range.1 = val;
                    }
                }
            }
            label { "Max DC Current (A)" }
            input {
                r#type: "number", step: "any",
                value: "{props.max_dc_current}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.max_dc_current = val;
                    }
                }
            }
            label { "Max Efficiency (%)" }
            input {
                r#type: "number", step: "any",
                value: "{props.max_efficiency}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.max_efficiency = val;
                    }
                }
            }
            label { "AC Frequency Range 1 (min)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ac_frequency_range.0.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ac_frequency_range.0.0 = val;
                    }
                }
            }
            label { "AC Frequency Range 1 (max)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ac_frequency_range.0.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ac_frequency_range.0.1 = val;
                    }
                }
            }
            label { "AC Frequency Range 2 (min)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ac_frequency_range.1.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ac_frequency_range.1.0 = val;
                    }
                }
            }
            label { "AC Frequency Range 2 (max)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ac_frequency_range.1.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ac_frequency_range.1.1 = val;
                    }
                }
            }
            label { "AC Voltage Range (min)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ac_voltage_range.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ac_voltage_range.0 = val;
                    }
                }
            }
            label { "AC Voltage Range (max)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ac_voltage_range.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ac_voltage_range.1 = val;
                    }
                }
            }
            label { "Power Factor Range (leading)" }
            input {
                r#type: "number", step: "any",
                value: "{props.power_factor_range.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.power_factor_range.0 = val;
                    }
                }
            }
            label { "Power Factor Range (unity)" }
            input {
                r#type: "number", step: "any",
                value: "{props.power_factor_range.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.power_factor_range.1 = val;
                    }
                }
            }
            label { "Power Factor Range (lagging)" }
            input {
                r#type: "number", step: "any",
                value: "{props.power_factor_range.2}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                       props.power_factor_range.2 = val;
                    }
                }
            }
            label { "Dimensions (length mm)" }
            input {
                r#type: "number", step: "any",
                value: "{props.dimensions.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                       props.dimensions.0 = val;
                    }
                }
            }
            label { "Dimensions (width mm)" }
            input {
                r#type: "number", step: "any",
                value: "{props.dimensions.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                       props.dimensions.1 = val;
                    }
                }
            }
            label { "Dimensions (height mm)" }
            input {
                r#type: "number", step: "any",
                value: "{props.dimensions.2}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.dimensions.2 = val;
                    }
                }
            }
            label { "Weight (kg)" }
            input {
                r#type: "number", step: "any",
                value: "{props.weight}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.weight = val;
                    }
                }
            }
            label { "Ambient Temp Range (min)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ambient_temperature_range.0}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ambient_temperature_range.0 = val;
                    }
                }
            }
            label { "Ambient Temp Range (max)" }
            input {
                r#type: "number", step: "any",
                value: "{props.ambient_temperature_range.1}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.ambient_temperature_range.1 = val;
                    }
                }
            }
            label { "Altitude Limit (m)" }
            input {
                r#type: "number", step: "any",
                value: "{props.altitude_limit}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.altitude_limit = val;
                    }
                }
            }
            label { "Max Short-Circuit Current from PV (A)" }
            input {
                r#type: "number", step: "any",
                value: "{props.max_current_sc}",
                oninput: move |e| {
                    if let Ok(val) = e.value().parse() {
                        props.max_current_sc = val;
                    }
                }
            }
        }
    }
}
