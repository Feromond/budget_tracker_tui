use crate::app::{App, AppMode};
use crate::model::{MonthlySummary, SortColumn, SortOrder, TransactionType, DATE_FORMAT};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::collections::HashMap;

pub(crate) fn ui(f: &mut Frame, app: &mut App) {
    let filter_bar_height = if app.mode == AppMode::Filtering { 3 } else { 0 };
    let status_bar_height = if app.status_message.is_some() { 3 } else { 0 };
    let summary_bar_height = 3;
    let help_bar_height = 3;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(filter_bar_height),
            Constraint::Length(summary_bar_height),
            Constraint::Length(status_bar_height),
            Constraint::Length(help_bar_height),
        ])
        .split(f.area()); // Use f.area() instead of f.size()

    let main_area = main_chunks[0];
    let filter_area = main_chunks[1];
    let summary_area = main_chunks[2];
    let status_area = main_chunks[3];
    let help_area = main_chunks[4];

    match app.mode {
        AppMode::Normal | AppMode::Filtering => {
            render_transaction_table(f, app, main_area);
        }
        AppMode::Adding | AppMode::Editing => {
            render_transaction_form(f, app, main_area);
        }
        AppMode::ConfirmDelete => {
            render_transaction_table(f, app, main_area);
            render_confirmation_dialog(f, "Confirm Delete? (y/n)", main_area);
        }
        AppMode::Summary => {
            render_summary_view(f, app, main_area);
        }
        AppMode::SelectingCategory | AppMode::SelectingSubcategory => {
            // Render form underneath the popup
            render_transaction_form(f, app, main_area);
            render_selection_popup(f, app, main_area);
        }
        AppMode::CategorySummary => {
            render_category_summary_view(f, app, main_area);
        }
        AppMode::Settings => {
            render_transaction_table(f, app, main_area);
            render_settings_popup(f, app, main_area);
        }
    }

    if app.mode == AppMode::Filtering {
        render_filter_input(f, app, filter_area);
    }

    render_summary_bar(f, app, summary_area);

    if let Some(msg) = &app.status_message {
        render_status_bar(f, msg, status_area);
    }

    render_help_bar(f, app, help_area);

    match app.mode {
        AppMode::Filtering => {
            let cursor_x = app.input_field_content[..app.input_field_cursor]
                .chars()
                .count() as u16;
            f.set_cursor_position(Position::new(filter_area.x + cursor_x + 1, filter_area.y + 1));
        }
        _ => {}
    }
}

