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
            // Check for valid day in month, including leap years
            let last_day = match month {
                2 => {
                    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                        29 // Leap year February
                    } else {
                        28 // Regular February
                    }
                }
                4 | 6 | 9 | 11 => 30, // 30-day months
                _ => 31,              // 31-day months
            };

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
