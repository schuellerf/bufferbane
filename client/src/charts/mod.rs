//! Chart generation (PNG and interactive HTML export)

use crate::config::Config;
use crate::testing::Measurement;
use anyhow::Result;
use plotters::prelude::*;
use std::collections::HashMap;
use std::path::Path;

/// Generate latency chart with min/max/avg/percentile lines and shaded variance area
pub fn generate_latency_chart(
    measurements: &[Measurement],
    output_path: &Path,
    config: &Config,
    num_segments: usize,
    db: Option<&crate::storage::Database>,
) -> Result<()> {
    if measurements.is_empty() {
        anyhow::bail!("No measurements to chart");
    }
    
    // Group measurements by target
    let mut by_target: HashMap<String, Vec<(i64, f64)>> = HashMap::new();
    
    for m in measurements {
        // Include both ICMP and server-based tests
        if (m.test_type == "icmp" || m.test_type == "server_echo") && m.status == "success" {
            if let Some(rtt) = m.rtt_ms {
                by_target
                    .entry(m.target.clone())
                    .or_insert_with(Vec::new)
                    .push((m.timestamp, rtt));
            }
        }
    }
    
    if by_target.is_empty() {
        anyhow::bail!("No successful measurements to chart");
    }
    
    // Calculate time range
    let min_time = measurements.iter().map(|m| m.timestamp).min().unwrap();
    let max_time = measurements.iter().map(|m| m.timestamp).max().unwrap();
    
    // Create chart
    let root = BitMapBackend::new(
        output_path,
        (config.export.chart_width, config.export.chart_height),
    ).into_drawing_area();
    
    root.fill(&WHITE)?;
    
    // Calculate global min/max for Y axis
    let all_rtts: Vec<f64> = by_target.values()
        .flat_map(|v| v.iter().map(|(_, rtt)| *rtt))
        .collect();
    
    let min_rtt = all_rtts.iter().copied().fold(f64::INFINITY, f64::min);
    let max_rtt = all_rtts.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    
    let y_margin = (max_rtt - min_rtt) * 0.1;
    let y_min = (min_rtt - y_margin).max(0.0);
    let y_max = max_rtt + y_margin;
    
    // Build chart
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Latency Over Time ({} to {})",
                chrono::DateTime::from_timestamp(min_time, 0).unwrap().format("%Y-%m-%d %H:%M"),
                chrono::DateTime::from_timestamp(max_time, 0).unwrap().format("%Y-%m-%d %H:%M")
            ),
            ("sans-serif", 40).into_font(),  // Larger title
        )
        .margin(15)
        .x_label_area_size(60)  // Larger area for labels
        .y_label_area_size(80)  // Larger area for labels
        .build_cartesian_2d(min_time..max_time, y_min..y_max)?;
    
    chart
        .configure_mesh()
        .x_label_formatter(&|x| {
            chrono::DateTime::from_timestamp(*x, 0)
                .map(|dt| dt.format("%H:%M").to_string())
                .unwrap_or_default()
        })
        .y_desc("Latency (ms)")
        .x_desc("Time")
        .label_style(("sans-serif", 20))  // Larger axis labels
        .axis_desc_style(("sans-serif", 24))  // Larger axis descriptions
        .draw()?;
    
    // Draw each target
    let colors = vec![
        &BLUE,
        &GREEN,
        &RED,
        &CYAN,
        &MAGENTA,
    ];
    
    for (idx, (target, points)) in by_target.iter().enumerate() {
        let color = colors[idx % colors.len()];
        
        // Sort points by time
        let mut sorted_points = points.clone();
        sorted_points.sort_by_key(|(t, _)| *t);
        
        // Split into segments where gap > 5 minutes (300 seconds)
        let segments = split_into_segments(&sorted_points, 300);
        
        // Process each continuous segment separately
        for (segment_idx, segment) in segments.iter().enumerate() {
            // Calculate statistics for each window within this segment
            let window_size = ((max_time - min_time) / num_segments as i64).max(1);
            let mut windowed_stats = Vec::new();
            
            for window_start in (min_time..=max_time).step_by(window_size as usize) {
                let window_end = window_start + window_size;
                let window_points: Vec<f64> = segment
                    .iter()
                    .filter(|(t, _)| *t >= window_start && *t < window_end)
                    .map(|(_, rtt)| *rtt)
                    .collect();
                
                if !window_points.is_empty() {
                    let stats = calculate_statistics(&window_points);
                    windowed_stats.push((window_start + window_size / 2, stats));
                }
            }
            
            if windowed_stats.is_empty() {
                continue;
            }
            
            // Draw shaded area between min and max
            let area_points: Vec<_> = windowed_stats
                .iter()
                .map(|(t, stats)| (*t, stats.min))
                .chain(
                    windowed_stats
                        .iter()
                        .rev()
                        .map(|(t, stats)| (*t, stats.max))
                )
                .collect();
            
            chart.draw_series(
                std::iter::once(Polygon::new(
                    area_points,
                    color.mix(0.1).filled(),
                ))
            )?;
            
            // Draw min line (thin)
            chart.draw_series(LineSeries::new(
                windowed_stats.iter().map(|(t, stats)| (*t, stats.min)),
                color.stroke_width(1),
            ))?;
            
            // Draw max line (thin)
            chart.draw_series(LineSeries::new(
                windowed_stats.iter().map(|(t, stats)| (*t, stats.max)),
                color.stroke_width(1),
            ))?;
            
            // Draw P95 line (dashed)
            // Note: plotters doesn't easily support dashed lines, so we'll use thin lines
            chart.draw_series(LineSeries::new(
                windowed_stats.iter().map(|(t, stats)| (*t, stats.p95)),
                color.mix(0.7).stroke_width(1),
            ))?;
            
            // Draw P99 line (dashed)
            chart.draw_series(LineSeries::new(
                windowed_stats.iter().map(|(t, stats)| (*t, stats.p99)),
                color.mix(0.5).stroke_width(1),
            ))?;
            
            // Draw avg line (bold) - only add legend for first segment of each target
            if segment_idx == 0 {
                chart.draw_series(LineSeries::new(
                    windowed_stats.iter().map(|(t, stats)| (*t, stats.avg)),
                    color.stroke_width(3),
                ))?.label(target.clone())
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(3)));
            } else {
                chart.draw_series(LineSeries::new(
                    windowed_stats.iter().map(|(t, stats)| (*t, stats.avg)),
                    color.stroke_width(3),
                ))?;
            }
        }
    }
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 18))  // Larger legend font
        .draw()?;
    
    // Draw events if database is available
    if let Some(database) = db {
        if let Ok(events) = database.query_events(min_time, max_time) {
            // Draw event markers
            for event in events {
                let event_x = event.timestamp;
                
                // Determine color based on event type
                let color = match event.event_type.as_str() {
                    "high_latency" => RGBColor(255, 165, 0),      // Orange
                    "packet_loss" => RGBColor(255, 0, 0),         // Red
                    "error" => RGBColor(139, 0, 0),               // Dark Red
                    "ip_change" => RGBColor(46, 134, 222),        // Blue
                    "gateway_change" => RGBColor(155, 89, 182),   // Purple
                    _ => RGBColor(136, 136, 136),                 // Gray
                };
                
                // Draw vertical line from top to bottom
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(event_x, y_min), (event_x, y_max)],
                    ShapeStyle {
                        color: color.mix(0.3).to_rgba(),
                        filled: false,
                        stroke_width: 1,
                    },
                )))?;
                
                // Draw marker at top
                chart.draw_series(std::iter::once(Circle::new(
                    (event_x, y_max - (y_max - y_min) * 0.02),
                    5,
                    ShapeStyle {
                        color: color.mix(0.8).to_rgba(),
                        filled: true,
                        stroke_width: 2,
                    },
                )))?;
            }
        }
    }
    
    root.present()?;
    
    Ok(())
}

