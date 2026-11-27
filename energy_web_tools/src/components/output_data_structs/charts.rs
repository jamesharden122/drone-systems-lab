use chrono::NaiveDateTime;
use chrono::{DateTime, TimeZone, Utc};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Props, PartialEq, Clone)]
pub struct ChartProps {
    pub data: Vec<HashMap<String, String>>,
    pub metric: String, // e.g. "revenue", "expense", "ppw"
}

#[component]
pub fn ChartFromMapTable(props: ChartProps) -> Element {
    let width = 1500.0;
    let height = 600.0;
    let y_padding = 100.0;
    let x_padding = y_padding;
    //let max_points = 300;

    // Extract data
    let mut combined: Vec<(f64, f64)> = props
        .data
        .iter()
        .filter_map(|map| {
            let date_str = map.get("date_time")?;
            let date_clean = date_str.trim_matches('"');

            let date = DateTime::parse_from_rfc3339(date_clean)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    NaiveDateTime::parse_from_str(date_clean, "%Y-%m-%d %H:%M:%S")
                        .map(|naive| DateTime::<Utc>::from_utc(naive, Utc))
                })
                .ok()?
                .timestamp_nanos() as f64;

            let metric_val = map.get(&props.metric)?.parse::<f64>().ok()?;
            Some((date, metric_val))
        })
        .collect();

    combined.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let (x_values, y_values): (Vec<_>, Vec<_>) = combined.iter().cloned().unzip();
    if x_values.is_empty() || y_values.is_empty() {
        return rsx!(div { "No data available" });
    }

    // Axis bounds
    let x_min = *x_values
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let x_max = *x_values
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let x_min_sec = (x_min / 1_000_000_000.0).floor();
    let x_max_sec = (x_max / 1_000_000_000.0).floor();
    let range_secs = x_max_sec - x_min_sec; // convert from ns to seconds
    let (y_min, y_max) = (
        *y_values
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        *y_values
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
    );

    // Scale functions
    let scale_x = |x: f64| ((x - x_min) / (x_max - x_min) * (width - 2.0 * x_padding)) + x_padding;
    let scale_y =
        |y: f64| height - ((y - y_min) / (y_max - y_min) * (height - 2.0 * y_padding)) - y_padding;
    // Chart points
    let points: Vec<String> = x_values
        .iter()
        .zip(y_values.iter())
        .map(|(&x, &y)| format!("{},{}", scale_x(x), scale_y(y)))
        .collect();
    let polyline_points = points.join(" ");

    // Generate ticks - ensures first tick is x_min and last tick is x_max
    let x_ticks = 10;
    let y_ticks = 6;
    // Convert min & max to DateTime<Utc>

    let start_dt = chrono::Utc.timestamp((x_min / 1_000_000_000.0) as i64, 0);
    let end_dt = chrono::Utc.timestamp((x_max / 1_000_000_000.0) as i64, 0);
    tracing::debug!("Start Date: {:?}", start_dt);
    tracing::debug!("End Date: {:?}", end_dt);

    // Total range duration
    let total_duration = end_dt - start_dt;
    let step = total_duration / (x_ticks - 1);
    // Generate ticks based on seconds
    // Generate tick DateTimes
    // Generate evenly spaced ticks in seconds
    let x_tick_values: Vec<f64> = (0..x_ticks)
        .map(|i| {
            let tick_sec = x_min_sec + ((i as f64) / ((x_ticks - 1) as f64)) * range_secs;
            tick_sec * 1_000_000_000.0 // convert back to ns for scaling
        })
        .collect();
    let y_tick_values: Vec<f64> = (0..y_ticks)
        .map(|i| y_min + ((i as f64) / ((y_ticks - 1) as f64)) * (y_max - y_min))
        .collect();

    // Create tick elements
    // Create tick elements
    let x_tick_elements: Vec<_> = x_tick_values
        .iter()
        .map(|&xv| {
            let x_pos = scale_x(xv);
            let timestamp = chrono::Utc.timestamp((xv / 1_000_000_000.0) as i64, 0);

            // Format depending on range
            let formatted = if range_secs > 10.0 * 86_400.0 {
                timestamp.format("%m-%d").to_string() // multi-day range → date only
            } else {
                timestamp.format("%m-%d %H:%M").to_string() // shorter range → date + time
            };

            rsx! {
                line {
                    x1: {x_pos}, y1: {height - y_padding},
                    x2: {x_pos}, y2: {height - y_padding + 6.0},
                    stroke: "#555", stroke_width: "1"
                }
                text {
                    x: {x_pos}, y: {height - y_padding + 20.0},
                    font_size: "18", text_anchor: "middle", fill: "#FFF",
                    "{formatted}"
                }
            }
        })
        .collect();

    let y_tick_elements: Vec<_> = y_tick_values
        .iter()
        .map(|&yv| {
            let y_pos = scale_y(yv);
            let label = format!("{:.2}", yv); // format to 2 decimal places
            rsx! {
                line {
                    x1: x_padding - 6.0, y1: y_pos,
                    x2: x_padding,       y2: y_pos,
                    stroke: "black", stroke_width: "1"
                }
                text {
                    x: x_padding - 10.0, y: y_pos + 4.0,
                    font_size: "18", text_anchor: "end",fill: "#FFF",
                    "{label}"
                }
            }
        })
        .collect();

    rsx! {
    svg {
        width: "{width}",
        height: "{height}",
        // Background
        rect {
            x: 0,
            y: 0,
            width: "{width}",
            height: "{height}",
            fill: "#353935" // dark background
        }

        // Grid lines (X)
        for xv in &x_tick_values {
            line {
                x1: "{scale_x(*xv)}", y1: "{y_padding/4.0}",
                x2: "{scale_x(*xv)}", y2: "{height-y_padding}",
                stroke: "#2A2A2A", stroke_width: "1",
                stroke_dasharray: "4 4"
            }
        }

        // Grid lines (Y)
        for yv in &y_tick_values {
            line {
                x1: "{x_padding}", y1: "{scale_y(*yv)}",
                x2: "{width-x_padding}", y2: "{scale_y(*yv)}",
                stroke: "#2A2A2A", stroke_width: "1",
                stroke_dasharray: "4 4"
            }
        }

        // Chart line
        polyline {
            points: "{polyline_points}",
            fill: "none",
            stroke: "#00BFFF", // bright blue
            stroke_width: "2",
            stroke_linejoin: "round",
            stroke_linecap: "round"
        }

        // Axes
        line {
            x1: "{x_padding}",
            y1: "{height-y_padding}",
            x2: "{width-x_padding}",
            y2: "{height-y_padding}",
            stroke: "#888",
            stroke_width: "3"
        }
        line {
            x1: "{x_padding}",
            y1: "{y_padding/4.0}",
            x2: "{x_padding}",
            y2: "{height-y_padding}",
            stroke: "#888",
            stroke_width: "3"
        }
        // Inject ticks
        {x_tick_elements.into_iter()}
        {y_tick_elements.into_iter()}

        // Labels
        text {
            x: "{width/2.0}", y: "{height-20.0}",
            text_anchor: "middle", font_size: "28", fill: "#FFF", "Time"
        }
        text {
            x: 30, y: "{height/2.0}",
            transform: "rotate(-90, 30, {height/2.0})",
            text_anchor: "middle", font_size: "28", fill: "#FFF",
            "{props.metric}"
        }
    }

    }
}
