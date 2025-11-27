use super::state::App;
use crate::app::settings_types::{SettingType, SettingsState};
use crate::config::{save_settings, AppSettings};
use crate::persistence::{load_transactions, save_transactions};
use std::path::PathBuf;

impl App {
    // --- Settings Mode Logic ---
    // Handles entering/exiting settings mode, saving settings, and resetting the data file path.
    pub(crate) fn enter_settings_mode(&mut self) {
        self.mode = crate::app::state::AppMode::Settings;

        // Initialize Settings State
        self.settings_state = SettingsState::new();

        // Load Config
        let loaded_settings = crate::config::load_settings().unwrap_or_default();

        // --- Data Management Section ---
        self.settings_state.add_setting(
            "header_data",
            "Data Management",
            "".to_string(),
            SettingType::SectionHeader,
            "",
        );

        // 1. Data File Path
        let path_str = self.data_file_path.to_string_lossy().to_string();
        let path_val = crate::validation::strip_path_quotes(&path_str);
        self.settings_state.add_setting(
            "data_file_path",
            "Data File Path",
            path_val,
            SettingType::Path,
            "Absolute path to your transactions CSV file.",
        );

        // --- Monthly Summary View Section ---
        self.settings_state.add_setting(
            "header_monthly",
            "Monthly Summary View",
            "".to_string(),
            SettingType::SectionHeader,
            "",
        );

        // 2. Target Budget
        let budget_val = loaded_settings
            .target_budget
            .map(|v| v.to_string())
            .unwrap_or_default();
        self.settings_state.add_setting(
            "target_budget",
            "Target Budget",
            budget_val,
            SettingType::Number,
            "Monthly spending goal. Displayed in Monthly Summary view only when cumulative mode.",
        );

        // --- Transaction View Section ---
        self.settings_state.add_setting(
            "header_transactions",
            "Transaction View",
            "".to_string(),
            SettingType::SectionHeader,
            "",
        );

        // 3. Hourly Rate
        let hourly_rate_val = loaded_settings
            .hourly_rate
            .map(|v| v.to_string())
            .unwrap_or_default();
        self.settings_state.add_setting(
            "hourly_rate",
            "Hourly Rate ($)",
            hourly_rate_val.clone(),
            SettingType::Number,
            "Optional. Enter your hourly earning rate to see costs in hours.",
        );

        // 4. Show Hours Toggle
        // Only show this if hourly rate is present
        if !hourly_rate_val.is_empty() {
            let show_hours_val = if loaded_settings.show_hours.unwrap_or(false) {
                "< Yes >"
            } else {
                "< No >"
            };
            self.settings_state.add_setting(
                "show_hours",
                "Show Costs in Hours",
                show_hours_val.to_string(),
                SettingType::Toggle,
                "Toggle to display transaction amounts as hours worked.",
            );
        }

        // --- Input Preferences Section ---
        self.settings_state.add_setting(
            "header_input",
            "Input Preferences",
            "".to_string(),
            SettingType::SectionHeader,
            "",
        );

        // 5. Fuzzy Search Mode
        let fuzzy_search_val = if loaded_settings.fuzzy_search_mode.unwrap_or(false) {
            "< Yes >"
        } else {
            "< No >"
        };
        self.settings_state.add_setting(
            "fuzzy_search_mode",
            "Fuzzy Search Categories",
            fuzzy_search_val.to_string(),
            SettingType::Toggle,
            "Toggle to enable fuzzy searching for categories/subcategories.",
        );

        // Select first valid item (skip headers)
        self.settings_state.selected_index = 0;
        while self.settings_state.selected_index < self.settings_state.items.len() {
            if self.settings_state.items[self.settings_state.selected_index].setting_type
                != SettingType::SectionHeader
            {
                break;
            }
            self.settings_state.selected_index += 1;
        }

        if let Some(item) = self
            .settings_state
            .items
            .get(self.settings_state.selected_index)
        {
            self.settings_state.edit_cursor = item.value.len();
        }

        self.status_message = None;
    }

