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
            RecurrenceFrequency::SemiMonthly | RecurrenceFrequency::SemiMonthlyWorkday => {
                // Semi-monthly follows a fixed-day pattern rather than a recurring interval.
                let workdays_only = frequency == RecurrenceFrequency::SemiMonthlyWorkday;
                generated.extend(generate_semi_monthly_instances(
                    recurring_tx,
                    up_to_date,
                    workdays_only,
                ));
            }
            _ => {
                let mut occurrence: i32 = 0;
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

                    // Calculate next occurrence from the anchor date
                    occurrence += 1;
                    let next_date = match frequency {
                        RecurrenceFrequency::Daily => {
                            recurring_tx.date + Duration::days(occurrence as i64)
                        }
                        RecurrenceFrequency::Weekly => {
                            recurring_tx.date + Duration::weeks(occurrence as i64)
                        }
                        RecurrenceFrequency::BiWeekly => {
                            recurring_tx.date + Duration::weeks(2 * occurrence as i64)
                        }
                        RecurrenceFrequency::Monthly => {
                            crate::validation::add_months(recurring_tx.date, occurrence)
                        }
                        RecurrenceFrequency::Quarterly => {
                            crate::validation::add_months(recurring_tx.date, 3 * occurrence)
                        }
                        RecurrenceFrequency::Yearly => {
                            // add_months clamps Feb 29 to Feb 28 in non-leap years and
                            // restores Feb 29 when a leap year comes around again.
                            crate::validation::add_months(recurring_tx.date, 12 * occurrence)
                        }
                        RecurrenceFrequency::SemiMonthly
                        | RecurrenceFrequency::SemiMonthlyWorkday => unreachable!(), // Handled above
                    };

                    // add_months falls back to the input date at its bounds; bail out
                    // rather than loop forever if the date stops advancing.
                    if next_date <= current_date {
                        break;
                    }
                    current_date = next_date;
                }
            }
        }
    }

    generated
}

/// Generates semi-monthly transaction instances on the 15th and last day of each month. When
/// `workdays_only` is set, a target landing on a weekend is moved back to the preceding weekday
/// (e.g. payroll paid on the last business day on or before the 15th / month end).
fn generate_semi_monthly_instances(
    recurring_tx: &Transaction,
    up_to_date: NaiveDate,
    workdays_only: bool,
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
            if let Some(mut target_date) = NaiveDate::from_ymd_opt(year, month, day) {
                if workdays_only {
                    target_date = previous_weekday_on_or_before(target_date);
                }
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

/// Moves a date back to the nearest weekday on or before it (Saturday/Sunday roll to Friday).
fn previous_weekday_on_or_before(mut date: NaiveDate) -> NaiveDate {
    while matches!(date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
        date -= Duration::days(1);
    }
    date
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

    if let Some(end_date) = recurring_tx.recurrence_end_date
        && target_date > end_date
    {
        return None;
    }

    let mut new_tx = recurring_tx.clone();
    new_tx.date = target_date;
    new_tx.is_generated_from_recurring = true;
    // Generated occurrences are not persisted; clear the DB id and link back to the source.
    new_tx.parent_id = recurring_tx.id;
    new_tx.id = None;
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
