use super::state::App;
use crate::app::state::DateUnit;
use crate::model::{TransactionType, DATE_FORMAT};
use chrono::NaiveDate;
use ratatui::widgets::ListState;
use std::collections::HashSet;

impl App {
    // --- Filtering Logic ---
    // Handles entering/exiting filtering mode, applying basic filter, and updating filtered indices.
    pub(crate) fn start_filtering(&mut self) {
        self.mode = crate::app::state::AppMode::Filtering;
        self.input_field_cursor = self.input_field_content.len();
        self.status_message = None;
    }
    pub(crate) fn exit_filtering(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.status_message = None;
    }
    pub(crate) fn apply_filter(&mut self) {
        crate::app::util::sort_transactions_impl(
            &mut self.transactions,
            self.sort_by,
            self.sort_order,
        );
        let query = self.input_field_content.to_lowercase();
        self.filtered_indices = self
            .transactions
            .iter()
            .enumerate()
            .filter(|(_, tx)| {
                if query.is_empty() {
                    true
                } else {
                    tx.description.to_lowercase().contains(&query)
                }
            })
            .map(|(index, _)| index)
            .collect();
        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
        } else {
            let current_selection = self.table_state.selected().unwrap_or(0);
            self.table_state
                .select(Some(current_selection.min(self.filtered_indices.len() - 1)));
        }
        self.calculate_category_summaries();
        self.calculate_monthly_summaries();
    }
    // --- Advanced Filtering Logic ---
    // Handles advanced filter UI, field navigation, and applying advanced filters to transactions.
    pub(crate) fn start_advanced_filtering(&mut self) {
        self.mode = crate::app::state::AppMode::AdvancedFiltering;
        self.advanced_filter_fields = Default::default();
        self.current_advanced_filter_field = 0;
        self.status_message = None;
    }
    pub(crate) fn cancel_advanced_filtering(&mut self) {
        self.mode = crate::app::state::AppMode::Filtering;
        self.status_message = None;
    }
    pub(crate) fn finish_advanced_filtering(&mut self) {
        self.apply_advanced_filter();
        self.mode = crate::app::state::AppMode::Normal;
        self.status_message = None;
    }
    pub(crate) fn clear_advanced_filter_fields(&mut self) {
        for f in self.advanced_filter_fields.iter_mut() {
            f.clear();
        }
        self.current_advanced_filter_field = 0;
        self.apply_advanced_filter();
    }
    pub(crate) fn next_advanced_filter_field(&mut self) {
        self.current_advanced_filter_field =
            (self.current_advanced_filter_field + 1) % self.advanced_filter_fields.len();
    }
    pub(crate) fn previous_advanced_filter_field(&mut self) {
        if self.current_advanced_filter_field == 0 {
            self.current_advanced_filter_field = self.advanced_filter_fields.len() - 1;
        } else {
            self.current_advanced_filter_field -= 1;
        }
    }
    pub(crate) fn insert_char_advanced_filter(&mut self, c: char) {
        let idx = self.current_advanced_filter_field;
        let field = &mut self.advanced_filter_fields[idx];
        match idx {
            0 | 1 => {
                // Date fields: use centralized date validation
                if let Some(new_content) =
                    crate::validation::validate_and_insert_date_char(field, c)
                {
                    *field = new_content;
                }
                // Note: No error message here since this is filter input, not form validation
            }
            5 => { /* Type field: toggle only via arrows/enter */ }
            6 | 7 => {
                // Amount fields: use centralized amount validation
                crate::validation::insert_amount_char(field, c);
            }
            3 | 4 => { /* Category/Subcategory: selections only, no free text */ }
            _ => {
                // Description (idx 2)
                field.push(c);
            }
        }
    }
    pub(crate) fn delete_char_advanced_filter(&mut self) {
        let idx = self.current_advanced_filter_field;
        let field = &mut self.advanced_filter_fields[idx];
        match idx {
            0 | 1 => {
                // Date fields: use centralized date backspace handling
                crate::validation::handle_date_backspace(field);
            }
            5 => { /* Type field: nothing */ }
            6 | 7 => {
                field.pop();
            } // Amount fields: simple pop
            3 | 4 => { /* Category/Subcategory: no deletion */ }
            _ => {
                field.pop();
            } // Description
        }
    }
    pub(crate) fn toggle_advanced_transaction_type(&mut self) {
        let ft = self.advanced_filter_fields[5].trim();
        let new_val = if ft.is_empty() {
            "Income"
        } else if ft.eq_ignore_ascii_case("Income") {
            "Expense"
        } else {
            ""
        };
        self.advanced_filter_fields[5] = new_val.to_string();
    }
    pub(crate) fn start_advanced_category_selection(&mut self) {
        self.selecting_field_index = Some(3);
        self.mode = crate::app::state::AppMode::SelectingFilterCategory;
        let mut unique: HashSet<String> =
            self.categories.iter().map(|c| c.category.clone()).collect();
        let mut opts: Vec<String> = unique.drain().collect();
        opts.sort_unstable();
        self.current_selection_list = opts;
        self.selection_list_state = ListState::default();
        if !self.current_selection_list.is_empty() {
            self.selection_list_state.select(Some(0));
        }
    }
    pub(crate) fn start_advanced_subcategory_selection(&mut self) {
        self.selecting_field_index = Some(4);
        self.mode = crate::app::state::AppMode::SelectingFilterSubcategory;
        let current_cat = self.advanced_filter_fields[3].trim();
        let mut unique: HashSet<String> = self
            .categories
            .iter()
            .filter(|c| current_cat.is_empty() || c.category.eq_ignore_ascii_case(current_cat))
            .filter(|c| !c.subcategory.is_empty())
            .map(|c| c.subcategory.clone())
            .collect();
        let mut opts: Vec<String> = unique.drain().collect();
        opts.sort_unstable();
        opts.insert(0, "(None)".to_string());
        self.current_selection_list = opts;
        self.selection_list_state = ListState::default();
        if !self.current_selection_list.is_empty() {
            self.selection_list_state.select(Some(0));
        }
    }
    pub(crate) fn confirm_advanced_selection(&mut self) {
        if let Some(idx) = self.selection_list_state.selected() {
            if let Some(fi) = self.selecting_field_index {
                if let Some(val) = self.current_selection_list.get(idx) {
                    let v = if fi == 4 && val == "(None)" {
                        ""
                    } else {
                        val.as_str()
                    };
                    self.advanced_filter_fields[fi] = v.to_string();
                    if fi == 3 {
                        self.start_advanced_subcategory_selection();
                        return;
                    }
                }
            }
        }
        self.mode = crate::app::state::AppMode::AdvancedFiltering;
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }
    pub(crate) fn cancel_advanced_selection(&mut self) {
        self.mode = crate::app::state::AppMode::AdvancedFiltering;
        if let Some(fi) = self.selecting_field_index {
            self.current_advanced_filter_field = fi;
        }
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }
    pub(crate) fn apply_advanced_filter(&mut self) {
        crate::app::util::sort_transactions_impl(
            &mut self.transactions,
            self.sort_by,
            self.sort_order,
        );
        let date_from =
            NaiveDate::parse_from_str(&self.advanced_filter_fields[0], DATE_FORMAT).ok();
        let date_to = NaiveDate::parse_from_str(&self.advanced_filter_fields[1], DATE_FORMAT).ok();
        let desc_q = self.advanced_filter_fields[2].to_lowercase();
        let cat_q = self.advanced_filter_fields[3].to_lowercase();
        let sub_q = self.advanced_filter_fields[4].to_lowercase();
        let type_q = self.advanced_filter_fields[5].trim();
        let amt_from = self.advanced_filter_fields[6].parse::<f64>().ok();
        let amt_to = self.advanced_filter_fields[7].parse::<f64>().ok();
        self.filtered_indices = self
            .transactions
            .iter()
            .enumerate()
            .filter(|(_, tx)| {
                if let Some(d) = date_from {
                    if tx.date < d {
                        return false;
                    }
                }
                if let Some(d) = date_to {
                    if tx.date > d {
                        return false;
                    }
                }
                if !desc_q.is_empty() && !tx.description.to_lowercase().contains(&desc_q) {
                    return false;
                }
                if !cat_q.is_empty() && !tx.category.to_lowercase().contains(&cat_q) {
                    return false;
                }
                if !sub_q.is_empty() && !tx.subcategory.to_lowercase().contains(&sub_q) {
                    return false;
                }
                if type_q.eq_ignore_ascii_case("Income")
                    && tx.transaction_type != TransactionType::Income
                {
                    return false;
                }
                if type_q.eq_ignore_ascii_case("Expense")
                    && tx.transaction_type != TransactionType::Expense
                {
                    return false;
                }
                if let Some(f) = amt_from {
                    if tx.amount < f {
                        return false;
                    }
                }
                if let Some(t) = amt_to {
                    if tx.amount > t {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();
        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
        } else {
            let cur = self.table_state.selected().unwrap_or(0);
            self.table_state
                .select(Some(cur.min(self.filtered_indices.len() - 1)));
        }
        self.calculate_category_summaries();
        self.calculate_monthly_summaries();
    }
    pub(crate) fn adjust_advanced_date(&mut self, amount: i64, unit: DateUnit) {
        let idx = self.current_advanced_filter_field;
        if idx == 0 || idx == 1 {
            if let Ok(current_date) =
                NaiveDate::parse_from_str(&self.advanced_filter_fields[idx], DATE_FORMAT)
            {
                let new_date = match unit {
                    DateUnit::Day => {
                        if amount > 0 {
                            current_date + chrono::Duration::days(amount)
                        } else {
                            current_date - chrono::Duration::days(-amount)
                        }
                    }
                    DateUnit::Month => {
                        // Use centralized month arithmetic
                        crate::validation::add_months(current_date, amount as i32)
                    }
                };
                self.advanced_filter_fields[idx] = new_date.format(DATE_FORMAT).to_string();
            }
        }
    }
    pub(crate) fn increment_advanced_date(&mut self) {
        self.adjust_advanced_date(1, DateUnit::Day);
    }
    pub(crate) fn decrement_advanced_date(&mut self) {
        self.adjust_advanced_date(-1, DateUnit::Day);
    }
    pub(crate) fn increment_advanced_month(&mut self) {
        self.adjust_advanced_date(1, DateUnit::Month);
    }
    pub(crate) fn decrement_advanced_month(&mut self) {
        self.adjust_advanced_date(-1, DateUnit::Month);
    }
}
