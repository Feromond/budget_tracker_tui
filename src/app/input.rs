use super::state::App;
use crate::model::DATE_FORMAT;
use chrono::NaiveDate;

impl App {
    // --- Input Handling ---
    // Handles cursor movement and character insertion/deletion for the generic input field and add/edit fields.
    pub(crate) fn move_cursor_left(&mut self) {
        if self.input_field_cursor > 0 {
            self.input_field_cursor -= 1;
        }
    }
    pub(crate) fn move_cursor_right(&mut self) {
        if self.input_field_cursor < self.input_field_content.len() {
            self.input_field_cursor += 1;
        }
    }
    pub(crate) fn insert_char_at_cursor(&mut self, c: char) {
        self.input_field_content.insert(self.input_field_cursor, c);
        self.move_cursor_right();
    }
    pub(crate) fn delete_char_before_cursor(&mut self) {
        if self.input_field_cursor > 0 {
            self.move_cursor_left();
            self.input_field_content.remove(self.input_field_cursor);
        }
    }
    pub(crate) fn delete_char_after_cursor(&mut self) {
        if self.input_field_cursor < self.input_field_content.len() {
            self.input_field_content.remove(self.input_field_cursor);
        }
    }
    pub(crate) fn insert_char_add_edit(&mut self, c: char) {
        let current_field = self.current_add_edit_field;
        let field_content = &mut self.add_edit_fields[current_field];

        match current_field {
            0 => {
                // Date field - use centralized validation
                if let Some(new_content) =
                    crate::validation::validate_and_insert_date_char(field_content, c)
                {
                    *field_content = new_content;
                }
            }
            2 => {
                // Amount field - use centralized validation
                crate::validation::insert_amount_char(field_content, c);
            }
            _ => {
                // Default behavior for other fields
                field_content.push(c);
            }
        }
    }
    pub(crate) fn delete_char_add_edit(&mut self) {
        let current_field = self.current_add_edit_field;
        let field_content = &mut self.add_edit_fields[current_field];

        if current_field == 0 {
            // Date field - use centralized backspace handling
            crate::validation::handle_date_backspace(field_content);
        } else if !field_content.is_empty() {
            // Default behavior for other fields
            field_content.pop();
        }
    }
    pub(crate) fn next_add_edit_field(&mut self) {
        // Move to the next add/edit field
        let next_field = (self.current_add_edit_field + 1) % self.add_edit_fields.len();
        self.current_add_edit_field = next_field;
    }
    pub(crate) fn previous_add_edit_field(&mut self) {
        // Move to the previous add/edit field
        if self.current_add_edit_field == 0 {
            self.current_add_edit_field = self.add_edit_fields.len() - 1;
        } else {
            self.current_add_edit_field -= 1;
        }
    }

    // --- Settings Input ---

    fn strip_quotes_from_current_setting(&mut self) {
        let idx = self.settings_state.selected_index;
        if let Some(item) = self.settings_state.items.get_mut(idx) {
            if item.setting_type == crate::app::settings_types::SettingType::Path {
                let stripped = crate::validation::strip_path_quotes(&item.value);
                item.value = stripped;
                self.settings_state.edit_cursor = item.value.len();
            }
        }
    }

    pub(crate) fn next_settings_field(&mut self) {
        self.strip_quotes_from_current_setting();
        
        let len = self.settings_state.items.len();
        if len == 0 { return; }

        loop {
            self.settings_state.selected_index = (self.settings_state.selected_index + 1) % len;
            let idx = self.settings_state.selected_index;
            
            // Skip headers
            if self.settings_state.items[idx].setting_type != crate::app::settings_types::SettingType::SectionHeader {
                 self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
                 break;
            }
        }
    }

    pub(crate) fn previous_settings_field(&mut self) {
        self.strip_quotes_from_current_setting();
        
        let len = self.settings_state.items.len();
        if len == 0 { return; }
        
        loop {
            if self.settings_state.selected_index == 0 {
                 self.settings_state.selected_index = len - 1;
            } else {
                 self.settings_state.selected_index -= 1;
            }
            let idx = self.settings_state.selected_index;
            
             // Skip headers
            if self.settings_state.items[idx].setting_type != crate::app::settings_types::SettingType::SectionHeader {
                 self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
                 break;
            }
        }
    }

    pub(crate) fn move_cursor_left_settings(&mut self) {
        let idx = self.settings_state.selected_index;
        if let Some(item) = self.settings_state.items.get_mut(idx) {
            if item.setting_type == crate::app::settings_types::SettingType::Toggle {
                // Left sets to No
                item.value = "< No >".to_string();
                self.settings_state.edit_cursor = item.value.len();
                return;
            }
        }

        if self.settings_state.edit_cursor > 0 {
            self.settings_state.edit_cursor -= 1;
        }
    }

    pub(crate) fn move_cursor_right_settings(&mut self) {
        let idx = self.settings_state.selected_index;
        if let Some(item) = self.settings_state.items.get_mut(idx) {
            if item.setting_type == crate::app::settings_types::SettingType::Toggle {
                // Right sets to Yes
                item.value = "< Yes >".to_string();
                self.settings_state.edit_cursor = item.value.len();
                return;
            }
            if self.settings_state.edit_cursor < item.value.len() {
                self.settings_state.edit_cursor += 1;
            }
        }
    }

