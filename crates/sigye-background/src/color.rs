//! Color utility functions for background animations.

use ratatui::style::Color;

/// Map a resource value (0.0-1.0) to a color from cool blue to warm red.
pub fn resource_to_color(value: f32) -> Color {
    let value = value.clamp(0.0, 1.0);

    // Hue: 240 (blue) -> 60 (yellow) -> 0 (red)
    let hue = 240.0 - (value * 240.0);

    // Higher usage = more saturated and brighter
    let saturation = 0.6 + (value * 0.4);
    let lightness = 0.15 + (value * 0.25);

    hsl_to_rgb(hue, saturation, lightness)
}

/// Convert HSL to RGB color.
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
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