// Renders the main transaction table
fn render_transaction_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_titles = [
        "Date",
        "Description",
        "Category",
        "Subcategory",
        "Type",
        "Amount",
    ];
    let sort_columns = [
        SortColumn::Date,
        SortColumn::Description,
        SortColumn::Category,
        SortColumn::Subcategory,
        SortColumn::Type,
        SortColumn::Amount,
    ];

    let header_cells = header_titles
        .iter()
        .zip(sort_columns.iter())
        .map(|(title, column)| {
            let style = Style::default().fg(Color::Cyan).bold();
            let symbol = if app.sort_by == *column {
                match app.sort_order {
                    SortOrder::Ascending => " ▲",
                    SortOrder::Descending => " ▼",
                }
            } else {
                ""
            };
            Cell::from(format!("{}{}", title, symbol)).style(style)
        });

    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1)
        .bottom_margin(1);

    let rows = app.filtered_indices.iter().map(|&original_index| {
        if original_index >= app.transactions.len() {
            return Row::new(vec![Cell::from("Error: Invalid Index").fg(Color::Red)])
                .height(1)
                .bottom_margin(0);
        }
        let tx = &app.transactions[original_index];
        let amount_style = match tx.transaction_type {
            TransactionType::Income => Style::default().fg(Color::LightGreen),
            TransactionType::Expense => Style::default().fg(Color::LightRed),
        };
        let cells = vec![
            Cell::from(tx.date.format(DATE_FORMAT).to_string()),
            Cell::from(tx.description.clone()),
            Cell::from(tx.category.clone()),
            Cell::from(tx.subcategory.clone()),
            Cell::from(format!("{:?}", tx.transaction_type)),
            Cell::from(format!("{:.2}", tx.amount)).style(amount_style),
        ];
        Row::new(cells).height(1).bottom_margin(0)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(10),
            Constraint::Percentage(35),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(7),
            Constraint::Percentage(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("Transactions"))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(" > ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

// Renders the Add/Edit transaction form
fn render_transaction_form(f: &mut Frame, app: &App, area: Rect) {
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Date
            Constraint::Length(3), // Description
            Constraint::Length(3), // Amount
            Constraint::Length(3), // Type (Toggle)
            Constraint::Length(3), // Category (Select)
            Constraint::Length(3), // Subcategory (Select)
            Constraint::Min(0),
        ])
        .split(area);

    // Field titles and hints
    let field_definitions = [
        ("Date (YYYY-MM-DD)", "(◀/▶ or +/- adjust, Digits to enter)"),
        ("Description", ""),
        ("Amount", ""),
        ("Type", "(◀/▶ or Enter to toggle)"),
        ("Category", "(Enter to select)"),
        ("Subcategory", "(Enter to select)"),
    ];

    let input_widgets: Vec<_> = app
        .add_edit_fields
        .iter()
        .zip(field_definitions.iter())
        .enumerate()
        .map(|(i, (text, (base_title, hint)))| {
            let is_focused = app.current_add_edit_field == i;
            let title = format!("{} {}", base_title, hint).trim_end().to_string(); // Combine title and hint

            let content = if i == 3 {
                // Show as < Value >
                Span::styled(
                    format!(" < {} > ", text),
                    Style::default().fg(Color::White).bold(),
                )
            } else {
                Span::raw(text.as_str())
            };

            let input = Paragraph::new(content)
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .border_style(if is_focused {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        }),
                );
            input
        })
        .collect();

    for i in 0..input_widgets.len() {
        if i < form_chunks.len() {
            f.render_widget(input_widgets[i].clone(), form_chunks[i]);
        }
    }

    let form_title_text = if app.mode == AppMode::Editing {
        "Edit Transaction"
    } else {
        "Add New Transaction"
    };
    // Removed redundant key instructions from title - they are in the Help bar now
    let form_block = Block::default()
        .title(form_title_text)
        .borders(Borders::ALL);
    f.render_widget(form_block, area);

    // Set cursor only for editable text fields (not Type, Category, or Subcategory)
    if ![3, 4, 5].contains(&app.current_add_edit_field) {
        if let Some(focused_area) = form_chunks.get(app.current_add_edit_field) {
            let text_len = app.add_edit_fields[app.current_add_edit_field].len() as u16;
            f.set_cursor_position(Position::new(
                focused_area.x + text_len + 1,
                focused_area.y + 1,
            ))
        }
    }
}

fn render_summary_bar(f: &mut Frame, app: &App, area: Rect) {
    let (total_income, total_expense) = app
        .filtered_indices
        .iter()
        .filter_map(|&idx| app.transactions.get(idx))
        .fold((0.0, 0.0), |(inc, exp), tx| match tx.transaction_type {
            TransactionType::Income => (inc + tx.amount, exp),
            TransactionType::Expense => (inc, exp + tx.amount),
        });
    let net_balance = total_income - total_expense;

    let income_span = Span::styled(
        format!("Income: {:.2}", total_income),
        Style::default().fg(Color::LightGreen),
    );
    let expense_span = Span::styled(
        format!("Expenses: {:.2}", total_expense),
        Style::default().fg(Color::LightRed),
    );
    let net_style = if net_balance >= 0.0 {
        Style::default().fg(Color::LightGreen)
    } else {
        Style::default().fg(Color::LightRed)
    };
    let net_str = if net_balance >= 0.0 {
        format!("+{:.2}", net_balance)
    } else {
        format!("{:.2}", net_balance)
    };
    let net_span = Span::styled(format!("Net: {}", net_str), net_style);

    let summary_line = Line::from(vec![
        income_span,
        Span::raw(" | "),
        expense_span,
        Span::raw(" | "),
        net_span,
    ])
    .alignment(Alignment::Center);

    let summary_paragraph = Paragraph::new(summary_line).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Summary (Filtered)"),
    );

    f.render_widget(summary_paragraph, area);
}

