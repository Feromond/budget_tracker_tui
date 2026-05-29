use super::state::{App, AppMode};
use crate::persistence::{load_transactions, save_transactions};
use crate::transaction_store::TransactionStore;
use chrono::Duration;
use std::path::PathBuf;

impl App {
    pub(crate) fn open_transaction_io(&mut self, mode: AppMode) {
        self.mode = mode;
        self.io_path_input = self.default_io_path_value();
        self.io_path_cursor = self.io_path_input.len();
        self.clear_status_message();
    }

    pub(crate) fn cancel_transaction_io(&mut self) {
        self.mode = AppMode::Settings;
        self.io_path_input.clear();
        self.io_path_cursor = 0;
        self.clear_status_message();
    }

    pub(crate) fn reset_transaction_io_path(&mut self) {
        self.io_path_input = self.default_io_path_value();
        self.io_path_cursor = self.io_path_input.len();
    }

    pub(crate) fn clear_transaction_io_path(&mut self) {
        self.io_path_input.clear();
        self.io_path_cursor = 0;
    }

    /// Defaults the prompt to the legacy data directory, the likeliest place to keep a CSV.
    fn default_io_path_value(&self) -> String {
        crate::validation::strip_path_quotes(&self.data_file_path.to_string_lossy())
    }

    pub(crate) fn import_transactions(&mut self) {
        let path_str = crate::validation::strip_path_quotes(&self.io_path_input);
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
        // Generated occurrences are re-derived from their sources, so only real rows are merged.
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

    pub(crate) fn export_transactions(&mut self) {
        let path_str = crate::validation::strip_path_quotes(&self.io_path_input);
        if path_str.trim().is_empty() {
            self.set_status_message("Error: enter a destination path to export.", None);
            return;
        }
        let path = PathBuf::from(&path_str);

        // Export the materialized view (real rows plus generated occurrences) for a complete CSV.
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
}
