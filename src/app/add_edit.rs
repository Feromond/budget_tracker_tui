use super::state::App;
use crate::db::transaction_store::TransactionStore;
use crate::model::TransactionType;
use crate::model::{TransactionDraft, DATE_FORMAT};
use chrono::{Duration, NaiveDate};

impl App {
    // Helper function to find the original recurring transaction for a generated one
    pub(crate) fn find_original_recurring_transaction(
        &self,
        generated_tx: &crate::model::Transaction,
    ) -> Option<usize> {
        // Only generated occurrences point back to a source.
        let parent_id = generated_tx.parent_id?;
        self.transactions
            .iter()
            .position(|tx| tx.id == Some(parent_id) && !tx.is_generated_from_recurring)
    }

    // Shared function for jumping to original recurring transaction
    pub(crate) fn jump_to_original_if_needed(
        &mut self,
        tx: &crate::model::Transaction,
        original_index: usize,
        action: crate::app::util::JumpToOriginalAction,
    ) -> Option<usize> {
        if !tx.is_generated_from_recurring {
            return Some(original_index); // Not generated, use current index
        }

        if let Some(original_recurring_index) = self.find_original_recurring_transaction(tx) {
            // Find the view index for the original transaction
            if let Some(original_view_index) = self
                .filtered_indices
                .iter()
                .position(|&idx| idx == original_recurring_index)
            {
                // Select the original transaction in the table
                self.table_state.select(Some(original_view_index));
                self.set_status_message(action.message(), None);
                Some(original_recurring_index)
            } else {
                self.set_status_message(
                    "Original recurring transaction not visible in current filter.",
                    None,
                );
                None
            }
        } else {
            self.set_status_message("Could not find original recurring transaction.", None);
            None
        }
    }

