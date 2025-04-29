use crate::app::state::App;
use crate::ui::helpers::month_to_short_str;
use ratatui::prelude::*;
use ratatui::widgets::*;

fn cell_income(amount: f64) -> Cell<'static> {
    Cell::from(format!("{:.2}", amount)).style(Style::default().fg(Color::LightGreen))
}
fn cell_expense(amount: f64) -> Cell<'static> {
    Cell::from(format!("{:.2}", amount)).style(Style::default().fg(Color::LightRed))
}
fn cell_net(net: f64) -> Cell<'static> {
    let s = if net >= 0.0 {
        format!("+{:.2}", net)
    } else {
        format!("{:.2}", net)
    };
    let style = if net >= 0.0 {
        Style::default().fg(Color::LightGreen)
    } else {
        Style::default().fg(Color::LightRed)
    };
    Cell::from(s).style(style)
}

pub fn render_summary_view(f: &mut Frame, app: &mut App, area: Rect) {
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
            let month_name = month_to_short_str(month);
            let inc_cell = cell_income(summary.income);
            let exp_cell = cell_expense(summary.expense);
            let net_cell = cell_net(summary.income - summary.expense);
            table_rows.push(
                Row::new(vec![Cell::from(month_name), inc_cell, exp_cell, net_cell])
                    .height(1)
                    .bottom_margin(0),
            );

            let net = summary.income - summary.expense;
            let net_i64 = net.round() as i64;
            chart_data_styled.push(
                Bar::default()
                    .label(month_name.into())
                    .value(net_i64.unsigned_abs())
                    .style(if net >= 0.0 {
                        Style::default().fg(Color::LightGreen)
                    } else {
                        Style::default().fg(Color::LightRed)
                    }),
            );
            max_abs_chart_value = max_abs_chart_value.max(net_i64.abs());
        }
    } else {
        // Add a row indicating no data if no year is selected
        table_rows.push(Row::new(vec![Cell::from("N/A")]).height(1));
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

pub fn render_summary_bar(f: &mut Frame, app: &App, area: Rect) {
    let (total_income, total_expense) = app
        .filtered_indices
        .iter()
        .filter_map(|&idx| app.transactions.get(idx))
        .fold((0.0, 0.0), |(inc, exp), tx| match tx.transaction_type {
            crate::model::TransactionType::Income => (inc + tx.amount, exp),
            crate::model::TransactionType::Expense => (inc, exp + tx.amount),
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
