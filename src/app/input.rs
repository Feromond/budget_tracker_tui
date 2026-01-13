use super::state::App;
use crate::app::state::AppMode;
use crate::model::DATE_FORMAT;
use chrono::NaiveDate;

#[derive(PartialEq)]
pub enum InputType {
    Text,
    Date,
    Amount,
}

impl App {
    // Helper to get the active mutable input field state based on the current mode and field index.
    // Returns: (content_string, cursor_position, input_type)
    fn get_active_input_mut(&mut self) -> Option<(&mut String, &mut usize, InputType)> {
        match self.mode {
            AppMode::Filtering => Some((
                &mut self.simple_filter_content,
                &mut self.simple_filter_cursor,
                InputType::Text,
            )),
            AppMode::Adding | AppMode::Editing => {
                let idx = self.current_add_edit_field;
                let input_type = match idx {
                    0 => InputType::Date,
                    2 => InputType::Amount,
                    1 => InputType::Text,
                    _ => return None, // Other fields (Type, Category, Subcategory) are not standard text inputs
                };
                Some((
                    &mut self.add_edit_fields[idx],
                    &mut self.add_edit_cursor,
                    input_type,
                ))
            }
            AppMode::AdvancedFiltering => {
                let idx = self.current_advanced_filter_field;
                let input_type = match idx {
                    0 | 1 => InputType::Date,
                    2 => InputType::Text, // Description
                    6 | 7 => InputType::Amount,
                    _ => return None, // Category(3), Subcategory(4), Type(5) are selections/toggles
                };
                Some((
                    &mut self.advanced_filter_fields[idx],
                    &mut self.advanced_filter_cursor,
                    input_type,
                ))
            }
            _ => None,
        }
    }

    // --- Input Handling ---

    pub(crate) fn move_cursor_left(&mut self) {
        if let Some((_, cursor, _)) = self.get_active_input_mut() {
            if *cursor > 0 {
                *cursor -= 1;
            }
        }
    }

    pub(crate) fn move_cursor_right(&mut self) {
        if let Some((content, cursor, _)) = self.get_active_input_mut() {
            if *cursor < content.len() {
                *cursor += 1;
            }
        }
    }

    pub(crate) fn insert_char_at_cursor(&mut self, c: char) {
        if let Some((content, cursor, input_type)) = self.get_active_input_mut() {
            match input_type {
                InputType::Date => {
                    if let Some(new_content) =
                        crate::validation::validate_and_insert_date_char(content, c)
                    {
                        *content = new_content;
                        *cursor = content.len();
                    }
                }
                InputType::Amount => {
                    if crate::validation::validate_amount_char(content, c) {
                        if *cursor >= content.len() {
                            content.push(c);
                        } else {
                            content.insert(*cursor, c);
                        }
                        *cursor += 1;
                    }
                }
                InputType::Text => {
                    if *cursor >= content.len() {
                        content.push(c);
                    } else {
                        content.insert(*cursor, c);
                    }
                    *cursor += 1;
                }
            }
        }
    }

    pub(crate) fn delete_char_before_cursor(&mut self) {
        if let Some((content, cursor, input_type)) = self.get_active_input_mut() {
            match input_type {
                InputType::Date => {
                    // Date backspace logic is specific
                    crate::validation::handle_date_backspace(content);
                    *cursor = content.len();
                }
                _ => {
                    if *cursor > 0 {
                        if *cursor <= content.len() {
                            content.remove(*cursor - 1);
                            *cursor -= 1;
                        } else {
                            *cursor = content.len();
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn delete_char_after_cursor(&mut self) {
        if let Some((content, cursor, _)) = self.get_active_input_mut() {
            if *cursor < content.len() {
                content.remove(*cursor);
            }
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
        if len == 0 {
            return;
        }

        loop {
            self.settings_state.selected_index = (self.settings_state.selected_index + 1) % len;
            let idx = self.settings_state.selected_index;

            // Skip headers
            if self.settings_state.items[idx].setting_type
                != crate::app::settings_types::SettingType::SectionHeader
            {
                self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
                break;
            }
        }
    }

    pub(crate) fn previous_settings_field(&mut self) {
        self.strip_quotes_from_current_setting();

        let len = self.settings_state.items.len();
        if len == 0 {
            return;
        }

        loop {
            if self.settings_state.selected_index == 0 {
                self.settings_state.selected_index = len - 1;
            } else {
                self.settings_state.selected_index -= 1;
            }
            let idx = self.settings_state.selected_index;

            // Skip headers
            if self.settings_state.items[idx].setting_type
                != crate::app::settings_types::SettingType::SectionHeader
            {
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
                item.value = " No ▶".to_string();
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
                item.value = "◀ Yes ".to_string();
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
        if idx >= self.settings_state.items.len() {
            return;
        }

        let setting_type = self.settings_state.items[idx].setting_type.clone();

        match setting_type {
            crate::app::settings_types::SettingType::SectionHeader => {
                // Do nothing
            }
            crate::app::settings_types::SettingType::Number => {
                crate::validation::insert_amount_char(&mut self.settings_state.items[idx].value, c);
                self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
            }
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
            }
            crate::app::settings_types::SettingType::Toggle => {}
        }
        self.update_settings_visibility();
    }

    pub(crate) fn delete_char_settings(&mut self) {
        let idx = self.settings_state.selected_index;
        if idx >= self.settings_state.items.len() {
            return;
        }

        let setting_type = self.settings_state.items[idx].setting_type.clone();

        match setting_type {
            crate::app::settings_types::SettingType::SectionHeader => {}
            crate::app::settings_types::SettingType::Number => {
                self.settings_state.items[idx].value.pop();
                self.settings_state.edit_cursor = self.settings_state.items[idx].value.len();
            }
            crate::app::settings_types::SettingType::Toggle => {
                // No-op for delete on toggle
            }
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
        self.update_settings_visibility();
    }

    pub(crate) fn clear_settings_field(&mut self) {
        let idx = self.settings_state.selected_index;
        if let Some(item) = self.settings_state.items.get_mut(idx) {
            if item.setting_type != crate::app::settings_types::SettingType::SectionHeader {
                item.value.clear();
                self.settings_state.edit_cursor = 0;
            }
        }
        self.update_settings_visibility();
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