// Renders the help bar based on the current mode
fn render_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let help_spans = match app.mode {
        AppMode::Normal => vec![
            Span::raw("↑↓ Nav | "),
            Span::styled("a", Style::default().fg(Color::LightGreen)),
            Span::raw(" Add | "),
            Span::styled("e", Style::default().fg(Color::LightYellow)),
            Span::raw(" Edit | "),
            Span::styled("d", Style::default().fg(Color::LightRed)),
            Span::raw(" Del | "),
            Span::styled("f", Style::default().fg(Color::Cyan)),
            Span::raw(" Filter | "),
            Span::raw("s Summ | c CatSum | "),
            Span::styled("1-6", Style::default().fg(Color::LightBlue)),
            Span::raw(" Sort | "),
            Span::styled("q/Esc", Style::default().fg(Color::Magenta)),
            Span::raw(" Quit"),
        ],
        AppMode::Adding | AppMode::Editing => vec![
            Span::raw("Tab/↑↓ Nav | "),
            Span::raw("←→ Toggle | "),
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(" Save/Select | "),
            Span::styled("Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Cancel"),
        ],
        AppMode::ConfirmDelete => vec![
            Span::styled("y", Style::default().fg(Color::LightGreen)),
            Span::raw(": Confirm | "),
            Span::styled("n/Esc", Style::default().fg(Color::LightRed)),
            Span::raw(": Cancel"),
        ],
        AppMode::Filtering => vec![
            Span::raw("Type Filter | "),
            Span::raw("← → Cursor | "),
            Span::raw("Bksp/Del Edit | "),
            Span::styled("Enter/Esc", Style::default().fg(Color::LightGreen)),
            Span::raw(" Apply/Exit"),
        ],
        AppMode::Summary => vec![
            Span::raw("↑↓ Nav | "),
            Span::raw("←→/[] Year | "),
            Span::styled("q/Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Back"),
        ],
        AppMode::SelectingCategory | AppMode::SelectingSubcategory => vec![
            Span::raw("↑↓ Nav | "),
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(": Confirm | "),
            Span::styled("Esc", Style::default().fg(Color::LightRed)),
            Span::raw(": Cancel"),
        ],
        AppMode::CategorySummary => vec![
            Span::raw("↑↓ Nav | "),
            Span::raw("←→/[] Year | "),
            Span::styled("q/Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Back"),
        ],
        AppMode::Settings => vec![
            Span::styled("Enter", Style::default().fg(Color::LightGreen)),
            Span::raw(" Save Path | "),
            Span::styled("q/Esc", Style::default().fg(Color::LightRed)),
            Span::raw(" Back"),
        ],
    };

    let help_paragraph = Paragraph::new(Line::from(help_spans))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_paragraph, area);
}

// Renders the status bar (for errors/info)
fn render_status_bar(f: &mut Frame, message: &str, area: Rect) {
    let style = if message.starts_with("Error:") {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default().fg(Color::LightYellow)
    };
    let status_text = Paragraph::new(message)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .alignment(Alignment::Center);
    f.render_widget(status_text, area);
}

// Renders a confirmation dialog popup
fn render_confirmation_dialog(f: &mut Frame, message: &str, area: Rect) {
    let dialog_area = centered_rect(60, 20, area);

    let dialog_block = Block::default()
        .title("Confirmation")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let dialog_text = Paragraph::new(message)
        .block(dialog_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, dialog_area); // Clear the area behind the dialog
    f.render_widget(dialog_text, dialog_area);
}

// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Renders the filter input box
fn render_filter_input(f: &mut Frame, app: &App, area: Rect) {
    let input = Paragraph::new(app.input_field_content.as_str())
        .style(Style::default().fg(Color::LightYellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filter (Description)"),
        );
    f.render_widget(input, area);
    // Cursor setting is handled in the main `ui` function
}

// Renders the monthly summary view (table + chart)
fn render_summary_view(f: &mut Frame, app: &mut App, area: Rect) {
    let summary_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let table_area = summary_chunks[0];
    let chart_area = summary_chunks[1];

    let current_year = app
        .summary_years
        .get(app.selected_summary_year_index)
        .copied();
    let year_str = current_year.map_or_else(|| "N/A".to_string(), |y| y.to_string());
    let year_progress = format!(
        "({}/{})",
        app.selected_summary_year_index + 1,
        app.summary_years.len().max(1)
    );

    // Table
    let table_title = format!("Monthly Summary - {} {}", year_str, year_progress);
    let header_titles = ["Month", "Income", "Expense", "Net"];
    let header_cells = header_titles
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).bold()));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1)
        .bottom_margin(1);

    let mut table_rows = Vec::new();
    let mut chart_data_styled: Vec<Bar> = Vec::with_capacity(12);
    let mut max_abs_chart_value = 0i64;

    if let Some(year) = current_year {
        for month in 1..=12 {
            let summary = app
                .monthly_summaries
                .get(&(year, month))
                .cloned()
                .unwrap_or_default();
            let net = summary.income - summary.expense;
            let net_i64 = net.round() as i64;
            let net_style = if net >= 0.0 {
                Style::default().fg(Color::LightGreen)
            } else {
                Style::default().fg(Color::LightRed)
            };
            let month_name = month_to_short_str(month);
            let net_str = if net >= 0.0 {
                format!("+{:.2}", net)
            } else {
                format!("{:.2}", net)
            };

            table_rows.push(
                Row::new(vec![
                    Cell::from(month_name),
                    Cell::from(format!("{:.2}", summary.income))
                        .style(Style::default().fg(Color::LightGreen)),
                    Cell::from(format!("{:.2}", summary.expense))
                        .style(Style::default().fg(Color::LightRed)),
                    Cell::from(net_str).style(net_style),
                ])
                .height(1)
                .bottom_margin(0),
            );

            chart_data_styled.push(
                Bar::default()
                    .label(month_name.into())
                    .value(net_i64.unsigned_abs())
                    .style(net_style),
            );
            max_abs_chart_value = max_abs_chart_value.max(net_i64.abs());
        }
    } else {
        // Add a row indicating no data if no year is selected
        table_rows.push(Row::new(vec![Cell::from("No Data")]).height(1)); // Simple row with one cell
        chart_data_styled.push(Bar::default().label("N/A".into()).value(0));
    }

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(table_title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(" > ");

    // Chart
    let chart_title = format!("Net Balance Chart - {}", year_str);
    let bar_group = BarGroup::default().bars(&chart_data_styled);

    let num_bars = 12u16;
    let usable_width = chart_area.width.saturating_sub(2);
    let width_per_bar_and_gap = (usable_width / num_bars.max(1)).max(1);
    let bar_gap = if width_per_bar_and_gap > 1 {
        1u16
    } else {
        0u16
    };
    let bar_width = width_per_bar_and_gap.saturating_sub(bar_gap).max(1);

    let bar_chart = BarChart::default()
        .block(Block::default().title(chart_title).borders(Borders::ALL))
        .data(bar_group)
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .group_gap(0)
        .label_style(Style::default().fg(Color::White))
        .max(max_abs_chart_value.max(10) as u64);

    f.render_stateful_widget(table, table_area, &mut app.table_state);
    f.render_widget(bar_chart, chart_area);
}

