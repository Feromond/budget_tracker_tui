use super::state::App;
use crate::model::TransactionType;
use crate::model::DATE_FORMAT;
use crate::persistence::save_transactions;
use chrono::NaiveDate;

impl App {
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
        self.status_message = None;
    }
    pub(crate) fn exit_adding(&mut self) { // TODO: Add status message for exiting without saving
        self.mode = crate::app::state::AppMode::Normal;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default(); 
    }
    pub(crate) fn add_transaction(&mut self) {
        // Parse and validate all fields for a new transaction.
        let date_res = NaiveDate::parse_from_str(&self.add_edit_fields[0], DATE_FORMAT);
        let description = self.add_edit_fields[1].trim();
        let amount_res = self.add_edit_fields[2].parse::<f64>();
        let type_str = self.add_edit_fields[3].trim().to_lowercase();
        let category = self.add_edit_fields[4].trim();
        let subcategory = self.add_edit_fields[5].trim();
        let transaction_type = if type_str.starts_with('i') {
            TransactionType::Income
        } else {
            TransactionType::Expense
        };
        if let Err(cat_err) = self.validate_category(transaction_type, category, subcategory) {
            self.status_message = Some(format!("Error: {}", cat_err));
            return;
        }
        match (date_res, description, amount_res) {
            (Ok(date), desc, Ok(amount)) if !desc.is_empty() && amount > 0.0 => {
                let new_transaction = crate::model::Transaction {
                    date,
                    description: desc.to_string(),
                    amount,
                    transaction_type,
                    category: category.to_string(),
                    subcategory: subcategory.to_string(),
                };
                self.transactions.push(new_transaction);
                self.sort_transactions();
                self.apply_filter();
                self.calculate_monthly_summaries();
                match save_transactions(&self.transactions, &self.data_file_path) {
                    Ok(_) => {
                        self.status_message = Some("Transaction added successfully.".to_string());
                        self.exit_adding();
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving transaction: {}", e));
                    }
                }
            }
            (Err(_), _, _) => {
                self.status_message = Some(format!(
                    "Error: Invalid Date Format (Expected {})",
                    DATE_FORMAT
                ));
            }
            (_, _, Err(_)) => {
                self.status_message = Some("Error: Invalid Amount (Must be a number)".to_string());
            }
            (_, "", _) => {
                self.status_message = Some("Error: Description cannot be empty".to_string());
            }
            (_, _, Ok(amount)) if amount <= 0.0 => {
                self.status_message = Some("Error: Amount must be positive".to_string());
            }
            _ => {
                self.status_message = Some("Error: Could not add transaction".to_string());
            }
        }
    }
    // --- Editing Logic ---
    pub(crate) fn start_editing(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                let tx = &self.transactions[original_index];
                self.mode = crate::app::state::AppMode::Editing;
                self.editing_index = Some(original_index);
                self.current_add_edit_field = 0;
                self.add_edit_fields = [
                    tx.date.format(DATE_FORMAT).to_string(),
                    tx.description.clone(),
                    format!("{:.2}", tx.amount),
                    if tx.transaction_type == TransactionType::Income {
                        "Income".to_string()
                    } else {
                        "Expense".to_string()
                    },
                    tx.category.clone(),
                    tx.subcategory.clone(),
                ];
                self.status_message = None;
            } else {
                self.status_message =
                    Some("Error: Could not map view index to transaction".to_string());
            }
        } else {
            self.status_message = Some("Select a transaction to edit first".to_string());
        }
    }
    pub(crate) fn exit_editing(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default();
        self.status_message = None;
    }
    pub(crate) fn update_transaction(&mut self) {
        if let Some(index) = self.editing_index {
            let date_res = NaiveDate::parse_from_str(&self.add_edit_fields[0], DATE_FORMAT);
            let description = self.add_edit_fields[1].trim();
            let amount_res = self.add_edit_fields[2].parse::<f64>();
            let type_str = self.add_edit_fields[3].trim().to_lowercase();
            let category = self.add_edit_fields[4].trim();
            let subcategory = self.add_edit_fields[5].trim();
            let transaction_type = if type_str.starts_with('i') {
                TransactionType::Income
            } else {
                TransactionType::Expense
            };
            if let Err(cat_err) = self.validate_category(transaction_type, category, subcategory) {
                self.status_message = Some(format!("Error: {}", cat_err));
                return;
            }
            match (date_res, description, amount_res) {
                (Ok(date), desc, Ok(amount)) if !desc.is_empty() && amount > 0.0 => {
                    if index < self.transactions.len() {
                        self.transactions[index] = crate::model::Transaction {
                            date,
                            description: desc.to_string(),
                            amount,
                            transaction_type,
                            category: category.to_string(),
                            subcategory: subcategory.to_string(),
                        };
                        match save_transactions(&self.transactions, &self.data_file_path) {
                            Ok(_) => {
                                self.status_message =
                                    Some("Transaction updated successfully.".to_string());
                                self.apply_filter();
                                self.calculate_monthly_summaries();
                                self.exit_editing();
                            }
                            Err(e) => {
                                self.status_message =
                                    Some(format!("Error saving updated transaction: {}", e));
                            }
                        }
                    } else {
                        self.status_message = Some("Error: Invalid index during edit".to_string());
                        self.exit_editing();
                    }
                }
                (Err(_), _, _) => {
                    self.status_message = Some(format!(
                        "Error: Invalid Date Format (Expected {})",
                        DATE_FORMAT
                    ));
                }
                (_, _, Err(_)) => {
                    self.status_message =
                        Some("Error: Invalid Amount (Must be a number)".to_string());
                }
                (_, "", _) => {
                    self.status_message = Some("Error: Description cannot be empty".to_string());
                }
                (_, _, Ok(amount)) if amount <= 0.0 => {
                    self.status_message = Some("Error: Amount must be positive".to_string());
                }
                _ => {
                    self.status_message = Some("Error: Could not update transaction".to_string());
                }
            }
        } else {
            self.status_message = Some("Error: No transaction selected for editing".to_string());
            self.exit_editing();
        }
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
    // --- Deleting Logic ---
    // Handles preparing, confirming, and canceling transaction deletion.
    pub(crate) fn prepare_for_delete(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                self.delete_index = Some(original_index);
                self.mode = crate::app::state::AppMode::ConfirmDelete;
                self.status_message = Some("Confirm Delete? (y/n)".to_string());
            } else {
                self.status_message =
                    Some("Error: Could not map view index to transaction".to_string());
            }
        } else {
            self.status_message = Some("Select a transaction to delete first".to_string());
        }
    }
    pub(crate) fn confirm_delete(&mut self) {
        if let Some(original_index) = self.delete_index {
            self.transactions.remove(original_index);
            self.apply_filter();
            if let Some(selected) = self.table_state.selected() {
                if selected >= self.filtered_indices.len() && !self.filtered_indices.is_empty() {
                    self.table_state
                        .select(Some(self.filtered_indices.len() - 1));
                }
            }
            match save_transactions(&self.transactions, &self.data_file_path) {
                Ok(_) => {
                    self.status_message = Some("Transaction deleted successfully.".to_string());
                    self.delete_index = None;
                    self.mode = crate::app::state::AppMode::Normal;
                    self.calculate_monthly_summaries();
                    self.calculate_category_summaries();
                }
                Err(e) => {
                    self.status_message = Some(format!("Error saving after delete: {}", e));
                }
            }
        } else {
            self.cancel_delete();
        }
    }
    pub(crate) fn cancel_delete(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.delete_index = None;
        self.status_message = None;
    }
    // --- Validation Helper ---
    // Checks if the given category/subcategory is valid for the transaction type.
    fn validate_category(
        &self,
        transaction_type: TransactionType,
        category: &str,
        subcategory: &str,
    ) -> Result<(), String> {
        // Allow "Uncategorized" or empty category
        if category.is_empty() || category.eq_ignore_ascii_case("Uncategorized") {
            return Ok(());
        }
        let category_lower = category.to_lowercase();
        let subcategory_lower = subcategory.to_lowercase();
        let is_valid = self.categories.iter().any(|cat_info| {
            cat_info.transaction_type == transaction_type
                && cat_info.category.to_lowercase() == category_lower
                && (cat_info.subcategory.to_lowercase() == subcategory_lower
                    || cat_info.subcategory.is_empty())
        });
        if is_valid {
            Ok(())
        } else {
            Err(format!(
                "Invalid Category/Subcategory: '{}' / '{}' for {:?}",
                category, subcategory, transaction_type
            ))
        }
    }
}
