use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_settings_mode(app: &mut App, key_event: KeyEvent) {
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Esc, KeyModifiers::NONE) => app.exit_settings_mode(),
        (KeyCode::Enter, KeyModifiers::NONE) => app.save_settings(),
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => app.reset_settings_path_to_default(),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => app.clear_settings_field(),
        (KeyCode::Tab, KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            app.next_settings_field()
        }
        (KeyCode::BackTab, KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            app.previous_settings_field()
        }
        (KeyCode::Left, KeyModifiers::NONE) => app.move_cursor_left_settings(),
        (KeyCode::Right, KeyModifiers::NONE) => app.move_cursor_right_settings(),
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            app.insert_char_settings(c);
        }
        (KeyCode::Backspace, KeyModifiers::NONE) => app.delete_char_settings(),
        (KeyCode::Delete, KeyModifiers::NONE) => app.clear_settings_field(),
        _ => {}
    }
}
