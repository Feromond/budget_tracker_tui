use super::state::App;
use crate::model::TransactionType;
use ratatui::widgets::ListState;
use std::collections::HashSet;

impl App {
    // --- Fuzzy Search Logic ---

    pub(crate) fn start_fuzzy_selection(&mut self) {
        self.selecting_field_index = Some(4); // Start pretending we are selecting category
        self.mode = crate::app::state::AppMode::FuzzyFinding;
        self.search_query = String::new();
        self.update_fuzzy_search_results();
        
        // Reset list state
        self.selection_list_state = ListState::default();
        if !self.current_selection_list.is_empty() {
            self.selection_list_state.select(Some(0));
        }
    }

    pub(crate) fn update_fuzzy_search_results(&mut self) {
        let current_type_str = self.add_edit_fields[3].trim();
        let Ok(current_type) = TransactionType::try_from(current_type_str) else {
            self.current_selection_list.clear();
            return;
        };

        let query = self.search_query.to_lowercase();
        
        // Collect all (Category, Subcategory) pairs for the transaction type
        // If subcategory is empty, just use category
        // Format in list: "Category > Subcategory" or just "Category"
        
        let mut options: Vec<String> = Vec::new();
        let mut unique_pairs: HashSet<(String, String)> = HashSet::new();

        for cat_info in &self.categories {
            if cat_info.transaction_type == current_type {
                unique_pairs.insert((cat_info.category.clone(), cat_info.subcategory.clone()));
            }
        }

        for (cat, sub) in unique_pairs {
            let display_str = if sub.is_empty() {
                cat.clone()
            } else {
                format!("{} > {}", cat, sub)
            };

            if query.is_empty() || display_str.to_lowercase().contains(&query) {
                options.push(display_str);
            }
        }

        options.sort_unstable();
        self.current_selection_list = options;
        
        // Adjust selection if out of bounds
        let len = self.current_selection_list.len();
        if len > 0 {
            let current = self.selection_list_state.selected().unwrap_or(0);
             if current >= len {
                self.selection_list_state.select(Some(len - 1));
            } else {
                self.selection_list_state.select(Some(current));
            }
        } else {
            self.selection_list_state.select(None);
        }
    }

    pub(crate) fn confirm_fuzzy_selection(&mut self) {
        if let Some(selected_index) = self.selection_list_state.selected() {
            if let Some(selected_value) = self.current_selection_list.get(selected_index) {
                // Parse "Category > Subcategory" or "Category"
                let parts: Vec<&str> = selected_value.split(" > ").collect();
                let category = parts[0];
                let subcategory = if parts.len() > 1 { parts[1] } else { "" };

                // Set fields
                // Index 4 is Category, 5 is Subcategory
                self.add_edit_fields[4] = category.to_string();
                self.add_edit_fields[5] = subcategory.to_string();
                self.current_add_edit_field = 0; 
            }
        }
        
        self.exit_fuzzy_mode();
    }

    pub(crate) fn cancel_fuzzy_selection(&mut self) {
        self.exit_fuzzy_mode();
        // Return to category field focus
         self.current_add_edit_field = 4;
    }
    
    fn exit_fuzzy_mode(&mut self) {
         self.mode = if self.editing_index.is_some() {
            crate::app::state::AppMode::Editing
        } else {
            crate::app::state::AppMode::Adding
        };
        self.selecting_field_index = None;
        self.current_selection_list.clear();
        self.search_query.clear();
    }
}
