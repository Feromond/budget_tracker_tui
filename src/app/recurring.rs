use super::state::App;
use crate::model::{RecurrenceFrequency, Transaction};
use crate::recurring::{generate_recurring_transactions, remove_generated_recurring_transactions};
use chrono::{NaiveDate, Datelike};

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
                let tx = &self.transactions[original_index];
                
                // If this is a generated recurring transaction, find and edit the original's settings instead
                if tx.is_generated_from_recurring {
                    if let Some(original_recurring_index) = self.find_original_recurring_transaction(tx) {
                        // Find the view index for the original transaction
                        if let Some(original_view_index) = self.filtered_indices.iter().position(|&idx| idx == original_recurring_index) {
                            // Select the original transaction in the table
                            self.table_state.select(Some(original_view_index));
                            self.status_message = Some("Jumped to original recurring transaction for settings.".to_string());
                            
                            // Now edit the original transaction's recurring settings
                            let original_tx = &self.transactions[original_recurring_index];
                            self.mode = crate::app::state::AppMode::RecurringSettings;
                            self.recurring_transaction_index = Some(original_recurring_index);
                            self.current_recurring_field = 0;
                            
                            // Initialize fields with current values
                            self.recurring_settings_fields[0] = if original_tx.is_recurring { "Yes" } else { "No" }.to_string();
                            self.recurring_settings_fields[1] = original_tx.recurrence_frequency
                                .map(|f| f.to_string().to_string())
                                .unwrap_or(String::from("Monthly"));
                            self.recurring_settings_fields[2] = original_tx.recurrence_end_date
                                .map(|d| d.format(crate::model::DATE_FORMAT).to_string())
                                .unwrap_or(String::new());
                            return;
                        } else {
                            self.status_message = Some("Original recurring transaction not visible in current filter.".to_string());
                            return;
                        }
                    } else {
                        self.status_message = Some("Could not find original recurring transaction.".to_string());
                        return;
                    }
                }
                
                self.mode = crate::app::state::AppMode::RecurringSettings;
                self.recurring_transaction_index = Some(original_index);
                self.current_recurring_field = 0;
                
                // Initialize fields with current values
                self.recurring_settings_fields[0] = if tx.is_recurring { "Yes" } else { "No" }.to_string();
                self.recurring_settings_fields[1] = tx.recurrence_frequency
                    .map(|f| f.to_string().to_string())
                    .unwrap_or(String::from("Monthly"));
                self.recurring_settings_fields[2] = tx.recurrence_end_date
                    .map(|d| d.format(crate::model::DATE_FORMAT).to_string())
                    .unwrap_or(String::new());
                
                self.status_message = None;
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
            // Use the same sophisticated date validation as add/edit forms
            if let Some(new_date) = crate::app::util::validate_and_insert_date_char(
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
            let current_field = &mut self.recurring_settings_fields[2];
            let len = current_field.len();

            if current_field.ends_with('-') && (current_field.len() == 5 || current_field.len() == 8) {
                if current_field
                    .chars()
                    .nth(len - 2)
                    .is_some_and(|ch| ch.is_ascii_digit())
                {
                    current_field.pop(); // Remove the hyphen
                    current_field.pop(); // Remove the preceding digit
                } else {
                    current_field.pop(); 
                }
            } else if !current_field.is_empty() {
                current_field.pop();
            }
            self.status_message = None;
        }
    }
    

    pub(crate) fn increment_date_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if self.recurring_settings_fields[2].is_empty() {
                // If empty, jump to today's date
                let today = chrono::Local::now().date_naive();
                self.recurring_settings_fields[2] = today.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            } else if let Ok(date) = chrono::NaiveDate::parse_from_str(&self.recurring_settings_fields[2], crate::model::DATE_FORMAT) {
                let new_date = date + chrono::Duration::days(1);
                self.recurring_settings_fields[2] = new_date.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            }
        }
    }

    pub(crate) fn decrement_date_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if self.recurring_settings_fields[2].is_empty() {
                // If empty, jump to today's date
                let today = chrono::Local::now().date_naive();
                self.recurring_settings_fields[2] = today.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            } else if let Ok(date) = chrono::NaiveDate::parse_from_str(&self.recurring_settings_fields[2], crate::model::DATE_FORMAT) {
                let new_date = date - chrono::Duration::days(1);
                self.recurring_settings_fields[2] = new_date.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            }
        }
    }

    pub(crate) fn increment_month_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if self.recurring_settings_fields[2].is_empty() {
                // If empty, jump to today's date
                let today = chrono::Local::now().date_naive();
                self.recurring_settings_fields[2] = today.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            } else if let Ok(date) = chrono::NaiveDate::parse_from_str(&self.recurring_settings_fields[2], crate::model::DATE_FORMAT) {
                // Add one month using chrono's date arithmetic
                let new_date = if date.month() == 12 {
                    NaiveDate::from_ymd_opt(date.year() + 1, 1, date.day())
                        .unwrap_or_else(|| NaiveDate::from_ymd_opt(date.year() + 1, 1, 28).unwrap())
                } else {
                    NaiveDate::from_ymd_opt(date.year(), date.month() + 1, date.day())
                        .unwrap_or_else(|| NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 28).unwrap())
                };
                self.recurring_settings_fields[2] = new_date.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            }
        }
    }

    pub(crate) fn decrement_month_recurring(&mut self) {
        if self.current_recurring_field == 2 {
            if self.recurring_settings_fields[2].is_empty() {
                // If empty, jump to today's date
                let today = chrono::Local::now().date_naive();
                self.recurring_settings_fields[2] = today.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            } else if let Ok(date) = chrono::NaiveDate::parse_from_str(&self.recurring_settings_fields[2], crate::model::DATE_FORMAT) {
                // Subtract one month using chrono's date arithmetic
                let new_date = if date.month() == 1 {
                    NaiveDate::from_ymd_opt(date.year() - 1, 12, date.day())
                        .unwrap_or_else(|| NaiveDate::from_ymd_opt(date.year() - 1, 12, 28).unwrap())
                } else {
                    NaiveDate::from_ymd_opt(date.year(), date.month() - 1, date.day())
                        .unwrap_or_else(|| NaiveDate::from_ymd_opt(date.year(), date.month() - 1, 28).unwrap())
                };
                self.recurring_settings_fields[2] = new_date.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None;
            }
        }
    }
} 