use super::helpers::categorize_expenses;
use crate::app::state::App;
use crate::config::load_settings;
use crate::model::{Transaction, TransactionType};
use crate::persistence::load_transactions;
use crate::ui::helpers::month_to_short_str;
use crate::validation::days_in_month;
use chrono::Datelike;
use ratatui::prelude::*;
use ratatui::text::Line;
use ratatui::widgets::{
    Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, GraphType, Paragraph,
};
use std::path::Path;

pub fn render_summary_view(f: &mut Frame, app: &mut App, area: Rect) {
    let summary_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let line_chart_area = summary_chunks[0];
    let bar_chart_area = summary_chunks[1];

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

    // --- Line Chart: Daily Expenses for Selected Month or All Months (Multi Mode) ---
    let mut months: Vec<u32> = vec![];
    if let Some(year) = current_year {
        months = app.sorted_months_for_year(year);
    }
    let color_palette = [
        Color::LightRed,
        Color::LightGreen,
        Color::LightBlue,
        Color::LightYellow,
        Color::LightMagenta,
        Color::LightCyan,
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Yellow,
        Color::Magenta,
        Color::Cyan,
    ];
    let mut all_line_data: Vec<Vec<(f64, f64)>> = vec![];
    let mut legend_labels = vec![];
    let mut max_expense = 0.0;
    let mut datasets = vec![];
    if app.summary_multi_month_mode {
        for &month in &months {
            let year = current_year.unwrap_or(0);
            let num_days = days_in_month(year, month) as usize;
            let mut daily_expenses = vec![0.0; num_days];
            for &idx in &app.filtered_indices {
                let tx = &app.transactions[idx];
                if tx.date.year() == year && tx.date.month() == month {
                    if let crate::model::TransactionType::Expense = tx.transaction_type {
                        let day = tx.date.day() as usize;
                        if day > 0 && day <= num_days {
                            daily_expenses[day - 1] += tx.amount;
                        }
                    }
                }
            }
            let line_data: Vec<(f64, f64)> = if app.summary_cumulative_mode {
                let mut cum = 0.0;
                daily_expenses
                    .iter()
                    .enumerate()
                    .map(|(i, &amt)| {
                        cum += amt;
                        if cum > max_expense {
                            max_expense = cum;
                        }
                        ((i + 1) as f64, cum)
                    })
                    .collect()
            } else {
                daily_expenses
                    .iter()
                    .enumerate()
                    .map(|(i, &amt)| {
                        if amt > max_expense {
                            max_expense = amt;
                        }
                        ((i + 1) as f64, amt)
                    })
                    .collect()
            };
            all_line_data.push(line_data);
        }
        for (i, (&month, line_data)) in months.iter().zip(all_line_data.iter()).enumerate() {
            datasets.push(
                Dataset::default()
                    .name(month_to_short_str(month))
                    .marker(ratatui::symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(color_palette[i % color_palette.len()]))
                    .data(line_data),
            );
            legend_labels.push(Span::styled(
                month_to_short_str(month),
                Style::default().fg(color_palette[i % color_palette.len()]),
            ));
        }
    } else {
        all_line_data.clear();
        if let (Some(year), Some(month)) = (current_year, app.selected_summary_month) {
            let num_days = days_in_month(year, month) as usize;
            let mut daily_expenses = vec![0.0; num_days];
            for &idx in &app.filtered_indices {
                let tx = &app.transactions[idx];
                if tx.date.year() == year && tx.date.month() == month {
                    if let crate::model::TransactionType::Expense = tx.transaction_type {
                        let day = tx.date.day() as usize;
                        if day > 0 && day <= num_days {
                            daily_expenses[day - 1] += tx.amount;
                        }
                    }
                }
            }
            let line_data: Vec<(f64, f64)> = if app.summary_cumulative_mode {
                let mut cum = 0.0;
                daily_expenses
                    .iter()
                    .enumerate()
                    .map(|(i, &amt)| {
                        cum += amt;
                        if cum > max_expense {
                            max_expense = cum;
                        }
                        ((i + 1) as f64, cum)
                    })
                    .collect()
            } else {
                daily_expenses
                    .iter()
                    .enumerate()
                    .map(|(i, &amt)| {
                        if amt > max_expense {
                            max_expense = amt;
                        }
                        ((i + 1) as f64, amt)
                    })
                    .collect()
            };
            all_line_data.push(line_data);
            let data_ref = all_line_data.last().unwrap();
            // Determine color for this month (same as in title)
            let month_color = app
                .selected_summary_month
                .and_then(|m| months.iter().position(|&x| x == m))
                .map(|idx| color_palette[idx % color_palette.len()])
                .unwrap_or(Color::White);
            datasets.push(
                Dataset::default()
                    .name(month_to_short_str(month))
                    .marker(ratatui::symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(month_color))
                    .data(data_ref),
            );
            legend_labels.push(Span::styled(
                month_to_short_str(month),
                Style::default().fg(month_color),
            ));
        }
    }
    let month_index =
        if let (Some(m), false) = (app.selected_summary_month, app.summary_multi_month_mode) {
            months
                .iter()
                .position(|&x| x == m)
                .map(|i| i + 1)
                .unwrap_or(1)
        } else {
            1
        };
    let month_count = months.len().max(1);
    let year_str_owned = year_str.clone();
    let year_progress_owned = year_progress.clone();
    let is_filtered = app.filtered_indices.len() != app.transactions.len();
    let chart_title = if app.summary_multi_month_mode {
        let y = year_str_owned.clone();
        let yp = year_progress_owned.clone();
        let mut title_spans = vec![];
        if is_filtered {
            title_spans.push(Span::styled(
                "(Filtered) ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        title_spans.push(Span::styled(
            "Daily Spending",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        if app.summary_cumulative_mode {
            title_spans.push(Span::styled(
                " (Cumulative)",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        title_spans.push(Span::styled(
            " (All Months)",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            " - ",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            y,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            " ",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            yp,
            Style::default().add_modifier(Modifier::BOLD),
        ));
        Line::from(title_spans)
    } else {
        let month_color = app
            .selected_summary_month
            .and_then(|m| months.iter().position(|&x| x == m))
            .map(|idx| color_palette[idx % color_palette.len()])
            .unwrap_or(Color::White);
        let month_str = app
            .selected_summary_month
            .map(month_to_short_str)
            .unwrap_or("-")
            .to_string();
        let y = year_str_owned.clone();
        let mut title_spans = vec![];
        if is_filtered {
            title_spans.push(Span::styled(
                "(Filtered) ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        title_spans.push(Span::styled(
            "Daily Spending",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        if app.summary_cumulative_mode {
            title_spans.push(Span::styled(
                " (Cumulative)",
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        title_spans.push(Span::styled(
            " - ",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            y,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            " ",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            month_str,
            Style::default()
                .fg(month_color)
                .add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            format!(" ({}/{})", month_index, month_count),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        Line::from(title_spans)
    };
    let year = current_year.unwrap_or(0);
    let max_days = months
        .iter()
        .map(|&m| days_in_month(year, m))
        .max()
        .unwrap_or(31);
    let x_max = max_days as f64;
    // Generate x-axis labels at key points: start, 1/4, 1/2, 3/4, end
    let mut x_labels = vec![];
    let n_ticks = 5;
    for i in 0..n_ticks {
        let pos: f64 = 1.0 + ((x_max - 1.0) * (i as f64) / (n_ticks as f64 - 1.0));
        x_labels.push(Span::raw(format!("{:.0}", pos.round())));
    }
    // Y-axis ticks: 0, 1/4 max, 1/2 max, 3/4 max, max
    let y_max = max_expense.max(10.0);
    let y_labels = vec![
        Span::raw("0"),
        Span::raw(format!("{:.0}", y_max * 0.25)),
        Span::raw(format!("{:.0}", y_max * 0.5)),
        Span::raw(format!("{:.0}", y_max * 0.75)),
        Span::raw(format!("{:.0}", y_max)),
    ];

    // Add cumulative budget line if in cumulative mode and budget is set
    let mut cumulative_budget_line: Option<Vec<(f64, f64)>> = None;
    if app.summary_cumulative_mode && app.target_budget.is_some() && !app.summary_multi_month_mode {
        if let (Some(year), Some(month)) = (current_year, app.selected_summary_month) {
            let num_days = days_in_month(year, month) as usize;
            let budget = app.target_budget.unwrap();
            if num_days > 0 {
                let daily_budget = budget / num_days as f64;
                let budget_line: Vec<(f64, f64)> = (1..=num_days)
                    .map(|d| (d as f64, daily_budget * d as f64))
                    .collect();
                cumulative_budget_line = Some(budget_line);
                legend_labels.push(Span::styled(
                    "CumuBudget",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::DIM),
                ));
            }
        }
    }
    if let Some(ref budget_line) = cumulative_budget_line {
        datasets.push(
            Dataset::default()
                .name("CumuBudget")
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::DIM),
                )
                .data(budget_line),
        );
    }
    let chart = Chart::new(datasets)
        .block(Block::default().title(chart_title).borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("Day")
                .bounds([1.0, x_max])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("Spending")
                .bounds([0.0, y_max])
                .labels(y_labels),
        );
    f.render_widget(chart, line_chart_area);
    // Show legend if needed (single or multi month)
    if (app.summary_multi_month_mode && !legend_labels.is_empty())
        || (!app.summary_multi_month_mode
            && (!legend_labels.is_empty() || cumulative_budget_line.is_some()))
    {
        let legend_line = Line::from(legend_labels);
        let legend_area = Rect {
            x: line_chart_area.x + 2,
            y: line_chart_area.y + 2, // below the title
            width: line_chart_area.width.saturating_sub(4),
            height: 1,
        };
        f.render_widget(legend_line, legend_area);
    }

    // --- Bar Chart: Monthly Net Balance ---
    let table_title = {
        let y = year_str_owned.clone();
        let yp = year_progress_owned.clone();
        let mut title_spans = vec![];
        if is_filtered {
            title_spans.push(Span::styled(
                "(Filtered) ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        }
        title_spans.push(Span::styled(
            "Monthly Net Balance - ",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            y,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            " ",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        title_spans.push(Span::styled(
            yp,
            Style::default().add_modifier(Modifier::BOLD),
        ));
        Line::from(title_spans)
    };
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
            chart_data_styled.push(
                Bar::default()
                    .label(month_to_short_str(month).into())
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
        chart_data_styled.push(Bar::default().label("N/A".into()).value(0));
    }
    let num_bars = chart_data_styled.len() as u16;
    let usable_width = bar_chart_area.width.saturating_sub(2);
    let width_per_bar_and_gap = (usable_width / num_bars.max(1)).max(1);
    let bar_gap = if width_per_bar_and_gap > 1 {
        1u16
    } else {
        0u16
    };
    let bar_width = width_per_bar_and_gap.saturating_sub(bar_gap).max(1);
    let bar_chart = BarChart::default()
        .block(Block::default().title(table_title).borders(Borders::ALL))
        .data(BarGroup::default().bars(&chart_data_styled))
        .bar_width(bar_width)
        .bar_gap(bar_gap)
        .group_gap(0)
        .label_style(Style::default().fg(Color::White))
        .max(max_abs_chart_value.max(10) as u64);
    f.render_widget(bar_chart, bar_chart_area);
}

pub fn render_spending_goals_bar(f: &mut Frame, app: &App, area: Rect) {
    let now = chrono::Utc::now();
    let year = now.year();
    let month = now.month();

    let expenses: Vec<Transaction> = load_transactions(Path::new(
        load_settings()
            .unwrap_or_default()
            .data_file_path
            .unwrap_or_default()
            .as_str(),
    ))
    .unwrap_or_default()
    .into_iter()
    .filter(|transaction| transaction.transaction_type == TransactionType::Expense)
    .filter(|transaction| (transaction.date.month0() + 1) == month)
    .collect();

    let actual_expenses = categorize_expenses(expenses.clone());

    let total_income = app
        .monthly_summaries
        .get(&(year, month))
        .cloned()
        .unwrap_or_default()
        .income;
    let settings = load_settings().unwrap_or_default();
    let spending_goals_percentages = (
        settings.necessary_expenses_percentage.unwrap_or_default(),
        settings
            .discretionary_expenses_percentage
            .unwrap_or_default(),
        settings.saving_or_investment_percentage.unwrap_or_default(),
        settings.tax_setaside_percentage.unwrap_or_default(),
    );
    let percentages = (
        total_income * (spending_goals_percentages.0 / 100.0),
        total_income * (spending_goals_percentages.1 / 100.0),
        total_income * (spending_goals_percentages.2 / 100.0),
        total_income * (spending_goals_percentages.3 / 100.0),
    );
    let necessary_span = Span::styled(
        format!("Necessities: {:.2}", percentages.0),
        Style::default().add_modifier(Modifier::BOLD),
    );
    let necessary_actual = Span::styled(
        format!(" ({:.2}) ", actual_expenses.0),
        if actual_expenses.0 <= percentages.0 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        },
    );
    let discretionary_span = Span::styled(
        format!("Discretionary: {:.2}", percentages.1),
        Style::default().add_modifier(Modifier::BOLD),
    );
    let discretionary_actual = Span::styled(
        format!(" ({:.2}) ", actual_expenses.1),
        if actual_expenses.0 <= percentages.1 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        },
    );
    let savings_span = Span::styled(
        format!("Savings/investments: {:.2}", percentages.2),
        Style::default().add_modifier(Modifier::BOLD),
    );
    let savings_actual = Span::styled(
        format!(" ({:.2}) ", actual_expenses.2),
        // NOTE: Here is green if you saved as much you planned or more
        if actual_expenses.2 >= percentages.2 {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        },
    );
    let tax_span = Span::styled(
        format!("Tax setaside: {:.2}", percentages.3),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Blue),
    );
    let spending_goals_line = Line::from(vec![
        necessary_span,
        necessary_actual,
        Span::raw(" | "),
        discretionary_span,
        discretionary_actual,
        Span::raw(" | "),
        savings_span,
        savings_actual,
        Span::raw(" | "),
        tax_span,
    ])
    .alignment(Alignment::Center);
    let spending_goals_paragraph = Paragraph::new(spending_goals_line).block(
        Block::default().borders(Borders::ALL).title(Span::styled(
            format!("Spending goals for {} {}", month_to_short_str(month), year),
            Style::default().add_modifier(Modifier::BOLD),
        )),
    );

    f.render_widget(spending_goals_paragraph, area);
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
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
    );
    let expense_span = Span::styled(
        format!("Expenses: {:.2}", total_expense),
        Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD),
    );
    let net_style = if net_balance >= 0.0 {
        Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD)
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

    let is_filtered = app.filtered_indices.len() != app.transactions.len();
    let title = if is_filtered {
        "Grand Total (Filtered)"
    } else {
        "Grand Total (All Transactions)"
    };

    let summary_paragraph =
        Paragraph::new(summary_line).block(Block::default().borders(Borders::ALL).title(
            Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
        ));

    f.render_widget(summary_paragraph, area);
}
