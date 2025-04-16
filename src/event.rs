use crate::app::{App, AppMode};
use crate::model::SortColumn;
use crate::ui::ui;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
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
            if let Event::Key(key) = event::read()? {
                if app.mode != AppMode::ConfirmDelete
                    && app.mode != AppMode::SelectingCategory
                    && app.mode != AppMode::SelectingSubcategory
                {
                    app.status_message = None;
                }
                update(app, key);
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
                KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                KeyCode::Char('a') => app.start_adding(),
                KeyCode::Char('d') => app.prepare_for_delete(),
                KeyCode::Char('e') => app.start_editing(),
                KeyCode::Char('f') => app.start_filtering(),
                KeyCode::Char('s') => app.enter_summary_mode(),
                KeyCode::Char('c') => app.enter_category_summary_mode(),
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
                KeyCode::Char(c) => match app.current_add_edit_field {
                    0 if c == '+' || c == '=' => app.increment_date(),
                    0 if c == '-' => app.decrement_date(),
                    // Prevent text input in Type, Category, Subcategory fields
                    field if ![3, 4, 5].contains(&field) => app.insert_char_add_edit(c),
                    _ => {} // Ignore char input for fields 3, 4, 5
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
        AppMode::Filtering => match key_code {
            KeyCode::Esc | KeyCode::Enter => app.exit_filtering(),
            KeyCode::Char(c) => {
                app.insert_char_at_cursor(c);
                app.apply_filter();
            }
            KeyCode::Backspace => {
                app.delete_char_before_cursor();
                app.apply_filter();
            }
            KeyCode::Delete => {
                app.delete_char_after_cursor();
                app.apply_filter();
            }
            KeyCode::Left => app.move_cursor_left(),
            KeyCode::Right => app.move_cursor_right(),
            _ => {}
        },
        AppMode::Summary => {
            match key_code {
                KeyCode::Char('q') | KeyCode::Esc => app.exit_summary_mode(),
                KeyCode::Down | KeyCode::Char('j') => app.next_item(), // Navigate months
                KeyCode::Up | KeyCode::Char('k') => app.previous_item(), // Navigate months
                KeyCode::Char(']') | KeyCode::PageDown | KeyCode::Right => app.next_summary_year(),
                KeyCode::Char('[') | KeyCode::PageUp | KeyCode::Left => app.previous_summary_year(),
                _ => {}
            }
        }
        AppMode::SelectingCategory | AppMode::SelectingSubcategory => match key_code {
            KeyCode::Esc => app.cancel_selection(),
            KeyCode::Enter => app.confirm_selection(),
            KeyCode::Down | KeyCode::Char('j') => app.select_next_list_item(),
            KeyCode::Up | KeyCode::Char('k') => app.select_previous_list_item(),
            KeyCode::Tab => {}
            _ => {}
        },
        AppMode::CategorySummary => match key_code {
            KeyCode::Char('q') | KeyCode::Esc => app.exit_category_summary_mode(),
            KeyCode::Down | KeyCode::Char('j') => app.next_category_summary_item(),
            KeyCode::Up | KeyCode::Char('k') => app.previous_category_summary_item(),
            KeyCode::Char(']') | KeyCode::PageDown | KeyCode::Right => {
                app.next_category_summary_year()
            }
            KeyCode::Char('[') | KeyCode::PageUp | KeyCode::Left => {
                app.previous_category_summary_year()
            }
            _ => {}
        },
    }
}
