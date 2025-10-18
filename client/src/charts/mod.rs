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

/// Generate interactive HTML chart with hover tooltips
pub fn generate_interactive_chart(
    measurements: &[Measurement],
    output_path: &Path,
    _config: &Config,
    num_segments: usize,
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
    
    // Generate HTML
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Bufferbane - Latency Chart</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            margin: 20px;
            background: #f5f5f5;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        h1 {{
            margin-top: 0;
            color: #333;
            font-size: 28px;
        }}
        .chart-container {{
            position: relative;
            margin: 20px 0;
        }}
        canvas {{
            border: 1px solid #ddd;
            cursor: crosshair;
        }}
        #tooltip {{
            position: absolute;
            background: rgba(0, 0, 0, 0.9);
            color: white;
            padding: 12px 16px;
            border-radius: 6px;
            font-size: 14px;
            pointer-events: none;
            display: none;
            z-index: 1000;
            white-space: nowrap;
            box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        }}
        .legend {{
            display: flex;
            gap: 20px;
            margin-top: 20px;
            flex-wrap: wrap;
        }}
        .legend-item {{
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 16px;
        }}
        .legend-color {{
            width: 24px;
            height: 4px;
            border-radius: 2px;
        }}
        .stats {{
            margin-top: 20px;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
        }}
        .stat-card {{
            padding: 15px;
            background: #f8f9fa;
            border-radius: 6px;
            border-left: 4px solid #3366CC;
        }}
        .stat-label {{
            font-size: 12px;
            color: #666;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        .stat-value {{
            font-size: 24px;
            font-weight: 600;
            color: #333;
            margin-top: 5px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Bufferbane - Latency Over Time</h1>
        <p><strong>Period:</strong> {} to {}</p>
        
        <div class="chart-container">
            <canvas id="chart" width="1200" height="600"></canvas>
            <div id="tooltip"></div>
        </div>
        
        <div class="legend" id="legend"></div>
        
        <div class="stats" id="stats"></div>
    </div>

    <script>
        const data = {};
        const colors = {};
        const canvas = document.getElementById('chart');
        const ctx = canvas.getContext('2d');
        const tooltip = document.getElementById('tooltip');
        
        const minTime = {};
        const maxTime = {};
        const minRtt = {};
        const maxRtt = {};
        
        const padding = {{ left: 80, right: 40, top: 60, bottom: 60 }};
        const chartWidth = canvas.width - padding.left - padding.right;
        const chartHeight = canvas.height - padding.top - padding.bottom;
        
        // Helper functions
        function timeToX(timestamp) {{
            return padding.left + (timestamp - minTime) / (maxTime - minTime) * chartWidth;
        }}
        
        function rttToY(rtt) {{
            return padding.top + chartHeight - (rtt - minRtt) / (maxRtt - minRtt) * chartHeight;
        }}
        
        function formatTime(timestamp) {{
            const date = new Date(timestamp * 1000);
            return date.toLocaleTimeString('en-US', {{ hour: '2-digit', minute: '2-digit', second: '2-digit' }});
        }}
        
        function formatDateTime(timestamp) {{
            const date = new Date(timestamp * 1000);
            return date.toLocaleString('en-US', {{ 
                year: 'numeric', month: 'short', day: 'numeric',
                hour: '2-digit', minute: '2-digit', second: '2-digit'
            }});
        }}
        
        // Draw chart
        function drawChart() {{
            // Clear canvas
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            
            // Draw grid
            ctx.strokeStyle = '#e0e0e0';
            ctx.lineWidth = 1;
            
            // Horizontal grid lines
            for (let i = 0; i <= 5; i++) {{
                const y = padding.top + (chartHeight / 5) * i;
                ctx.beginPath();
                ctx.moveTo(padding.left, y);
                ctx.lineTo(padding.left + chartWidth, y);
                ctx.stroke();
                
                // Y axis labels
                const rtt = maxRtt - (maxRtt - minRtt) / 5 * i;
                ctx.fillStyle = '#666';
                ctx.font = '14px sans-serif';
                ctx.textAlign = 'right';
                ctx.fillText(rtt.toFixed(1) + 'ms', padding.left - 10, y + 5);
            }}
            
            // Vertical grid lines
            for (let i = 0; i <= 6; i++) {{
                const x = padding.left + (chartWidth / 6) * i;
                ctx.beginPath();
                ctx.moveTo(x, padding.top);
                ctx.lineTo(x, padding.top + chartHeight);
                ctx.stroke();
                
                // X axis labels
                const timestamp = minTime + (maxTime - minTime) / 6 * i;
                ctx.fillStyle = '#666';
                ctx.font = '14px sans-serif';
                ctx.textAlign = 'center';
                ctx.fillText(formatTime(timestamp), x, padding.top + chartHeight + 25);
            }}
            
            // Axes
            ctx.strokeStyle = '#333';
            ctx.lineWidth = 2;
            ctx.beginPath();
            ctx.moveTo(padding.left, padding.top);
            ctx.lineTo(padding.left, padding.top + chartHeight);
            ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight);
            ctx.stroke();
            
            // Axis labels
            ctx.fillStyle = '#333';
            ctx.font = 'bold 18px sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText('Time', canvas.width / 2, canvas.height - 10);
            
            ctx.save();
            ctx.translate(15, canvas.height / 2);
            ctx.rotate(-Math.PI / 2);
            ctx.fillText('Latency (ms)', 0, 0);
            ctx.restore();
            
            // Draw data lines (breaking at gaps > 5 minutes)
            // Data format: [window_start, window_end, count, min, max, avg, p95, p99]
            const MAX_GAP_SECONDS = 300;  // 5 minutes
            Object.entries(data).forEach(([target, windows], idx) => {{
                // Determine line style based on target label
                let lineWidth = 2;
                let dashPattern = [];
                let alphaMultiplier = 1.0;
                
                if (target.includes('↑ Upload')) {{
                    // Upload: dashed line, thinner, lighter
                    dashPattern = [8, 4];
                    lineWidth = 2;
                    alphaMultiplier = 0.8;
                }} else if (target.includes('↓ Download')) {{
                    // Download: dotted line, thinner, medium
                    dashPattern = [2, 3];
                    lineWidth = 2;
                    alphaMultiplier = 0.9;
                }} else if (target.includes('RTT')) {{
                    // RTT: solid line, thicker, full color
                    dashPattern = [];
                    lineWidth = 3;
                    alphaMultiplier = 1.0;
                }} else {{
                    // ICMP: solid line, medium
                    dashPattern = [];
                    lineWidth = 2;
                    alphaMultiplier = 1.0;
                }}
                
                // Split windows into continuous segments (no gaps > 5 min)
                const segments = [];
                let currentSegment = [];
                
                windows.forEach((window, i) => {{
                    if (i === 0) {{
                        currentSegment.push(window);
                    }} else {{
                        const prevTime = windows[i - 1][1];  // prev window end
                        const currTime = window[0];  // curr window start
                        const gap = currTime - prevTime;
                        
                        if (gap > MAX_GAP_SECONDS) {{
                            // Gap detected - save current segment and start new one
                            if (currentSegment.length > 0) {{
                                segments.push(currentSegment);
                            }}
                            currentSegment = [window];
                        }} else {{
                            currentSegment.push(window);
                        }}
                    }}
                }});
                
                // Don't forget the last segment
                if (currentSegment.length > 0) {{
                    segments.push(currentSegment);
                }}
                
                // Draw each segment separately
                segments.forEach(segment => {{
                    // Draw shaded area between min and max
                    ctx.fillStyle = colors[idx];
                    ctx.globalAlpha = 0.1;
                    ctx.beginPath();
                    
                    // Draw min line (bottom of shaded area)
                    segment.forEach((window, i) => {{
                        const window_center = (window[0] + window[1]) / 2;
                        const min = window[3];
                        const x = timeToX(window_center);
                        const y = rttToY(min);
                        if (i === 0) ctx.moveTo(x, y);
                        else ctx.lineTo(x, y);
                    }});
                    
                    // Draw max line in reverse (top of shaded area)
                    for (let i = segment.length - 1; i >= 0; i--) {{
                        const window = segment[i];
                        const window_center = (window[0] + window[1]) / 2;
                        const max = window[4];
                        const x = timeToX(window_center);
                        const y = rttToY(max);
                        ctx.lineTo(x, y);
                    }}
                    
                    ctx.closePath();
                    ctx.fill();
                    ctx.globalAlpha = 1.0;
                    
                    // Draw min line (thin, lighter color)
                    ctx.strokeStyle = colors[idx];
                    ctx.globalAlpha = 0.3 * alphaMultiplier;
                    ctx.lineWidth = 1;
                    ctx.setLineDash(dashPattern);
                    ctx.beginPath();
                    segment.forEach((window, i) => {{
                        const window_center = (window[0] + window[1]) / 2;
                        const min = window[3];
                        const x = timeToX(window_center);
                        const y = rttToY(min);
                        if (i === 0) ctx.moveTo(x, y);
                        else ctx.lineTo(x, y);
                    }});
                    ctx.stroke();
                    
                    // Draw max line (thin, lighter color)
                    ctx.setLineDash(dashPattern);
                    ctx.beginPath();
                    segment.forEach((window, i) => {{
                        const window_center = (window[0] + window[1]) / 2;
                        const max = window[4];
                        const x = timeToX(window_center);
                        const y = rttToY(max);
                        if (i === 0) ctx.moveTo(x, y);
                        else ctx.lineTo(x, y);
                    }});
                    ctx.stroke();
                    ctx.setLineDash([]);
                    ctx.globalAlpha = 1.0;
                    
                    // Draw avg line (bold, primary)
                    ctx.strokeStyle = colors[idx];
                    ctx.globalAlpha = alphaMultiplier;
                    ctx.lineWidth = lineWidth;
                    ctx.setLineDash(dashPattern);
                    ctx.beginPath();
                    segment.forEach((window, i) => {{
                        const window_center = (window[0] + window[1]) / 2;
                        const avg = window[5];
                        const x = timeToX(window_center);
                        const y = rttToY(avg);
                        if (i === 0) ctx.moveTo(x, y);
                        else ctx.lineTo(x, y);
                    }});
                    ctx.stroke();
                    ctx.setLineDash([]);  // Reset dash pattern
                    ctx.globalAlpha = 1.0;  // Reset alpha
                }});
            }});
        }}
        
        // Handle mouse move for tooltips
        canvas.addEventListener('mousemove', (e) => {{
            const rect = canvas.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            
            // Find closest window
            let closestDist = Infinity;
            let closestWindow = null;
            let closestTarget = null;
            
            Object.entries(data).forEach(([target, windows]) => {{
                windows.forEach(window => {{
                    // window format: [start, end, count, min, max, avg, p95, p99]
                    const window_center = (window[0] + window[1]) / 2;
                    const avg = window[5];
                    const x = timeToX(window_center);
                    const y = rttToY(avg);
                    const dist = Math.sqrt((mouseX - x) ** 2 + (mouseY - y) ** 2);
                    
                    if (dist < closestDist && dist < 30) {{
                        closestDist = dist;
                        closestWindow = window;
                        closestTarget = target;
                    }}
                }});
            }});
            
            if (closestWindow) {{
                tooltip.style.display = 'block';
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
                
                const start = closestWindow[0];
                const end = closestWindow[1];
                const count = closestWindow[2];
                const min = closestWindow[3];
                const max = closestWindow[4];
                const avg = closestWindow[5];
                const p95 = closestWindow[6];
                const p99 = closestWindow[7];
                const variance = max - min;
                
                tooltip.innerHTML = `
                    <strong>${{closestTarget}}</strong><br>
                    <div style="font-size: 11px; color: #ccc; margin: 4px 0;">
                        ${{formatTime(start)}} - ${{formatTime(end)}}<br>
                        (${{count}} samples)
                    </div>
                    <div style="border-top: 1px solid #555; padding-top: 6px; margin-top: 6px;">
                        <strong>Min:</strong> ${{min.toFixed(2)}}ms<br>
                        <strong>Avg:</strong> ${{avg.toFixed(2)}}ms<br>
                        <strong>Max:</strong> ${{max.toFixed(2)}}ms<br>
                        <strong>P95:</strong> ${{p95.toFixed(2)}}ms<br>
                        <strong>P99:</strong> ${{p99.toFixed(2)}}ms<br>
                        <div style="font-size: 11px; color: #ccc; margin-top: 4px;">
                            Variance: ${{variance.toFixed(2)}}ms
                        </div>
                    </div>
                `;
            }} else {{
                tooltip.style.display = 'none';
            }}
        }});
        
        canvas.addEventListener('mouseleave', () => {{
            tooltip.style.display = 'none';
        }});
        
        // Create legend
        const legendEl = document.getElementById('legend');
        Object.keys(data).forEach((target, idx) => {{
            const item = document.createElement('div');
            item.className = 'legend-item';
            item.innerHTML = `
                <div class="legend-color" style="background: ${{colors[idx]}}"></div>
                <span>${{target}}</span>
            `;
            legendEl.appendChild(item);
        }});
        
        // Create stats (overall statistics across all windows)
        const statsEl = document.getElementById('stats');
        Object.entries(data).forEach(([target, windows]) => {{
            // Aggregate stats across all windows
            let overall_min = Infinity;
            let overall_max = -Infinity;
            let sum_avg = 0;
            let sum_p95 = 0;
            let sum_p99 = 0;
            let total_count = 0;
            
            windows.forEach(window => {{
                // window format: [start, end, count, min, max, avg, p95, p99]
                overall_min = Math.min(overall_min, window[3]);
                overall_max = Math.max(overall_max, window[4]);
                sum_avg += window[5] * window[2];  // weighted by count
                sum_p95 += window[6] * window[2];
                sum_p99 += window[7] * window[2];
                total_count += window[2];
            }});
            
            const overall_avg = sum_avg / total_count;
            const overall_p95 = sum_p95 / total_count;
            const overall_p99 = sum_p99 / total_count;
            
            const card = document.createElement('div');
            card.className = 'stat-card';
            card.innerHTML = `
                <div class="stat-label">${{target}} (${{windows.length}} windows, ${{total_count}} samples)</div>
                <div class="stat-value">${{overall_avg.toFixed(2)}}ms</div>
                <div style="font-size: 12px; color: #666; margin-top: 5px;">
                    Min: ${{overall_min.toFixed(2)}}ms | Max: ${{overall_max.toFixed(2)}}ms<br>
                    P95: ${{overall_p95.toFixed(2)}}ms | P99: ${{overall_p99.toFixed(2)}}ms
                </div>
            `;
            statsEl.appendChild(card);
        }});
        
        // Initial draw
        drawChart();
    </script>
</body>
</html>"#,
        chrono::DateTime::from_timestamp(min_time, 0).unwrap().format("%Y-%m-%d %H:%M"),
        chrono::DateTime::from_timestamp(max_time, 0).unwrap().format("%Y-%m-%d %H:%M"),
        data_json,
        colors_str,
        min_time,
        max_time,
        format!("{:.2}", y_min),
        format!("{:.2}", y_max),
    );
    
    std::fs::write(output_path, html)?;
    
    Ok(())
}

