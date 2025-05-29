use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_recurring_mode(app: &mut App, key_event: KeyEvent) {
    match (key_event.modifiers, key_event.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => app.exit_recurring_settings(true),
        (KeyModifiers::NONE, KeyCode::Enter) => match app.current_recurring_field {
            1 => app.start_frequency_selection(),
            _ => app.save_recurring_settings(),
        },
        (KeyModifiers::NONE, KeyCode::Tab) | (KeyModifiers::NONE, KeyCode::Down) => {
            app.next_recurring_field()
        }
        (KeyModifiers::NONE, KeyCode::BackTab) | (KeyModifiers::NONE, KeyCode::Up) => {
            app.previous_recurring_field()
        }
        (KeyModifiers::NONE, KeyCode::Left) => match app.current_recurring_field {
            0 => app.toggle_recurring_enabled(),
            2 => app.decrement_date_recurring(),
            _ => {}
        },
        (KeyModifiers::NONE, KeyCode::Right) => match app.current_recurring_field {
            0 => app.toggle_recurring_enabled(),
            2 => app.increment_date_recurring(),
            _ => {}
        },
        (KeyModifiers::SHIFT, KeyCode::Left) => {
            if app.current_recurring_field == 2 {
                app.decrement_month_recurring();
            }
        }
        (KeyModifiers::SHIFT, KeyCode::Right) => {
            if app.current_recurring_field == 2 {
                app.increment_month_recurring();
            }
        }
        (KeyModifiers::NONE, KeyCode::Char(c)) => match app.current_recurring_field {
            2 if c == '+' || c == '=' => app.increment_date_recurring(),
            2 if c == '-' => app.decrement_date_recurring(),
            2 if c.is_ascii_digit() => app.insert_char_recurring(c),
            _ => {}
        },
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            if app.current_recurring_field == 2 {
                app.delete_char_recurring();
            }
        }
        _ => {}
    }
}