struct Statistics {
    min: f64,
    max: f64,
    avg: f64,
    p95: f64,
    p99: f64,
}

/// Split time series data into continuous segments, breaking when gap > max_gap_seconds
fn split_into_segments(points: &[(i64, f64)], max_gap_seconds: i64) -> Vec<Vec<(i64, f64)>> {
    if points.is_empty() {
        return vec![];
    }
    
    let mut segments = Vec::new();
    let mut current_segment = Vec::new();
    
    for (i, point) in points.iter().enumerate() {
        if i > 0 {
            let prev_time = points[i - 1].0;
            let curr_time = point.0;
            let gap = curr_time - prev_time;
            
            // If gap is too large, start a new segment
            if gap > max_gap_seconds {
                if !current_segment.is_empty() {
                    segments.push(current_segment.clone());
                    current_segment.clear();
                }
            }
        }
        
        current_segment.push(*point);
    }
    
    // Add the last segment if not empty
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }
    
    segments
}

fn calculate_statistics(values: &[f64]) -> Statistics {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let len = sorted.len();
    let min = sorted[0];
    let max = sorted[len - 1];
    let avg = sorted.iter().sum::<f64>() / len as f64;
    
    let p95_idx = ((len as f64) * 0.95) as usize;
    let p99_idx = ((len as f64) * 0.99) as usize;
    
    let p95 = sorted.get(p95_idx).copied().unwrap_or(max);
    let p99 = sorted.get(p99_idx).copied().unwrap_or(max);
    
    Statistics {
        min,
        max,
        avg,
        p95,
        p99,
    }
}

