use super::state::App;
use crate::app::state::{AppMode, CategorySummaryItem};
use crate::model::MonthlySummary;
use chrono;
use chrono::Datelike;

impl App {
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
            self.table_state.select(Some(0));
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
                            for subcategory in subcategories {
                                if let Some(summary) =
                                    month_map.get(&(category.clone(), subcategory.clone()))
                                {
                                    items.push(CategorySummaryItem::Subcategory(
                                        month,
                                        category.clone(),
                                        subcategory.clone(),
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
}