    pub(crate) fn exit_settings_mode(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.settings_state = SettingsState::default();
        self.status_message = None;
    }

    pub(crate) fn save_settings(&mut self) {
        // Retrieve values from state
        let mut new_path_str = String::new();
        let mut target_budget_str = String::new();
        let mut hourly_rate_str = String::new();
        let mut show_hours_val = None;
        let mut fuzzy_search_val = None;

        if let Some(val) = self.settings_state.get_value("data_file_path") {
            new_path_str = crate::validation::strip_path_quotes(val);
        }
        if let Some(val) = self.settings_state.get_value("target_budget") {
            target_budget_str = val.trim().to_string();
        }
        if let Some(val) = self.settings_state.get_value("hourly_rate") {
            hourly_rate_str = val.trim().to_string();
        }
        if let Some(val) = self.settings_state.get_value("show_hours") {
            show_hours_val = Some(val.to_lowercase().contains("yes"));
        }
        if let Some(val) = self.settings_state.get_value("fuzzy_search_mode") {
            fuzzy_search_val = Some(val.to_lowercase().contains("yes"));
        }

        // Validate Target Budget
        let target_budget = if target_budget_str.is_empty() {
            None
        } else {
            match crate::validation::validate_amount_string(&target_budget_str) {
                Ok(val) => Some(val),
                Err(msg) => {
                    self.status_message = Some(format!("Error: Target budget - {}", msg));
                    return;
                }
            }
        };

        // Validate Hourly Rate
        let hourly_rate = if hourly_rate_str.is_empty() {
            None
        } else {
            match crate::validation::validate_amount_string(&hourly_rate_str) {
                Ok(val) => Some(val),
                Err(msg) => {
                    self.status_message = Some(format!("Error: Hourly rate - {}", msg));
                    return;
                }
            }
        };

        // Validate Path
        if new_path_str.is_empty() {
            self.status_message = Some("Error: Path cannot be empty.".to_string());
            return;
        }
        let new_path = PathBuf::from(&new_path_str);
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

        // Save to Config
        let settings = AppSettings {
            data_file_path: Some(new_path_str.clone()),
            target_budget,
            hourly_rate,
            show_hours: show_hours_val,
            fuzzy_search_mode: fuzzy_search_val,
        };
        if let Err(e) = save_settings(&settings) {
            self.status_message = Some(format!("Error saving config file: {}", e));
            return;
        }

        // Reload Transactions
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

        // Re-init app state (sorting, summaries)
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
        self.hourly_rate = hourly_rate;
        self.show_hours = show_hours_val.unwrap_or(false);
        self.fuzzy_search_mode = fuzzy_search_val.unwrap_or(false);
    }

    pub(crate) fn reset_settings_path_to_default(&mut self) {
        match Self::get_default_data_file_path() {
            Ok(default_path) => {
                let path_str = default_path.to_string_lossy().to_string();
                let clean_path = crate::validation::strip_path_quotes(&path_str);

                // Find index
                if let Some(idx) = self
                    .settings_state
                    .items
                    .iter()
                    .position(|i| i.key == "data_file_path")
                {
                    self.settings_state.items[idx].value = clean_path;
                    if self.settings_state.selected_index == idx {
                        self.settings_state.edit_cursor =
                            self.settings_state.items[idx].value.len();
                    }
                }

                self.status_message =
                    Some("Path reset to default. Press Enter to save.".to_string());
            }
            Err(e) => {
                let fallback_path = "transactions.csv";

                if let Some(idx) = self
                    .settings_state
                    .items
                    .iter()
                    .position(|i| i.key == "data_file_path")
                {
                    self.settings_state.items[idx].value = fallback_path.to_string();
                    if self.settings_state.selected_index == idx {
                        self.settings_state.edit_cursor =
                            self.settings_state.items[idx].value.len();
                    }
                }

                self.status_message = Some(format!(
                    "Error getting default path ({}). Reset to local '{}'. Press Enter to save.",
                    e, fallback_path
                ));
            }
        }
    }
}
