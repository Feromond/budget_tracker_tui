use super::state::App;
use crate::config::{save_settings, AppSettings};
use crate::persistence::{load_transactions, save_transactions};
use std::path::PathBuf;

impl App {
    // --- Settings Mode Logic ---
    // Handles entering/exiting settings mode, saving settings, and resetting the data file path.
    pub(crate) fn enter_settings_mode(&mut self) {
        self.mode = crate::app::state::AppMode::Settings;
        self.input_field_content = self.data_file_path.to_string_lossy().to_string();
        self.input_field_cursor = self.input_field_content.len();
        self.status_message = None;
    }
    pub(crate) fn exit_settings_mode(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.input_field_content.clear();
        self.input_field_cursor = 0;
        self.status_message = None;
    }
    pub(crate) fn save_settings(&mut self) {
        // Save the new data file path in config and reload transactions from the new path.
        let new_path_str = self.input_field_content.trim();
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
        // Save the new data file path in config and reload transactions from the new path.
        let settings = AppSettings {
            data_file_path: Some(new_path_str.to_string()),
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
    }
    pub(crate) fn reset_settings_path_to_default(&mut self) {
        // Reset the data file path to the default location.
        match Self::get_default_data_file_path() {
            Ok(default_path) => {
                let path_str = default_path.to_string_lossy().to_string();
                self.input_field_content = path_str;
                self.input_field_cursor = self.input_field_content.len();
                self.status_message =
                    Some("Path reset to default. Press Enter to save.".to_string());
            }
            Err(e) => {
                let fallback_path = "transactions.csv";
                self.input_field_content = fallback_path.to_string();
                self.input_field_cursor = self.input_field_content.len();
                self.status_message = Some(format!(
                    "Error getting default path ({}). Reset to local '{}'. Press Enter to save.",
                    e, fallback_path
                ));
            }
        }
    }
}
