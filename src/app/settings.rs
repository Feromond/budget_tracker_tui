use super::state::App;
use crate::config::{save_settings, AppSettings};
use crate::persistence::{load_transactions, save_transactions};
use std::path::PathBuf;

impl App {
    // --- Settings Mode Logic ---
    // Handles entering/exiting settings mode, saving settings, and resetting the data file path.
    pub(crate) fn enter_settings_mode(&mut self) {
        self.mode = crate::app::state::AppMode::Settings;
        self.settings_fields[0] = self.data_file_path.to_string_lossy().to_string();
        let loaded_settings = crate::config::load_settings().unwrap_or_default();
        self.settings_fields[1] = loaded_settings
            .target_budget
            .map(|v| v.to_string())
            .unwrap_or_default();
        self.current_settings_field = 0;
        self.input_field_cursor = self.settings_fields[0].len();
        self.status_message = None;
    }
    pub(crate) fn exit_settings_mode(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.settings_fields = Default::default();
        self.current_settings_field = 0;
        self.input_field_cursor = 0;
        self.status_message = None;
    }
    pub(crate) fn save_settings(&mut self) {
        // Save the new data file path in config and reload transactions from the new path.
        let new_path_str = self.settings_fields[0].trim();
        let target_budget_str = self.settings_fields[1].trim();
        let necessary_expenses_percentage_str = self.settings_fields[2].trim();
        let discretionary_expenses_percentage_str = self.settings_fields[3].trim();
        let savings_or_investment_percentage_str = self.settings_fields[4].trim();
        let tax_setaside_percentage_str = self.settings_fields[5].trim();
        let necessary_expenses_percentage = if necessary_expenses_percentage_str.is_empty() {
            None
        } else {
            match necessary_expenses_percentage_str.parse::<u8>() {
                Ok(val) => Some(val),
                _ => None,
            }
        };
        let discretionary_expenses_percentage = if discretionary_expenses_percentage_str.is_empty()
        {
            None
        } else {
            match discretionary_expenses_percentage_str.parse::<u8>() {
                Ok(val) => Some(val),
                _ => None,
            }
        };
        let saving_or_investment_percentage = if savings_or_investment_percentage_str.is_empty() {
            None
        } else {
            match savings_or_investment_percentage_str.parse::<u8>() {
                Ok(val) => Some(val),
                _ => None,
            }
        };
        let tax_setaside_percentage = if tax_setaside_percentage_str.is_empty() {
            None
        } else {
            match tax_setaside_percentage_str.parse::<u8>() {
                Ok(val) => Some(val),
                _ => None,
            }
        };
        let target_budget = if target_budget_str.is_empty() {
            None
        } else {
            match target_budget_str.parse::<f64>() {
                Ok(val) if val > 0.0 => Some(val),
                _ => {
                    self.status_message = Some(
                        "Error: Target budget must be a positive number or blank.".to_string(),
                    );
                    return;
                }
            }
        };
        if new_path_str.is_empty() {
            self.status_message = Some("Error: Path cannot be empty.".to_string());
            return;
        }
        let new_path = PathBuf::from(new_path_str);
        if !new_path.exists() {
            if let Err(e) = save_transactions(&self.transactions, &new_path) {
                self.status_message = Some(format!(
                    "Error creating transactions file '{}': {}. Check path and permissions.",
                    new_path.display(),
                    e
                ));
                return;
            }
        }
        // Return an error if the spending goals' total does not add up to 100 %
        let percentage: u8 = necessary_expenses_percentage.unwrap_or(0)
            + discretionary_expenses_percentage.unwrap_or(0)
            + saving_or_investment_percentage.unwrap_or(0)
            + tax_setaside_percentage.unwrap_or(0);
        if percentage != 100 && percentage != 0 {
            self.status_message = Some(String::from(
                "The percentages of your spending goals do not add up to 100%!",
            ));
            return;
        }
        // Save the new data file path and target budget in config (future: add to AppSettings)
        let settings = AppSettings {
            data_file_path: Some(new_path_str.to_string()),
            target_budget,
            necessary_expenses_percentage,
            discretionary_expenses_percentage,
            saving_or_investment_percentage,
            tax_setaside_percentage,
        };
        if let Err(e) = save_settings(&settings) {
            self.status_message = Some(format!("Error saving config file: {}", e));
            return;
        }
        // Reload transactions from the new path.
        self.data_file_path = new_path.clone();
        let txs = match load_transactions(&self.data_file_path) {
            Ok(tx) => tx,
            Err(e) => {
                self.status_message = Some(format!(
                    "Error loading transactions from '{}': {}. Check file format and permissions.",
                    self.data_file_path.display(),
                    e
                ));
                return;
            }
        };
        self.transactions = txs;
        self.exit_settings_mode();
        self.sort_transactions();
        self.filtered_indices = (0..self.transactions.len()).collect();
        if !self.filtered_indices.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
        self.calculate_monthly_summaries();
        self.calculate_category_summaries();
        self.status_message = Some(format!(
            "Settings saved. Data file set to: {}",
            self.data_file_path.display()
        ));
        self.target_budget = target_budget;
        self.input_field_cursor = 0;
    }
    pub(crate) fn reset_settings_path_to_default(&mut self) {
        // Reset the data file path to the default location.
        match Self::get_default_data_file_path() {
            Ok(default_path) => {
                let path_str = default_path.to_string_lossy().to_string();
                self.settings_fields[0] = path_str;
                self.current_settings_field = 0;
                self.input_field_cursor = self.settings_fields[0].len();
                self.status_message =
                    Some("Path reset to default. Press Enter to save.".to_string());
            }
            Err(e) => {
                let fallback_path = "transactions.csv";
                self.settings_fields[0] = fallback_path.to_string();
                self.current_settings_field = 0;
                self.input_field_cursor = self.settings_fields[0].len();
                self.status_message = Some(format!(
                    "Error getting default path ({}). Reset to local '{}'. Press Enter to save.",
                    e, fallback_path
                ));
            }
        }
    }
}
