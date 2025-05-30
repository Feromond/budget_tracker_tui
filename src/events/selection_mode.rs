use crate::app::state::{App, AppMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_selection_mode(app: &mut App, key_event: KeyEvent) {
    match app.mode {
        AppMode::SelectingCategory | AppMode::SelectingSubcategory => {
            handle_transaction_selection(app, key_event)
        }
        AppMode::SelectingFilterCategory | AppMode::SelectingFilterSubcategory => {
            handle_filter_selection(app, key_event)
        }
        AppMode::SelectingRecurrenceFrequency => {
            handle_recurrence_frequency_selection(app, key_event)
        }
        _ => {}
    }
}

fn handle_transaction_selection(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => app.cancel_selection(),
        KeyCode::Enter => app.confirm_selection(),
        KeyCode::Down => app.select_next_list_item(),
        KeyCode::Up => app.select_previous_list_item(),
        KeyCode::Tab => {}
        _ => {}
    }
}

fn handle_filter_selection(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => app.cancel_advanced_selection(),
        KeyCode::Enter => app.confirm_advanced_selection(),
        KeyCode::Down => app.select_next_list_item(),
        KeyCode::Up => app.select_previous_list_item(),
        KeyCode::Tab => {}
        _ => {}
    }
}

fn handle_recurrence_frequency_selection(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => {
            app.mode = AppMode::RecurringSettings;
        }
        KeyCode::Enter => {
            if let Some(selected) = app.selection_list_state.selected() {
                if let Some(frequency) = app.current_selection_list.get(selected) {
                    app.recurring_settings_fields[1] = frequency.clone();
                }
            }
            app.mode = AppMode::RecurringSettings;
        }
        KeyCode::Down => app.select_next_list_item(),
        KeyCode::Up => app.select_previous_list_item(),
        _ => {}
    }
}
