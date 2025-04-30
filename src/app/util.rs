use crate::model::*;
use std::cmp::Ordering;

pub fn validate_and_insert_date_char(field: &str, c: char) -> Option<String> {
    // Only allow ASCII digits for date input
    if !c.is_ascii_digit() {
        return None;
    }

    let len = field.len();

    if len >= 10 {
        return None;
    }

    let mut result = field.to_string();

    // Validate month digits as they're entered
    if len == 5 {
        let month_digit = c.to_digit(10).unwrap_or(0);
        if month_digit > 1 {
            // First digit of month can only be 0 or 1
            // Allow direct entry of single-digit months
            result.push(c);
            result.push('-');
            return Some(result);
        }
    } else if len == 6 {
        let first_digit = field
            .chars()
            .nth(5)
            .and_then(|ch| ch.to_digit(10))
            .unwrap_or(0);
        let month = first_digit * 10 + c.to_digit(10).unwrap_or(0);

        if month == 0 || month > 12 {
            return None;
        }
    }

    // Validate day digits
    if len == 8 {
        let day_digit = c.to_digit(10).unwrap_or(0);
        if day_digit > 3 {
            // First digit of day can only be 0, 1, 2, or 3
            return None;
        }
    } else if len == 9 {
        if let (Ok(year), Ok(month)) = (field[0..4].parse::<i32>(), field[5..7].parse::<u32>()) {
            let first_digit = field
                .chars()
                .nth(8)
                .and_then(|ch| ch.to_digit(10))
                .unwrap_or(0);
            let day = first_digit * 10 + c.to_digit(10).unwrap_or(0);
            // Use days_in_month utility for validation
            let last_day = days_in_month(year, month);

            if day == 0 || day > last_day {
                return None;
            }
        }
    }

    // Add the digit
    result.push(c);

    // Auto-insert hyphens
    if result.len() == 4 {
        // Validate year
        if let Ok(year) = result.parse::<i32>() {
            if !(1900..=2100).contains(&year) {
                return None; // Reject invalid year
            }
        }
        result.push('-');
    } else if result.len() == 7 {
        result.push('-');
    }

    Some(result)
}

// Sorts transactions by the selected column and order.
pub fn sort_transactions_impl(
    transactions: &mut [Transaction],
    sort_by: SortColumn,
    sort_order: SortOrder,
) {
    transactions.sort_by(|a, b| {
        let ordering = match sort_by {
            SortColumn::Date => a.date.cmp(&b.date),
            SortColumn::Description => a.description.cmp(&b.description),
            SortColumn::Amount => a.amount.partial_cmp(&b.amount).unwrap_or(Ordering::Equal),
            SortColumn::Type => a.transaction_type.cmp(&b.transaction_type),
            SortColumn::Category => a.category.cmp(&b.category),
            SortColumn::Subcategory => a.subcategory.cmp(&b.subcategory),
        };
        if sort_order == SortOrder::Descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 31,
    }
}
