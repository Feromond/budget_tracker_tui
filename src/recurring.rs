/// Core business logic for recurring transactions
/// 
/// This module contains pure functions that handle recurring transaction generation
/// and management. These are domain-level operations independent of UI or app state.
use crate::model::{RecurrenceFrequency, Transaction};
use chrono::{Datelike, Duration, NaiveDate};


/// Generates recurring transaction instances from a list of recurring transactions
/// up to a specified date.
/// 
/// # Arguments
/// * `recurring_transactions` - Slice of transactions marked as recurring
/// * `up_to_date` - Generate instances up to this date (inclusive)
/// 
/// # Returns
/// Vector of generated transaction instances with `is_generated_from_recurring` = true
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
                    crate::validation::add_months(current_date, 1)
                }
                RecurrenceFrequency::Yearly => {
                    // Handle leap year edge case for Feb 29
                    let next_year = current_date.year() + 1;
                    if current_date.month() == 2 && current_date.day() == 29 {
                        // If it's Feb 29 and next year is not a leap year, use Feb 28
                        if !crate::validation::is_leap_year(next_year) {
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

/// Removes all generated recurring transactions from a transaction list
/// 
/// This is used to clean up before regenerating recurring transactions
/// to avoid duplicates.
/// 
/// # Arguments
/// * `transactions` - Mutable reference to transaction vector to clean
pub fn remove_generated_recurring_transactions(transactions: &mut Vec<Transaction>) {
    transactions.retain(|tx| !tx.is_generated_from_recurring);
} 