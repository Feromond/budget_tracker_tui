use crate::app::state::{App, AppMode};
use crate::ui::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::Backend;
use ratatui::Terminal;
use std::result::Result as StdResult;
use std::time::Duration;

use super::{
    add_edit_mode, filter_mode, normal_mode, recurring_mode, selection_mode, settings_mode,
    summary_mode,
};

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> StdResult<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                // Handle Paste Event
                Event::Paste(text) => {
                    if app.mode == AppMode::Settings && app.current_settings_field == 0 {
                        let field = &mut app.settings_fields[0];
                        let cursor = app.input_field_cursor;
                        field.insert_str(cursor, &text);
                        app.input_field_cursor += text.chars().count();
                    }
                    // Potentially handle paste in other modes later if needed
                }
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press
                        && (key.modifiers == KeyModifiers::NONE
                                || (app.mode == AppMode::Settings && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('d') | KeyCode::Char('u') | KeyCode::Char('v')))
                                // Let Shift+Char pass through for typing capitals/symbols in settings path
                                || (app.mode == AppMode::Settings && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_)))
                                // Allow Shift+Char in Adding and Editing modes
                                || ((app.mode == AppMode::Adding || app.mode == AppMode::Editing) && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_)))
                                // Allow Shift+Arrow in Adding, Editing, AdvancedFiltering, and RecurringSettings modes for month changes
                                || ((app.mode == AppMode::Adding || app.mode == AppMode::Editing || app.mode == AppMode::AdvancedFiltering || app.mode == AppMode::RecurringSettings)
                                    && key.modifiers == KeyModifiers::SHIFT
                                    && matches!(key.code, KeyCode::Left | KeyCode::Right))
                                // Allow Ctrl+F/R in simple Filtering mode and Ctrl+R in AdvancedFiltering mode
                                || (app.mode == AppMode::Filtering && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('f') | KeyCode::Char('r')))
                                || (app.mode == AppMode::AdvancedFiltering && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('r')))
                                || (app.mode == AppMode::Filtering && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_))))
                    {
                        if app.mode != AppMode::ConfirmDelete
                            && app.mode != AppMode::SelectingCategory
                            && app.mode != AppMode::SelectingSubcategory
                        {
                            app.status_message = None;
                        }
                        update(app, key);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

// Main input handler, dispatching based on mode
fn update(app: &mut App, key_event: KeyEvent) {
    match app.mode {
        AppMode::Normal => normal_mode::handle_normal_mode(app, key_event),
        AppMode::Adding | AppMode::Editing => add_edit_mode::handle_add_edit_mode(app, key_event),
        AppMode::ConfirmDelete => add_edit_mode::handle_confirm_delete(app, key_event),
        AppMode::Filtering | AppMode::AdvancedFiltering => {
            filter_mode::handle_filter_mode(app, key_event)
        }
        AppMode::Summary | AppMode::CategorySummary => {
            summary_mode::handle_summary_mode(app, key_event)
        }
        AppMode::SelectingCategory
        | AppMode::SelectingSubcategory
        | AppMode::SelectingFilterCategory
        | AppMode::SelectingFilterSubcategory
        | AppMode::SelectingRecurrenceFrequency => {
            selection_mode::handle_selection_mode(app, key_event)
        }
        AppMode::Settings => settings_mode::handle_settings_mode(app, key_event),
        AppMode::RecurringSettings => recurring_mode::handle_recurring_mode(app, key_event),
    }
}
