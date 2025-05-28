use crate::model::{RecurrenceFrequency, Transaction};
use chrono::{Datelike, Duration, NaiveDate};

pub fn generate_recurring_transactions(
    recurring_transactions: &[Transaction],
    up_to_date: NaiveDate,
) -> Vec<Transaction> {
    let mut generated = Vec::new();

    for recurring_tx in recurring_transactions {
        if !recurring_tx.is_recurring || recurring_tx.recurrence_frequency.is_none() {
            continue;
        }

        let frequency = recurring_tx.recurrence_frequency.unwrap();
        let mut current_date = recurring_tx.date;

        // Generate transactions from the original date up to the specified date
        while current_date <= up_to_date {
            // Skip the original transaction date (it's already in the list)
            if current_date != recurring_tx.date {
                // Check if we've exceeded the end date (if specified)
                if let Some(end_date) = recurring_tx.recurrence_end_date {
                    if current_date > end_date {
                        break;
                    }
                }

                let mut new_tx = recurring_tx.clone();
                new_tx.date = current_date;
                new_tx.is_generated_from_recurring = true;
                generated.push(new_tx);
            }

            // Calculate next occurrence
            current_date = match frequency {
                RecurrenceFrequency::Daily => current_date + Duration::days(1),
                RecurrenceFrequency::Weekly => current_date + Duration::weeks(1),
                RecurrenceFrequency::BiWeekly => current_date + Duration::weeks(2),
                RecurrenceFrequency::Monthly => {
                    // Handle month boundaries properly
                    let next_month = if current_date.month() == 12 {
                        current_date.with_year(current_date.year() + 1).unwrap().with_month(1).unwrap()
                    } else {
                        current_date.with_month(current_date.month() + 1).unwrap()
                    };
                    
                    // Handle cases where the day doesn't exist in the next month (e.g., Jan 31 -> Feb 28)
                    let target_day = current_date.day();
                    let days_in_next_month = days_in_month(next_month.year(), next_month.month());
                    let actual_day = target_day.min(days_in_next_month);
                    
                    next_month.with_day(actual_day).unwrap()
                }
                RecurrenceFrequency::Yearly => {
                    // Handle leap year edge case for Feb 29
                    let next_year = current_date.year() + 1;
                    if current_date.month() == 2 && current_date.day() == 29 {
                        // If it's Feb 29 and next year is not a leap year, use Feb 28
                        if !is_leap_year(next_year) {
                            NaiveDate::from_ymd_opt(next_year, 2, 28).unwrap()
                        } else {
                            current_date.with_year(next_year).unwrap()
                        }
                    } else {
                        current_date.with_year(next_year).unwrap()
                    }
                }
            };
        }
    }

    generated
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => panic!("Invalid month: {}", month),
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn remove_generated_recurring_transactions(transactions: &mut Vec<Transaction>) {
    transactions.retain(|tx| !tx.is_generated_from_recurring);
} 