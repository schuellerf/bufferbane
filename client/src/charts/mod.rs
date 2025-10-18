//! Chart generation (PNG export)

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
) -> Result<()> {
    if measurements.is_empty() {
        anyhow::bail!("No measurements to chart");
    }
    
    // Group measurements by target
    let mut by_target: HashMap<String, Vec<(i64, f64)>> = HashMap::new();
    
    for m in measurements {
        if m.test_type == "icmp" && m.status == "success" {
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
            ("sans-serif", 30).into_font(),
        )
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
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
        
        // Calculate statistics for each window
        let window_size = ((max_time - min_time) / 100).max(1);
        let mut windowed_stats = Vec::new();
        
        for window_start in (min_time..=max_time).step_by(window_size as usize) {
            let window_end = window_start + window_size;
            let window_points: Vec<f64> = sorted_points
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
        
        // Draw avg line (bold)
        chart.draw_series(LineSeries::new(
            windowed_stats.iter().map(|(t, stats)| (*t, stats.avg)),
            color.stroke_width(3),
        ))?.label(target.clone())
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(3)));
    }
    
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    
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

