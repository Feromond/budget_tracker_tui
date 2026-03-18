use super::state::App;
use crate::app::state::AppMode;
use crate::category_store::CategoryStore;
use crate::model::{CategoryDraft, CategoryRecord, TransactionType};
use crate::persistence::save_transactions;
use chrono::Duration;

impl App {
    pub(crate) fn open_category_catalog(&mut self) {
        if let Err(err) = self.reload_categories_from_store() {
            self.set_status_message(format!("Error loading categories: {}", err), None);
            return;
        }

        self.mode = AppMode::CategoryCatalog;
        self.editing_category_id = None;
        self.category_delete_id = None;
        let selection = if self.category_records.is_empty() {
            None
        } else {
            Some(
                self.category_table_state
                    .selected()
                    .unwrap_or(0)
                    .min(self.category_records.len() - 1),
            )
        };
        self.category_table_state.select(selection);
        self.clear_status_message();
    }

    pub(crate) fn exit_category_catalog(&mut self) {
        self.mode = AppMode::Settings;
        self.category_delete_id = None;
        self.clear_status_message();
    }

    pub(crate) fn next_category_record(&mut self) {
        let len = self.category_records.len();
        if len == 0 {
            return;
        }

        let index = match self.category_table_state.selected() {
            Some(current) if current + 1 < len => current + 1,
            _ => 0,
        };
        self.category_table_state.select(Some(index));
    }

    pub(crate) fn previous_category_record(&mut self) {
        let len = self.category_records.len();
        if len == 0 {
            return;
        }

        let index = match self.category_table_state.selected() {
            Some(0) | None => len - 1,
            Some(current) => current - 1,
        };
        self.category_table_state.select(Some(index));
    }

