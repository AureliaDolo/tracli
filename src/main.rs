use std::{
    error::Error,
    fmt::Display,
    io::{stdout, Stdout},
};

use chrono::{Datelike, NaiveDate};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use inquire::{Confirm, DateSelect, Select};
use ratatui::{
    prelude::CrosstermBackend,
    style::Style,
    widgets::{
        calendar::{CalendarEventStore, Monthly},
        Widget,
    },
};

use rusqlite::Connection;
use time::Date;

type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;
type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

const DATABASE_URL: &str = "./data.db3";

#[derive(Debug)]
struct Period {
    date: NaiveDate,
    flow: Flow,
    // note: String
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(u8)]
enum Flow {
    #[default]
    None,
    Spotting,
    Light,
    Medium,
    Heavy,
    Apocalyptic,
}

impl TryFrom<u8> for Flow {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Flow::None),
            1 => Ok(Flow::Spotting),
            2 => Ok(Flow::Light),
            3 => Ok(Flow::Medium),
            4 => Ok(Flow::Heavy),
            5 => Ok(Flow::Apocalyptic),
            _ => Err("Unsupported flow quantifier".to_string()),
        }
    }
}

const FLOW_OPTION: [Flow; 6] = [
    Flow::None,
    Flow::Spotting,
    Flow::Light,
    Flow::Medium,
    Flow::Heavy,
    Flow::Apocalyptic,
];

impl Display for Flow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flow::None => write!(f, "None"),
            Flow::Spotting => write!(f, "Spotting"),
            Flow::Light => write!(f, "Light"),
            Flow::Medium => write!(f, "Medium"),
            Flow::Heavy => write!(f, "Heavy"),
            Flow::Apocalyptic => write!(f, "Apocalyptic"),
        }
    }
}

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
    let conn = Connection::open(DATABASE_URL)?;
    conn.execute(
        "CREATE TABLE if not exists period (logdate date PRIMARY KEY, flow TINYINT);",
        (),
    )?;
    loop {
        let date = DateSelect::new("Select date").prompt()?;
        let flow = Select::new("Select flow intensity", FLOW_OPTION.to_vec()).prompt()?;
        if Confirm::new(&format!("Save {flow} flow for {date} ?")).prompt()? {
            // save
            let mut stmt = conn.prepare("SELECT * FROM period WHERE logdate = ?;")?;
            if let Some(old) = stmt
                .query_and_then((date,), |row| row.get::<_, u8>(1))?
                .next()
            {
                if Confirm::new(&format!(
                    "Data {} already present at {date}, overwrite it ?",
                    Flow::try_from(old?)?
                ))
                .prompt()?
                {
                    conn.execute("DELETE FROM period  WHERE logdate = ?;", (date,))?;
                } else {
                    continue;
                }
            };
            conn.execute(
                "INSERT INTO period (logdate, flow) VALUES (?1, ?2);;",
                (date, flow as u8),
            )?;
        }
        if Confirm::new("Exit ?").prompt()? {
            break;
        }
    }

    // let c = Monthly::new(
    //     Date::from_calendar_date(
    //         date.year(),
    //         time::Month::try_from(date.month() as u8)?,
    //         date.day() as u8,
    //     )?,
    //     CalendarEventStore::default(),
    // )
    // .show_weekdays_header(Style::default())
    // .show_month_header(Style::default());
    // enable_raw_mode()?;
    // let mut terminal = setup_terminal()?;
    // for _ in 0..1000 {
    //     terminal.draw(|f| ui(f, c.clone()))?;
    // }
    // restore_terminal(terminal)?;

    Ok(())
}
