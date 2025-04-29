use super::state::App;

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
        // Special handling for the Date field (index 0)
        if current_field == 0 {
            if let Some(new_content) =
                crate::app::util::validate_and_insert_date_char(field_content, c)
            {
                *field_content = new_content;
            }
        } else if current_field == 2 {
            // Only allow digits and one decimal point for Amount field
            if c.is_ascii_digit() || (c == '.' && !field_content.contains('.')) {
                field_content.push(c);
            }
        } else {
            // Default behavior for other fields
            field_content.push(c);
        }
    }
    pub(crate) fn delete_char_add_edit(&mut self) {
        let current_field = self.current_add_edit_field;
        let field_content = &mut self.add_edit_fields[current_field];
        // Special handling for the Date field (index 0)
        if current_field == 0 {
            let len = field_content.len();
            // If the last character is a hyphen that we likely auto-inserted,
            // remove it and the preceding digit.
            if (len == 5 || len == 8) && field_content.ends_with('-') {
                // Check if the character before the hyphen is a digit (sanity check)
                if field_content
                    .chars()
                    .nth(len - 2)
                    .is_some_and(|ch| ch.is_ascii_digit())
                {
                    field_content.pop(); // Remove the hyphen
                    field_content.pop(); // Remove the preceding digit
                } else {
                    // Should not happen with current logic, but handle gracefully
                    field_content.pop(); // Just remove the hyphen
                }
            } else if !field_content.is_empty() {
                field_content.pop();
            }
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
}
