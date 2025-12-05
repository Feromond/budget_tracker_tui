use crate::app::state::{App, AppMode};
use crate::ui::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::Backend;
use ratatui::Terminal;
use std::result::Result as StdResult;
use std::time::Duration;

use super::{
    add_edit_mode, filter_mode, fuzzy_search_mode, help_mode, normal_mode, recurring_mode,
    selection_mode, settings_mode, summary_mode,
};

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> StdResult<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        // Check for status expiry
        if let Some(expiry) = app.status_expiry {
            if std::time::Instant::now() > expiry {
                app.clear_status_message();
            }
        }

        // Check for update in background channel
        if let Ok(Some(version)) = app.update_rx.try_recv() {
            app.update_available_version = Some(version);
            app.show_update_popup = true;
        }

        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                // Handle Paste Event
                Event::Paste(text) => {
                    if app.mode == AppMode::Settings {
                        let idx = app.settings_state.selected_index;
                        if let Some(item) = app.settings_state.items.get_mut(idx) {
                            // Allow paste for Path type
                            if matches!(
                                item.setting_type,
                                crate::app::settings_types::SettingType::Path
                            ) {
                                let cursor = app.settings_state.edit_cursor;
                                if cursor <= item.value.len() {
                                    item.value.insert_str(cursor, &text);
                                    app.settings_state.edit_cursor += text.chars().count();

                                    if item.setting_type
                                        == crate::app::settings_types::SettingType::Path
                                    {
                                        let stripped =
                                            crate::validation::strip_path_quotes(&item.value);
                                        item.value = stripped;
                                        app.settings_state.edit_cursor = item.value.len();
                                    }
                                }
                            }
                        }
                    }
                    // Potentially handle paste in other modes later if needed
                }
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press
                        && (key.modifiers == KeyModifiers::NONE
                                || (app.mode == AppMode::Settings && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('d') | KeyCode::Char('u') | KeyCode::Char('v')))
                                // Let Shift+Char pass through for typing capitals/symbols in settings path
                                || (app.mode == AppMode::Settings && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_)))
                                // Allow Shift+Char in Adding, Editing and FuzzyFinding modes
                                || ((app.mode == AppMode::Adding || app.mode == AppMode::Editing || app.mode == AppMode::FuzzyFinding) && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_)))
                                // Allow Shift+Arrow in Adding, Editing, AdvancedFiltering, and RecurringSettings modes for month changes
                                || ((app.mode == AppMode::Adding || app.mode == AppMode::Editing || app.mode == AppMode::AdvancedFiltering || app.mode == AppMode::RecurringSettings)
                                    && key.modifiers == KeyModifiers::SHIFT
                                    && matches!(key.code, KeyCode::Left | KeyCode::Right))
                                // Allow Ctrl+F/R in simple Filtering mode and Ctrl+R in AdvancedFiltering mode
                                || (app.mode == AppMode::Filtering && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('f') | KeyCode::Char('r')))
                                || (app.mode == AppMode::AdvancedFiltering && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('r')))
                                || ((app.mode == AppMode::Filtering || app.mode == AppMode::AdvancedFiltering) && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_)))
                                // Allow Ctrl+Up/Down for jump navigation in Normal mode
                                || (app.mode == AppMode::Normal && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Up | KeyCode::Down))
                                // Allow Ctrl+H for Help Toggle
                                || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('h')))
                    {
                        if app.mode != AppMode::ConfirmDelete
                            && app.mode != AppMode::SelectingCategory
                            && app.mode != AppMode::SelectingSubcategory
                            && app.mode != AppMode::KeybindingsInfo
                        {
                            app.clear_status_message();
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
    if app.show_update_popup {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Enter | KeyCode::Char('o') => {
                    let _ = crate::app::util::open_url(
                        "https://github.com/Feromond/budget_tracker_tui/releases",
                    );
                    app.show_update_popup = false;
                }
                _ => {
                    app.show_update_popup = false;
                }
            }
        }
        return;
    }

    // Global Toggle for Keybindings Help
    if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('h') {
        if app.mode == AppMode::KeybindingsInfo || app.mode == AppMode::KeybindingDetail {
            let prev = app.previous_mode.unwrap_or(AppMode::Normal);
            app.mode = prev;
            app.previous_mode = None;
        } else {
            app.previous_mode = Some(app.mode);
            app.mode = AppMode::KeybindingsInfo;
            app.help_table_state.select(Some(0));
        }
        return;
    }

    match app.mode {
        AppMode::KeybindingsInfo | AppMode::KeybindingDetail => {
            help_mode::handle_help_mode(app, key_event)
        }
        AppMode::Normal => normal_mode::handle_normal_mode(app, key_event),
        AppMode::Adding | AppMode::Editing => add_edit_mode::handle_add_edit_mode(app, key_event),
        AppMode::ConfirmDelete => add_edit_mode::handle_confirm_delete(app, key_event),
        AppMode::Filtering | AppMode::AdvancedFiltering => {
            filter_mode::handle_filter_mode(app, key_event)
        }
        AppMode::FuzzyFinding => fuzzy_search_mode::handle_fuzzy_search_mode(app, key_event),
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
