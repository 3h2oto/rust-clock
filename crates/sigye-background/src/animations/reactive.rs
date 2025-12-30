//! Reactive background animations that respond to system metrics.

use ratatui::{
    Frame,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use sigye_core::{AnimationSpeed, SystemMetrics};

use crate::color::resource_to_color;

/// Render system pulse background - CPU drives pulse rate and size.
pub fn render_system_pulse(
    frame: &mut Frame,
    elapsed_ms: u64,
    speed: AnimationSpeed,
    metrics: &SystemMetrics,
) {
    let area = frame.area();
    let width = area.width as f32;
    let height = area.height as f32;

    // CPU usage controls pulse rate and size
    let cpu = metrics.cpu_usage;
    let base_period = match speed {
        AnimationSpeed::Slow => 3000.0,
        AnimationSpeed::Medium => 2000.0,
        AnimationSpeed::Fast => 1000.0,
    };

    // Higher CPU = faster pulse
    let period = base_period * (1.0 - cpu * 0.5);
    let phase = (elapsed_ms as f32 % period) / period;
    let pulse = (phase * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;

    let color = resource_to_color(cpu);

    // Render pulsing effect from center
    let lines: Vec<Line> = (0..area.height)
        .map(|y| {
            let spans: Vec<Span> = (0..area.width)
                .map(|x| {
                    let dx = x as f32 - width / 2.0;
                    let dy = (y as f32 - height / 2.0) * 2.0; // Adjust for terminal aspect ratio
                    let dist = (dx * dx + dy * dy).sqrt();
                    let max_dist = (width * width / 4.0 + height * height).sqrt();
                    let normalized = dist / max_dist;

                    // Pulse expands from center
                    let intensity = (1.0 - normalized) * pulse * (0.3 + cpu * 0.7);

                    if intensity > 0.05 {
                        let ch = if intensity > 0.6 {
                            '█'
                        } else if intensity > 0.4 {
                            '▓'
                        } else if intensity > 0.2 {
                            '▒'
                        } else if intensity > 0.1 {
                            '░'
                        } else {
                            '·'
                        };
                        Span::styled(ch.to_string(), Style::new().fg(color))
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), area);
}

/// Render resource wave background - memory drives wave amplitude.
pub fn render_resource_wave(
    frame: &mut Frame,
    elapsed_ms: u64,
    speed: AnimationSpeed,
    metrics: &SystemMetrics,
) {
    let area = frame.area();
    let width = area.width as f32;
    let height = area.height as f32;

    // Memory controls wave amplitude
    let mem = metrics.memory_usage;
    let amplitude = mem * (height / 3.0);
    let color = resource_to_color(mem);

    let period = speed.wave_period_ms();
    let time_phase = (elapsed_ms % period) as f32 / period as f32;

    let lines: Vec<Line> = (0..area.height)
        .map(|y| {
            let spans: Vec<Span> = (0..area.width)
                .map(|x| {
                    let x_norm = x as f32 / width;
                    let wave_y = (height / 2.0)
                        + amplitude
                            * ((x_norm * 4.0 + time_phase * 2.0 * std::f32::consts::PI).sin());

                    let dist = (y as f32 - wave_y).abs();

                    if dist < 3.0 {
                        let ch = if dist < 0.5 {
                            '█'
                        } else if dist < 1.5 {
                            '▓'
                        } else {
                            '░'
                        };
                        Span::styled(ch.to_string(), Style::new().fg(color))
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), area);
}

/// Render data flow background - network I/O drives particle density and speed.
pub fn render_data_flow(
    frame: &mut Frame,
    elapsed_ms: u64,
    speed: AnimationSpeed,
    metrics: &SystemMetrics,
) {
    let area = frame.area();

    // Network rate controls particle density and speed
    let net_combined = (metrics.network_rx_rate + metrics.network_tx_rate) / 2.0;
    let color = resource_to_color(net_combined);

    let base_speed = match speed {
        AnimationSpeed::Slow => 0.5,
        AnimationSpeed::Medium => 1.0,
        AnimationSpeed::Fast => 2.0,
    };
    let flow_speed = base_speed + net_combined * 2.0;

    let lines: Vec<Line> = (0..area.height)
        .map(|y| {
            let spans: Vec<Span> = (0..area.width)
                .map(|x| {
                    // Flowing particles based on position and time
                    let seed = (x as usize)
                        .wrapping_mul(17)
                        .wrapping_add((y as usize).wrapping_mul(31));
                    let particle_phase =
                        ((elapsed_ms as f32 * flow_speed / 100.0) + seed as f32) % 100.0;

                    // Show particle if it's in the "visible" part of its cycle
                    // Higher network = more particles visible
                    let threshold = 95.0 - (net_combined * 70.0);
                    if particle_phase > threshold && seed % 15 < 2 {
                        let chars = ['·', '•', '○', '●'];
                        let ch = chars[seed % chars.len()];
                        Span::styled(ch.to_string(), Style::new().fg(color))
                    } else {
                        Span::raw(" ")
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), area);
}

/// Render heat map background - combined metrics drive heat intensity.
pub fn render_heat_map(
    frame: &mut Frame,
    elapsed_ms: u64,
    speed: AnimationSpeed,
    metrics: &SystemMetrics,
) {
    let area = frame.area();
    let width = area.width;
    let height = area.height;

    // Combined metric for overall "heat"
    let combined = (metrics.cpu_usage
        + metrics.memory_usage
        + metrics.network_rx_rate
        + metrics.network_tx_rate)
        / 4.0;

    let period = speed.gradient_scroll_period_ms();
    let time_phase = (elapsed_ms % period) as f32 / period as f32;

    let lines: Vec<Line> = (0..height)
        .map(|y| {
            let spans: Vec<Span> = (0..width)
                .map(|x| {
                    // Heat spreads from edges
                    let edge_dist = (x.min(width - 1 - x).min(y).min(height - 1 - y)) as f32;
                    let max_edge = (width.min(height) / 2) as f32;
                    let edge_factor = 1.0 - (edge_dist / max_edge.max(1.0)).min(1.0);

                    // Add some noise/variation
                    let noise =
                        (x as f32 * 0.1 + y as f32 * 0.15 + time_phase * 10.0).sin() * 0.3 + 0.7;

                    let heat = edge_factor * (0.2 + combined * 0.8) * noise;
                    let color = resource_to_color(heat);

                    let ch = if heat > 0.5 {
                        '█'
                    } else if heat > 0.35 {
                        '▓'
                    } else if heat > 0.2 {
                        '▒'
                    } else if heat > 0.1 {
                        '░'
                    } else {
                        ' '
                    };

                    if ch == ' ' {
                        Span::raw(" ")
                    } else {
                        Span::styled(ch.to_string(), Style::new().fg(color))
                    }
                })
                .collect();
            Line::from(spans)
        })
        .collect();

    frame.render_widget(Paragraph::new(lines), area);
}
