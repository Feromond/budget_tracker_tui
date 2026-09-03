use super::state::{App, AppMode, BudgetCategoryComparison};
use crate::db::category_store::CategoryStore;
use crate::model::{CategoryRecord, TransactionType};
use chrono::{Datelike, Duration};
use rust_decimal::Decimal;

fn normalize_budget_key(category: &str, subcategory: &str) -> (String, String) {
    let category = category.trim();
    let subcategory = subcategory.trim();
    let normalized_category = if category.is_empty() {
        "Uncategorized".to_string()
    } else {
        category.to_string()
    };

    (normalized_category, subcategory.to_string())
}

fn record_label(record: &CategoryRecord) -> String {
    if record.subcategory.trim().is_empty() {
        record.category.clone()
    } else {
        format!("{} / {}", record.category, record.subcategory)
    }
}

fn comparison_from_record(
    record: &CategoryRecord,
    actual_expense: Decimal,
) -> Option<BudgetCategoryComparison> {
    if record.transaction_type != TransactionType::Expense {
        return None;
    }

    let target_budget = record.target_budget?;
    let (category, subcategory) = normalize_budget_key(&record.category, &record.subcategory);

    Some(BudgetCategoryComparison {
        id: record.id,
        category,
        subcategory,
        target_budget,
        actual_expense,
    })
}

impl App {
    fn clamp_budget_selection(&mut self, len: usize) {
        if len == 0 {
            self.budget_table_state.select(None);
            return;
        }

        let selected = self.budget_table_state.selected().unwrap_or(0).min(len - 1);
        self.budget_table_state.select(Some(selected));
    }

    fn update_selected_budget_month(&mut self, year: i32) {
        let current_date = chrono::Local::now();
        let current_year = current_date.year();
        let current_month = current_date.month();
        let months = self.sorted_months_for_year(year);
        if year == current_year && months.contains(&current_month) {
            self.selected_budget_month = Some(current_month);
        } else {
            self.selected_budget_month = months.last().copied();
        }
    }

    pub(crate) fn refresh_budget_years(&mut self) {
        self.budget_years = self.summary_years.clone();
        if self.budget_years.is_empty() {
            self.budget_year_index = 0;
            self.selected_budget_month = None;
            self.budget_table_state.select(None);
            return;
        }

        self.budget_year_index = self.budget_year_index.min(self.budget_years.len() - 1);
        if let Some(year) = self.budget_years.get(self.budget_year_index).copied() {
            let months = self.sorted_months_for_year(year);
            if !matches!(self.selected_budget_month, Some(month) if months.contains(&month)) {
                self.update_selected_budget_month(year);
            }
        }
        let len = self.current_budget_category_comparisons().len();
        self.clamp_budget_selection(len);
    }

    pub(crate) fn enter_budget_mode(&mut self) {
        self.mode = AppMode::Budget;
        self.calculate_monthly_summaries();
        self.calculate_category_summaries();
        self.refresh_budget_years();
        if !self.budget_years.is_empty() {
            let current_year = chrono::Local::now().year();
            if let Some(index) = self
                .budget_years
                .iter()
                .position(|&year| year == current_year)
            {
                self.budget_year_index = index;
            } else {
                self.budget_year_index = self.budget_years.len() - 1;
            }
            if let Some(year) = self.budget_years.get(self.budget_year_index).copied() {
                self.update_selected_budget_month(year);
            }
        } else {
            self.selected_budget_month = None;
        }
        let len = self.current_budget_category_comparisons().len();
        self.clamp_budget_selection(len);
        self.clear_status_message();
    }

