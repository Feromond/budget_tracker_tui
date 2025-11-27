/// App-specific utility functions
///
/// This module contains utilities that are specific to app state management
/// and operations, as opposed to general validation or business logic.
use crate::model::*;
use std::cmp::Ordering;

/// Sorts transactions by the selected column and order
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

/// Opens a URL in the default browser using system commands.
/// Returns true if successful, false otherwise.
pub fn open_url(url: &str) -> bool {
    let result = if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).spawn()
    } else if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
    } else if cfg!(target_os = "linux") {
        std::process::Command::new("xdg-open").arg(url).spawn()
    } else {
        return false;
    };

    result.map(|_| true).unwrap_or(false)
}

// --- Recurring Transaction Utilities ---

/// Action types for jumping to original recurring transactions
pub enum JumpToOriginalAction {
    Edit,
    Delete,
    RecurringSettings,
}

impl JumpToOriginalAction {
    pub fn message(&self) -> &'static str {
        match self {
            JumpToOriginalAction::Edit => "Jumped to original recurring transaction for editing.",
            JumpToOriginalAction::Delete => {
                "Jumped to original recurring transaction for deletion."
            }
            JumpToOriginalAction::RecurringSettings => {
                "Jumped to original recurring transaction for settings."
            }
        }
    }
}
