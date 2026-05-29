mod app;
mod config;
mod db;
mod events;
mod model;
mod persistence;
mod recurring;
mod ui;
mod validation;

use crate::app::state::App;
use events::run_app;

use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use std::result::Result as StdResult;

fn main() -> StdResult<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    stdout()
        .execute(EnterAlternateScreen)?
        .execute(EnableBracketedPaste)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let initial_status = app.status_message.clone();

    let run_result = run_app(&mut terminal, &mut app);

    stdout().execute(DisableBracketedPaste)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Some(msg) = initial_status {
        eprintln!("Initial Status: {}", msg);
    }

    match run_result {
        Ok(_) => Ok(()),
        Err(run_err) => {
            eprintln!("Application Error: {}", run_err);
            Err(run_err)
        }
    }
}
