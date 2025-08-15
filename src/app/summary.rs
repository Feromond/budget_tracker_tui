use super::state::App;
use crate::app::state::{AppMode, CategorySummaryItem};
use crate::model::MonthlySummary;
use chrono;
use chrono::Datelike;

impl App {
    // --- Private Helpers for Summary Navigation ---
    pub fn sorted_months_for_year(&self, year: i32) -> Vec<u32> {
        let mut months: Vec<u32> = self
            .monthly_summaries
            .keys()
            .filter_map(|(y, m)| if *y == year { Some(*m) } else { None })
            .collect();
        months.sort_unstable();
        months
    }

    fn update_selected_summary_month(&mut self, year: i32) {
        let current_date = chrono::Local::now();
        let current_year = current_date.year();
        let current_month = current_date.month();
        let months = self.sorted_months_for_year(year);
        if year == current_year && months.contains(&current_month) {
            self.selected_summary_month = Some(current_month);
        } else {
            self.selected_summary_month = months.last().copied();
        }
    }

    // --- Private Helpers for Category Summary Navigation ---
    pub fn sorted_category_months_for_year(&self, year: i32) -> Vec<u32> {
        let mut months: Vec<u32> = self
            .category_summaries
            .keys()
            .filter_map(|(y, m)| if *y == year { Some(*m) } else { None })
            .collect();
        months.sort_unstable();
        months.dedup();
        months
    }