    pub(crate) fn start_adding_category(&mut self) {
        self.mode = AppMode::CategoryEditor;
        self.editing_category_id = None;
        self.current_category_field = 0;
        self.category_edit_fields = [
            TransactionType::Expense.to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        self.category_edit_cursor = self.category_edit_fields[0].len();
        self.clear_status_message();
    }

    pub(crate) fn start_editing_category(&mut self) {
        let Some(record) = self.selected_category_record().cloned() else {
            self.set_status_message("Select a category first.", None);
            return;
        };

        self.mode = AppMode::CategoryEditor;
        self.editing_category_id = Some(record.id);
        self.current_category_field = 0;
        let draft = record.to_draft();
        self.category_edit_fields = [
            draft.transaction_type.to_string(),
            draft.category,
            draft.subcategory,
            draft.tag.unwrap_or_default(),
            draft
                .target_budget
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
        ];
        self.category_edit_cursor = self.category_edit_fields[0].len();
        self.clear_status_message();
    }

    pub(crate) fn exit_category_editor(&mut self, cancelled: bool) {
        self.mode = AppMode::CategoryCatalog;
        self.editing_category_id = None;
        self.current_category_field = 0;
        self.category_edit_fields = Default::default();
        self.category_edit_cursor = 0;

        if cancelled {
            self.set_status_message("Category edit cancelled.", Some(Duration::seconds(3)));
        } else {
            self.clear_status_message();
        }
    }

    pub(crate) fn next_category_field(&mut self) {
        self.current_category_field =
            (self.current_category_field + 1) % self.category_edit_fields.len();
        self.category_edit_cursor = self.category_edit_fields[self.current_category_field].len();
    }

    pub(crate) fn previous_category_field(&mut self) {
        if self.current_category_field == 0 {
            self.current_category_field = self.category_edit_fields.len() - 1;
        } else {
            self.current_category_field -= 1;
        }
        self.category_edit_cursor = self.category_edit_fields[self.current_category_field].len();
    }

    pub(crate) fn toggle_category_transaction_type(&mut self) {
        if self.current_category_field != 0 {
            return;
        }

        self.category_edit_fields[0] =
            if self.category_edit_fields[0].eq_ignore_ascii_case("income") {
                TransactionType::Expense.to_string()
            } else {
                TransactionType::Income.to_string()
            };
        self.category_edit_cursor = self.category_edit_fields[0].len();
    }

    pub(crate) fn prepare_delete_category(&mut self) {
        let Some(record) = self.selected_category_record().cloned() else {
            self.set_status_message("Select a category first.", None);
            return;
        };

        self.category_delete_id = Some(record.id);
        self.mode = AppMode::ConfirmCategoryDelete;
        self.set_status_message(
            format!(
                "Delete {} / {}? Press y to confirm.",
                record.category,
                if record.subcategory.is_empty() {
                    "(No subcategory)"
                } else {
                    record.subcategory.as_str()
                }
            ),
            None,
        );
    }

    pub(crate) fn cancel_delete_category(&mut self) {
        self.mode = AppMode::CategoryCatalog;
        self.category_delete_id = None;
        self.clear_status_message();
    }

    pub(crate) fn confirm_delete_category(&mut self) {
        let Some(id) = self.category_delete_id else {
            self.cancel_delete_category();
            return;
        };

        let Some(record) = self
            .category_records
            .iter()
            .find(|record| record.id == id)
            .cloned()
        else {
            self.cancel_delete_category();
            self.set_status_message("The selected category no longer exists.", None);
            return;
        };

        let store = self.category_store();
        if let Err(err) = store.delete(id) {
            self.set_status_message(format!("Error deleting category: {}", err), None);
            return;
        }

        crate::app::util::apply_category_delete(&mut self.transactions, &record);

        if let Err(err) = save_transactions(&self.transactions, &self.data_file_path) {
            self.set_status_message(
                format!("Category deleted, but saving transactions failed: {}", err),
                None,
            );
            return;
        }

        if let Err(err) = self.reload_categories_from_store() {
            self.set_status_message(
                format!("Category deleted, but refresh failed: {}", err),
                None,
            );
            return;
        }

        self.apply_filter();
        self.calculate_monthly_summaries();
        self.calculate_category_summaries();
        self.mode = AppMode::CategoryCatalog;
        self.category_delete_id = None;
        self.clamp_category_catalog_selection();
        self.set_status_message("Category deleted successfully.", Some(Duration::seconds(3)));
    }

    pub(crate) fn save_category(&mut self) {
        let draft = match self.build_category_draft_from_editor() {
            Ok(draft) => draft,
            Err(message) => {
                self.set_status_message(format!("Error: {}", message), None);
                return;
            }
        };

        let existing_record = self.editing_category_id.and_then(|id| {
            self.category_records
                .iter()
                .find(|record| record.id == id)
                .cloned()
        });
        let editing_category_id = self.editing_category_id;

        let store = self.category_store();
        let result = if let Some(id) = editing_category_id {
            store.update(id, &draft).map(|_| id)
        } else {
            store.insert(&draft).map(|record| record.id)
        };

        let saved_id = match result {
            Ok(saved_id) => saved_id,
            Err(err) => {
                self.set_status_message(format!("Error saving category: {}", err), None);
                return;
            }
        };

        if let Some(old_record) = existing_record {
            crate::app::util::apply_category_update(&mut self.transactions, &old_record, &draft);
        }

        if let Err(err) = save_transactions(&self.transactions, &self.data_file_path) {
            self.set_status_message(
                format!("Category saved, but saving transactions failed: {}", err),
                None,
            );
            return;
        }

        if let Err(err) = self.reload_categories_from_store() {
            self.set_status_message(format!("Category saved, but refresh failed: {}", err), None);
            return;
        }

        self.apply_filter();
        self.calculate_monthly_summaries();
        self.calculate_category_summaries();
        self.mode = AppMode::CategoryCatalog;
        self.editing_category_id = Some(saved_id);
        self.current_category_field = 0;
        self.category_edit_fields = Default::default();
        self.category_edit_cursor = 0;
        self.select_saved_category();
        self.editing_category_id = None;
        self.set_status_message("Category saved successfully.", Some(Duration::seconds(3)));
    }

    fn selected_category_record(&self) -> Option<&CategoryRecord> {
        self.category_table_state
            .selected()
            .and_then(|index| self.category_records.get(index))
    }

    fn clamp_category_catalog_selection(&mut self) {
        let selection = if self.category_records.is_empty() {
            None
        } else {
            Some(
                self.category_table_state
                    .selected()
                    .unwrap_or(0)
                    .min(self.category_records.len() - 1),
            )
        };
        self.category_table_state.select(selection);
    }

    fn select_saved_category(&mut self) {
        let selection = self.editing_category_id.and_then(|id| {
            self.category_records
                .iter()
                .position(|record| record.id == id)
        });

        if selection.is_some() {
            self.category_table_state.select(selection);
        } else {
            self.clamp_category_catalog_selection();
        }
    }

    fn build_category_draft_from_editor(&self) -> Result<CategoryDraft, String> {
        let transaction_type = TransactionType::try_from(self.category_edit_fields[0].trim())
            .map_err(|_| "Transaction type must be Income or Expense.".to_string())?;
        let category = self.category_edit_fields[1].trim().to_string();
        let subcategory = self.category_edit_fields[2].trim().to_string();
        let tag = self.category_edit_fields[3].trim();
        let target_budget_str = self.category_edit_fields[4].trim();

        if category.is_empty() {
            return Err("Category cannot be empty.".to_string());
        }

        let target_budget = if target_budget_str.is_empty() {
            None
        } else {
            Some(crate::validation::validate_amount_string(
                target_budget_str,
            )?)
        };

        Ok(CategoryDraft {
            transaction_type,
            category,
            subcategory,
            tag: if tag.is_empty() {
                None
            } else {
                Some(tag.to_string())
            },
            target_budget,
        })
    }
}
