use super::state::App;
use crate::model::{TransactionType, DATE_FORMAT};
use chrono::{Duration, NaiveDate};
use ratatui::widgets::ListState;
use rust_decimal::Decimal;
use std::collections::HashSet;

impl App {
    pub(crate) fn is_filter_active(&self) -> bool {
        !self.simple_filter_content.is_empty()
            || self.advanced_filter_fields.iter().any(|f| !f.is_empty())
    }
    // --- Filtering Logic ---
    // Handles entering/exiting filtering mode, applying basic filter, and updating filtered indices.
    pub(crate) fn start_filtering(&mut self) {
        self.mode = crate::app::state::AppMode::Filtering;
        self.simple_filter_cursor = self.simple_filter_content.len();
        self.clear_status_message();
    }
    pub(crate) fn exit_filtering(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.clear_status_message();
    }
    pub(crate) fn apply_filter(&mut self) {
        crate::app::util::sort_transactions_impl(
            &mut self.transactions,
            self.sort_by,
            self.sort_order,
        );
        let query = self.simple_filter_content.to_lowercase();
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
        self.current_advanced_filter_field = 0;
        self.advanced_filter_cursor = self.advanced_filter_fields[0].len();
        self.clear_status_message()
    }
    pub(crate) fn cancel_advanced_filtering(&mut self) {
        self.mode = crate::app::state::AppMode::Normal;
        self.clear_status_message()
    }
    pub(crate) fn finish_advanced_filtering(&mut self) {
        self.clear_simple_filter_field_only();
        self.apply_advanced_filter();
        self.mode = crate::app::state::AppMode::Normal;
        self.clear_status_message()
    }

    pub(crate) fn reset_all_filters(&mut self) {
        let was_active = self.is_filter_active();
        // Clear simple filter field
        self.simple_filter_content.clear();
        self.simple_filter_cursor = 0;

        // Clear advanced filter fields
        for f in self.advanced_filter_fields.iter_mut() {
            f.clear();
        }
        self.current_advanced_filter_field = 0;

        // Apply basic filter (shows all transactions) and return to normal mode
        self.apply_filter();
        self.mode = crate::app::state::AppMode::Normal;
        if was_active {
            self.set_status_message("All filters cleared", Some(Duration::seconds(3)));
        } else {
            self.clear_status_message();
        }
    }
    pub(crate) fn clear_advanced_filter_fields_only(&mut self) {
        // Clear advanced filter fields without changing mode
        for f in self.advanced_filter_fields.iter_mut() {
            f.clear();
        }
        self.current_advanced_filter_field = 0;
    }
    pub(crate) fn clear_simple_filter_field_only(&mut self) {
        // Clear simple filter field without changing mode
        self.simple_filter_content.clear();
        self.simple_filter_cursor = 0;
    }
    pub(crate) fn next_advanced_filter_field(&mut self) {
        self.current_advanced_filter_field =
            (self.current_advanced_filter_field + 1) % self.advanced_filter_fields.len();
        self.advanced_filter_cursor = self.advanced_filter_fields[self.current_advanced_filter_field].len();
    }
    pub(crate) fn previous_advanced_filter_field(&mut self) {
        if self.current_advanced_filter_field == 0 {
            self.current_advanced_filter_field = self.advanced_filter_fields.len() - 1;
        } else {
            self.current_advanced_filter_field -= 1;
        }
        self.advanced_filter_cursor = self.advanced_filter_fields[self.current_advanced_filter_field].len();
    }
    pub(crate) fn toggle_advanced_transaction_type(&mut self) {
        self.clear_simple_filter_field_only();
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
        self.type_to_select.clear();
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
        self.type_to_select.clear();
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
                    let val_clone = val.clone();
                    self.clear_simple_filter_field_only();
                    let v = if fi == 4 && val_clone == "(None)" {
                        ""
                    } else {
                        val_clone.as_str()
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
        let amt_from = self.advanced_filter_fields[6].parse::<Decimal>().ok();
        let amt_to = self.advanced_filter_fields[7].parse::<Decimal>().ok();
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
    pub(crate) fn increment_advanced_date(&mut self) {
        let idx = self.current_advanced_filter_field;
        if idx == 0 || idx == 1 {
            self.clear_simple_filter_field_only();
            if let Some(new_date) = self.increment_date_field(&self.advanced_filter_fields[idx]) {
                self.advanced_filter_fields[idx] = new_date;
                self.advanced_filter_cursor = self.advanced_filter_fields[idx].len();
            }
        }
    }
    pub(crate) fn decrement_advanced_date(&mut self) {
        let idx = self.current_advanced_filter_field;
        if idx == 0 || idx == 1 {
            self.clear_simple_filter_field_only();
            if let Some(new_date) = self.decrement_date_field(&self.advanced_filter_fields[idx]) {
                self.advanced_filter_fields[idx] = new_date;
                self.advanced_filter_cursor = self.advanced_filter_fields[idx].len();
            }
        }
    }
    pub(crate) fn increment_advanced_month(&mut self) {
        let idx = self.current_advanced_filter_field;
        if idx == 0 || idx == 1 {
            self.clear_simple_filter_field_only();
            if let Some(new_date) = self.increment_month_field(&self.advanced_filter_fields[idx]) {
                self.advanced_filter_fields[idx] = new_date;
                self.advanced_filter_cursor = self.advanced_filter_fields[idx].len();
            }
        }
    }
    pub(crate) fn decrement_advanced_month(&mut self) {
        let idx = self.current_advanced_filter_field;
        if idx == 0 || idx == 1 {
            self.clear_simple_filter_field_only();
            if let Some(new_date) = self.decrement_month_field(&self.advanced_filter_fields[idx]) {
                self.advanced_filter_fields[idx] = new_date;
                self.advanced_filter_cursor = self.advanced_filter_fields[idx].len();
            }
        }
    }
}
