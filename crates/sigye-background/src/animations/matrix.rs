//! Matrix rain animation (stateful).

use ratatui::{
    style::{Color, Style},
    text::Span,
};
use sigye_core::AnimationSpeed;

use crate::chars::MATRIX_CHARS;

/// State for a single matrix rain column.
#[derive(Debug, Clone)]
pub struct MatrixColumn {
    /// Current y position of the raindrop head.
    pub y: f32,
    /// Speed multiplier for this column.
    pub speed: f32,
    /// Length of the trail.
    pub trail_length: usize,
    /// Seed for character generation.
    pub char_seed: usize,
}

/// Initialize matrix columns for the given dimensions.
pub fn init_columns(width: u16, height: u16) -> Vec<MatrixColumn> {
    (0..width)
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
        .collect()
}

/// Update matrix column positions.
pub fn update(columns: &mut [MatrixColumn], delta_ms: u64, height: u16, speed: AnimationSpeed) {
    let fall_speed = speed.matrix_fall_speed();
    let delta_y = (delta_ms as f32 / 50.0) * fall_speed;

    for col in columns {
        col.y += delta_y * col.speed;
        // Reset column when it goes off screen
        if col.y > (height as f32 + col.trail_length as f32) {
            col.y = -(col.trail_length as f32);
            col.char_seed = col.char_seed.wrapping_add(1);
        }
    }
}

/// Render a matrix rain character.
pub fn render_char(columns: &[MatrixColumn], x: u16, y: u16) -> Span<'static> {
    let x = x as usize;
    let y = y as f32;

    if x >= columns.len() {
        return Span::raw(" ");
    }

    let col = &columns[x];
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
