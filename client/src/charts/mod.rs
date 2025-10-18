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
            let window_size = ((max_time - min_time) / 100).max(1);
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
    
    // Calculate global min/max for Y axis
    let all_rtts: Vec<f64> = by_target.values()
        .flat_map(|v| v.iter().map(|(_, rtt)| *rtt))
        .collect();
    
    let min_rtt = all_rtts.iter().copied().fold(f64::INFINITY, f64::min);
    let max_rtt = all_rtts.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    
    let y_margin = (max_rtt - min_rtt) * 0.1;
    let y_min = (min_rtt - y_margin).max(0.0);
    let y_max = max_rtt + y_margin;
    
    // Prepare data for JavaScript
    let mut data_json = String::from("{\n");
    for (idx, (target, points)) in by_target.iter().enumerate() {
        let mut sorted_points = points.clone();
        sorted_points.sort_by_key(|(t, _)| *t);
        
        data_json.push_str(&format!("  \"{}\": [\n", target));
        for (timestamp, rtt) in &sorted_points {
            data_json.push_str(&format!("    [{}, {:.2}],\n", timestamp, rtt));
        }
        data_json.push_str("  ]");
        if idx < by_target.len() - 1 {
            data_json.push(',');
        }
        data_json.push('\n');
    }
    data_json.push_str("}");
    
    // Colors for each target
    let colors = vec![
        "#3366CC", // Blue
        "#109618", // Green
        "#DC3912", // Red
        "#FF9900", // Orange
        "#990099", // Purple
    ];
    
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
        
        // Calculate statistics
        function calculateStats(points) {{
            const rtts = points.map(p => p[1]).sort((a, b) => a - b);
            return {{
                min: rtts[0],
                max: rtts[rtts.length - 1],
                avg: rtts.reduce((a, b) => a + b, 0) / rtts.length,
                p50: rtts[Math.floor(rtts.length * 0.5)],
                p95: rtts[Math.floor(rtts.length * 0.95)],
                p99: rtts[Math.floor(rtts.length * 0.99)]
            }};
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
            const MAX_GAP_SECONDS = 300;  // 5 minutes
            Object.entries(data).forEach(([target, points], idx) => {{
                ctx.strokeStyle = colors[idx];
                ctx.lineWidth = 3;
                ctx.beginPath();
                
                points.forEach((point, i) => {{
                    const x = timeToX(point[0]);
                    const y = rttToY(point[1]);
                    
                    if (i === 0) {{
                        ctx.moveTo(x, y);
                    }} else {{
                        // Check if there's a gap > 5 minutes
                        const prevTime = points[i - 1][0];
                        const currTime = point[0];
                        const gap = currTime - prevTime;
                        
                        if (gap > MAX_GAP_SECONDS) {{
                            // Start a new line segment (don't connect across gap)
                            ctx.stroke();
                            ctx.beginPath();
                            ctx.moveTo(x, y);
                        }} else {{
                            ctx.lineTo(x, y);
                        }}
                    }}
                }});
                
                ctx.stroke();
            }});
        }}
        
        // Handle mouse move for tooltips
        canvas.addEventListener('mousemove', (e) => {{
            const rect = canvas.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            
            // Find closest point
            let closestDist = Infinity;
            let closestPoint = null;
            let closestTarget = null;
            
            Object.entries(data).forEach(([target, points]) => {{
                points.forEach(point => {{
                    const x = timeToX(point[0]);
                    const y = rttToY(point[1]);
                    const dist = Math.sqrt((mouseX - x) ** 2 + (mouseY - y) ** 2);
                    
                    if (dist < closestDist && dist < 20) {{
                        closestDist = dist;
                        closestPoint = point;
                        closestTarget = target;
                    }}
                }});
            }});
            
            if (closestPoint) {{
                tooltip.style.display = 'block';
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
                tooltip.innerHTML = `
                    <strong>${{closestTarget}}</strong><br>
                    <strong>Time:</strong> ${{formatDateTime(closestPoint[0])}}<br>
                    <strong>Latency:</strong> ${{closestPoint[1].toFixed(2)}}ms
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
        
        // Create stats
        const statsEl = document.getElementById('stats');
        Object.entries(data).forEach(([target, points]) => {{
            const stats = calculateStats(points);
            const card = document.createElement('div');
            card.className = 'stat-card';
            card.innerHTML = `
                <div class="stat-label">${{target}}</div>
                <div class="stat-value">${{stats.avg.toFixed(2)}}ms</div>
                <div style="font-size: 12px; color: #666; margin-top: 5px;">
                    Min: ${{stats.min.toFixed(2)}}ms | Max: ${{stats.max.toFixed(2)}}ms<br>
                    P95: ${{stats.p95.toFixed(2)}}ms | P99: ${{stats.p99.toFixed(2)}}ms
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
        colors.iter().enumerate().map(|(_i, c)| format!("\"{}\"", c)).collect::<Vec<_>>().join(", "),
        min_time,
        max_time,
        format!("{:.2}", y_min),
        format!("{:.2}", y_max),
    );
    
    std::fs::write(output_path, html)?;
    
    Ok(())
}