// Renders the category/subcategory selection popup list
fn render_selection_popup(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_title = match app.mode {
        AppMode::SelectingCategory => "Select Category (Enter/Esc)",
        AppMode::SelectingSubcategory => "Select Subcategory (Enter/Esc)",
        _ => "Select Option",
    };

    let items: Vec<ListItem> = app
        .current_selection_list
        .iter()
        .map(|i| ListItem::new(i.as_str()).style(Style::default().fg(Color::White)))
        .collect();

    let list = List::new(items)
        .block(Block::default().title(popup_title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let popup_area = centered_rect(60, 50, area);

    f.render_widget(Clear, popup_area);
    f.render_stateful_widget(list, popup_area, &mut app.selection_list_state);
}

// Renders the category summary view (table + chart)
fn render_category_summary_view(f: &mut Frame, app: &mut App, area: Rect) {
    let summary_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let table_area = summary_chunks[0];
    let chart_area = summary_chunks[1];

    // Table Setup
    let header_titles = [
        "Month",
        "Category",
        "Subcategory",
        "Income",
        "Expense",
        "Net",
    ];
    let header_cells = header_titles
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Cyan).bold()));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1)
        .bottom_margin(1);

    // Data for Table
    let current_year = app
        .category_summary_years
        .get(app.category_summary_year_index)
        .copied();
    let year_str = current_year.map_or_else(|| "N/A".to_string(), |y| y.to_string());
    let month_category_subcategory_list = app.get_current_category_summary_list();

    let rows =
        month_category_subcategory_list
            .iter()
            .map(|(month, category_name, subcategory_name)| {
                let summary = current_year
                    .and_then(|year| {
                        app.category_summaries
                            .get(&(year, *month))
                            .and_then(|month_map| {
                                month_map.get(&(category_name.clone(), subcategory_name.clone()))
                            })
                    })
                    .cloned()
                    .unwrap_or_default();

                let net = summary.income - summary.expense;
                let net_style = if net >= 0.0 {
                    Style::default().fg(Color::LightGreen)
                } else {
                    Style::default().fg(Color::LightRed)
                };
                let month_name = month_to_short_str(*month);
                let subcat_display = if subcategory_name.is_empty() {
                    "-".to_string()
                } else {
                    subcategory_name.clone()
                };
                let net_str = if net >= 0.0 {
                    format!("+{:.2}", net)
                } else {
                    format!("{:.2}", net)
                };

                Row::new(vec![
                    Cell::from(month_name),
                    Cell::from(category_name.as_str()),
                    Cell::from(subcat_display),
                    Cell::from(format!("{:.2}", summary.income))
                        .style(Style::default().fg(Color::LightGreen)),
                    Cell::from(format!("{:.2}", summary.expense))
                        .style(Style::default().fg(Color::LightRed)),
                    Cell::from(net_str).style(net_style),
                ])
                .height(1)
                .bottom_margin(0)
            });

    let table_title = format!(
        "Category/Subcategory Summary - {} ({}/{})",
        year_str,
        app.category_summary_year_index + 1,
        app.category_summary_years.len().max(1)
    );
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(11),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(table_title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(" > ");

    f.render_stateful_widget(table, table_area, &mut app.category_summary_table_state);

    // Chart Setup
    let mut chart_title = "Category Net Balance".to_string();
    let mut bars: Vec<Bar> = Vec::new();
    let mut max_abs_chart_value: u64 = 10;

    if let Some(selected_index) = app.category_summary_table_state.selected() {
        if let Some((selected_month, _, _)) = month_category_subcategory_list.get(selected_index) {
            if let Some(year) = current_year {
                let mut category_totals: HashMap<String, MonthlySummary> = HashMap::new();
                if let Some(month_map) = app.category_summaries.get(&(year, *selected_month)) {
                    for ((category, _), summary) in month_map.iter() {
                        let cat_summary = category_totals.entry(category.clone()).or_default();
                        cat_summary.income += summary.income;
                        cat_summary.expense += summary.expense;
                    }
                }

                let mut category_data_for_chart: Vec<(String, f64)> = category_totals
                    .into_iter()
                    .map(|(cat, summary)| {
                        let net_balance = summary.income - summary.expense;
                        (cat, net_balance)
                    })
                    .collect();

                category_data_for_chart.sort_by(|(c1, _), (c2, _)| c1.cmp(c2));

                let mut current_max: i64 = 0;
                bars = category_data_for_chart
                    .iter()
                    .map(|(cat, net)| {
                        let net_val = *net;
                        let net_i64 = net_val.round() as i64;
                        let net_style = if net_val >= 0.0 {
                            Style::default().fg(Color::LightGreen)
                        } else {
                            Style::default().fg(Color::LightRed)
                        };
                        current_max = current_max.max(net_i64.abs());
                        Bar::default()
                            .label(cat.chars().take(15).collect::<String>().into()) // Truncate label (increased length)
                            .value(net_i64.unsigned_abs())
                            .style(net_style)
                    })
                    .collect();
                max_abs_chart_value = (current_max as u64).max(10); // Ensure max is at least 10
                chart_title = format!(
                    "Category Net Balance - {} {}",
                    month_to_short_str(*selected_month),
                    year_str
                );
            }
        }
    }

    if bars.is_empty() {
        bars.push(Bar::default().label("No Data".into()).value(0));
        chart_title = format!("Category Net Balance - {} (Select Row)", year_str);
    }

    let num_bars = bars.len() as u16;
    let usable_width = chart_area.width.saturating_sub(2);
    let width_per_bar_and_gap = (usable_width / num_bars.max(1)).max(1);
    let bar_gap = if width_per_bar_and_gap > 1 {
        1u16
    } else {
        0u16
    };
    let bar_width = width_per_bar_and_gap.saturating_sub(bar_gap).max(1);

    let bar_chart = BarChart::default()
        .block(Block::default().title(chart_title).borders(Borders::ALL))
        .data(BarGroup::default().bars(&bars))
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .group_gap(0)
        .label_style(Style::default().fg(Color::White))
        .max(max_abs_chart_value); // Use calculated max

    f.render_widget(bar_chart, chart_area);
}

