use crate::app::state::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_filter_mode(app: &mut App, key_event: KeyEvent) {
    match app.mode {
        AppMode::Filtering => handle_simple_filtering(app, key_event),
        AppMode::AdvancedFiltering => handle_advanced_filtering(app, key_event),
        _ => {}
    }
}

fn handle_simple_filtering(app: &mut App, key_event: KeyEvent) {
    match (key_event.modifiers, key_event.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('f')) => app.start_advanced_filtering(),
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
            app.clear_input_field();
            app.apply_filter();
        }
        (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Enter) => {
            app.exit_filtering()
        }
        (KeyModifiers::NONE, KeyCode::Char(c)) => {
            app.insert_char_at_cursor(c);
            app.apply_filter();
        }
        (KeyModifiers::SHIFT, KeyCode::Char(c)) => {
            app.insert_char_at_cursor(c);
            app.apply_filter();
        }
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            app.delete_char_before_cursor();
            app.apply_filter();
        }
        (KeyModifiers::NONE, KeyCode::Delete) => {
            app.delete_char_after_cursor();
            app.apply_filter();
        }
        (KeyModifiers::NONE, KeyCode::Left) => app.move_cursor_left(),
        (KeyModifiers::NONE, KeyCode::Right) => app.move_cursor_right(),
        _ => {}
    }
}

fn handle_advanced_filtering(app: &mut App, key_event: KeyEvent) {
    match (key_event.modifiers, key_event.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => app.clear_advanced_filter_fields(),
        (KeyModifiers::NONE, KeyCode::Esc) => app.cancel_advanced_filtering(),
        (KeyModifiers::NONE, KeyCode::Enter) => match app.current_advanced_filter_field {
            3 => app.start_advanced_category_selection(),
            4 => app.start_advanced_subcategory_selection(),
            5 => app.toggle_advanced_transaction_type(),
            _ => app.finish_advanced_filtering(),
        },
        (KeyModifiers::NONE, KeyCode::Tab) => app.next_advanced_filter_field(),
        (KeyModifiers::NONE, KeyCode::BackTab) => app.previous_advanced_filter_field(),
        (KeyModifiers::NONE, KeyCode::Up) => app.previous_advanced_filter_field(),
        (KeyModifiers::NONE, KeyCode::Down) => app.next_advanced_filter_field(),
        (KeyModifiers::NONE, KeyCode::Left) => match app.current_advanced_filter_field {
            0 | 1 => app.decrement_advanced_date(),
            5 => app.toggle_advanced_transaction_type(),
            _ => {}
        },
        (KeyModifiers::NONE, KeyCode::Right) => match app.current_advanced_filter_field {
            0 | 1 => app.increment_advanced_date(),
            5 => app.toggle_advanced_transaction_type(),
            _ => {}
        },
        (KeyModifiers::SHIFT, KeyCode::Left) => match app.current_advanced_filter_field {
            0 | 1 => app.decrement_advanced_month(),
            _ => {}
        },
        (KeyModifiers::SHIFT, KeyCode::Right) => match app.current_advanced_filter_field {
            0 | 1 => app.increment_advanced_month(),
            _ => {}
        },
        (KeyModifiers::NONE, KeyCode::Char(c)) => app.insert_char_advanced_filter(c),
        (KeyModifiers::SHIFT, KeyCode::Char(c)) => app.insert_char_advanced_filter(c),
        (KeyModifiers::NONE, KeyCode::Backspace) => app.delete_char_advanced_filter(),
        (KeyModifiers::NONE, KeyCode::Delete) => app.delete_char_advanced_filter(),
        _ => {}
    }
}
