use std::time::Duration;

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::Paragraph,
    DefaultTerminal, Frame,
};

/// Large 7-segment style digits (7 lines tall, 6 chars wide)
const DIGITS: [[&str; 7]; 10] = [
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
const COLON: [&str; 7] = [
    "  ",
    "██",
    "  ",
    "  ",
    "  ",
    "██",
    "  ",
];

/// Letter A
const LETTER_A: [&str; 7] = [
    " ████ ",
    "██  ██",
    "██  ██",
    "██████",
    "██  ██",
    "██  ██",
    "██  ██",
];

/// Letter P
const LETTER_P: [&str; 7] = [
    "█████ ",
    "██  ██",
    "██  ██",
    "█████ ",
    "██    ",
    "██    ",
    "██    ",
];

/// Letter M
const LETTER_M: [&str; 7] = [
    "██   ██",
    "███ ███",
    "███████",
    "██ █ ██",
    "██   ██",
    "██   ██",
    "██   ██",
];

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// Time format for the clock display.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TimeFormat {
    #[default]
    TwentyFourHour,
    TwelveHour,
}

impl TimeFormat {
    /// Toggle between 12-hour and 24-hour format.
    fn toggle(&self) -> Self {
        match self {
            TimeFormat::TwentyFourHour => TimeFormat::TwelveHour,
            TimeFormat::TwelveHour => TimeFormat::TwentyFourHour,
        }
    }
}

/// Color theme for the clock display.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ColorTheme {
    #[default]
    Cyan,
    Green,
    White,
    Magenta,
    Yellow,
    Red,
    Blue,
}

impl ColorTheme {
    /// Cycle to the next color theme.
    fn next(&self) -> Self {
        match self {
            ColorTheme::Cyan => ColorTheme::Green,
            ColorTheme::Green => ColorTheme::Magenta,
            ColorTheme::Magenta => ColorTheme::Yellow,
            ColorTheme::Yellow => ColorTheme::Red,
            ColorTheme::Red => ColorTheme::Blue,
            ColorTheme::Blue => ColorTheme::White,
            ColorTheme::White => ColorTheme::Cyan,
        }
    }

    /// Convert theme to Ratatui Color.
    fn color(self) -> Color {
        match self {
            ColorTheme::Cyan => Color::Cyan,
            ColorTheme::Green => Color::Green,
            ColorTheme::White => Color::White,
            ColorTheme::Magenta => Color::Magenta,
            ColorTheme::Yellow => Color::Yellow,
            ColorTheme::Red => Color::Red,
            ColorTheme::Blue => Color::Blue,
        }
    }
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    /// Current time format (12h or 24h).
    time_format: TimeFormat,
    /// Current color theme.
    color_theme: ColorTheme,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Build large ASCII art time string
    fn build_time_art(&self, hours: u32, minutes: u32, seconds: u32, is_pm: bool) -> Vec<String> {
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
            if self.time_format == TimeFormat::TwelveHour {
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

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        let now = Local::now();

        // Get time components
        let (hours, is_pm) = match self.time_format {
            TimeFormat::TwentyFourHour => (now.format("%H").to_string().parse().unwrap_or(0), false),
            TimeFormat::TwelveHour => {
                let h: u32 = now.format("%I").to_string().parse().unwrap_or(12);
                let pm = now.format("%p").to_string() == "PM";
                (h, pm)
            }
        };
        let minutes: u32 = now.format("%M").to_string().parse().unwrap_or(0);
        let seconds: u32 = now.format("%S").to_string().parse().unwrap_or(0);

        // Format date
        let date_str = now.format("%A, %B %d, %Y").to_string();

        let color = self.color_theme.color();
        let area = frame.area();

        // Build the large time display
        let time_lines = self.build_time_art(hours, minutes, seconds, is_pm);

        // Create vertical layout for centering
        let chunks = Layout::vertical([
            Constraint::Fill(1),   // Top padding
            Constraint::Length(7), // Big digits (7 lines)
            Constraint::Length(2), // Spacing
            Constraint::Length(1), // Date
            Constraint::Fill(1),   // Bottom padding
            Constraint::Length(1), // Help text
        ])
        .split(area);

        // Render big time
        let time_text: Vec<Line> = time_lines
            .into_iter()
            .map(|s| Line::from(s).style(Style::new().fg(color)))
            .collect();

        let time_widget = Paragraph::new(time_text).alignment(Alignment::Center);
        frame.render_widget(time_widget, chunks[1]);

        // Render date
        let date = Paragraph::new(date_str)
            .style(Style::new().fg(color))
            .alignment(Alignment::Center);
        frame.render_widget(date, chunks[3]);

        // Render help text
        let help = Line::from(vec![
            "q".bold().fg(color),
            " quit  ".dark_gray(),
            "t".bold().fg(color),
            " toggle 12/24h  ".dark_gray(),
            "c".bold().fg(color),
            " cycle color".dark_gray(),
        ])
        .centered();
        frame.render_widget(help, chunks[5]);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    /// Uses polling with timeout for real-time clock updates.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        // Poll for events with 100ms timeout for smooth clock updates
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('t')) => self.toggle_time_format(),
            (_, KeyCode::Char('c')) => self.cycle_color_theme(),
            _ => {}
        }
    }

    /// Toggle between 12-hour and 24-hour time format.
    fn toggle_time_format(&mut self) {
        self.time_format = self.time_format.toggle();
    }

    /// Cycle through available color themes.
    fn cycle_color_theme(&mut self) {
        self.color_theme = self.color_theme.next();
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
