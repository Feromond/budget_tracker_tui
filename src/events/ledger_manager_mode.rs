use crate::app::state::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_ledger_manager_mode(app: &mut App, key_event: KeyEvent) {
    match app.mode {
        AppMode::LedgerManager => handle_ledger_list(app, key_event),
        AppMode::LedgerEditor => handle_ledger_editor(app, key_event),
        AppMode::ConfirmLedgerDelete => handle_confirm_ledger_delete(app, key_event),
        _ => {}
    }
}

fn handle_ledger_list(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
            app.exit_ledger_manager()
        }
        (KeyCode::Down, KeyModifiers::NONE) => app.next_ledger(),
        (KeyCode::Up, KeyModifiers::NONE) => app.previous_ledger(),
        (KeyCode::Enter, KeyModifiers::NONE) => app.activate_selected_ledger(),
        (KeyCode::Char('a'), KeyModifiers::NONE) => app.start_adding_ledger(),
        (KeyCode::Char('e'), KeyModifiers::NONE) => app.start_renaming_ledger(),
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.start_copying_ledger(),
        (KeyCode::Char('d'), KeyModifiers::NONE) => app.prepare_delete_ledger(),
        _ => {}
    }
}

fn handle_ledger_editor(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) => app.cancel_ledger_editor(),
        (KeyCode::Enter, KeyModifiers::NONE) => app.save_ledger(),
        (KeyCode::Left, KeyModifiers::NONE) => app.move_cursor_left(),
        (KeyCode::Right, KeyModifiers::NONE) => app.move_cursor_right(),
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            app.insert_char_at_cursor(c)
        }
        (KeyCode::Backspace, KeyModifiers::NONE) => app.delete_char_before_cursor(),
        (KeyCode::Delete, KeyModifiers::NONE) => app.delete_char_after_cursor(),
        _ => {}
    }
}

fn handle_confirm_ledger_delete(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete_ledger(),
        _ => app.cancel_delete_ledger(),
    }
}