    // --- Summary Logic ---
    // Handles entering/exiting summary mode, navigating years, and updating monthly summaries.
    pub(crate) fn enter_summary_mode(&mut self) {
        self.mode = AppMode::Summary;
        self.calculate_monthly_summaries();
        if !self.summary_years.is_empty() {
            let current_year = chrono::Local::now().year();
            if let Some(idx) = self.summary_years.iter().position(|&y| y == current_year) {
                self.selected_summary_year_index = idx;
            } else {
                self.selected_summary_year_index = self.summary_years.len() - 1;
            }
        }
        // Set selected_summary_month to current month if present, else latest month with data
        if let Some(year) = self
            .summary_years
            .get(self.selected_summary_year_index)
            .copied()
        {
            self.update_selected_summary_month(year);
        } else {
            self.selected_summary_month = None;
        }
        self.table_state.select(Some(0));
        self.status_message = None;
    }
    pub(crate) fn exit_summary_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.status_message = None;
    }
    pub(crate) fn next_summary_year(&mut self) {
        if !self.summary_years.is_empty() {
            self.selected_summary_year_index =
                (self.selected_summary_year_index + 1) % self.summary_years.len();
            // Reset month selection for new year
            if let Some(year) = self
                .summary_years
                .get(self.selected_summary_year_index)
                .copied()
            {
                self.update_selected_summary_month(year);
            } else {
                self.selected_summary_month = None;
            }
            self.table_state.select(Some(0));
        }
    }
    pub(crate) fn previous_summary_year(&mut self) {
        if !self.summary_years.is_empty() {
            if self.selected_summary_year_index > 0 {
                self.selected_summary_year_index -= 1;
            } else {
                self.selected_summary_year_index = self.summary_years.len() - 1;
            }
            // Reset month selection for new year
            if let Some(year) = self
                .summary_years
                .get(self.selected_summary_year_index)
                .copied()
            {
                self.update_selected_summary_month(year);
            } else {
                self.selected_summary_month = None;
            }
            self.table_state.select(Some(0));
        }
    }
    pub(crate) fn next_summary_month(&mut self) {
        if let Some(year) = self
            .summary_years
            .get(self.selected_summary_year_index)
            .copied()
        {
            let months = self.sorted_months_for_year(year);
            if let Some(current) = self.selected_summary_month {
                if let Some(idx) = months.iter().position(|&m| m == current) {
                    let next_idx = (idx + 1) % months.len();
                    self.selected_summary_month = Some(months[next_idx]);
                }
            } else if let Some(&first) = months.first() {
                self.selected_summary_month = Some(first);
            }
        }
    }
    pub(crate) fn previous_summary_month(&mut self) {
        if let Some(year) = self
            .summary_years
            .get(self.selected_summary_year_index)
            .copied()
        {
            let months = self.sorted_months_for_year(year);
            if let Some(current) = self.selected_summary_month {
                if let Some(idx) = months.iter().position(|&m| m == current) {
                    let prev_idx = if idx == 0 { months.len() - 1 } else { idx - 1 };
                    self.selected_summary_month = Some(months[prev_idx]);
                }
            } else if let Some(&first) = months.first() {
                self.selected_summary_month = Some(first);
            }
        }
    }
    // --- Category Summary Logic ---
    // Handles entering/exiting category summary mode, navigating years/items, and updating category summaries.
    pub(crate) fn enter_category_summary_mode(&mut self) {
        self.mode = AppMode::CategorySummary;
        self.calculate_category_summaries();
        if !self.category_summary_years.is_empty() {
            let current_year = chrono::Local::now().year();
            if let Some(idx) = self
                .category_summary_years
                .iter()
                .position(|&y| y == current_year)
            {
                self.category_summary_year_index = idx;
            } else {
                self.category_summary_year_index = self.category_summary_years.len() - 1;
            }
        }
        self.cached_visible_category_items = self.get_visible_category_summary_items();
        let len = self.cached_visible_category_items.len();
        self.category_summary_table_state
            .select(if len > 0 { Some(0) } else { None });
        self.status_message = None;
    }
    pub(crate) fn exit_category_summary_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.status_message = None;
    }
    pub(crate) fn next_category_summary_item(&mut self) {
        let list_len = self.cached_visible_category_items.len();
        if list_len == 0 {
            return;
        }
        let i = match self.category_summary_table_state.selected() {
            Some(i) => {
                if i >= list_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.category_summary_table_state.select(Some(i));
    }
    pub(crate) fn previous_category_summary_item(&mut self) {
        let list_len = self.cached_visible_category_items.len();
        if list_len == 0 {
            return;
        }
        let i = match self.category_summary_table_state.selected() {
            Some(i) => {
                if i == 0 {
                    list_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.category_summary_table_state.select(Some(i));
    }
    pub(crate) fn next_category_summary_year(&mut self) {
        if !self.category_summary_years.is_empty() {
            self.category_summary_year_index =
                (self.category_summary_year_index + 1) % self.category_summary_years.len();
            self.cached_visible_category_items = self.get_visible_category_summary_items();
            let len = self.cached_visible_category_items.len();
            self.category_summary_table_state
                .select(if len > 0 { Some(0) } else { None });
        }
    }
    pub(crate) fn previous_category_summary_year(&mut self) {
        if !self.category_summary_years.is_empty() {
            if self.category_summary_year_index > 0 {
                self.category_summary_year_index -= 1;
            } else {
                self.category_summary_year_index = self.category_summary_years.len() - 1;
            }
            self.cached_visible_category_items = self.get_visible_category_summary_items();
            let len = self.cached_visible_category_items.len();
            self.category_summary_table_state
                .select(if len > 0 { Some(0) } else { None });
        }
    }
    pub(crate) fn get_visible_category_summary_items(&self) -> Vec<CategorySummaryItem> {
        let mut items = Vec::new();
        if let Some(year) = self
            .category_summary_years
            .get(self.category_summary_year_index)
            .copied()
        {
            let mut months: Vec<u32> = self
                .category_summaries
                .keys()
                .filter_map(|(y, m)| if *y == year { Some(*m) } else { None })
                .collect();
            months.sort_unstable();
            months.dedup();
            for month in months {
                if let Some(month_map) = self.category_summaries.get(&(year, month)) {
                    let mut month_total = MonthlySummary::default();
                    for summary in month_map.values() {
                        month_total.income += summary.income;
                        month_total.expense += summary.expense;
                    }
                    items.push(CategorySummaryItem::Month(month, month_total));
                    if self.expanded_category_summary_months.contains(&month) {
                        let mut categories: Vec<String> =
                            month_map.keys().map(|(cat, _)| cat.clone()).collect();
                        categories.sort_unstable();
                        categories.dedup();
                        for category in categories {
                            let mut subcategories: Vec<String> = month_map
                                .keys()
                                .filter_map(|(cat, sub)| {
                                    if cat == &category && !sub.is_empty() {
                                        Some(sub.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            subcategories.sort_unstable();
                            // Always include the case where subcategory is empty (Uncategorized)
                            if month_map.contains_key(&(category.clone(), String::new())) {
                                subcategories.insert(0, String::new());
                            }
                            for subcategory in subcategories {
                                let display_category = if category.is_empty() {
                                    "Uncategorized".to_string()
                                } else {
                                    category.clone()
                                };
                                let display_subcategory = if subcategory.is_empty() {
                                    "Uncategorized".to_string()
                                } else {
                                    subcategory.clone()
                                };
                                if let Some(summary) =
                                    month_map.get(&(category.clone(), subcategory.clone()))
                                {
                                    items.push(CategorySummaryItem::Subcategory(
                                        month,
                                        display_category,
                                        display_subcategory,
                                        *summary,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        items
    }

    /// Jump to the next month in CategorySummary mode
    /// Finds the next Month item in the visible items list
    pub(crate) fn next_category_summary_month(&mut self) {
        let current_selection = self.category_summary_table_state.selected().unwrap_or(0);
        let items = &self.cached_visible_category_items;

        // Find the next month item after the current selection
        for (index, item) in items.iter().enumerate().skip(current_selection + 1) {
            if matches!(item, CategorySummaryItem::Month(_, _)) {
                self.category_summary_table_state.select(Some(index));
                return;
            }
        }

        // If no month found after current selection, wrap to first month
        for (index, item) in items.iter().enumerate() {
            if matches!(item, CategorySummaryItem::Month(_, _)) {
                self.category_summary_table_state.select(Some(index));
                return;
            }
        }
    }

    /// Jump to the previous month in CategorySummary mode
    /// Finds the previous Month item in the visible items list
    pub(crate) fn previous_category_summary_month(&mut self) {
        let current_selection = self.category_summary_table_state.selected().unwrap_or(0);
        let items = &self.cached_visible_category_items;

        // Find the previous month item before the current selection
        for (index, item) in items.iter().enumerate().take(current_selection).rev() {
            if matches!(item, CategorySummaryItem::Month(_, _)) {
                self.category_summary_table_state.select(Some(index));
                return;
            }
        }

        // If no month found before current selection, wrap to last month
        for (index, item) in items.iter().enumerate().rev() {
            if matches!(item, CategorySummaryItem::Month(_, _)) {
                self.category_summary_table_state.select(Some(index));
                return;
            }
        }
    }
}