    // --- Adding Logic ---
    // Handles entering add mode, setting up default values, and resetting state.
    pub(crate) fn start_adding(&mut self) {
        self.mode = crate::app::state::AppMode::Adding;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default();
        let today = chrono::Local::now().date_naive();
        self.add_edit_fields[0] = today.format(DATE_FORMAT).to_string();
        self.add_edit_fields[3] = "Expense".to_string();
        self.add_edit_cursor = self.add_edit_fields[0].len();
        self.clear_status_message();
    }
    pub(crate) fn exit_adding(&mut self, cancelled: bool) {
        self.mode = crate::app::state::AppMode::Normal;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default();
        if cancelled {
            self.set_status_message("Add transaction cancelled.", Some(Duration::seconds(3)));
        }
    }
    pub(crate) fn add_transaction(&mut self) {
        // Parse and validate all fields for a new transaction.
        let date_res = NaiveDate::parse_from_str(&self.add_edit_fields[0], DATE_FORMAT);
        let description = self.add_edit_fields[1].trim();
        let amount_str = self.add_edit_fields[2].trim();
        let type_str = self.add_edit_fields[3].trim().to_lowercase();
        let category = self.add_edit_fields[4].trim();
        let subcategory = self.add_edit_fields[5].trim();

        let transaction_type = if type_str.starts_with('i') {
            TransactionType::Income
        } else {
            TransactionType::Expense
        };

        let amount = match crate::validation::validate_amount_string(amount_str) {
            Ok(amount) => amount,
            Err(msg) => {
                self.set_status_message(format!("Error: {}", msg), None);
                return;
            }
        };

        let date = match date_res {
            Ok(date) => date,
            Err(_) => {
                self.set_status_message(
                    format!("Error: Invalid Date Format (Expected {})", DATE_FORMAT),
                    None,
                );
                return;
            }
        };

        if description.is_empty() {
            self.set_status_message("Error: Description cannot be empty", None);
            return;
        }

        if let Err(cat_err) = crate::validation::validate_category(
            &self.categories,
            transaction_type,
            category,
            subcategory,
        ) {
            self.set_status_message(format!("Error: {}", cat_err), None);
            return;
        }

        let draft = TransactionDraft {
            date,
            description: description.to_string(),
            amount,
            transaction_type,
            category: category.to_string(),
            subcategory: subcategory.to_string(),
            is_recurring: false,
            recurrence_frequency: None,
            recurrence_end_date: None,
        };

        match self.transaction_store().insert(&draft) {
            Ok(_) => match self.reload_transactions_from_db() {
                Ok(_) => {
                    self.set_status_message(
                        "Transaction added successfully.",
                        Some(Duration::seconds(3)),
                    );
                    self.exit_adding(false);
                }
                Err(e) => {
                    self.set_status_message(format!("Error reloading transactions: {}", e), None);
                }
            },
            Err(e) => {
                self.set_status_message(format!("Error saving transaction: {}", e), None);
            }
        }
    }
    // --- Editing Logic ---
    pub(crate) fn start_editing(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                let tx = self.transactions[original_index].clone();

                // Jump to original if this is a generated transaction, or use current if not
                if let Some(target_index) = self.jump_to_original_if_needed(
                    &tx,
                    original_index,
                    crate::app::util::JumpToOriginalAction::Edit,
                ) {
                    let target_tx = &self.transactions[target_index];

                    self.mode = crate::app::state::AppMode::Editing;
                    self.editing_index = Some(target_index);
                    self.current_add_edit_field = 0;
                    self.add_edit_fields = [
                        target_tx.date.format(DATE_FORMAT).to_string(),
                        target_tx.description.clone(),
                        format!("{:.2}", target_tx.amount),
                        if target_tx.transaction_type == TransactionType::Income {
                            "Income".to_string()
                        } else {
                            "Expense".to_string()
                        },
                        target_tx.category.clone(),
                        target_tx.subcategory.clone(),
                    ];
                    self.add_edit_cursor = self.add_edit_fields[0].len();

                    if target_index == original_index {
                        self.clear_status_message();
                    }
                }
            } else {
                self.set_status_message("Error: Could not map view index to transaction", None);
            }
        } else {
            self.set_status_message("Select a transaction to edit first", None);
        }
    }
    pub(crate) fn exit_editing(&mut self, cancelled: bool) {
        self.mode = crate::app::state::AppMode::Normal;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default();
        if cancelled {
            self.set_status_message("Edit transaction cancelled.", Some(Duration::seconds(3)));
        } else {
            self.clear_status_message();
        }
    }
    pub(crate) fn update_transaction(&mut self) {
        if let Some(index) = self.editing_index {
            let date_res = NaiveDate::parse_from_str(&self.add_edit_fields[0], DATE_FORMAT);
            let description = self.add_edit_fields[1].trim();
            let amount_str = self.add_edit_fields[2].trim();
            let type_str = self.add_edit_fields[3].trim().to_lowercase();
            let category = self.add_edit_fields[4].trim();
            let subcategory = self.add_edit_fields[5].trim();

            let transaction_type = if type_str.starts_with('i') {
                TransactionType::Income
            } else {
                TransactionType::Expense
            };

            // Validate amount using centralized utility
            let amount = match crate::validation::validate_amount_string(amount_str) {
                Ok(amount) => amount,
                Err(msg) => {
                    self.set_status_message(format!("Error: {}", msg), None);
                    return;
                }
            };

            // Validate date and description
            let date = match date_res {
                Ok(date) => date,
                Err(_) => {
                    self.set_status_message(
                        format!("Error: Invalid Date Format (Expected {})", DATE_FORMAT),
                        None,
                    );
                    return;
                }
            };

            if description.is_empty() {
                self.set_status_message("Error: Description cannot be empty", None);
                return;
            }

            // Validate category using centralized utility
            if let Err(cat_err) = crate::validation::validate_category(
                &self.categories,
                transaction_type,
                category,
                subcategory,
            ) {
                self.set_status_message(format!("Error: {}", cat_err), None);
                return;
            }

            // Update transaction
            if index < self.transactions.len() {
                let existing_tx = &self.transactions[index];
                // The edit form only edits the core fields; preserve the recurring rule.
                let draft = TransactionDraft {
                    date,
                    description: description.to_string(),
                    amount,
                    transaction_type,
                    category: category.to_string(),
                    subcategory: subcategory.to_string(),
                    is_recurring: existing_tx.is_recurring,
                    recurrence_frequency: existing_tx.recurrence_frequency,
                    recurrence_end_date: existing_tx.recurrence_end_date,
                };
                let Some(id) = existing_tx.id else {
                    self.set_status_message("Error: transaction has no database id", None);
                    self.exit_editing(true);
                    return;
                };

                match self
                    .transaction_store()
                    .update(id, &draft)
                    .and_then(|_| self.reload_transactions_from_db())
                {
                    Ok(_) => {
                        self.set_status_message(
                            "Transaction updated successfully.",
                            Some(Duration::seconds(3)),
                        );
                        self.exit_editing(false);
                    }
                    Err(e) => {
                        self.set_status_message(
                            format!("Error saving updated transaction: {}", e),
                            None,
                        );
                    }
                }
            } else {
                self.set_status_message("Error: Invalid index during edit", None);
                self.exit_editing(true);
            }
        } else {
            self.set_status_message("Error: No transaction selected for editing", None);
            self.exit_editing(true);
        }
    }
    // --- Field Navigation ---
    pub(crate) fn next_add_edit_field(&mut self) {
        self.current_add_edit_field =
            (self.current_add_edit_field + 1) % self.add_edit_fields.len();
        self.add_edit_cursor = self.add_edit_fields[self.current_add_edit_field].len();
    }

    pub(crate) fn previous_add_edit_field(&mut self) {
        if self.current_add_edit_field == 0 {
            self.current_add_edit_field = self.add_edit_fields.len() - 1;
        } else {
            self.current_add_edit_field -= 1;
        }
        self.add_edit_cursor = self.add_edit_fields[self.current_add_edit_field].len();
    }

    // --- Toggle Transaction Type ---
    // Switches between Income and Expense, and clears category/subcategory if type changes.
    pub(crate) fn toggle_transaction_type(&mut self) {
        if self.current_add_edit_field == 3 {
            self.add_edit_fields[3] = if self.add_edit_fields[3].eq_ignore_ascii_case("income") {
                "Expense".to_string()
            } else {
                "Income".to_string()
            };
            self.add_edit_fields[4] = String::new();
            self.add_edit_fields[5] = String::new();
        }
    }
    // --- Copying Logic ---
    pub(crate) fn copy_transaction(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                let tx = self.transactions[original_index].clone();
                let today = chrono::Local::now().date_naive();

                // A copy is always a plain (non-recurring) transaction dated today.
                let draft = TransactionDraft {
                    date: today,
                    description: tx.description.clone(),
                    amount: tx.amount,
                    transaction_type: tx.transaction_type,
                    category: tx.category.clone(),
                    subcategory: tx.subcategory.clone(),
                    is_recurring: false,
                    recurrence_frequency: None,
                    recurrence_end_date: None,
                };

                match self
                    .transaction_store()
                    .insert(&draft)
                    .and_then(|_| self.reload_transactions_from_db())
                {
                    Ok(_) => {
                        if let Some(new_view_index) =
                            self.filtered_indices.iter().position(|&idx| {
                                let t = &self.transactions[idx];
                                t.date == today
                                    && t.description == tx.description
                                    && t.amount == tx.amount
                                    && t.transaction_type == tx.transaction_type
                                    && t.category == tx.category
                                    && t.subcategory == tx.subcategory
                                    && !t.is_recurring
                                    && !t.is_generated_from_recurring
                            })
                        {
                            self.table_state.select(Some(new_view_index));
                        }
                        self.set_status_message(
                            "Transaction copied with today's date.",
                            Some(Duration::seconds(3)),
                        );
                    }
                    Err(e) => {
                        self.set_status_message(
                            format!("Error saving copied transaction: {}", e),
                            None,
                        );
                    }
                }
            } else {
                self.set_status_message("Error: Could not map view index to transaction", None);
            }
        } else {
            self.set_status_message("Select a transaction to copy first", None);
        }
    }

    // --- Deleting Logic ---
    // Handles preparing, confirming, and canceling transaction deletion.
    pub(crate) fn prepare_for_delete(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                // Clone the transaction to avoid borrowing issues
                let tx = self.transactions[original_index].clone();

                // Jump to original if this is a generated transaction, or use current if not
                if let Some(target_index) = self.jump_to_original_if_needed(
                    &tx,
                    original_index,
                    crate::app::util::JumpToOriginalAction::Delete,
                ) {
                    self.delete_index = Some(target_index);
                    self.mode = crate::app::state::AppMode::ConfirmDelete;

                    // Only show delete confirmation if we didn't jump (to preserve jump message)
                    if target_index == original_index {
                        self.set_status_message("Confirm Delete? (y/n)", None);
                    }
                }
            } else {
                self.set_status_message("Error: Could not map view index to transaction", None);
            }
        } else {
            self.set_status_message("Select a transaction to delete first", None);
        }
    }
    pub(crate) fn confirm_delete(&mut self) {
        if let Some(original_index) = self.delete_index {
            // Deletion always targets a real row (generated rows jump to their source first).
            let Some(id) = self.transactions[original_index].id else {
                self.set_status_message("Error: transaction has no database id", None);
                self.cancel_delete();
                return;
            };

            match self
                .transaction_store()
                .delete(id)
                .and_then(|_| self.reload_transactions_from_db())
            {
                Ok(_) => {
                    if let Some(selected) = self.table_state.selected() {
                        if selected >= self.filtered_indices.len()
                            && !self.filtered_indices.is_empty()
                        {
                            self.table_state
                                .select(Some(self.filtered_indices.len() - 1));
                        }
                    }
                    self.set_status_message(
                        "Transaction deleted successfully.",
                        Some(Duration::seconds(3)),
                    );
                    self.delete_index = None;
                    self.mode = crate::app::state::AppMode::Normal;
                }
                Err(e) => {
                    self.set_status_message(format!("Error deleting transaction: {}", e), None);
                }
            }
        } else {
            self.cancel_delete();
        }
    }
    pub(crate) fn cancel_delete(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.delete_index = None;
        self.clear_status_message();
    }
}
