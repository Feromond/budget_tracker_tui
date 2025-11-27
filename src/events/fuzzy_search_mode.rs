use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_fuzzy_search_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_fuzzy_selection();
        }
        KeyCode::Enter => {
            app.confirm_fuzzy_selection();
        }
        KeyCode::Up => {
            app.select_previous_list_item();
        }
        KeyCode::Down => {
            app.select_next_list_item();
        }
        KeyCode::Backspace => {
            if !app.search_query.is_empty() {
                app.search_query.pop();
                app.update_fuzzy_search_results();
            }
        }
        KeyCode::Char(c) => {
            // Allow normal typing
            app.search_query.push(c);
            app.update_fuzzy_search_results();
        }
        _ => {}
    }
}
