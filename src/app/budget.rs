use super::state::{App, AppMode, BudgetCategoryComparison};
use crate::model::{CategoryRecord, TransactionType};
use chrono::Datelike;
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
            if let Some(index) = self.budget_years.iter().position(|&year| year == current_year) {
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

    pub(crate) fn budget_category_monthly_expenses(
        &self,
        year: i32,
        comparison: &BudgetCategoryComparison,
    ) -> Vec<(u32, Decimal)> {
        (1..=12)
            .map(|month| {
                let expense = self
                    .category_summaries
                    .get(&(year, month))
                    .and_then(|month_map| {
                        month_map.get(&(
                            comparison.category.clone(),
                            comparison.subcategory.clone(),
                        ))
                    })
                    .map(|summary| summary.expense)
                    .unwrap_or(Decimal::ZERO);
                (month, expense)
            })
            .collect()
    }
}
