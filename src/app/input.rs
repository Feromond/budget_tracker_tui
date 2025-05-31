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
    pub(crate) fn clear_input_field(&mut self) {
        self.input_field_content.clear();
        self.input_field_cursor = 0;
    }

    // --- Settings Input ---
    pub(crate) fn next_settings_field(&mut self) {
        let next_field = (self.current_settings_field + 1) % self.settings_fields.len();
        self.current_settings_field = next_field;
        if self.current_settings_field == 0 {
            self.input_field_cursor = self.settings_fields[0].len();
        }
    }
    pub(crate) fn previous_settings_field(&mut self) {
        if self.current_settings_field == 0 {
            self.current_settings_field = self.settings_fields.len() - 1;
        } else {
            self.current_settings_field -= 1;
        }
        if self.current_settings_field == 0 {
            self.input_field_cursor = self.settings_fields[0].len();
        }
    }
    pub(crate) fn move_cursor_left_settings(&mut self) {
        if self.current_settings_field == 0 && self.input_field_cursor > 0 {
            self.input_field_cursor -= 1;
        }
    }
    pub(crate) fn move_cursor_right_settings(&mut self) {
        if self.current_settings_field == 0
            && self.input_field_cursor < self.settings_fields[0].len()
        {
            self.input_field_cursor += 1;
        }
    }
    pub(crate) fn insert_char_settings(&mut self, c: char) {
        let idx = self.current_settings_field;

        match idx {
            0 => {
                // Data File Path: insert at cursor
                let field = &mut self.settings_fields[0];
                field.insert(self.input_field_cursor, c);
                self.input_field_cursor += 1;
            }
            _ => {
                // Numeric setting fields
                crate::validation::insert_amount_char(&mut self.settings_fields[idx], c);
            }
        }
    }
    pub(crate) fn delete_char_settings(&mut self) {
        let idx = self.current_settings_field;
        if idx == 0 {
            // Data File Path: delete before cursor
            let field = &mut self.settings_fields[0];
            if self.input_field_cursor > 0 && !field.is_empty() {
                field.remove(self.input_field_cursor - 1);
                self.input_field_cursor -= 1;
            }
        } else {
            let field = &mut self.settings_fields[idx];
            field.pop();
        }
    }
    pub(crate) fn clear_settings_field(&mut self) {
        let idx = self.current_settings_field;
        self.settings_fields[idx].clear();
        if idx == 0 {
            self.input_field_cursor = 0;
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
