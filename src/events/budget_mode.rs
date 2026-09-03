use crate::app::state::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_budget_mode(app: &mut App, key_event: KeyEvent) {
    if app.mode == AppMode::BudgetCategoryEditor {
        handle_budget_target_editor(app, key_event);
        return;
    }

    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => app.exit_budget_mode(),
        (KeyCode::Char('e'), KeyModifiers::NONE) => app.start_editing_budget_target(),
        (KeyCode::Char('t'), KeyModifiers::NONE) => app.start_editing_monthly_budget(),
        (KeyCode::Char('c'), KeyModifiers::NONE) => app.open_category_catalog(AppMode::Budget),
        (KeyCode::Down, KeyModifiers::NONE) => app.next_budget_category(),
        (KeyCode::Up, KeyModifiers::NONE) => app.previous_budget_category(),
        (KeyCode::Right, KeyModifiers::NONE) => app.next_budget_month(),
        (KeyCode::Left, KeyModifiers::NONE) => app.previous_budget_month(),
        (KeyCode::Right, KeyModifiers::SHIFT) => app.next_budget_year(),
        (KeyCode::Left, KeyModifiers::SHIFT) => app.previous_budget_year(),
        _ => {}
    }
}

fn handle_budget_target_editor(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) => app.cancel_budget_edit(),
        (KeyCode::Enter, KeyModifiers::NONE) => app.save_budget_edit(),
        (KeyCode::Left, KeyModifiers::NONE) => app.move_cursor_left(),
        (KeyCode::Right, KeyModifiers::NONE) => app.move_cursor_right(),
        (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::Tab, KeyModifiers::NONE) => {
            app.cycle_budget_edit_scope(true)
        }
        (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::BackTab, KeyModifiers::NONE) => {
            app.cycle_budget_edit_scope(false)
        }
        (KeyCode::Char(c), KeyModifiers::NONE) => app.insert_char_at_cursor(c),
        (KeyCode::Backspace, KeyModifiers::NONE) => app.delete_char_before_cursor(),
        (KeyCode::Delete, KeyModifiers::NONE) => app.delete_char_after_cursor(),
        _ => {}
    }
}
