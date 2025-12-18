//! ASCII art fonts for the sigye clock application.

use sigye_core::TimeFormat;

/// Large 7-segment style digits (7 lines tall, 6 chars wide)
pub const DIGITS: [[&str; 7]; 10] = [
    // 0
    [
        " ████ ",
        "██  ██",
        "██  ██",
        "██  ██",
        "██  ██",
        "██  ██",
        " ████ ",
    ],
    // 1
    [
        "  ██  ",
        " ███  ",
        "  ██  ",
        "  ██  ",
        "  ██  ",
        "  ██  ",
        " ████ ",
    ],
    // 2
    [
        " ████ ",
        "██  ██",
        "    ██",
        "  ██  ",
        " ██   ",
        "██    ",
        "██████",
    ],
    // 3
    [
        " ████ ",
        "██  ██",
        "    ██",
        "  ███ ",
        "    ██",
        "██  ██",
        " ████ ",
    ],
    // 4
    [
        "██  ██",
        "██  ██",
        "██  ██",
        "██████",
        "    ██",
        "    ██",
        "    ██",
    ],
    // 5
    [
        "██████",
        "██    ",
        "██    ",
        "█████ ",
        "    ██",
        "██  ██",
        " ████ ",
    ],
    // 6
    [
        " ████ ",
        "██    ",
        "██    ",
        "█████ ",
        "██  ██",
        "██  ██",
        " ████ ",
    ],
    // 7
    [
        "██████",
        "    ██",
        "   ██ ",
        "  ██  ",
        "  ██  ",
        "  ██  ",
        "  ██  ",
    ],
    // 8
    [
        " ████ ",
        "██  ██",
        "██  ██",
        " ████ ",
        "██  ██",
        "██  ██",
        " ████ ",
    ],
    // 9
    [
        " ████ ",
        "██  ██",
        "██  ██",
        " █████",
        "    ██",
        "    ██",
        " ████ ",
    ],
];

/// Colon separator (7 lines tall, 2 chars wide)
pub const COLON: [&str; 7] = ["  ", "██", "  ", "  ", "  ", "██", "  "];

/// Letter A
pub const LETTER_A: [&str; 7] = [
    " ████ ",
    "██  ██",
    "██  ██",
    "██████",
    "██  ██",
    "██  ██",
    "██  ██",
];

/// Letter P
pub const LETTER_P: [&str; 7] = [
    "█████ ",
    "██  ██",
    "██  ██",
    "█████ ",
    "██    ",
    "██    ",
    "██    ",
];

/// Letter M
pub const LETTER_M: [&str; 7] = [
    "██   ██",
    "███ ███",
    "███████",
    "██ █ ██",
    "██   ██",
    "██   ██",
    "██   ██",
];

/// Build large ASCII art time string.
///
/// # Arguments
/// * `time_format` - Whether to use 12-hour or 24-hour format
/// * `hours` - Hour value (0-23 for 24h, 1-12 for 12h)
/// * `minutes` - Minute value (0-59)
/// * `seconds` - Second value (0-59)
/// * `is_pm` - Whether it's PM (only used for 12-hour format)
///
/// # Returns
/// A vector of 7 strings, each representing one line of the ASCII art.
pub fn build_time_art(
    time_format: TimeFormat,
    hours: u32,
    minutes: u32,
    seconds: u32,
    is_pm: bool,
) -> Vec<String> {
    let h1 = (hours / 10) as usize;
    let h2 = (hours % 10) as usize;
    let m1 = (minutes / 10) as usize;
    let m2 = (minutes % 10) as usize;
    let s1 = (seconds / 10) as usize;
    let s2 = (seconds % 10) as usize;

    let mut lines = Vec::with_capacity(7);

    for row in 0..7 {
        let mut line = String::new();
        line.push_str(DIGITS[h1][row]);
        line.push(' ');
        line.push_str(DIGITS[h2][row]);
        line.push(' ');
        line.push_str(COLON[row]);
        line.push(' ');
        line.push_str(DIGITS[m1][row]);
        line.push(' ');
        line.push_str(DIGITS[m2][row]);
        line.push(' ');
        line.push_str(COLON[row]);
        line.push(' ');
        line.push_str(DIGITS[s1][row]);
        line.push(' ');
        line.push_str(DIGITS[s2][row]);

        // Add AM/PM for 12-hour format
        if time_format == TimeFormat::TwelveHour {
            line.push_str("  ");
            if is_pm {
                line.push_str(LETTER_P[row]);
            } else {
                line.push_str(LETTER_A[row]);
            }
            line.push_str(LETTER_M[row]);
        }

        lines.push(line);
    }

    lines
}
