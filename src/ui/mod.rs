pub mod category_summary;
pub mod dialog;
pub mod filter;
pub mod help;
pub mod helpers;
pub mod recurring;
pub mod settings;
pub mod status;
pub mod summary;
pub mod transaction_form;
pub mod transaction_table;

use crate::app::state::{App, AppMode};
use ratatui::Frame;

pub(crate) fn ui(f: &mut Frame, app: &mut App) {
    let filter_bar_height = if app.mode == AppMode::Filtering { 3 } else { 0 };
    let status_bar_height = if app.status_message.is_some() { 3 } else { 0 };
    let summary_bar_height = 3;
    let help_bar_height = 3;

    let main_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Min(0),
            ratatui::layout::Constraint::Length(filter_bar_height),
            ratatui::layout::Constraint::Length(summary_bar_height),
            ratatui::layout::Constraint::Length(status_bar_height),
            ratatui::layout::Constraint::Length(help_bar_height),
        ])
        .split(f.area());

    let main_area = main_chunks[0];
    let filter_area = main_chunks[1];
    let summary_area = main_chunks[2];
    let status_area = main_chunks[3];
    let help_area = main_chunks[4];

    match app.mode {
        AppMode::Normal | AppMode::Filtering => {
            transaction_table::render_transaction_table(f, app, main_area);
        }
        AppMode::AdvancedFiltering => {
            filter::render_advanced_filter_form(f, app, main_area);
        }
        AppMode::SelectingFilterCategory | AppMode::SelectingFilterSubcategory => {
            filter::render_advanced_filter_form(f, app, main_area);
            dialog::render_selection_popup(f, app, main_area);
        }
        AppMode::Adding | AppMode::Editing => {
            transaction_form::render_transaction_form(f, app, main_area);
        }
        AppMode::ConfirmDelete => {
            transaction_table::render_transaction_table(f, app, main_area);
            dialog::render_confirmation_dialog(f, "Confirm Delete? (y/n)", main_area);
        }
        AppMode::Summary => {
            summary::render_summary_view(f, app, main_area);
        }
        AppMode::SelectingCategory | AppMode::SelectingSubcategory => {
            transaction_form::render_transaction_form(f, app, main_area);
            dialog::render_selection_popup(f, app, main_area);
        }
        AppMode::CategorySummary => {
            category_summary::render_category_summary_view(f, app, main_area);
        }
        AppMode::Settings => {
            transaction_table::render_transaction_table(f, app, main_area);
            settings::render_settings_form(f, app, main_area);
        }
        AppMode::RecurringSettings => {
            recurring::render_recurring_settings(f, app, main_area);
        }
        AppMode::SelectingRecurrenceFrequency => {
            recurring::render_recurring_settings(f, app, main_area);
            dialog::render_selection_popup(f, app, main_area);
        }
    }

    if app.mode == AppMode::Filtering {
        filter::render_filter_input(f, app, filter_area);
    }

    // Determine year filter based on current mode
    let year_filter = match app.mode {
        AppMode::Summary => {
            app.summary_years.get(app.selected_summary_year_index).copied()
        }
        AppMode::CategorySummary => {
            app.category_summary_years.get(app.category_summary_year_index).copied()
        }
        _ => None, // No year filter for other modes - show all transactions as before
    };
    
    summary::render_summary_bar(f, app, summary_area, year_filter);

    if let Some(msg) = &app.status_message {
        status::render_status_bar(f, msg, status_area);
    }

    help::render_help_bar(f, app, help_area);

    if app.mode == AppMode::Filtering {
        let cursor_x = app.input_field_content[..app.input_field_cursor]
            .chars()
            .count() as u16;
        f.set_cursor_position(ratatui::layout::Position::new(
            filter_area.x + cursor_x + 1,
            filter_area.y + 1,
        ));
    }
}
