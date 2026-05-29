use super::state::App;
use crate::app::settings_types::{SettingType, SettingsState};
use crate::config::{save_settings, AppSettings};
use crate::persistence::{load_categories, load_transactions, save_transactions};
use crate::transaction_store::TransactionStore;
use chrono::Duration;
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

        // 1. SQLite Database Path (primary storage for transactions and categories)
        let database_path_str = self.database_path.to_string_lossy().to_string();
        let database_path_val = crate::validation::strip_path_quotes(&database_path_str);
        self.settings_state.add_setting(
            "database_path",
            "Database Path",
            database_path_val,
            SettingType::Path,
            "Absolute path to your SQLite database (transactions and categories).",
        );
        // 2. Manage Categories
        self.settings_state.add_setting(
            "manage_categories",
            "Manage Categories",
            "Open Category Catalog".to_string(),
            SettingType::Action,
            "Open the category catalog to add, edit, or delete categories.",
        );
        // 3. Import Transactions (CSV) — type a path and press Enter to merge it in.
        self.settings_state.add_setting(
            "import_transactions",
            "Import Transactions (CSV)",
            self.default_io_path_value(),
            SettingType::Path,
            "Type a CSV path and press Enter to import (new rows are added, duplicates skipped).",
        );
        // 4. Export Transactions (CSV) — type a destination and press Enter to write a CSV.
        self.settings_state.add_setting(
            "export_transactions",
            "Export Transactions (CSV)",
            self.default_io_path_value(),
            SettingType::Path,
            "Type a destination path and press Enter to export all transactions to CSV.",
        );

        // --- Monthly Summary View Section ---
        self.settings_state.add_setting(
            "header_monthly",
            "Monthly Summary View",
            "".to_string(),
            SettingType::SectionHeader,
            "",
        );

        // 5. Target Budget
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

        // 6. Hourly Rate
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

        // 7. Show Hours Toggle
        // Only show this if hourly rate is present
        if !hourly_rate_val.is_empty() {
            let show_hours_val = if loaded_settings.show_hours.unwrap_or(false) {
                "◀ Yes "
            } else {
                " No ▶"
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

        // 8. Fuzzy Search Mode
        let fuzzy_search_val = if loaded_settings.fuzzy_search_mode.unwrap_or(false) {
            "◀ Yes "
        } else {
            " No ▶"
        };
        self.settings_state.add_setting(
            "fuzzy_search_mode",
            "Fuzzy Search Categories",
            fuzzy_search_val.to_string(),
            SettingType::Toggle,
            "Toggle to enable fuzzy searching for categories/subcategories.",
        );

        // --- General Preferences Section ---
        self.settings_state.add_setting(
            "header_general",
            "General Preferences",
            "".to_string(),
            SettingType::SectionHeader,
            "",
        );

        // 9. Hide Help Bar
        let hide_help_bar_val = if loaded_settings.hide_help_bar.unwrap_or(false) {
            "◀ Yes "
        } else {
            " No ▶"
        };
        self.settings_state.add_setting(
            "hide_help_bar",
            "Hide Help Bar (NOT RECOMMENDED)",
            hide_help_bar_val.to_string(),
            SettingType::Toggle,
            "Toggle to hide the bottom help bar (Ctrl+H will still work).",
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

        self.clear_status_message();
    }

    pub(crate) fn exit_settings_mode(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.settings_state = SettingsState::default();
        self.clear_status_message();
    }

    pub(crate) fn save_settings(&mut self) {
        // Retrieve values from state
        let mut new_database_path_str = String::new();
        let mut target_budget_str = String::new();
        let mut hourly_rate_str = String::new();
        let mut show_hours_val = None;
        let mut fuzzy_search_val = None;
        let mut hide_help_bar_val = None;

        if let Some(val) = self.settings_state.get_value("database_path") {
            new_database_path_str = crate::validation::strip_path_quotes(val);
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
        if let Some(val) = self.settings_state.get_value("hide_help_bar") {
            hide_help_bar_val = Some(val.to_lowercase().contains("yes"));
        }

        // Validate Target Budget
        let target_budget = if target_budget_str.is_empty() {
            None
        } else {
            match crate::validation::validate_amount_string(&target_budget_str) {
                Ok(val) => Some(val),
                Err(msg) => {
                    self.set_status_message(format!("Error: Target budget - {}", msg), None);
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
                    self.set_status_message(format!("Error: Hourly rate - {}", msg), None);
                    return;
                }
            }
        };

        // Validate Path
        if new_database_path_str.is_empty() {
            self.set_status_message("Error: Database path cannot be empty.", None);
            return;
        }
        let new_database_path = PathBuf::from(&new_database_path_str);

        let seed_categories = if self.categories.is_empty() {
            load_categories().unwrap_or_default()
        } else {
            self.categories.clone()
        };
        if let Err(e) = Self::prepare_category_database_for_path_change(
            &self.database_path,
            &new_database_path,
            &seed_categories,
        ) {
            self.set_status_message(
                format!(
                    "Error preparing database '{}': {}. Check path and permissions.",
                    new_database_path.display(),
                    e
                ),
                None,
            );
            return;
        }

        // Save to Config. The legacy data file path is retained only so a one-time CSV
        // migration can still locate it and as the default for import/export.
        let settings = AppSettings {
            data_file_path: Some(self.data_file_path.to_string_lossy().to_string()),
            database_path: Some(new_database_path_str.clone()),
            target_budget,
            hourly_rate,
            show_hours: show_hours_val,
            fuzzy_search_mode: fuzzy_search_val,
            hide_help_bar: hide_help_bar_val,
        };
        if let Err(e) = save_settings(&settings) {
            self.set_status_message(format!("Error saving config file: {}", e), None);
            return;
        }

        // Point at the new database and reload everything from it.
        self.database_path = new_database_path.clone();
        if let Err(e) = self.reload_categories_from_store() {
            self.set_status_message(
                format!(
                    "Error loading categories from '{}': {}. Check database path and permissions.",
                    self.database_path.display(),
                    e
                ),
                None,
            );
            return;
        }
        if let Err(e) = self.reload_transactions_from_db() {
            self.set_status_message(
                format!(
                    "Error loading transactions from '{}': {}. Check database path and permissions.",
                    self.database_path.display(),
                    e
                ),
                None,
            );
            return;
        }

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

        self.set_status_message(
            format!("Settings saved. Database: {}", self.database_path.display()),
            Some(Duration::seconds(3)),
        );
        self.target_budget = target_budget;
        self.hourly_rate = hourly_rate;
        self.show_hours = show_hours_val.unwrap_or(false);
        self.fuzzy_search_mode = fuzzy_search_val.unwrap_or(false);
        self.hide_help_bar = hide_help_bar_val.unwrap_or(false);
    }

    /// Default value shown in the Import/Export path fields: the legacy data directory, which
    /// is the most likely place a user keeps a transactions CSV.
    fn default_io_path_value(&self) -> String {
        crate::validation::strip_path_quotes(&self.data_file_path.to_string_lossy())
    }

    /// Reset the currently-selected Import/Export path field back to the default location.
    pub(crate) fn reset_settings_io_path_to_default(&mut self) {
        let default = self.default_io_path_value();
        let idx = self.settings_state.selected_index;
        if let Some(item) = self.settings_state.items.get_mut(idx) {
            item.value = default;
            self.settings_state.edit_cursor = item.value.len();
        }
        self.set_status_message("Path reset to default location.", None);
    }

    pub(crate) fn reset_settings_database_path_to_default(&mut self) {
        let data_path_value =
            crate::validation::strip_path_quotes(&self.data_file_path.to_string_lossy());

        let default_path = if data_path_value.trim().is_empty() {
            Self::get_default_database_file_path().unwrap_or_else(|_| PathBuf::from("budget.db"))
        } else {
            Self::default_database_path_for_data_path(PathBuf::from(data_path_value).as_path())
        };

        let clean_path =
            crate::validation::strip_path_quotes(default_path.to_string_lossy().as_ref());

        if let Some(idx) = self
            .settings_state
            .items
            .iter()
            .position(|i| i.key == "database_path")
        {
            self.settings_state.items[idx].value = clean_path;
            if self.settings_state.selected_index == idx {
                self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
            }
        }

        self.set_status_message(
            "Database path reset to default for the current data file. Press Enter to save.",
            None,
        );
    }

    pub(crate) fn activate_selected_setting(&mut self) {
        let selected_key = self
            .settings_state
            .items
            .get(self.settings_state.selected_index)
            .map(|item| item.key.clone());

        match selected_key.as_deref() {
            Some("manage_categories") => self.open_category_catalog(),
            Some("import_transactions") => self.import_transactions(),
            Some("export_transactions") => self.export_transactions(),
            _ => self.save_settings(),
        }
    }

    /// Import transactions from a CSV chosen in the Import field. Generated recurring rows in
    /// the file are dropped (they are re-derived from sources); the rest are merged with the
    /// database, skipping exact duplicates.
    pub(crate) fn import_transactions(&mut self) {
        let raw = match self.settings_state.get_value("import_transactions") {
            Some(value) => value.clone(),
            None => {
                self.set_status_message("Error: import path field is missing.", None);
                return;
            }
        };
        let path_str = crate::validation::strip_path_quotes(&raw);
        if path_str.trim().is_empty() {
            self.set_status_message("Error: enter a CSV path to import.", None);
            return;
        }
        let path = PathBuf::from(&path_str);
        if !path.exists() {
            self.set_status_message(format!("Error: file '{}' not found.", path.display()), None);
            return;
        }

        let rows = match load_transactions(&path) {
            Ok(rows) => rows,
            Err(e) => {
                self.set_status_message(format!("Error reading '{}': {}", path.display(), e), None);
                return;
            }
        };
        let real_rows: Vec<_> = rows
            .into_iter()
            .filter(|tx| !tx.is_generated_from_recurring)
            .collect();

        let summary = match self.transaction_store().import_merge(&real_rows) {
            Ok(summary) => summary,
            Err(e) => {
                self.set_status_message(format!("Error importing transactions: {}", e), None);
                return;
            }
        };
        if let Err(e) = self.reload_transactions_from_db() {
            self.set_status_message(format!("Imported, but reloading failed: {}", e), None);
            return;
        }

        self.exit_settings_mode();
        self.filtered_indices = (0..self.transactions.len()).collect();
        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
        }
        self.set_status_message(
            format!(
                "Imported {} new, skipped {} duplicates.",
                summary.added, summary.skipped
            ),
            Some(Duration::seconds(4)),
        );
    }

    /// Export every transaction (including the materialized recurring occurrences shown in the
    /// app) to the CSV path chosen in the Export field.
    pub(crate) fn export_transactions(&mut self) {
        let raw = match self.settings_state.get_value("export_transactions") {
            Some(value) => value.clone(),
            None => {
                self.set_status_message("Error: export path field is missing.", None);
                return;
            }
        };
        let path_str = crate::validation::strip_path_quotes(&raw);
        if path_str.trim().is_empty() {
            self.set_status_message("Error: enter a destination path to export.", None);
            return;
        }
        let path = PathBuf::from(&path_str);

        match save_transactions(&self.transactions, &path) {
            Ok(_) => {
                let count = self.transactions.len();
                self.exit_settings_mode();
                self.set_status_message(
                    format!("Exported {} transactions to {}.", count, path.display()),
                    Some(Duration::seconds(4)),
                );
            }
            Err(e) => {
                self.set_status_message(
                    format!("Error exporting to '{}': {}", path.display(), e),
                    None,
                );
            }
        }
    }

    /// Generalized method to ensure a setting is visible or hidden based on a condition.
    /// This handles finding the correct position and maintaining list integrity.
    fn ensure_setting_visibility<F>(
        &mut self,
        target_key: &str,
        should_be_visible: bool,
        insert_after_key: &str,
        item_creator: F,
    ) where
        F: FnOnce() -> crate::app::settings_types::SettingItem,
    {
        let target_idx = self
            .settings_state
            .items
            .iter()
            .position(|i| i.key == target_key);

        if should_be_visible {
            if target_idx.is_none() {
                // Find where to insert
                if let Some(after_idx) = self
                    .settings_state
                    .items
                    .iter()
                    .position(|i| i.key == insert_after_key)
                {
                    let item = item_creator();
                    self.settings_state.items.insert(after_idx + 1, item);

                    // If we inserted before the selection, shift selection down
                    if self.settings_state.selected_index > after_idx {
                        self.settings_state.selected_index += 1;
                    }
                }
            }
        } else if let Some(idx) = target_idx {
            self.settings_state.items.remove(idx);

            // If we removed the item before or at the selection, shift selection up
            if self.settings_state.selected_index >= idx {
                self.settings_state.selected_index =
                    self.settings_state.selected_index.saturating_sub(1);
            }
        }
    }

    pub(crate) fn update_settings_visibility(&mut self) {
        // Rule 1: "show_hours" depends on "hourly_rate" having a value
        let hourly_rate_has_value = self
            .settings_state
            .get_value("hourly_rate")
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false);

        self.ensure_setting_visibility("show_hours", hourly_rate_has_value, "hourly_rate", || {
            crate::app::settings_types::SettingItem {
                key: "show_hours".to_string(),
                label: "Show Costs in Hours".to_string(),
                value: " No ▶".to_string(), // Default to No
                setting_type: crate::app::settings_types::SettingType::Toggle,
                help: "Toggle to display transaction amounts as hours worked.".to_string(),
            }
        });
    }
}
