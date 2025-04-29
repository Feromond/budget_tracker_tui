use crate::app::state::App;
use crate::model::{SortColumn, SortOrder, TransactionType, DATE_FORMAT};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render_transaction_table(f: &mut Frame, app: &mut App, area: Rect) {
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
            Cell::from(tx.description.as_str()),
            Cell::from(tx.category.as_str()),
            Cell::from(tx.subcategory.as_str()),
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