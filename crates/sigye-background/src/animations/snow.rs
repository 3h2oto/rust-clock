//! Snowfall animation (stateful).

use ratatui::{
    style::{Color, Style},
    text::Span,
};
use sigye_core::AnimationSpeed;

use crate::chars::SNOW_CHARS;

/// State for a single snowfall column.
#[derive(Debug, Clone)]
pub struct SnowColumn {
    /// Current y position of the snowflake.
    pub y: f32,
    /// Speed multiplier for this column.
    pub speed: f32,
    /// Horizontal drift phase offset.
    pub drift_phase: f32,
    /// Size category (0=small, 1=medium, 2=large).
    pub size: u8,
    /// Seed for character generation.
    pub char_seed: usize,
}

/// Initialize snowfall columns for the given dimensions.
pub fn init_columns(width: u16, height: u16, init_seed: u64) -> Vec<SnowColumn> {
    (0..width)
        .map(|x| {
            let x = x as usize;
            // Mix column index with time-based seed for better randomness
            let mixed = x.wrapping_mul(31).wrapping_add(init_seed as usize);
            let stagger = ((mixed.wrapping_mul(11).wrapping_add(7)) % (height as usize * 3)) as f32;
            SnowColumn {
                y: -stagger,
                speed: 0.2 + ((mixed.wrapping_mul(17)) % 10) as f32 / 20.0,
                drift_phase: ((mixed.wrapping_mul(23)) % 100) as f32 / 100.0,
                size: ((mixed.wrapping_mul(13)) % 3) as u8,
                char_seed: mixed.wrapping_mul(19),
            }
        })
        .collect()
}

/// Update snowfall column positions.
pub fn update(columns: &mut [SnowColumn], delta_ms: u64, height: u16, speed: AnimationSpeed) {
    let fall_speed = speed.snow_fall_speed();
    let delta_y = (delta_ms as f32 / 80.0) * fall_speed;

    for col in columns {
        col.y += delta_y * col.speed;
        if col.y > height as f32 + 2.0 {
            col.y = -2.0;
            col.char_seed = col.char_seed.wrapping_add(1);
        }
    }
}

/// Render a snowfall character.
pub fn render_char(columns: &[SnowColumn], x: u16, y: u16, elapsed_ms: u64) -> Span<'static> {
    let x_idx = x as usize;
    let y_f = y as f32;

    if x_idx >= columns.len() {
        return Span::raw(" ");
    }

    let col = &columns[x_idx];

    // Calculate horizontal drift for visual effect
    let drift_period = 3000.0;
    let drift = ((elapsed_ms as f32 / drift_period + col.drift_phase) * 2.0 * std::f32::consts::PI)
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