// Helper for month abbreviation
fn month_to_short_str(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "?",
    }
}

// Renders the settings popup
fn render_settings_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(75, 20, area);

    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(3), // Input field
            Constraint::Length(1), // Instructions
            Constraint::Min(0),
        ])
        .split(popup_area);

    let title = Paragraph::new("Settings: Data File Path")
        .alignment(Alignment::Center)
        .style(Style::default().bold());

    let input_chunk = popup_chunks[1];

    let input = Paragraph::new(app.input_field_content.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Enter Path")
                .border_style(Style::default().fg(Color::Yellow)),
        );

    let instructions = Paragraph::new("Esc: Cancel, Enter: Save, r: Reset to Default")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    // Render the popup background
    let clear_area = Clear;
    f.render_widget(clear_area, popup_area);
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title("Settings")
        .style(Style::default().bg(Color::Black));
    f.render_widget(popup_block, popup_area);

    // Render widgets inside the popup area
    f.render_widget(title, popup_chunks[0]);
    f.render_widget(input, input_chunk);
    f.render_widget(instructions, popup_chunks[2]);

    let cursor_x = app.input_field_content[..app.input_field_cursor]
        .chars()
        .count() as u16;
    f.set_cursor_position(Position::new(input_chunk.x + cursor_x + 1, input_chunk.y + 1));
}