/// Load and process HTML template for interactive charts
const INTERACTIVE_TEMPLATE: &str = include_str!("../templates/interactive_chart.html");

/// Generate interactive HTML chart with hover tooltips
pub fn generate_interactive_chart(
    measurements: &[Measurement],
    output_path: &Path,
    _config: &Config,
    num_segments: usize,
    db: Option<&crate::storage::Database>,
) -> Result<()> {
    if measurements.is_empty() {
        anyhow::bail!("No measurements to chart");
    }
    
    // Group measurements by target and metric type
    // For server tests, we'll have upload, download, and rtt
    // For ICMP tests, we'll only have rtt
    #[derive(Clone)]
    struct SeriesData {
        label: String,
        color: String,
        points: Vec<(i64, f64)>,
        #[allow(dead_code)]
        is_server_metric: bool,
    }
    
    let mut series: Vec<SeriesData> = Vec::new();
    
    // Group measurements by target and collect all metrics
    let mut by_target: HashMap<String, Vec<&Measurement>> = HashMap::new();
    for m in measurements {
        if (m.test_type == "icmp" || m.test_type == "server_echo") && m.status == "success" {
            by_target.entry(m.target.clone()).or_insert_with(Vec::new).push(m);
        }
    }
    
    if by_target.is_empty() {
        anyhow::bail!("No successful measurements to chart");
    }
    
    // Color palette for different targets
    let colors = ["#FF6B6B", "#4ECDC4", "#45B7D1", "#FFA07A", "#98D8C8", "#F7DC6F"];
    let mut color_idx = 0;
    
    for (target, target_measurements) in by_target.iter() {
        // Check if this is a server target
        let is_server = target_measurements.iter().any(|m| m.test_type == "server_echo");
        
        if is_server {
            // For server targets, create 3 series: upload, download, rtt
            let base_color = colors[color_idx % colors.len()];
            color_idx += 1;
            
            // Upload latency series (lighter, dashed)
            let mut upload_points = Vec::new();
            for m in target_measurements.iter() {
                if let Some(upload) = m.upload_latency_ms {
                    upload_points.push((m.timestamp, upload));
                }
            }
            if !upload_points.is_empty() {
                series.push(SeriesData {
                    label: format!("{} ↑ Upload", target),
                    color: format!("{}80", base_color), // Add transparency
                    points: upload_points,
                    is_server_metric: true,
                });
            }
            
            // Download latency series (medium)
            let mut download_points = Vec::new();
            for m in target_measurements.iter() {
                if let Some(download) = m.download_latency_ms {
                    download_points.push((m.timestamp, download));
                }
            }
            if !download_points.is_empty() {
                series.push(SeriesData {
                    label: format!("{} ↓ Download", target),
                    color: format!("{}B0", base_color), // Medium transparency
                    points: download_points,
                    is_server_metric: true,
                });
            }
            
            // RTT series (full color, thicker)
            let mut rtt_points = Vec::new();
            for m in target_measurements.iter() {
                if let Some(rtt) = m.rtt_ms {
                    rtt_points.push((m.timestamp, rtt));
                }
            }
            if !rtt_points.is_empty() {
                series.push(SeriesData {
                    label: format!("{} RTT", target),
                    color: base_color.to_string(),
                    points: rtt_points,
                    is_server_metric: true,
                });
            }
        } else {
            // For ICMP targets, just RTT
            let mut rtt_points = Vec::new();
            for m in target_measurements.iter() {
                if let Some(rtt) = m.rtt_ms {
                    rtt_points.push((m.timestamp, rtt));
                }
            }
            if !rtt_points.is_empty() {
                series.push(SeriesData {
                    label: format!("{} ICMP", target),
                    color: colors[color_idx % colors.len()].to_string(),
                    points: rtt_points,
                    is_server_metric: false,
                });
                color_idx += 1;
            }
        }
    }
    
    // Calculate time range
    let min_time = measurements.iter().map(|m| m.timestamp).min().unwrap();
    let max_time = measurements.iter().map(|m| m.timestamp).max().unwrap();
    
    // Calculate windowing parameters
    let window_size = ((max_time - min_time) / num_segments as i64).max(1);
    
    // Aggregate data into windows with statistics for each series
    let mut windowed_data: HashMap<String, Vec<(i64, i64, usize, Statistics)>> = HashMap::new();
    
    for series_data in &series {
        let mut sorted_points = series_data.points.clone();
        sorted_points.sort_by_key(|(t, _)| *t);
        
        let mut series_windows = Vec::new();
        
        // Split into segments and skip gaps > 5 minutes
        let segments = split_into_segments(&sorted_points, 300);
        
        for segment in segments {
            // Create windows within each segment
            for window_start in (min_time..=max_time).step_by(window_size as usize) {
                let window_end = window_start + window_size;
                let window_points: Vec<f64> = segment
                    .iter()
                    .filter(|(t, _)| *t >= window_start && *t < window_end)
                    .map(|(_, value)| *value)
                    .collect();
                
                if !window_points.is_empty() {
                    let stats = calculate_statistics(&window_points);
                    let count = window_points.len();
                    // Store: (window_start, window_end, sample_count, statistics)
                    series_windows.push((window_start, window_end, count, stats));
                }
            }
        }
        
        windowed_data.insert(series_data.label.clone(), series_windows);
    }
    
    // Calculate global min/max for Y axis
    let all_stats: Vec<&Statistics> = windowed_data.values()
        .flat_map(|v| v.iter().map(|(_, _, _, stats)| stats))
        .collect();
    
    let min_rtt = all_stats.iter().map(|s| s.min).fold(f64::INFINITY, f64::min);
    let max_rtt = all_stats.iter().map(|s| s.max).fold(f64::NEG_INFINITY, f64::max);
    
    let y_margin = (max_rtt - min_rtt) * 0.1;
    let y_min = (min_rtt - y_margin).max(0.0);
    let y_max = max_rtt + y_margin;
    
    // Query events from database (alerts, packet loss, errors)
    let events_json = if let Some(database) = db {
        match database.query_events(min_time, max_time) {
            Ok(events) => {
                let mut json = String::from("[\n");
                for (idx, event) in events.iter().enumerate() {
                    json.push_str(&format!(
                        "  {{\"timestamp\": {}, \"type\": \"{}\", \"target\": \"{}\", \"severity\": \"{}\", \"message\": \"{}\", \"value\": {}, \"threshold\": {}}}",
                        event.timestamp,
                        event.event_type.replace("\"", "\\\""),
                        event.target.replace("\"", "\\\""),
                        event.severity.replace("\"", "\\\""),
                        event.message.replace("\"", "\\\""),
                        event.value.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "null".to_string()),
                        event.threshold.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "null".to_string())
                    ));
                    if idx < events.len() - 1 {
                        json.push(',');
                    }
                    json.push('\n');
                }
                json.push_str("]");
                json
            }
            Err(e) => {
                eprintln!("Warning: Failed to query events: {}", e);
                "[]".to_string()
            }
        }
    } else {
        "[]".to_string()
    };
    
    // Prepare data for JavaScript with window statistics
    let mut data_json = String::from("{\n");
    for (idx, (label, windows)) in windowed_data.iter().enumerate() {
        data_json.push_str(&format!("  \"{}\": [\n", label));
        for (window_start, window_end, count, stats) in windows {
            // Format: [window_start, window_end, count, min, max, avg, p95, p99]
            data_json.push_str(&format!(
                "    [{}, {}, {}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}],\n",
                window_start, window_end, count,
                stats.min, stats.max, stats.avg, stats.p95, stats.p99
            ));
        }
        data_json.push_str("  ]");
        if idx < windowed_data.len() - 1 {
            data_json.push(',');
        }
        data_json.push('\n');
    }
    data_json.push_str("}");
    
    // Extract colors from series data
    let colors_json: Vec<String> = series.iter()
        .map(|s| format!("\"{}\"", s.color))
        .collect();
    let colors_str = format!("[{}]", colors_json.join(", "));
    
    // Format time range for display
    let start_time = chrono::DateTime::from_timestamp(min_time, 0)
        .unwrap()
        .format("%Y-%m-%d %H:%M")
        .to_string();
    let end_time = chrono::DateTime::from_timestamp(max_time, 0)
        .unwrap()
        .format("%Y-%m-%d %H:%M")
        .to_string();
    
    // Generate HTML from template
    let html = INTERACTIVE_TEMPLATE
        .replace("{{START_TIME}}", &start_time)
        .replace("{{END_TIME}}", &end_time)
        .replace("{{DATA_JSON}}", &data_json)
        .replace("{{COLORS_JSON}}", &colors_str)
        .replace("{{EVENTS_JSON}}", &events_json)
        .replace("{{MIN_TIME}}", &min_time.to_string())
        .replace("{{MAX_TIME}}", &max_time.to_string())
        .replace("{{MIN_RTT}}", &format!("{:.2}", y_min))
        .replace("{{MAX_RTT}}", &format!("{:.2}", y_max));
    
    std::fs::write(output_path, html)?;
    
    Ok(())
}
