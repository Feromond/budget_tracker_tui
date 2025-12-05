use super::state::App;
use crate::model::TransactionType;
use ratatui::widgets::ListState;
use std::collections::HashSet;

impl App {
    // --- Category/Subcategory Selection Logic ---
    pub(crate) fn start_category_selection(&mut self) {
        // If fuzzy search is enabled, redirect to that mode
        if self.fuzzy_search_mode {
            self.start_fuzzy_selection();
            return;
        }
        self.type_to_select.clear();

        self.selecting_field_index = Some(4);
        self.mode = crate::app::state::AppMode::SelectingCategory;
        let current_type_str = self.add_edit_fields[3].trim();
        let Ok(current_type) = TransactionType::try_from(current_type_str) else {
            self.set_status_message("Error: Invalid transaction type for category lookup.", None);
            self.mode = if self.editing_index.is_some() {
                crate::app::state::AppMode::Editing
            } else {
                crate::app::state::AppMode::Adding
            };
            return;
        };
        let mut unique_categories: HashSet<String> = self
            .categories
            .iter()
            .filter(|cat_info| cat_info.transaction_type == current_type)
            .map(|cat_info| cat_info.category.clone())
            .collect();
        let mut options: Vec<String> = unique_categories.drain().collect();
        options.sort_unstable();
        self.current_selection_list = options;
        self.selection_list_state = ListState::default();
        if !self.current_selection_list.is_empty() {
            self.selection_list_state.select(Some(0));
        }
    }
    pub(crate) fn start_subcategory_selection(&mut self) {
        self.type_to_select.clear();
        self.selecting_field_index = Some(5);
        self.mode = crate::app::state::AppMode::SelectingSubcategory;
        let current_type_str = self.add_edit_fields[3].trim();
        let current_category = self.add_edit_fields[4].trim();
        let Ok(current_type) = TransactionType::try_from(current_type_str) else {
            self.set_status_message(
                "Error: Invalid transaction type for subcategory lookup.",
                None,
            );
            self.mode = if self.editing_index.is_some() {
                crate::app::state::AppMode::Editing
            } else {
                crate::app::state::AppMode::Adding
            };
            return;
        };
        if current_category.is_empty() || current_category.eq_ignore_ascii_case("Uncategorized") {
            self.current_selection_list = vec!["(None)".to_string()];
        } else {
            let mut unique_subcategories: HashSet<String> = self
                .categories
                .iter()
                .filter(|cat_info| {
                    cat_info.transaction_type == current_type
                        && cat_info.category.eq_ignore_ascii_case(current_category)
                        && !cat_info.subcategory.is_empty()
                })
                .map(|cat_info| cat_info.subcategory.clone())
                .collect();
            let mut options: Vec<String> = unique_subcategories.drain().collect();
            options.sort_unstable();
            options.insert(0, "(None)".to_string());
            self.current_selection_list = options;
        }
        self.selection_list_state = ListState::default();
        if !self.current_selection_list.is_empty() {
            self.selection_list_state.select(Some(0));
        }
    }
    pub(crate) fn confirm_selection(&mut self) {
        if let Some(selected_index) = self.selection_list_state.selected() {
            if let Some(field_index) = self.selecting_field_index {
                if let Some(selected_value) = self.current_selection_list.get(selected_index) {
                    let value_to_set = if field_index == 5 && selected_value == "(None)" {
                        ""
                    } else {
                        selected_value.as_str()
                    };
                    self.add_edit_fields[field_index] = value_to_set.to_string();
                    if field_index == 4 {
                        self.current_add_edit_field = 5;
                        self.start_subcategory_selection();
                        return;
                    } else if field_index == 5 {
                        self.current_add_edit_field = 0;
                    }
                }
            }
        }
        self.mode = if self.editing_index.is_some() {
            crate::app::state::AppMode::Editing
        } else {
            crate::app::state::AppMode::Adding
        };
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }
    pub(crate) fn cancel_selection(&mut self) {
        self.mode = if self.editing_index.is_some() {
            crate::app::state::AppMode::Editing
        } else {
            crate::app::state::AppMode::Adding
        };
        if let Some(field_index) = self.selecting_field_index {
            self.current_add_edit_field = field_index;
        }
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }
    pub(crate) fn select_next_list_item(&mut self) {
        let list_len = self.current_selection_list.len();
        if list_len == 0 {
            return;
        }
        let i = match self.selection_list_state.selected() {
            Some(i) => {
                if i >= list_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selection_list_state.select(Some(i));
    }
    pub(crate) fn select_previous_list_item(&mut self) {
        let list_len = self.current_selection_list.len();
        if list_len == 0 {
            return;
        }
        let i = match self.selection_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    list_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selection_list_state.select(Some(i));
    }

    pub(crate) fn handle_type_to_select(&mut self, c: char) {
        if let Some(index) =
            self.type_to_select
                .handle_char(c, &self.current_selection_list, |item| item.as_str())
        {
            self.selection_list_state.select(Some(index));
        }
    }
}
