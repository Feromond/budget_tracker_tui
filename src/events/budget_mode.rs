use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_budget_mode(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => app.exit_budget_mode(),
        (KeyCode::Down, KeyModifiers::NONE) => app.next_budget_category(),
        (KeyCode::Up, KeyModifiers::NONE) => app.previous_budget_category(),
        (KeyCode::Right, KeyModifiers::NONE) => app.next_budget_month(),
        (KeyCode::Left, KeyModifiers::NONE) => app.previous_budget_month(),
        (KeyCode::Right, KeyModifiers::SHIFT) => app.next_budget_year(),
        (KeyCode::Left, KeyModifiers::SHIFT) => app.previous_budget_year(),
        _ => {}
    }
}
