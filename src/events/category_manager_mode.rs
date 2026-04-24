use crate::app::state::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_category_manager_mode(app: &mut App, key_event: KeyEvent) {
    match app.mode {
        AppMode::CategoryCatalog => handle_category_catalog(app, key_event),
        AppMode::CategoryEditor => handle_category_editor(app, key_event),
        AppMode::ConfirmCategoryDelete => handle_confirm_category_delete(app, key_event),
        _ => {}
    }
}

fn handle_category_catalog(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
            app.exit_category_catalog()
        }
        (KeyCode::Down, KeyModifiers::NONE) => app.next_category_record(),
        (KeyCode::Up, KeyModifiers::NONE) => app.previous_category_record(),
        (KeyCode::Char('a'), KeyModifiers::NONE) => app.start_adding_category(),
        (KeyCode::Char('e'), KeyModifiers::NONE) | (KeyCode::Enter, KeyModifiers::NONE) => {
            app.start_editing_category()
        }
        (KeyCode::Char('d'), KeyModifiers::NONE) => app.prepare_delete_category(),
        _ => {}
    }
}

fn handle_category_editor(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) => app.exit_category_editor(true),
        (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            app.next_category_field()
        }
        (KeyCode::BackTab, KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            app.previous_category_field()
        }
        (KeyCode::Enter, KeyModifiers::NONE) => {
            if app.current_category_field == 0 {
                app.toggle_category_transaction_type();
            } else {
                app.save_category();
            }
        }
        (KeyCode::Left, KeyModifiers::NONE) => {
            if app.current_category_field == 0 {
                app.toggle_category_transaction_type();
            } else {
                app.move_cursor_left();
            }
        }
        (KeyCode::Right, KeyModifiers::NONE) => {
            if app.current_category_field == 0 {
                app.toggle_category_transaction_type();
            } else {
                app.move_cursor_right();
            }
        }
        (KeyCode::Char(c), KeyModifiers::NONE) => match app.current_category_field {
            0 => {}
            4 if c.is_ascii_digit() || c == '.' => app.insert_char_at_cursor(c),
            1..=3 => app.insert_char_at_cursor(c),
            _ => {}
        },
        (KeyCode::Char(c), KeyModifiers::SHIFT)
            if (1..=3).contains(&app.current_category_field) =>
        {
            app.insert_char_at_cursor(c);
        }
        (KeyCode::Backspace, KeyModifiers::NONE) if app.current_category_field != 0 => {
            app.delete_char_before_cursor();
        }
        (KeyCode::Delete, KeyModifiers::NONE) if app.current_category_field != 0 => {
            app.delete_char_after_cursor();
        }
        _ => {}
    }
}

fn handle_confirm_category_delete(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete_category(),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete_category(),
        _ => {}
    }
}
