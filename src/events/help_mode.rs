use crate::app::help::get_help_for_mode;
use crate::app::state::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_help_mode(app: &mut App, key_event: KeyEvent) {
    // Handle Detail View Mode
    if app.mode == AppMode::KeybindingDetail {
        if matches!(key_event.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
            app.mode = AppMode::KeybindingsInfo;
        }
        return;
    }

    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            let prev = app.previous_mode.unwrap_or(AppMode::Normal);
            app.mode = prev;
            app.previous_mode = None;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let selected = app.help_table_state.selected().unwrap_or(0);
            let mode = app.previous_mode.unwrap_or(AppMode::Normal);
            let bindings = get_help_for_mode(mode);
            if !bindings.is_empty() {
                let new_selected = (selected + 1).min(bindings.len() - 1);
                app.help_table_state.select(Some(new_selected));
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
             let selected = app.help_table_state.selected().unwrap_or(0);
             let new_selected = selected.saturating_sub(1);
             app.help_table_state.select(Some(new_selected));
        }
        KeyCode::PageDown => {
             let selected = app.help_table_state.selected().unwrap_or(0);
             let mode = app.previous_mode.unwrap_or(AppMode::Normal);
             let bindings = get_help_for_mode(mode);
             if !bindings.is_empty() {
                 let new_selected = (selected + 10).min(bindings.len() - 1);
                 app.help_table_state.select(Some(new_selected));
             }
        }
        KeyCode::PageUp => {
             let selected = app.help_table_state.selected().unwrap_or(0);
             let new_selected = selected.saturating_sub(10);
             app.help_table_state.select(Some(new_selected));
        }
        KeyCode::Enter => {
            // Check if current selection has extended details
            let mode = app.previous_mode.unwrap_or(AppMode::Normal);
            let bindings = get_help_for_mode(mode);
            let selected = app.help_table_state.selected().unwrap_or(0);
            
            if let Some(binding) = bindings.get(selected) {
                if binding.extended_description.is_some() {
                    app.mode = AppMode::KeybindingDetail;
                }
            }
        }
        _ => {}
    }
}
