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
        if !recurring_tx.is_recurring {
            continue;
        }

        // Safely handle missing frequency
        let Some(frequency) = recurring_tx.recurrence_frequency else {
            continue; // Skip malformed recurring transactions
        };

        match frequency {
            RecurrenceFrequency::SemiMonthly => {
                // Handle semi-monthly separately since it follows a different pattern
                let semi_monthly_instances =
                    generate_semi_monthly_instances(recurring_tx, up_to_date);
                generated.extend(semi_monthly_instances);
            }
            _ => {
                // Handle other frequencies with the existing logic
                let mut current_date = recurring_tx.date;

                // Generate transactions from the original date up to the specified date
                while current_date <= up_to_date {
                    // Skip the original transaction date (it's already in the list)
                    if current_date != recurring_tx.date {
                        if let Some(new_tx) = create_generated_transaction(
                            recurring_tx,
                            current_date,
                            recurring_tx.date,
                            up_to_date,
                        ) {
                            generated.push(new_tx);
                        } else if current_date
                            > recurring_tx.recurrence_end_date.unwrap_or(NaiveDate::MAX)
                        {
                            break;
                        }
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
                                    NaiveDate::from_ymd_opt(next_year, 2, 28)
                                        .unwrap_or(current_date + Duration::days(365))
                                } else {
                                    current_date
                                        .with_year(next_year)
                                        .unwrap_or(current_date + Duration::days(365))
                                }
                            } else {
                                current_date
                                    .with_year(next_year)
                                    .unwrap_or(current_date + Duration::days(365))
                            }
                        }
                        RecurrenceFrequency::SemiMonthly => unreachable!(), // Handled above
                    };
                }
            }
        }
    }

    generated
}

/// Generates semi-monthly transaction instances (15th and last day of each month)
fn generate_semi_monthly_instances(
    recurring_tx: &Transaction,
    up_to_date: NaiveDate,
) -> Vec<Transaction> {
    let mut generated = Vec::new();
    let start_date = recurring_tx.date;

    // Start from the month of the original transaction
    let mut current_date =
        NaiveDate::from_ymd_opt(start_date.year(), start_date.month(), 1).unwrap_or(start_date);

    while current_date <= up_to_date {
        let year = current_date.year();
        let month = current_date.month();

        // Try to generate transactions for 15th and last day of the month
        for day in [15, crate::validation::days_in_month(year, month)] {
            if let Some(target_date) = NaiveDate::from_ymd_opt(year, month, day) {
                if let Some(new_tx) =
                    create_generated_transaction(recurring_tx, target_date, start_date, up_to_date)
                {
                    generated.push(new_tx);
                } else if target_date > recurring_tx.recurrence_end_date.unwrap_or(NaiveDate::MAX) {
                    return generated;
                }
            }
        }

        current_date = crate::validation::add_months(current_date, 1);

        if current_date > up_to_date {
            break;
        }
    }

    generated
}

/// Helper function to create a generated transaction if it meets the criteria
fn create_generated_transaction(
    recurring_tx: &Transaction,
    target_date: NaiveDate,
    start_date: NaiveDate,
    up_to_date: NaiveDate,
) -> Option<Transaction> {
    if target_date <= start_date || target_date > up_to_date {
        return None;
    }

    if let Some(end_date) = recurring_tx.recurrence_end_date {
        if target_date > end_date {
            return None;
        }
    }

    let mut new_tx = recurring_tx.clone();
    new_tx.date = target_date;
    new_tx.is_generated_from_recurring = true;
    Some(new_tx)
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
