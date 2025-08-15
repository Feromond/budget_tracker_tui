use crate::app::state::{App, AppMode, CategorySummaryItem};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_summary_mode(app: &mut App, key_event: KeyEvent) {
    match app.mode {
        AppMode::Summary => handle_regular_summary(app, key_event),
        AppMode::CategorySummary => handle_category_summary(app, key_event),
        _ => {}
    }
}

fn handle_regular_summary(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => app.exit_summary_mode(),
        KeyCode::Down => app.next_summary_month(),
        KeyCode::Up => app.previous_summary_month(),
        KeyCode::Char(']') | KeyCode::PageDown | KeyCode::Right => app.next_summary_year(),
        KeyCode::Char('[') | KeyCode::PageUp | KeyCode::Left => app.previous_summary_year(),
        KeyCode::Char('m') => app.summary_multi_month_mode = !app.summary_multi_month_mode,
        KeyCode::Char('c') => app.summary_cumulative_mode = !app.summary_cumulative_mode,
        _ => {}
    }
}

fn handle_category_summary(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') | KeyCode::Esc => app.exit_category_summary_mode(),
        KeyCode::Down => app.next_category_summary_item(),
        KeyCode::Up => app.previous_category_summary_item(),
        // PageUp/PageDown for month jumping
        KeyCode::PageUp => app.previous_category_summary_month(),
        KeyCode::PageDown => app.next_category_summary_month(),
        // Brackets and Left/Right for year navigation
        KeyCode::Char(']') | KeyCode::Right => app.next_category_summary_year(),
        KeyCode::Char('[') | KeyCode::Left => app.previous_category_summary_year(),
        KeyCode::Enter => {
            let items = app.get_visible_category_summary_items();
            if let Some(selected_index) = app.category_summary_table_state.selected() {
                if let Some(item) = items.get(selected_index) {
                    if let CategorySummaryItem::Month(month, _) = item {
                        if app.expanded_category_summary_months.contains(month) {
                            app.expanded_category_summary_months.remove(month);
                        } else {
                            app.expanded_category_summary_months.insert(*month);
                        }
                        app.cached_visible_category_items =
                            app.get_visible_category_summary_items();
                    }
                    // Clamp selection to valid range using cached list
                    let len = app.cached_visible_category_items.len();
                    if len == 0 {
                        app.category_summary_table_state.select(None);
                    } else if selected_index >= len {
                        app.category_summary_table_state.select(Some(len - 1));
                    } else {
                        app.category_summary_table_state
                            .select(Some(selected_index));
                    }
                }
            }
        }
        _ => {}
    }
}
