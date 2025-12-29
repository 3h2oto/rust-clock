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

/// Characters used for snowfall background.
const SNOW_CHARS: &[char] = &['*', '·', '•', '❄', '❅', '❆', '✦', '✧', '°'];

/// Characters used for frost crystals.
const FROST_CHARS: &[char] = &['·', '•', '*', '×', '✕', '✱', '░'];

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

/// State for a single snowfall column.
#[derive(Debug, Clone)]
struct SnowColumn {
    /// Current y position of the snowflake.
    y: f32,
    /// Speed multiplier for this column.
    speed: f32,
    /// Horizontal drift phase offset.
    drift_phase: f32,
    /// Size category (0=small, 1=medium, 2=large).
    size: u8,
    /// Seed for character generation.
    char_seed: usize,
}

/// Background animation state.
#[derive(Debug)]
pub struct BackgroundState {
    /// Matrix rain column states.
    matrix_columns: Vec<MatrixColumn>,
    /// Snowfall column states.
    snow_columns: Vec<SnowColumn>,
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
            snow_columns: Vec::new(),
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

    /// Initialize or reinitialize snowfall columns for the given dimensions.
    fn init_snow_columns(&mut self, width: u16, height: u16) {
        self.snow_columns = (0..width)
            .map(|x| {
                let x = x as usize;
                let stagger = ((x * 11 + 7) % (height as usize * 3)) as f32;
                SnowColumn {
                    y: -stagger,
                    speed: 0.2 + ((x * 17) % 10) as f32 / 20.0,
                    drift_phase: (x * 23) as f32 / 100.0,
                    size: ((x * 13) % 3) as u8,
                    char_seed: x * 19,
                }
            })
            .collect();
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

    /// Update snowfall column positions.
    fn update_snow(&mut self, elapsed_ms: u64, height: u16, speed: AnimationSpeed) {
        let delta_ms = elapsed_ms.saturating_sub(self.last_update_ms);
        self.last_update_ms = elapsed_ms;

        let fall_speed = speed.snow_fall_speed();
        let delta_y = (delta_ms as f32 / 80.0) * fall_speed;

        for col in &mut self.snow_columns {
            col.y += delta_y * col.speed;
            if col.y > height as f32 + 2.0 {
                col.y = -2.0;
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

        // Reinitialize if dimensions changed or columns not initialized
        let dimensions_changed = width != self.last_width || height != self.last_height;

        if style == BackgroundStyle::MatrixRain
            && (dimensions_changed || self.matrix_columns.is_empty())
        {
            self.init_matrix_columns(width, height);
        }
        if style == BackgroundStyle::Snowfall
            && (dimensions_changed || self.snow_columns.is_empty())
        {
            self.init_snow_columns(width, height);
        }

        if dimensions_changed {
            self.last_width = width;
            self.last_height = height;
        }

        // Update animation states
        if style == BackgroundStyle::MatrixRain {
            self.update_matrix(elapsed_ms, height, speed);
        }
        if style == BackgroundStyle::Snowfall {
            self.update_snow(elapsed_ms, height, speed);
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
            BackgroundStyle::Snowfall => self.render_snowfall_char(x, y, elapsed_ms, speed),
            BackgroundStyle::Frost => {
                self.render_frost_char(x, y, width, height, elapsed_ms, speed)
            }
            BackgroundStyle::Aurora => {
                self.render_aurora_char(x, y, width, height, elapsed_ms, speed)
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

    /// Render a snowfall character.
    fn render_snowfall_char(
        &self,
        x: u16,
        y: u16,
        elapsed_ms: u64,
        _speed: AnimationSpeed,
    ) -> Span<'static> {
        let x_idx = x as usize;
        let y_f = y as f32;

        if x_idx >= self.snow_columns.len() {
            return Span::raw(" ");
        }

        let col = &self.snow_columns[x_idx];

        // Calculate horizontal drift for visual effect
        let drift_period = 3000.0;
        let drift =
            ((elapsed_ms as f32 / drift_period + col.drift_phase) * 2.0 * std::f32::consts::PI)
                .sin()
                * 1.5;

        // Check if snowflake is at this position (applying drift effect)
        let flake_y = col.y + drift * 0.1;
        let distance = (y_f - flake_y).abs();

        if distance < 0.8 {
            // Select character based on size
            let char_idx = match col.size {
                0 => col.char_seed % 3,
                1 => 3 + col.char_seed % 3,
                _ => 6 + col.char_seed % 3,
            };
            let ch = SNOW_CHARS[char_idx % SNOW_CHARS.len()];

            // Color based on size - using deeper blues visible on both light and dark themes
            let color = match col.size {
                0 => Color::Rgb(70, 100, 160), // Small - dark steel blue
                1 => Color::Rgb(65, 105, 225), // Medium - royal blue
                _ => Color::Rgb(30, 144, 255), // Large - dodger blue
            };

            Span::styled(ch.to_string(), Style::new().fg(color))
        } else {
            Span::raw(" ")
        }
    }

    /// Render a frost crystal character.
    fn render_frost_char(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        elapsed_ms: u64,
        speed: AnimationSpeed,
    ) -> Span<'static> {
        let x_f = x as f32;
        let y_f = y as f32;
        let w_f = width as f32;
        let h_f = height as f32;

        // Calculate distance from nearest edge
        let edge_dist_x = x_f.min(w_f - 1.0 - x_f);
        let edge_dist_y = y_f.min(h_f - 1.0 - y_f);
        let edge_dist = edge_dist_x.min(edge_dist_y * 2.0);

        // Frost growth from edges - controlled by time
        let growth_period = speed.frost_growth_period_ms();
        let growth_phase =
            ((elapsed_ms % growth_period) as f32 / growth_period as f32) * std::f32::consts::PI;
        let growth_factor = growth_phase.sin() * 0.3 + 0.7;

        let max_frost_depth = (w_f.min(h_f) / 3.0) * growth_factor;

        if edge_dist > max_frost_depth {
            return Span::raw(" ");
        }

        // Crystal pattern using pseudo-random based on position
        let seed = (x as usize)
            .wrapping_mul(31)
            .wrapping_add((y as usize).wrapping_mul(17));

        // Density decreases toward center
        let density_threshold = ((edge_dist / max_frost_depth) * 85.0) as usize;
        if seed % 100 > (100 - density_threshold).max(15) {
            return Span::raw(" ");
        }

        // Character selection
        let char_idx = seed % FROST_CHARS.len();
        let ch = FROST_CHARS[char_idx];

        // Color based on distance from edge (darker toward center)
        let depth_ratio = edge_dist / max_frost_depth;
        let base_color = if depth_ratio < 0.3 {
            (200u8, 230u8, 255u8)
        } else if depth_ratio < 0.6 {
            (135, 206, 235)
        } else {
            (70, 130, 180)
        };

        // Add shimmer effect
        let shimmer = (elapsed_ms as f32 / 500.0 + seed as f32 * 0.1).sin() * 0.15 + 0.85;
        let r = (base_color.0 as f32 * shimmer) as u8;
        let g = (base_color.1 as f32 * shimmer) as u8;
        let b = (base_color.2 as f32 * shimmer) as u8;

        Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
    }

    /// Render an aurora borealis character.
    fn render_aurora_char(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        elapsed_ms: u64,
        speed: AnimationSpeed,
    ) -> Span<'static> {
        let x_norm = x as f32 / width.max(1) as f32;
        let y_norm = y as f32 / height.max(1) as f32;

        let period = speed.aurora_wave_period_ms();
        let time_phase = (elapsed_ms % period) as f32 / period as f32;

        // Multiple overlapping waves for aurora curtain effect
        let wave1 = ((x_norm * 3.0 + time_phase * 2.0 * std::f32::consts::PI).sin() + 1.0) / 2.0;
        let wave2 =
            ((x_norm * 5.0 - time_phase * 1.5 * std::f32::consts::PI + 1.0).sin() + 1.0) / 2.0;
        let wave3 = ((x_norm * 2.0 + time_phase * std::f32::consts::PI + 2.0).sin() + 1.0) / 2.0;

        // Combine waves
        let combined_wave = wave1 * 0.5 + wave2 * 0.3 + wave3 * 0.2;

        // Vertical falloff (aurora is brighter at top)
        let vertical_factor = 1.0 - y_norm.powf(0.5);

        // Final intensity
        let intensity = combined_wave * vertical_factor;

        if intensity < 0.15 {
            return Span::raw(" ");
        }

        // Select character based on intensity
        let ch = if intensity > 0.7 {
            '▓'
        } else if intensity > 0.5 {
            '▒'
        } else if intensity > 0.3 {
            '░'
        } else {
            return Span::raw(" ");
        };

        // Aurora colors - cycle through greens, blues, purples
        let color_phase = (elapsed_ms as f32 / 10000.0 + x_norm * 0.5) % 1.0;

        let (r, g, b) = if color_phase < 0.4 {
            // Green phase
            let t = color_phase / 0.4;
            (50, (127.0 + 128.0 * t) as u8, (80.0 + 50.0 * t) as u8)
        } else if color_phase < 0.7 {
            // Blue phase
            let t = (color_phase - 0.4) / 0.3;
            (
                (50.0 * (1.0 - t)) as u8,
                (255.0 - 100.0 * t) as u8,
                (150.0 + 105.0 * t) as u8,
            )
        } else {
            // Purple/pink phase
            let t = (color_phase - 0.7) / 0.3;
            (
                (80.0 + 80.0 * t) as u8,
                (155.0 - 50.0 * t) as u8,
                (255.0 - 30.0 * t) as u8,
            )
        };

        // Apply vertical dimming
        let dimming = 0.3 + vertical_factor * 0.7;
        let r = (r as f32 * dimming) as u8;
        let g = (g as f32 * dimming) as u8;
        let b = (b as f32 * dimming) as u8;

        Span::styled(ch.to_string(), Style::new().fg(Color::Rgb(r, g, b)))
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
