use super::state::App;
use crate::model::{RecurrenceFrequency, Transaction};
use crate::recurring::{generate_recurring_transactions, remove_generated_recurring_transactions};
use chrono::NaiveDate;

impl App {
    pub(crate) fn generate_recurring_transactions(&mut self) {
        // Remove any previously generated recurring transactions
        remove_generated_recurring_transactions(&mut self.transactions);
        
        // Get all recurring transactions
        let recurring_transactions: Vec<Transaction> = self.transactions
            .iter()
            .filter(|tx| tx.is_recurring)
            .cloned()
            .collect();
        
        // Generate new recurring transactions up to today
        let today = chrono::Local::now().date_naive();
        let generated = generate_recurring_transactions(&recurring_transactions, today);
        
        // Add generated transactions to the main list
        self.transactions.extend(generated);
        
        // Re-sort and recalculate
        self.sort_transactions();
        self.apply_filter();
        self.calculate_monthly_summaries();
        self.calculate_category_summaries();
    }
    
    pub(crate) fn start_recurring_settings(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                // Clone the transaction to avoid borrowing issues
                let tx = self.transactions[original_index].clone();
                
                // Jump to original if this is a generated transaction, or use current if not
                if let Some(target_index) = self.jump_to_original_if_needed(&tx, original_index, crate::app::util::JumpToOriginalAction::RecurringSettings) {
                    let target_tx = &self.transactions[target_index];
                    
                    self.mode = crate::app::state::AppMode::RecurringSettings;
                    self.recurring_transaction_index = Some(target_index);
                    self.current_recurring_field = 0;
                    
                    // Initialize fields with current values
                    self.recurring_settings_fields[0] = if target_tx.is_recurring { "Yes" } else { "No" }.to_string();
                    self.recurring_settings_fields[1] = target_tx.recurrence_frequency
                        .map(|f| f.to_string().to_string())
                        .unwrap_or(String::from("Monthly"));
                    self.recurring_settings_fields[2] = target_tx.recurrence_end_date
                        .map(|d| d.format(crate::model::DATE_FORMAT).to_string())
                        .unwrap_or_default();
                    
                    // Only clear status message if we didn't jump (to preserve jump message)
                    if target_index == original_index {
                        self.status_message = None;
                    }
                }
            } else {
                self.status_message = Some("Error: Could not map view index to transaction".to_string());
            }
        } else {
            self.status_message = Some("Select a transaction to configure recurring settings".to_string());
        }
    }
    
    pub(crate) fn exit_recurring_settings(&mut self, cancelled: bool) {
        self.mode = crate::app::state::AppMode::Normal;
        self.recurring_transaction_index = None;
        self.current_recurring_field = 0;
        self.recurring_settings_fields = Default::default();
        if cancelled {
            self.status_message = Some("Recurring settings cancelled.".to_string());
        }
    }
    
    pub(crate) fn save_recurring_settings(&mut self) {
        if let Some(index) = self.recurring_transaction_index {
            if index < self.transactions.len() {
                let is_recurring = self.recurring_settings_fields[0].to_lowercase() == "yes";
                let frequency_str = &self.recurring_settings_fields[1];
                let end_date_str = &self.recurring_settings_fields[2];
                
                // Parse frequency
                let frequency = if is_recurring {
                    match frequency_str.as_str() {
                        "Daily" => Some(RecurrenceFrequency::Daily),
                        "Weekly" => Some(RecurrenceFrequency::Weekly),
                        "Bi-Weekly" => Some(RecurrenceFrequency::BiWeekly),
                        "Monthly" => Some(RecurrenceFrequency::Monthly),
                        "Yearly" => Some(RecurrenceFrequency::Yearly),
                        _ => Some(RecurrenceFrequency::Monthly), // Default
                    }
                } else {
                    None
                };
                
                // Parse end date
                let end_date = if !end_date_str.is_empty() {
                    match NaiveDate::parse_from_str(end_date_str, crate::model::DATE_FORMAT) {
                        Ok(date) => Some(date),
                        Err(_) => {
                            self.status_message = Some(format!(
                                "Error: Invalid end date format (Expected {})",
                                crate::model::DATE_FORMAT
                            ));
                            return;
                        }
                    }
                } else {
                    None
                };
                
                // Update the transaction
                self.transactions[index].is_recurring = is_recurring;
                self.transactions[index].recurrence_frequency = frequency;
                self.transactions[index].recurrence_end_date = end_date;
                
                // Save to file
                match crate::persistence::save_transactions(&self.transactions, &self.data_file_path) {
                    Ok(_) => {
                        self.status_message = Some("Recurring settings saved successfully.".to_string());
                        
                        // Regenerate recurring transactions
                        self.generate_recurring_transactions();
                        
                        self.exit_recurring_settings(false);
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving recurring settings: {}", e));
                    }
                }
            } else {
                self.status_message = Some("Error: Invalid transaction index".to_string());
                self.exit_recurring_settings(true);
            }
        } else {
            self.status_message = Some("Error: No transaction selected for recurring settings".to_string());
            self.exit_recurring_settings(true);
        }
    }
    
    pub(crate) fn next_recurring_field(&mut self) {
        self.current_recurring_field = (self.current_recurring_field + 1) % 3;
    }
    
    pub(crate) fn previous_recurring_field(&mut self) {
        self.current_recurring_field = if self.current_recurring_field == 0 {
            2
        } else {
            self.current_recurring_field - 1
        };
    }
    
    pub(crate) fn toggle_recurring_enabled(&mut self) {
        if self.current_recurring_field == 0 {
            self.recurring_settings_fields[0] = if self.recurring_settings_fields[0] == "Yes" {
                "No".to_string()
            } else {
                "Yes".to_string()
            };
        }
    }
    
    pub(crate) fn start_frequency_selection(&mut self) {
        if self.current_recurring_field == 1 {
            self.mode = crate::app::state::AppMode::SelectingRecurrenceFrequency;
            self.current_selection_list = RecurrenceFrequency::all()
                .iter()
                .map(|f| f.to_string().to_string())
                .collect();
            self.selection_list_state.select(Some(0));
        }
    }
    
    pub(crate) fn insert_char_recurring(&mut self, c: char) {
        if self.current_recurring_field == 2 {
            // Use the centralized date validation from validation module
            if let Some(new_date) = crate::validation::validate_and_insert_date_char(
                &self.recurring_settings_fields[2], 
                c
            ) {
                self.recurring_settings_fields[2] = new_date;
                self.status_message = None; // Clear any previous error messages
            } else {
                // Invalid character or date, show error message
                self.status_message = Some("Invalid date input. Use YYYY-MM-DD format.".to_string());
            }
        }
    }
    
    pub(crate) fn delete_char_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            crate::validation::handle_date_backspace(&mut self.recurring_settings_fields[2]);
            self.status_message = None;
        }
    }
    
    pub(crate) fn increment_date_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if let Some(new_date) = self.increment_date_field(&self.recurring_settings_fields[2]) {
                self.recurring_settings_fields[2] = new_date;
                self.status_message = None;
            }
        }
    }

    pub(crate) fn decrement_date_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if let Some(new_date) = self.decrement_date_field(&self.recurring_settings_fields[2]) {
                self.recurring_settings_fields[2] = new_date;
                self.status_message = None;
            }
        }
    }

    pub(crate) fn increment_month_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if let Some(new_date) = self.increment_month_field(&self.recurring_settings_fields[2]) {
                self.recurring_settings_fields[2] = new_date;
                self.status_message = None;
            }
        }
    }

    pub(crate) fn decrement_month_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if let Some(new_date) = self.decrement_month_field(&self.recurring_settings_fields[2]) {
                self.recurring_settings_fields[2] = new_date;
                self.status_message = None;
            }
        }
    }
} 