mod app;
mod event;
mod model;
mod persistence;
mod ui;

use app::App;
use event::run_app;
use persistence::save_transactions;

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use std::result::Result as StdResult;

fn main() -> StdResult<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let initial_status = app.status_message.clone();

    let run_result = run_app(&mut terminal, &mut app);

    let save_result = save_transactions(&app.transactions);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    if let Some(msg) = initial_status {
        eprintln!("Initial Status: {}", msg);
    }

    match (run_result, save_result) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(run_err), Ok(_)) => {
            eprintln!("Application Error: {}", run_err);
            Err(run_err)
        }
        (Ok(_), Err(save_err)) => {
            eprintln!("Error Saving Transactions: {}", save_err);
            Err(save_err.into())
        }
        (Err(run_err), Err(save_err)) => {
            eprintln!("Application Error: {}", run_err);
            eprintln!("Error Saving Transactions: {}", save_err);
            Err(run_err)
        }
    }
}