    pub(crate) fn exit_budget_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.clear_status_message();
    }

    pub(crate) fn selected_budget_year(&self) -> Option<i32> {
        self.budget_years.get(self.budget_year_index).copied()
    }

    pub(crate) fn next_budget_year(&mut self) {
        if self.budget_years.is_empty() {
            return;
        }

        self.budget_year_index = (self.budget_year_index + 1) % self.budget_years.len();
        if let Some(year) = self.selected_budget_year() {
            self.update_selected_budget_month(year);
        }
        let len = self.current_budget_category_comparisons().len();
        self.clamp_budget_selection(len);
    }

    pub(crate) fn previous_budget_year(&mut self) {
        if self.budget_years.is_empty() {
            return;
        }

        if self.budget_year_index == 0 {
            self.budget_year_index = self.budget_years.len() - 1;
        } else {
            self.budget_year_index -= 1;
        }
        if let Some(year) = self.selected_budget_year() {
            self.update_selected_budget_month(year);
        }
        let len = self.current_budget_category_comparisons().len();
        self.clamp_budget_selection(len);
    }

    pub(crate) fn next_budget_month(&mut self) {
        if let Some(year) = self.selected_budget_year() {
            let months = self.sorted_months_for_year(year);
            if let Some(current) = self.selected_budget_month {
                if let Some(index) = months.iter().position(|&month| month == current) {
                    self.selected_budget_month = Some(months[(index + 1) % months.len()]);
                }
            } else if let Some(&first) = months.first() {
                self.selected_budget_month = Some(first);
            }
            let len = self.current_budget_category_comparisons().len();
            self.clamp_budget_selection(len);
        }
    }

    pub(crate) fn previous_budget_month(&mut self) {
        if let Some(year) = self.selected_budget_year() {
            let months = self.sorted_months_for_year(year);
            if let Some(current) = self.selected_budget_month {
                if let Some(index) = months.iter().position(|&month| month == current) {
                    let previous = if index == 0 {
                        months.len() - 1
                    } else {
                        index - 1
                    };
                    self.selected_budget_month = Some(months[previous]);
                }
            } else if let Some(&first) = months.first() {
                self.selected_budget_month = Some(first);
            }
            let len = self.current_budget_category_comparisons().len();
            self.clamp_budget_selection(len);
        }
    }

    pub(crate) fn next_budget_category(&mut self) {
        let len = self.current_budget_category_comparisons().len();
        if len == 0 {
            self.budget_table_state.select(None);
            return;
        }

        let next = match self.budget_table_state.selected() {
            Some(index) if index + 1 < len => index + 1,
            _ => 0,
        };
        self.budget_table_state.select(Some(next));
    }

    pub(crate) fn previous_budget_category(&mut self) {
        let len = self.current_budget_category_comparisons().len();
        if len == 0 {
            self.budget_table_state.select(None);
            return;
        }

        let previous = match self.budget_table_state.selected() {
            Some(0) | None => len - 1,
            Some(index) => index - 1,
        };
        self.budget_table_state.select(Some(previous));
    }

    pub(crate) fn budget_month_expense(&self, year: i32, month: u32) -> Decimal {
        self.monthly_summaries
            .get(&(year, month))
            .map(|summary| summary.expense)
            .unwrap_or(Decimal::ZERO)
    }

    pub(crate) fn budget_category_comparisons(
        &self,
        year: i32,
        month: u32,
    ) -> Vec<BudgetCategoryComparison> {
        let month_map = self.category_summaries.get(&(year, month));
        let mut comparisons: Vec<BudgetCategoryComparison> = self
            .category_records
            .iter()
            .filter_map(|record| {
                let (category, subcategory) =
                    normalize_budget_key(&record.category, &record.subcategory);
                let actual_expense = month_map
                    .and_then(|map| map.get(&(category.clone(), subcategory.clone())))
                    .map(|summary| summary.expense)
                    .unwrap_or(Decimal::ZERO);
                comparison_from_record(record, actual_expense)
            })
            .collect();

        comparisons.sort_by(|left, right| {
            left.category
                .cmp(&right.category)
                .then(left.subcategory.cmp(&right.subcategory))
        });
        comparisons
    }

    pub(crate) fn current_budget_category_comparisons(&self) -> Vec<BudgetCategoryComparison> {
        match (self.selected_budget_year(), self.selected_budget_month) {
            (Some(year), Some(month)) => self.budget_category_comparisons(year, month),
            _ => Vec::new(),
        }
    }

    pub(crate) fn selected_budget_category_comparison(&self) -> Option<BudgetCategoryComparison> {
        let comparisons = self.current_budget_category_comparisons();
        let selected = self.budget_table_state.selected().unwrap_or(0);
        comparisons.get(selected).cloned()
    }

    // --- Target budget editing ---

    pub(crate) fn start_editing_budget_target(&mut self) {
        let Some(comparison) = self.selected_budget_category_comparison() else {
            let message = if self.selected_budget_month.is_none() {
                "No month selected."
            } else {
                "No category budgets yet. Press c to set one in the catalog."
            };
            self.set_status_message(message, None);
            return;
        };

        self.mode = AppMode::BudgetCategoryEditor;
        self.budget_edit_category_id = Some(comparison.id);
        self.budget_edit_input = format!("{:.2}", comparison.target_budget);
        self.budget_edit_cursor = self.budget_edit_input.len();
        self.clear_status_message();
    }

    pub(crate) fn cancel_budget_target_edit(&mut self) {
        self.mode = AppMode::Budget;
        self.budget_edit_category_id = None;
        self.budget_edit_input.clear();
        self.budget_edit_cursor = 0;
        self.set_status_message("Budget edit cancelled.", Some(Duration::seconds(3)));
    }

    pub(crate) fn budget_edit_label(&self) -> Option<String> {
        self.budget_edit_record().map(record_label)
    }

    fn budget_edit_record(&self) -> Option<&CategoryRecord> {
        let id = self.budget_edit_category_id?;
        self.category_records.iter().find(|record| record.id == id)
    }

    pub(crate) fn save_budget_target(&mut self) {
        let Some(record) = self.budget_edit_record().cloned() else {
            self.cancel_budget_target_edit();
            return;
        };

        let input = self.budget_edit_input.trim();
        // Clearing it also drops the row from this table.
        let target_budget = if input.is_empty() {
            None
        } else {
            match crate::validation::validate_amount_string(input) {
                Ok(amount) => Some(amount),
                Err(message) => {
                    self.set_status_message(format!("Error: {}", message), None);
                    return;
                }
            }
        };

        let mut draft = record.to_draft();
        draft.target_budget = target_budget;

        if let Err(err) = self.category_store().update(record.id, &draft) {
            self.set_status_message(format!("Error saving budget: {}", err), None);
            return;
        }

        if let Err(err) = self.reload_categories_from_store() {
            self.set_status_message(format!("Budget saved, but refresh failed: {}", err), None);
            return;
        }

        self.mode = AppMode::Budget;
        self.budget_edit_category_id = None;
        self.budget_edit_input.clear();
        self.budget_edit_cursor = 0;
        self.select_budget_category(record.id);

        let label = record_label(&record);
        let message = match target_budget {
            Some(amount) => format!("Budget for {} set to {:.2}.", label, amount),
            None => format!("Budget for {} cleared.", label),
        };
        self.set_status_message(message, Some(Duration::seconds(3)));
    }

    fn select_budget_category(&mut self, id: i64) {
        let comparisons = self.current_budget_category_comparisons();
        match comparisons
            .iter()
            .position(|comparison| comparison.id == id)
        {
            Some(index) => self.budget_table_state.select(Some(index)),
            None => self.clamp_budget_selection(comparisons.len()),
        }
    }

    pub(crate) fn budget_category_monthly_expenses(
        &self,
        year: i32,
        comparison: &BudgetCategoryComparison,
    ) -> Vec<(u32, Decimal)> {
        let mut months: Vec<u32> = (1..=12)
            .filter(|&month| self.monthly_summaries.contains_key(&(year, month)))
            .collect();
        months.sort_unstable();
        months
            .into_iter()
            .map(|month| {
                let expense = self
                    .category_summaries
                    .get(&(year, month))
                    .and_then(|month_map| {
                        month_map
                            .get(&(comparison.category.clone(), comparison.subcategory.clone()))
                    })
                    .map(|summary| summary.expense)
                    .unwrap_or(Decimal::ZERO);
                (month, expense)
            })
            .collect()
    }
}
