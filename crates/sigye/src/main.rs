use std::time::Duration;

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::Paragraph,
    DefaultTerminal, Frame,
};
use sigye_core::{ColorTheme, TimeFormat};
use sigye_fonts::build_time_art;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
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

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        let now = Local::now();

        // Get time components
        let (hours, is_pm) = match self.time_format {
            TimeFormat::TwentyFourHour => {
                (now.format("%H").to_string().parse().unwrap_or(0), false)
            }
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
        let time_lines = build_time_art(self.time_format, hours, minutes, seconds, is_pm);

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
