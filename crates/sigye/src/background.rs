//! Background animation rendering for the sigye clock.

use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use sigye_core::{AnimationSpeed, BackgroundStyle};

use crate::system_metrics::SystemMetrics;

/// Characters used for starfield background.
const STAR_CHARS: &[char] = &['.', '*', '+', '·', '✦', '✧'];

/// Characters used for matrix rain.
const MATRIX_CHARS: &[char] = &[
    'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ', 'タ',
    'チ', 'ツ', 'テ', 'ト', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

/// State for a single matrix rain column.
#[derive(Debug, Clone)]
struct MatrixColumn {
    /// Current y position of the raindrop head.
    y: f32,
    /// Speed multiplier for this column.
    speed: f32,
    /// Length of the trail.
    trail_length: usize,
    /// Seed for character generation.
    char_seed: usize,
}

/// Background animation state.
#[derive(Debug)]
pub struct BackgroundState {
    /// Matrix rain column states.
    matrix_columns: Vec<MatrixColumn>,
    /// Last known terminal width.
    last_width: u16,
    /// Last known terminal height.
    last_height: u16,
    /// Last update time in milliseconds.
    last_update_ms: u64,
}

impl Default for BackgroundState {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundState {
    /// Create a new background state.
    pub fn new() -> Self {
        Self {
            matrix_columns: Vec::new(),
            last_width: 0,
            last_height: 0,
            last_update_ms: 0,
        }
    }

    /// Initialize or reinitialize matrix columns for the given dimensions.
    fn init_matrix_columns(&mut self, width: u16, height: u16) {
        self.matrix_columns = (0..width)
            .map(|x| {
                let x = x as usize;
                let stagger = ((x * 7 + 3) % (height as usize * 2)) as f32;
                MatrixColumn {
                    // Stagger start positions so columns don't all start at top
                    y: -stagger,
                    // Vary speeds between columns
                    speed: 0.3 + ((x * 13) % 10) as f32 / 15.0,
                    // Vary trail lengths
                    trail_length: 4 + (x * 11) % 8,
                    // Seed for character selection
                    char_seed: x * 17,
                }
            })
            .collect();
        self.last_width = width;
        self.last_height = height;
    }

    /// Update matrix column positions.
    fn update_matrix(&mut self, elapsed_ms: u64, height: u16, speed: AnimationSpeed) {
        let delta_ms = elapsed_ms.saturating_sub(self.last_update_ms);
        self.last_update_ms = elapsed_ms;

        let fall_speed = speed.matrix_fall_speed();
        let delta_y = (delta_ms as f32 / 50.0) * fall_speed;

        for col in &mut self.matrix_columns {
            col.y += delta_y * col.speed;
            // Reset column when it goes off screen
            if col.y > (height as f32 + col.trail_length as f32) {
                col.y = -(col.trail_length as f32);
                col.char_seed = col.char_seed.wrapping_add(1);
            }
        }
    }

    /// Render the background to the frame.
    pub fn render(
        &mut self,
        frame: &mut Frame,
        style: BackgroundStyle,
        elapsed_ms: u64,
        speed: AnimationSpeed,
        metrics: Option<&SystemMetrics>,
    ) {
        if style == BackgroundStyle::None {
            return;
        }

        let area = frame.area();
        let width = area.width;
        let height = area.height;

        // Handle reactive backgrounds separately
        if style.is_reactive() {
            if let Some(m) = metrics {
                self.render_reactive(frame, style, elapsed_ms, speed, m);
            }
            return;
        }

        // Reinitialize if dimensions changed
        if style == BackgroundStyle::MatrixRain
            && (width != self.last_width || height != self.last_height)
        {
            self.init_matrix_columns(width, height);
        }

        // Update matrix state
        if style == BackgroundStyle::MatrixRain {
            self.update_matrix(elapsed_ms, height, speed);
        }

        let lines: Vec<Line> = (0..height)
            .map(|y| {
                let spans: Vec<Span> = (0..width)
                    .map(|x| self.render_char(x, y, width, height, style, elapsed_ms, speed))
                    .collect();
                Line::from(spans)
            })
            .collect();

        frame.render_widget(Paragraph::new(lines), area);
    }

    /// Render a single background character at the given position.
    fn render_char(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        style: BackgroundStyle,
        elapsed_ms: u64,
        speed: AnimationSpeed,
    ) -> Span<'static> {
        match style {
            BackgroundStyle::None => Span::raw(" "),
            BackgroundStyle::Starfield => self.render_starfield_char(x, y, elapsed_ms, speed),
            BackgroundStyle::MatrixRain => self.render_matrix_char(x, y, height),
            BackgroundStyle::GradientWave => {
                self.render_gradient_char(x, y, width, height, elapsed_ms, speed)
            }
            // Reactive backgrounds are handled separately in render_reactive()
            BackgroundStyle::SystemPulse
            | BackgroundStyle::ResourceWave
            | BackgroundStyle::DataFlow
            | BackgroundStyle::HeatMap => Span::raw(" "),
        }
    }

    /// Render a starfield character using pseudo-random twinkling.
    fn render_starfield_char(
        &self,
        x: u16,
        y: u16,
        elapsed_ms: u64,
        speed: AnimationSpeed,
    ) -> Span<'static> {
        let x = x as usize;
        let y = y as usize;
        let period = speed.star_twinkle_period_ms();
        let frame_num = elapsed_ms / period;

        // Use deterministic "random" based on position and time
        let seed = (x.wrapping_mul(31))
            .wrapping_add(y.wrapping_mul(17))
            .wrapping_add(frame_num as usize);

        // Only show stars at ~3% of positions
        if seed % 100 < 3 {
            let char_idx = seed % STAR_CHARS.len();
            let ch = STAR_CHARS[char_idx];

            // Vary brightness based on position
            let brightness = (seed % 3) as u8;
            let color = match brightness {
                0 => Color::Rgb(60, 60, 80),    // Dim
                1 => Color::Rgb(100, 100, 140), // Medium
                _ => Color::Rgb(150, 150, 200), // Bright
            };

            Span::styled(ch.to_string(), Style::new().fg(color))
        } else {
            Span::raw(" ")
        }
    }

    /// Render a matrix rain character.
    fn render_matrix_char(&self, x: u16, y: u16, _height: u16) -> Span<'static> {
        let x = x as usize;
        let y = y as f32;

        if x >= self.matrix_columns.len() {
            return Span::raw(" ");
        }

        let col = &self.matrix_columns[x];
        let head_y = col.y;
        let tail_y = head_y - col.trail_length as f32;

        // Check if this position is within the rain trail
        if y >= tail_y && y <= head_y {
            let distance_from_head = head_y - y;
            let intensity = 1.0 - (distance_from_head / col.trail_length as f32);

            // Select character based on position and seed
            let char_idx = (col.char_seed.wrapping_add(y as usize)) % MATRIX_CHARS.len();
            let ch = MATRIX_CHARS[char_idx];

            // Head is bright white-green, trail fades to dark green
            let color = if distance_from_head < 1.0 {
                Color::Rgb(200, 255, 200) // Bright head
            } else {
                let g = (80.0 + 120.0 * intensity) as u8;
                Color::Rgb(0, g, 0)
            };

            Span::styled(ch.to_string(), Style::new().fg(color))
        } else {
            Span::raw(" ")
        }
    }

    /// Render a gradient wave character.
    fn render_gradient_char(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        elapsed_ms: u64,
        speed: AnimationSpeed,
    ) -> Span<'static> {
        let period = speed.gradient_scroll_period_ms();
        let time_phase = (elapsed_ms % period) as f32 / period as f32;

        let x_norm = x as f32 / width.max(1) as f32;
        let y_norm = y as f32 / height.max(1) as f32;

        // Create a diagonal wave pattern
        let wave = ((x_norm + y_norm * 0.5 + time_phase) * 2.0 * std::f32::consts::PI).sin();
        let intensity = (wave + 1.0) / 2.0; // Normalize to 0..1

        // Use block characters with varying density
        let ch = if intensity < 0.25 {
            ' '
        } else if intensity < 0.5 {
            '░'
        } else if intensity < 0.75 {
            '▒'
        } else {
            '▓'
        };

        // Color gradient from deep blue to cyan to purple
        let hue_offset = time_phase * 360.0;
        let base_hue = (x_norm * 60.0 + hue_offset) % 360.0;

        let color = hsl_to_rgb(base_hue, 0.7, 0.15 + intensity * 0.2);

        if ch == ' ' {
            Span::raw(" ")
        } else {
            Span::styled(ch.to_string(), Style::new().fg(color))
        }
    }

    /// Render reactive backgrounds that respond to system metrics.
    fn render_reactive(
        &mut self,
        frame: &mut Frame,
        style: BackgroundStyle,
        elapsed_ms: u64,
        speed: AnimationSpeed,
        metrics: &SystemMetrics,
    ) {
        match style {
            BackgroundStyle::SystemPulse => {
                self.render_system_pulse(frame, elapsed_ms, speed, metrics)
            }
            BackgroundStyle::ResourceWave => {
                self.render_resource_wave(frame, elapsed_ms, speed, metrics)
            }
            BackgroundStyle::DataFlow => self.render_data_flow(frame, elapsed_ms, speed, metrics),
            BackgroundStyle::HeatMap => self.render_heat_map(frame, elapsed_ms, speed, metrics),
            _ => {}
        }
    }

    /// Render system pulse background - CPU drives pulse rate and size.
    fn render_system_pulse(
        &self,
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
    fn render_resource_wave(
        &self,
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
    fn render_data_flow(
        &self,
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
    fn render_heat_map(
        &self,
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
                        let noise = (x as f32 * 0.1 + y as f32 * 0.15 + time_phase * 10.0).sin()
                            * 0.3
                            + 0.7;

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
}

/// Map a resource value (0.0-1.0) to a color from cool blue to warm red.
fn resource_to_color(value: f32) -> Color {
    let value = value.clamp(0.0, 1.0);

    // Hue: 240 (blue) -> 60 (yellow) -> 0 (red)
    let hue = 240.0 - (value * 240.0);

    // Higher usage = more saturated and brighter
    let saturation = 0.6 + (value * 0.4);
    let lightness = 0.15 + (value * 0.25);

    hsl_to_rgb(hue, saturation, lightness)
}

/// Convert HSL to RGB color.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return Color::Rgb(v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let h = h / 360.0;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    Color::Rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
