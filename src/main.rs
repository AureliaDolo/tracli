use std::{
    error::Error,
    io::{stdout, Stdout},
};

use chrono::Datelike;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use inquire::DateSelect;
use ratatui::{
    prelude::CrosstermBackend,
    style::Style,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        Widget,
    },
};
use time::Date;

type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;
type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

fn ui(frame: &mut Frame, widget: impl Widget) {
    frame.render_widget(widget, frame.size())
}

fn setup_terminal() -> Result<Terminal, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let date = DateSelect::new("When do you want to travel?").prompt()?;

    let c = Monthly::new(
        Date::from_calendar_date(
            date.year(),
            time::Month::try_from(date.month() as u8)?,
            date.day() as u8,
        )?,
        CalendarEventStore::default(),
    )
    .show_weekdays_header(Style::default())
    .show_month_header(Style::default());
    enable_raw_mode()?;
    let mut terminal = setup_terminal()?;
    for _ in 0..1000 {
        terminal.draw(|f| ui(f, c.clone()))?;
    }
    restore_terminal(terminal)?;

    Ok(())
}
