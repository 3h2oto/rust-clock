//! Background animation state management.

use ratatui::{
    Frame,
    text::{Line, Span},
    widgets::Paragraph,
};
use sigye_core::{AnimationSpeed, BackgroundStyle, SystemMetrics};

use crate::animations::{matrix, reactive, snow, stateless};

/// Background animation state.
#[derive(Debug)]
pub struct BackgroundState {
    /// Matrix rain column states.
    matrix_columns: Vec<matrix::MatrixColumn>,
    /// Snowfall column states.
    snow_columns: Vec<snow::SnowColumn>,
    /// Last known terminal width.
    last_width: u16,
    /// Last known terminal height.
    last_height: u16,
    /// Last update time in milliseconds.
    last_update_ms: u64,
    /// Seed captured at initialization for randomness.
    init_seed: u64,
}

impl Default for BackgroundState {
    fn default() -> Self {
        Self::new()
    }
}

impl BackgroundState {
    /// Create a new background state.
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Capture system time as seed for randomness
        let init_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        Self {
            matrix_columns: Vec::new(),
            snow_columns: Vec::new(),
            last_width: 0,
            last_height: 0,
            last_update_ms: 0,
            init_seed,
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
            self.matrix_columns = matrix::init_columns(width, height);
        }
        if style == BackgroundStyle::Snowfall
            && (dimensions_changed || self.snow_columns.is_empty())
        {
            self.snow_columns = snow::init_columns(width, height, self.init_seed);
        }

        if dimensions_changed {
            self.last_width = width;
            self.last_height = height;
        }

        // Calculate delta time for stateful animations
        let delta_ms = elapsed_ms.saturating_sub(self.last_update_ms);
        self.last_update_ms = elapsed_ms;

        // Update animation states
        if style == BackgroundStyle::MatrixRain {
            matrix::update(&mut self.matrix_columns, delta_ms, height, speed);
        }
        if style == BackgroundStyle::Snowfall {
            snow::update(&mut self.snow_columns, delta_ms, height, speed);
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
            BackgroundStyle::Starfield => stateless::render_starfield_char(x, y, elapsed_ms, speed),
            BackgroundStyle::MatrixRain => matrix::render_char(&self.matrix_columns, x, y),
            BackgroundStyle::GradientWave => {
                stateless::render_gradient_char(x, y, width, height, elapsed_ms, speed)
            }
            BackgroundStyle::Snowfall => snow::render_char(&self.snow_columns, x, y, elapsed_ms),
            BackgroundStyle::Frost => {
                stateless::render_frost_char(x, y, width, height, elapsed_ms, speed)
            }
            BackgroundStyle::Aurora => {
                stateless::render_aurora_char(x, y, width, height, elapsed_ms, speed)
            }
            // Reactive backgrounds are handled separately in render_reactive()
            BackgroundStyle::SystemPulse
            | BackgroundStyle::ResourceWave
            | BackgroundStyle::DataFlow
            | BackgroundStyle::HeatMap => Span::raw(" "),
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
                reactive::render_system_pulse(frame, elapsed_ms, speed, metrics)
            }
            BackgroundStyle::ResourceWave => {
                reactive::render_resource_wave(frame, elapsed_ms, speed, metrics)
            }
            BackgroundStyle::DataFlow => {
                reactive::render_data_flow(frame, elapsed_ms, speed, metrics)
            }
            BackgroundStyle::HeatMap => {
                reactive::render_heat_map(frame, elapsed_ms, speed, metrics)
            }
            _ => {}
        }
    }
}
