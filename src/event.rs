use crate::app::{App, AppMode, CategorySummaryItem};
use crate::model::SortColumn;
use crate::ui::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::Backend;
use ratatui::Terminal;
use std::result::Result as StdResult;
use std::time::Duration;

pub(crate) fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> StdResult<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                // Handle Paste Event
                Event::Paste(text) => {
                    if app.mode == AppMode::Settings {
                        app.input_field_content = text;
                        app.input_field_cursor = app.input_field_content.len();
                    }
                    // Potentially handle paste in other modes later if needed
                }
                Event::Key(key) => {
                    match key.kind {
                        KeyEventKind::Press => {
                            if key.modifiers == KeyModifiers::NONE
                                || (app.mode == AppMode::Settings && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('d') | KeyCode::Char('u')))
                                // Let Shift+Char pass through for typing capitals/symbols in settings path
                                || (app.mode == AppMode::Settings && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_)))
                                // Allow Shift+Char in Adding and Editing modes
                                || ((app.mode == AppMode::Adding || app.mode == AppMode::Editing) && key.modifiers == KeyModifiers::SHIFT && matches!(key.code, KeyCode::Char(_)))
                                // Allow Ctrl+F/R in simple Filtering mode
                                || (app.mode == AppMode::Filtering && key.modifiers == KeyModifiers::CONTROL && matches!(key.code, KeyCode::Char('f') | KeyCode::Char('r')))
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
                        _ => {} // Ignore other key event kinds (Release, Repeat)
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
    let key_code = key_event.code;

    match app.mode {
        AppMode::Normal => {
            match key_code {
                KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                KeyCode::Down => app.next_item(),
                KeyCode::Up => app.previous_item(),
                KeyCode::Char('a') => app.start_adding(),
                KeyCode::Char('d') => app.prepare_for_delete(),
                KeyCode::Char('e') => app.start_editing(),
                KeyCode::Char('f') => app.start_filtering(),
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
        AppMode::Adding | AppMode::Editing => {
            match key_code {
                KeyCode::Esc => {
                    if app.mode == AppMode::Adding {
                        app.exit_adding();
                    } else {
                        app.exit_editing();
                    }
                }
                KeyCode::Tab => {
                    app.next_add_edit_field();
                }
                KeyCode::BackTab => {
                    app.previous_add_edit_field();
                }
                KeyCode::Enter => {
                    // Toggle Type, trigger selection popups, or save transaction
                    match app.current_add_edit_field {
                        3 => app.toggle_transaction_type(), // Enter on Type field toggles it
                        4 => app.start_category_selection(), // Enter on Category field
                        5 => app.start_subcategory_selection(), // Enter on Subcategory field
                        _ => {
                            // Enter on any other field: Save
                            if app.mode == AppMode::Adding {
                                app.add_transaction();
                            } else {
                                app.update_transaction();
                            }
                        }
                    }
                }
                KeyCode::Up => app.previous_add_edit_field(),
                KeyCode::Down => app.next_add_edit_field(),
                KeyCode::Left => match app.current_add_edit_field {
                    0 => app.decrement_date(),
                    3 => app.toggle_transaction_type(),
                    _ => {}
                },
                KeyCode::Right => match app.current_add_edit_field {
                    0 => app.increment_date(),
                    3 => app.toggle_transaction_type(),
                    _ => {}
                },
                KeyCode::Char(c) => match app.current_add_edit_field {
                    0 if c == '+' || c == '=' => app.increment_date(),
                    0 if c == '-' => app.decrement_date(),
                    // Only allow digits for the date field (field 0)
                    0 if c.is_ascii_digit() => app.insert_char_add_edit(c),
                    // Allow any character for other non-special fields (1, 2)
                    field if ![0, 3, 4, 5].contains(&field) => app.insert_char_add_edit(c),
                    _ => {} // Ignore char input for fields 0 (non-digit), 3, 4, 5
                },
                KeyCode::Backspace => {
                    if ![3, 4, 5].contains(&app.current_add_edit_field) {
                        app.delete_char_add_edit();
                    }
                }
                _ => {}
            }
        }
        AppMode::ConfirmDelete => match key_code {
            KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete(),
            _ => {}
        },
        AppMode::Filtering => match (key_event.modifiers, key_code) {
            (KeyModifiers::CONTROL, KeyCode::Char('f')) => app.start_advanced_filtering(),
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                app.clear_input_field();
                app.apply_filter();
            }
            (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Enter) => {
                app.exit_filtering()
            }
            (KeyModifiers::NONE, KeyCode::Char(c)) => {
                app.insert_char_at_cursor(c);
                app.apply_filter();
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                app.delete_char_before_cursor();
                app.apply_filter();
            }
            (KeyModifiers::NONE, KeyCode::Delete) => {
                app.delete_char_after_cursor();
                app.apply_filter();
            }
            (KeyModifiers::NONE, KeyCode::Left) => app.move_cursor_left(),
            (KeyModifiers::NONE, KeyCode::Right) => app.move_cursor_right(),
            _ => {}
        },
        AppMode::AdvancedFiltering => match (key_event.modifiers, key_code) {
            (KeyModifiers::CONTROL, KeyCode::Char('r')) => app.clear_advanced_filter_fields(),
            (KeyModifiers::NONE, KeyCode::Esc) => app.cancel_advanced_filtering(),
            (KeyModifiers::NONE, KeyCode::Enter) => match app.current_advanced_filter_field {
                3 => app.start_advanced_category_selection(),
                4 => app.start_advanced_subcategory_selection(),
                5 => app.toggle_advanced_transaction_type(),
                _ => app.finish_advanced_filtering(),
            },
            (KeyModifiers::NONE, KeyCode::Tab) => app.next_advanced_filter_field(),
            (KeyModifiers::NONE, KeyCode::BackTab) => app.previous_advanced_filter_field(),
            (KeyModifiers::NONE, KeyCode::Up) => app.previous_advanced_filter_field(),
            (KeyModifiers::NONE, KeyCode::Down) => app.next_advanced_filter_field(),
            (KeyModifiers::NONE, KeyCode::Left) => match app.current_advanced_filter_field {
                0 | 1 => app.decrement_advanced_date(),
                5 => app.toggle_advanced_transaction_type(),
                _ => {}
            },
            (KeyModifiers::NONE, KeyCode::Right) => match app.current_advanced_filter_field {
                0 | 1 => app.increment_advanced_date(),
                5 => app.toggle_advanced_transaction_type(),
                _ => {}
            },
            (KeyModifiers::NONE, KeyCode::Char(c)) => app.insert_char_advanced_filter(c),
            (KeyModifiers::NONE, KeyCode::Backspace) => app.delete_char_advanced_filter(),
            (KeyModifiers::NONE, KeyCode::Delete) => app.delete_char_advanced_filter(),
            _ => {}
        },
        AppMode::Summary => match key_code {
            KeyCode::Char('q') | KeyCode::Esc => app.exit_summary_mode(),
            KeyCode::Down => app.next_item(),
            KeyCode::Up => app.previous_item(),
            KeyCode::Char(']') | KeyCode::PageDown | KeyCode::Right => app.next_summary_year(),
            KeyCode::Char('[') | KeyCode::PageUp | KeyCode::Left => app.previous_summary_year(),
            _ => {}
        },
        AppMode::SelectingCategory | AppMode::SelectingSubcategory => match key_code {
            KeyCode::Esc => app.cancel_selection(),
            KeyCode::Enter => app.confirm_selection(),
            KeyCode::Down => app.select_next_list_item(),
            KeyCode::Up => app.select_previous_list_item(),
            KeyCode::Tab => {}
            _ => {}
        },
        AppMode::SelectingFilterCategory | AppMode::SelectingFilterSubcategory => match key_code {
            KeyCode::Esc => app.cancel_advanced_selection(),
            KeyCode::Enter => app.confirm_advanced_selection(),
            KeyCode::Down => app.select_next_list_item(),
            KeyCode::Up => app.select_previous_list_item(),
            KeyCode::Tab => {}
            _ => {}
        },
        AppMode::CategorySummary => match key_code {
            KeyCode::Char('q') | KeyCode::Esc => app.exit_category_summary_mode(),
            KeyCode::Down => app.next_category_summary_item(),
            KeyCode::Up => app.previous_category_summary_item(),
            KeyCode::Char(']') | KeyCode::PageDown | KeyCode::Right => {
                app.next_category_summary_year()
            }
            KeyCode::Char('[') | KeyCode::PageUp | KeyCode::Left => {
                app.previous_category_summary_year()
            }
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
        },
        AppMode::Settings => match (key_code, key_event.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => app.exit_settings_mode(),
            (KeyCode::Enter, KeyModifiers::NONE) => app.save_settings(),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => app.reset_settings_path_to_default(),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => app.clear_input_field(),
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                app.insert_char_at_cursor(c);
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => app.delete_char_before_cursor(),
            (KeyCode::Delete, KeyModifiers::NONE) => app.delete_char_after_cursor(),
            (KeyCode::Left, KeyModifiers::NONE) => app.move_cursor_left(),
            (KeyCode::Right, KeyModifiers::NONE) => app.move_cursor_right(),
            _ => {}
        },
    }
}
