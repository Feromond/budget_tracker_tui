use crate::app::state::App;
use crate::model::SortColumn;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_normal_mode(app: &mut App, key_event: KeyEvent) {
    let key_code = key_event.code;

    match key_code {
        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
        KeyCode::Down => app.next_item(),
        KeyCode::Up => app.previous_item(),
        KeyCode::Char('a') => app.start_adding(),
        KeyCode::Char('d') => app.prepare_for_delete(),
        KeyCode::Char('e') => app.start_editing(),
        KeyCode::Char('f') => app.start_filtering(),
        KeyCode::Char('r') => app.start_recurring_settings(),
        KeyCode::Char('s') => app.enter_summary_mode(),
        KeyCode::Char('c') => app.enter_category_summary_mode(),
        KeyCode::Char('o') => app.enter_settings_mode(),
        // Sorting
        KeyCode::Char('1') | KeyCode::F(1) => app.set_sort_column(SortColumn::Date),
        KeyCode::Char('2') | KeyCode::F(2) => app.set_sort_column(SortColumn::Description),
        KeyCode::Char('3') | KeyCode::F(3) => app.set_sort_column(SortColumn::Category),
        KeyCode::Char('4') | KeyCode::F(4) => app.set_sort_column(SortColumn::Subcategory),
        KeyCode::Char('5') | KeyCode::F(5) => app.set_sort_column(SortColumn::Type),
        KeyCode::Char('6') | KeyCode::F(6) => app.set_sort_column(SortColumn::Amount),
        _ => {}
    }
} 