    pub(crate) fn insert_char_settings(&mut self, c: char) {
        let idx = self.settings_state.selected_index;
        if idx >= self.settings_state.items.len() { return; }
        
        let setting_type = self.settings_state.items[idx].setting_type.clone();
        
        match setting_type {
            crate::app::settings_types::SettingType::SectionHeader => {
                 // Do nothing
            },
            crate::app::settings_types::SettingType::Number => {
                 crate::validation::insert_amount_char(&mut self.settings_state.items[idx].value, c);
                 self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
            },
            crate::app::settings_types::SettingType::Path => {
                let item = &mut self.settings_state.items[idx];
                item.value.insert(self.settings_state.edit_cursor, c);
                
                let original_len = item.value.len();
                let stripped = crate::validation::strip_path_quotes(&item.value);
                let new_len = stripped.len();
                item.value = stripped;
                
                 let chars_removed = original_len - new_len;
                if chars_removed > 0 {
                    if self.settings_state.edit_cursor > chars_removed {
                         self.settings_state.edit_cursor -= chars_removed;
                    } else {
                         self.settings_state.edit_cursor = 0;
                    }
                    self.settings_state.edit_cursor = item.value.len();
                } else {
                    self.settings_state.edit_cursor += 1;
                }
            },
            crate::app::settings_types::SettingType::Toggle => {
                // Removed char input for toggle to reduce confusion, as per user request.
                // Use Left/Right arrows instead.
            }
        }
    }

    pub(crate) fn delete_char_settings(&mut self) {
        let idx = self.settings_state.selected_index;
        if idx >= self.settings_state.items.len() { return; }

        let setting_type = self.settings_state.items[idx].setting_type.clone();
        
        match setting_type {
             crate::app::settings_types::SettingType::SectionHeader => {},
             crate::app::settings_types::SettingType::Number => {
                 self.settings_state.items[idx].value.pop();
                 self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
             },
             crate::app::settings_types::SettingType::Toggle => {
                 // No-op for delete on toggle
             },
             _ => {
                 let cursor = self.settings_state.edit_cursor;
                 let item = &mut self.settings_state.items[idx];
                 if cursor > 0 && !item.value.is_empty() {
                     if cursor <= item.value.len() {
                         item.value.remove(cursor - 1);
                         self.settings_state.edit_cursor -= 1;
                     }
                     
                     if setting_type == crate::app::settings_types::SettingType::Path {
                         let stripped = crate::validation::strip_path_quotes(&item.value);
                         item.value = stripped;
                         if self.settings_state.edit_cursor > item.value.len() {
                             self.settings_state.edit_cursor = item.value.len();
                         }
                     }
                 }
             }
        }
    }

    pub(crate) fn clear_settings_field(&mut self) {
         let idx = self.settings_state.selected_index;
         if let Some(item) = self.settings_state.items.get_mut(idx) {
             if item.setting_type != crate::app::settings_types::SettingType::SectionHeader {
                 item.value.clear();
                 self.settings_state.edit_cursor = 0;
             }
         }
    }

    // --- Date Navigation ---
    // Date field navigation utilities for input fields - public for use by other modules
    pub fn increment_date_field(&self, field: &str) -> Option<String> {
        if field.is_empty() {
            let today = chrono::Local::now().date_naive();
            return Some(today.format(DATE_FORMAT).to_string());
        }

        if let Ok(date) = NaiveDate::parse_from_str(field, DATE_FORMAT) {
            let new_date = date + chrono::Duration::days(1);
            Some(new_date.format(DATE_FORMAT).to_string())
        } else {
            None
        }
    }

    pub fn decrement_date_field(&self, field: &str) -> Option<String> {
        if field.is_empty() {
            let today = chrono::Local::now().date_naive();
            return Some(today.format(DATE_FORMAT).to_string());
        }

        if let Ok(date) = NaiveDate::parse_from_str(field, DATE_FORMAT) {
            let new_date = date - chrono::Duration::days(1);
            Some(new_date.format(DATE_FORMAT).to_string())
        } else {
            None
        }
    }

    pub fn increment_month_field(&self, field: &str) -> Option<String> {
        if field.is_empty() {
            let today = chrono::Local::now().date_naive();
            return Some(today.format(DATE_FORMAT).to_string());
        }

        if let Ok(date) = NaiveDate::parse_from_str(field, DATE_FORMAT) {
            let new_date = crate::validation::add_months(date, 1);
            Some(new_date.format(DATE_FORMAT).to_string())
        } else {
            None
        }
    }

    pub fn decrement_month_field(&self, field: &str) -> Option<String> {
        if field.is_empty() {
            let today = chrono::Local::now().date_naive();
            return Some(today.format(DATE_FORMAT).to_string());
        }

        if let Ok(date) = NaiveDate::parse_from_str(field, DATE_FORMAT) {
            let new_date = crate::validation::add_months(date, -1);
            Some(new_date.format(DATE_FORMAT).to_string())
        } else {
            None
        }
    }
}
