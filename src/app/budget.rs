use super::state::{App, AppMode, BudgetCategoryComparison, BudgetEditTarget};
use crate::db::budget_store::BudgetStore;
use crate::model::{BudgetEditScope, BudgetMonth, BudgetWrite, CategoryRecord, TransactionType};
use crate::ui::helpers::month_to_short_str;
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

fn budget_edit_message(
    scope: BudgetEditScope,
    label: &str,
    month: &str,
    amount: Option<Decimal>,
    replaced: usize,
) -> String {
    match (scope, amount) {
        (BudgetEditScope::RemoveChange, _) => format!("{} change in {} removed.", label, month),
        (BudgetEditScope::ReplaceAllMonths, Some(amount)) => format!(
            "{} set to {:.2} for every month, replacing {} earlier change(s).",
            label, amount, replaced
        ),
        (BudgetEditScope::ReplaceAllMonths, None) => format!("{} cleared for every month.", label),
        (BudgetEditScope::ThisMonthOnly, Some(amount)) => {
            format!("{} set to {:.2} for {} only.", label, amount, month)
        }
        (BudgetEditScope::ThisMonthOnly, None) => format!("{} cleared for {} only.", label, month),
        (_, Some(amount)) => format!("{} set to {:.2} from {} on.", label, amount, month),
        (_, None) => format!("{} cleared from {} on.", label, month),
    }
}

