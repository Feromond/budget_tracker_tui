use crate::app::state::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_add_edit_mode(app: &mut App, key_event: KeyEvent) {
    match (key_event.modifiers, key_event.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => {
            if app.mode == AppMode::Adding {
                app.exit_adding(true);
            } else {
                app.exit_editing(true);
            }
        }
        (KeyModifiers::NONE, KeyCode::Tab) => {
            app.next_add_edit_field();
        }
        (KeyModifiers::NONE, KeyCode::BackTab) => {
            app.previous_add_edit_field();
        }
        (KeyModifiers::NONE, KeyCode::Enter) => {
            // Toggle Type, trigger selection popups, or save transaction
            match app.current_add_edit_field {
                3 => app.toggle_transaction_type(), // Enter on Type field toggles it
                4 => app.start_category_selection(), // Enter on Category field
                5 => app.start_subcategory_selection(), // Enter on Subcategory field
                _ => {
                    // Enter on any other field: Save
                    if app.mode == AppMode::Adding {
                        app.add_transaction();
                    } else {
                        app.update_transaction();
                    }
                }
            }
        }
        (KeyModifiers::NONE, KeyCode::Up) => app.previous_add_edit_field(),
        (KeyModifiers::NONE, KeyCode::Down) => app.next_add_edit_field(),
        (KeyModifiers::NONE, KeyCode::Left) => match app.current_add_edit_field {
            0 => app.decrement_date(),
            3 => app.toggle_transaction_type(),
            _ => {}
        },
        (KeyModifiers::NONE, KeyCode::Right) => match app.current_add_edit_field {
            0 => app.increment_date(),
            3 => app.toggle_transaction_type(),
            _ => {}
        },
        (KeyModifiers::SHIFT, KeyCode::Left) => {
            if app.current_add_edit_field == 0 {
                app.decrement_month()
            }
        }
        (KeyModifiers::SHIFT, KeyCode::Right) => {
            if app.current_add_edit_field == 0 {
                app.increment_month()
            }
        }
        (KeyModifiers::NONE, KeyCode::Char(c)) => match app.current_add_edit_field {
            0 if c == '+' || c == '=' => app.increment_date(),
            0 if c == '-' => app.decrement_date(),
            // Only allow digits for the date field (field 0)
            0 if c.is_ascii_digit() => app.insert_char_add_edit(c),
            // Allow any character for other non-special fields (1, 2)
            field if ![0, 3, 4, 5].contains(&field) => app.insert_char_add_edit(c),
            _ => {} // Ignore char input for fields 0 (non-digit), 3, 4, 5
        },
        (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
            if app.current_add_edit_field == 1 {
                app.insert_char_add_edit(c);
            }
        }
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            if ![3, 4, 5].contains(&app.current_add_edit_field) {
                app.delete_char_add_edit();
            }
        }
        _ => {}
    }
}

pub fn handle_confirm_delete(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete(),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete(),
        _ => {}
    }
} 