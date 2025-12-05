use ratatui::layout::{Constraint, Direction, Layout, Rect};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;

pub fn format_amount(amount: &Decimal) -> String {
    let value = amount.to_f64().unwrap_or(0.0);
    let s = format!("{:.2}", value.abs());
    let parts: Vec<&str> = s.split('.').collect();
    let int_part = parts[0];
    let frac_part = parts[1];

    let mut formatted_int = String::new();
    let mut count = 0;
    for c in int_part.chars().rev() {
        if count > 0 && count % 3 == 0 {
            formatted_int.push(',');
        }
        formatted_int.push(c);
        count += 1;
    }

    let formatted_int: String = formatted_int.chars().rev().collect();
    let sign = if value < 0.0 { "-" } else { "" };

    format!("{}{}.{}", sign, formatted_int, frac_part)
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub fn month_to_short_str(month: u32) -> &'static str {
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