fn record_label(record: &CategoryRecord) -> String {
    if record.subcategory.trim().is_empty() {
        record.category.clone()
    } else {
        format!("{} / {}", record.category, record.subcategory)
    }
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

    /// Calendar months, not just the ones holding transactions: budgets get planned ahead.
    fn budget_months(&self) -> Vec<u32> {
        (1..=12).collect()
    }

    fn update_selected_budget_month(&mut self, year: i32) {
        let current_date = chrono::Local::now();
        self.selected_budget_month = if year == current_date.year() {
            Some(current_date.month())
        } else {
            Some(1)
        };
    }

    pub(crate) fn refresh_budget_years(&mut self) {
        // Always offer the current year, so a budget can be set before anything is spent.
        self.budget_years = self.summary_years.clone();
        let current_year = chrono::Local::now().year();
        if !self.budget_years.contains(&current_year) {
            self.budget_years.push(current_year);
            self.budget_years.sort_unstable();
        }

        self.budget_year_index = self.budget_year_index.min(self.budget_years.len() - 1);
        if let Some(year) = self.budget_years.get(self.budget_year_index).copied()
            && self.selected_budget_month.is_none()
        {
            self.update_selected_budget_month(year);
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
        if self.selected_budget_year().is_some() {
            let months = self.budget_months();
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
        if self.selected_budget_year().is_some() {
            let months = self.budget_months();
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

    /// Summed from the same rows the table shows, so the two can never disagree.
    pub(crate) fn total_allocated_budget(&self, year: i32, month: u32) -> Decimal {
        self.budget_category_comparisons(year, month)
            .iter()
            .map(|comparison| comparison.budget)
            .sum()
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
            .budget_schedule
            .budgeted_categories(BudgetMonth::new(year, month))
            .into_iter()
            .filter_map(|(category_id, budget)| {
                let record = self
                    .category_records
                    .iter()
                    .find(|record| record.id == category_id)?;
                if record.transaction_type != TransactionType::Expense {
                    return None;
                }
                let (category, subcategory) =
                    normalize_budget_key(&record.category, &record.subcategory);
                let actual_expense = month_map
                    .and_then(|map| map.get(&(category.clone(), subcategory.clone())))
                    .map(|summary| summary.expense)
                    .unwrap_or(Decimal::ZERO);
                Some(BudgetCategoryComparison {
                    id: category_id,
                    category,
                    subcategory,
                    budget,
                    actual_expense,
                })
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

    // --- Budget editing ---

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

        let Some(month) = self.selected_budget_key() else {
            return;
        };
        self.open_budget_editor(
            BudgetEditTarget::Category(comparison.id),
            Some(comparison.budget),
            month,
            AppMode::Budget,
        );
    }

    pub(crate) fn start_editing_monthly_budget(&mut self) {
        let Some(start) = self.selected_budget_key() else {
            self.set_status_message("No month selected.", None);
            return;
        };
        let current = self.budget_schedule.monthly_budget(start);
        self.open_budget_editor(
            BudgetEditTarget::MonthlyBudget,
            current,
            start,
            AppMode::Budget,
        );
    }

    /// A category being added needs an id before a budget can point at it.
    pub(crate) fn save_category_then_edit_budget(&mut self) {
        self.save_category();
        // save_category lands back on the catalog with the saved row selected; anything
        // else means it failed and already reported why.
        if self.mode == AppMode::CategoryCatalog {
            self.start_editing_catalog_budget();
        }
    }

    /// Opens the same popup from the catalog, anchored on today since the catalog has no
    /// month of its own.
    pub(crate) fn start_editing_catalog_budget(&mut self) {
        let Some(record) = self.selected_category_record().cloned() else {
            self.set_status_message("Select a category first.", None);
            return;
        };
        if record.transaction_type != TransactionType::Expense {
            self.set_status_message("Budgets are only available for expense categories.", None);
            return;
        }

        let month = Self::current_budget_key();
        let current = self.budget_schedule.category_budget(record.id, month);
        self.open_budget_editor(
            BudgetEditTarget::Category(record.id),
            current,
            month,
            AppMode::CategoryCatalog,
        );
    }

    fn open_budget_editor(
        &mut self,
        target: BudgetEditTarget,
        current: Option<Decimal>,
        month: BudgetMonth,
        origin: AppMode,
    ) {
        self.mode = AppMode::BudgetCategoryEditor;
        self.budget_edit_target = Some(target);
        self.budget_edit_month = Some(month);
        self.budget_edit_origin = origin;
        self.budget_edit_scope_choice = BudgetEditScope::FromThisMonth;
        self.budget_edit_input = current.map(|v| format!("{v:.2}")).unwrap_or_default();
        self.budget_edit_cursor = self.budget_edit_input.len();
        self.clear_status_message();
    }

    pub(crate) fn cancel_budget_edit(&mut self) {
        self.close_budget_editor(self.budget_edit_target);
        self.set_status_message("Budget edit cancelled.", Some(Duration::seconds(3)));
    }

    pub(crate) fn budget_edit_label(&self) -> Option<String> {
        match self.budget_edit_target? {
            BudgetEditTarget::MonthlyBudget => Some("Monthly Budget".to_string()),
            BudgetEditTarget::Category(id) => self
                .category_records
                .iter()
                .find(|record| record.id == id)
                .map(record_label),
        }
    }

    pub(crate) fn budget_edit_scopes(&self) -> Vec<BudgetEditScope> {
        let mut scopes = vec![
            BudgetEditScope::FromThisMonth,
            BudgetEditScope::ThisMonthOnly,
            BudgetEditScope::ReplaceAllMonths,
        ];
        if let (Some(target), Some(start)) = (self.budget_edit_target, self.budget_edit_month)
            && self.budget_schedule.starts_at(target.category_id(), start)
        {
            scopes.push(BudgetEditScope::RemoveChange);
        }
        scopes
    }

    /// The choice, unless it is not on offer any more, in which case the safe default.
    pub(crate) fn budget_edit_scope(&self) -> BudgetEditScope {
        let scopes = self.budget_edit_scopes();
        if scopes.contains(&self.budget_edit_scope_choice) {
            self.budget_edit_scope_choice
        } else {
            BudgetEditScope::FromThisMonth
        }
    }

    pub(crate) fn cycle_budget_edit_scope(&mut self, forward: bool) {
        let scopes = self.budget_edit_scopes();
        let current = scopes
            .iter()
            .position(|scope| *scope == self.budget_edit_scope())
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % scopes.len()
        } else {
            (current + scopes.len() - 1) % scopes.len()
        };
        self.budget_edit_scope_choice = scopes[next];
    }

    pub(crate) fn save_budget_edit(&mut self) {
        let (Some(target), Some(label), Some(start)) = (
            self.budget_edit_target,
            self.budget_edit_label(),
            self.budget_edit_month,
        ) else {
            self.cancel_budget_edit();
            return;
        };

        let scope = self.budget_edit_scope();
        let amount = match self.parse_budget_edit_amount(scope) {
            Ok(amount) => amount,
            Err(message) => {
                self.set_status_message(format!("Error: {}", message), None);
                return;
            }
        };

        let category_id = target.category_id();
        let replaced = self.budget_schedule.period_count(category_id);
        let writes = self
            .budget_schedule
            .plan_edit(category_id, start, amount, scope);
        if let Err(err) = self.apply_budget_writes(category_id, &writes) {
            self.set_status_message(format!("Error saving budget: {}", err), None);
            return;
        }
        if let Err(err) = self.reload_budget_schedule() {
            self.set_status_message(format!("Budget saved, but refresh failed: {}", err), None);
            return;
        }

        self.close_budget_editor(Some(target));
        if self.mode == AppMode::CategoryCatalog
            && let BudgetEditTarget::Category(id) = target
        {
            self.resort_catalog_keeping(id);
        }
        let month = format!("{} {}", month_to_short_str(start.month), start.year);
        self.set_status_message(
            budget_edit_message(scope, &label, &month, amount, replaced),
            Some(Duration::seconds(3)),
        );
    }

    fn parse_budget_edit_amount(&self, scope: BudgetEditScope) -> Result<Option<Decimal>, String> {
        let input = self.budget_edit_input.trim();
        // Removing a change discards the typed amount, and an empty box clears the budget.
        if input.is_empty() || scope == BudgetEditScope::RemoveChange {
            return Ok(None);
        }
        crate::validation::validate_amount_string(input).map(Some)
    }

    fn apply_budget_writes(
        &self,
        category_id: Option<i64>,
        writes: &[BudgetWrite],
    ) -> Result<(), std::io::Error> {
        self.budget_store()
            .apply(self.active_ledger_id, category_id, writes)
    }

    fn close_budget_editor(&mut self, target: Option<BudgetEditTarget>) {
        let origin = self.budget_edit_origin;
        self.mode = origin;
        self.budget_edit_target = None;
        self.budget_edit_month = None;
        self.budget_edit_input.clear();
        self.budget_edit_cursor = 0;
        if origin == AppMode::Budget
            && let Some(BudgetEditTarget::Category(id)) = target
        {
            self.select_budget_category(id);
        }
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
