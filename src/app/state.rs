use crate::config::{load_settings, AppSettings};
use crate::model::*;
use crate::persistence::{load_categories, load_transactions};
use chrono::{Datelike, Duration, NaiveDate};
use ratatui::widgets::{ListState, TableState};
use std::collections::{HashMap, HashSet};
use std::fs::create_dir_all;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub(crate) enum DateUnit {
    Day,
    Month,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AppMode {
    Normal,
    Adding,
    Editing,
    ConfirmDelete,
    Filtering,
    AdvancedFiltering,
    SelectingFilterCategory,
    SelectingFilterSubcategory,
    Summary,
    SelectingCategory,
    SelectingSubcategory,
    CategorySummary,
    Settings,
}

#[derive(Debug)]
pub enum CategorySummaryItem {
    Month(u32, MonthlySummary),
    Subcategory(u32, String, String, MonthlySummary),
}

pub struct App {
    pub(crate) transactions: Vec<Transaction>,
    pub(crate) filtered_indices: Vec<usize>,
    pub(crate) categories: Vec<CategoryInfo>,
    pub(crate) data_file_path: PathBuf,
    pub(crate) should_quit: bool,
    pub(crate) table_state: TableState,
    pub(crate) mode: AppMode,
    pub(crate) input_field_content: String,
    pub(crate) input_field_cursor: usize,
    pub(crate) add_edit_fields: [String; 6],
    pub(crate) current_add_edit_field: usize,
    pub(crate) advanced_filter_fields: [String; 8],
    pub(crate) current_advanced_filter_field: usize,
    pub(crate) delete_index: Option<usize>,
    pub(crate) editing_index: Option<usize>,
    pub(crate) status_message: Option<String>,
    pub(crate) sort_by: SortColumn,
    pub(crate) sort_order: SortOrder,
    // Monthly Summary State
    pub(crate) monthly_summaries: HashMap<(i32, u32), MonthlySummary>,
    pub(crate) summary_years: Vec<i32>,
    pub(crate) selected_summary_year_index: usize,
    pub(crate) selected_summary_month: Option<u32>,
    pub(crate) summary_multi_month_mode: bool,
    pub(crate) summary_cumulative_mode: bool,
    // Category/Subcategory Selection Popup State
    pub(crate) selecting_field_index: Option<usize>,
    pub(crate) current_selection_list: Vec<String>,
    pub(crate) selection_list_state: ListState,
    // Category Summary State
    pub(crate) category_summary_table_state: TableState,
    pub(crate) category_summaries: HashMap<(i32, u32), HashMap<(String, String), MonthlySummary>>,
    pub(crate) category_summary_years: Vec<i32>,
    pub(crate) category_summary_year_index: usize,
    // Expansion state for hierarchical category summary
    pub(crate) expanded_category_summary_months: HashSet<u32>,
    // Flattened list of visible items for rendering and navigation
    pub(crate) cached_visible_category_items: Vec<CategorySummaryItem>,
    // Settings form state
    pub(crate) settings_fields: [String; 6],
    pub(crate) current_settings_field: usize,
    // Budget
    pub(crate) target_budget: Option<f64>,
}

impl App {
    pub fn new() -> Self {
        // --- Load Settings ---
        let (loaded_settings, load_settings_error_msg) = match load_settings() {
            Ok(settings) => (settings, None),
            Err(e) => (
                AppSettings::default(),
                Some(format!("Config Load Error: {}. Using defaults.", e)),
            ),
        };
        // --- Determine Data File Path (based on settings or default) ---
        let (initial_data_file_path, path_error_msg) = match loaded_settings.data_file_path {
            Some(path_str) => {
                let path = PathBuf::from(path_str);
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        if let Err(e) = create_dir_all(parent) {
                            {
                                let fallback = match Self::get_default_data_file_path() {
                                    Ok(p) => p,
                                    Err(_) => PathBuf::from("transactions.csv"),
                                };
                                (
                                    fallback,
                                    Some(format!("Config Path Error: Could not create parent dir for {}: {}. Using default.", path.display(), e)),
                                )
                            }
                        } else {
                            (path, None) // Parent created successfully
                        }
                    } else {
                        (path, None) // Parent already exists
                    }
                } else {
                    (path, None) // Path has no parent (e.g., relative path in current dir)
                }
            }
            None => match Self::get_default_data_file_path() {
                // No path in config, use default logic
                Ok(path) => (path, None),
                Err(e) => (
                    PathBuf::from("transactions.csv"),
                    Some(format!("Data Dir Error: {}. Using local.", e)),
                ),
            },
        };
        // --- Load Transactions ---
        let (mut transactions, load_tx_specific_error_msg) =
            match load_transactions(&initial_data_file_path) {
                Ok(txs) => (txs, None),
                Err(e) => (
                    vec![],
                    Some(format!(
                        "Load TX Error [{}]: {}",
                        initial_data_file_path.display(),
                        e
                    )),
                ),
            };
        // Combine potential errors (settings, path, tx load)
        let load_tx_error_msg = [
            load_settings_error_msg,
            path_error_msg,
            load_tx_specific_error_msg,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" | ");
        let load_tx_error_msg = if load_tx_error_msg.is_empty() {
            None
        } else {
            Some(load_tx_error_msg)
        };
        // --- Load Categories ---
        let (categories, load_cat_error_msg) = match load_categories() {
            Ok(cats) => (cats, None),
            Err(e) => (vec![], Some(format!("Load Category Error: {}", e))),
        };
        // --- Combine All Load Errors ---
        let load_error_msg = match (load_tx_error_msg, load_cat_error_msg) {
            (Some(tx_err), Some(cat_err)) => Some(format!("{} | {}", tx_err, cat_err)),
            (Some(tx_err), None) => Some(tx_err),
            (None, Some(cat_err)) => Some(cat_err),
            (None, None) => None,
        };
        let initial_sort_by = SortColumn::Date;
        let initial_sort_order = SortOrder::Descending;
        crate::app::util::sort_transactions_impl(
            &mut transactions,
            initial_sort_by,
            initial_sort_order,
        );
        let initial_filtered_indices = (0..transactions.len()).collect();
        let mut app = Self {
            transactions,
            filtered_indices: initial_filtered_indices,
            categories,
            data_file_path: initial_data_file_path,
            should_quit: false,
            table_state: TableState::default(),
            mode: AppMode::Normal,
            input_field_content: String::new(),
            input_field_cursor: 0,
            add_edit_fields: Default::default(),
            current_add_edit_field: 0,
            advanced_filter_fields: Default::default(),
            current_advanced_filter_field: 0,
            delete_index: None,
            editing_index: None,
            status_message: load_error_msg,
            sort_by: initial_sort_by,
            sort_order: initial_sort_order,
            monthly_summaries: HashMap::new(),
            summary_years: Vec::new(),
            selected_summary_year_index: 0,
            selected_summary_month: None,
            summary_multi_month_mode: false,
            summary_cumulative_mode: false,
            selecting_field_index: None,
            current_selection_list: Vec::new(),
            selection_list_state: ListState::default(),
            category_summaries: HashMap::new(),
            category_summary_years: Vec::new(),
            category_summary_year_index: 0,
            category_summary_table_state: TableState::default(),
            expanded_category_summary_months: HashSet::new(),
            cached_visible_category_items: Vec::new(),
            settings_fields: Default::default(),
            current_settings_field: 0,
            target_budget: loaded_settings.target_budget,
        };
        app.calculate_monthly_summaries();
        app.calculate_category_summaries();
        if !app.summary_years.is_empty() {
            app.selected_summary_year_index = app.summary_years.len() - 1;
        }
        if !app.transactions.is_empty() {
            app.table_state.select(Some(0));
        }
        app
    }
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    pub fn next_item(&mut self) {
        let list_len = match self.mode {
            AppMode::Normal | AppMode::Filtering => self.filtered_indices.len(),
            AppMode::Summary => 12,
            AppMode::CategorySummary => self.cached_visible_category_items.len(),
            _ => 0,
        };
        if list_len == 0 {
            return;
        }
        let current_selection = self.table_state.selected().unwrap_or(0);
        let next_selection = if current_selection >= list_len - 1 {
            0
        } else {
            current_selection + 1
        };
        self.table_state.select(Some(next_selection));
    }
    pub fn previous_item(&mut self) {
        let list_len = match self.mode {
            AppMode::Normal | AppMode::Filtering => self.filtered_indices.len(),
            AppMode::Summary => 12,
            AppMode::CategorySummary => self.cached_visible_category_items.len(),
            _ => 0,
        };
        if list_len == 0 {
            return;
        }
        let current_selection = self.table_state.selected().unwrap_or(0);
        let prev_selection = if current_selection == 0 {
            list_len - 1
        } else {
            current_selection - 1
        };
        self.table_state.select(Some(prev_selection));
    }
    pub fn decrement_date(&mut self) {
        self.adjust_date(-1, DateUnit::Day);
    }
    pub fn increment_date(&mut self) {
        self.adjust_date(1, DateUnit::Day);
    }
    pub fn decrement_month(&mut self) {
        self.adjust_date(-1, DateUnit::Month);
    }
    pub fn increment_month(&mut self) {
        self.adjust_date(1, DateUnit::Month);
    }
    pub(crate) fn get_original_index(&self, filtered_view_index: usize) -> Option<usize> {
        self.filtered_indices.get(filtered_view_index).copied()
    }
    pub(crate) fn sort_transactions(&mut self) {
        crate::app::util::sort_transactions_impl(
            &mut self.transactions,
            self.sort_by,
            self.sort_order,
        );
    }
    pub(crate) fn calculate_monthly_summaries(&mut self) {
        self.monthly_summaries.clear();
        let mut years = Vec::new();
        for &idx in &self.filtered_indices {
            if let Some(tx) = self.transactions.get(idx) {
                let year = tx.date.year();
                let month = tx.date.month();
                let summary = self.monthly_summaries.entry((year, month)).or_default();
                match tx.transaction_type {
                    TransactionType::Income => summary.income += tx.amount,
                    TransactionType::Expense => summary.expense += tx.amount,
                }
                if !years.contains(&year) {
                    years.push(year);
                }
            }
        }
        years.sort_unstable();
        self.summary_years = years;
        if !self.summary_years.is_empty() {
            self.selected_summary_year_index = self
                .selected_summary_year_index
                .min(self.summary_years.len() - 1);
        } else {
            self.selected_summary_year_index = 0;
        }
    }
    pub(crate) fn calculate_category_summaries(&mut self) {
        self.category_summaries.clear();
        let mut years = HashSet::new();
        for tx in self
            .filtered_indices
            .iter()
            .map(|&idx| &self.transactions[idx])
        {
            let year = tx.date.year();
            let month = tx.date.month();
            years.insert(year);
            let category_key = tx.category.trim();
            let subcategory_key = tx.subcategory.trim();
            let final_category = if category_key.is_empty() {
                "Uncategorized"
            } else {
                category_key
            };
            let month_map = self.category_summaries.entry((year, month)).or_default();
            let summary = month_map
                .entry((final_category.to_string(), subcategory_key.to_string()))
                .or_default();
            match tx.transaction_type {
                TransactionType::Income => summary.income += tx.amount,
                TransactionType::Expense => summary.expense += tx.amount,
            }
        }
        self.category_summary_years = years.into_iter().collect();
        self.category_summary_years.sort_unstable();
        if !self.category_summary_years.is_empty() {
            self.category_summary_year_index = self
                .category_summary_year_index
                .min(self.category_summary_years.len() - 1);
        } else {
            self.category_summary_year_index = 0;
        }
        let list_len = self.cached_visible_category_items.len();
        if list_len == 0 {
            self.category_summary_table_state.select(None);
        } else {
            let current_selection = self.category_summary_table_state.selected().unwrap_or(0);
            let new_selection = current_selection.min(list_len - 1);
            self.category_summary_table_state
                .select(Some(new_selection));
        }
    }
    pub(crate) fn adjust_date(&mut self, amount: i64, unit: DateUnit) {
        if self.current_add_edit_field == 0 {
            if let Ok(current_date) =
                NaiveDate::parse_from_str(&self.add_edit_fields[0], crate::model::DATE_FORMAT)
            {
                let new_date = match unit {
                    DateUnit::Day => {
                        if amount > 0 {
                            current_date + Duration::days(amount)
                        } else {
                            current_date - Duration::days(-amount)
                        }
                    }
                    DateUnit::Month => {
                        let day = current_date.day();
                        let month = current_date.month() as i32;
                        let year = current_date.year();
                        let new_month = month + amount as i32;
                        let mut target_year = year + (new_month - 1) / 12;
                        let mut target_month = ((new_month - 1) % 12) + 1;
                        if target_month <= 0 {
                            target_month += 12;
                            target_year -= 1;
                        }
                        NaiveDate::from_ymd_opt(target_year, target_month as u32, day)
                            .unwrap_or_else(|| {
                                // Get the last day of the target month
                                let last_day = if target_month == 12 {
                                    NaiveDate::from_ymd_opt(target_year + 1, 1, 1).unwrap()
                                } else {
                                    NaiveDate::from_ymd_opt(target_year, target_month as u32 + 1, 1)
                                        .unwrap()
                                };
                                last_day - Duration::days(1)
                            })
                    }
                };
                self.add_edit_fields[0] = new_date.format(crate::model::DATE_FORMAT).to_string();
                self.status_message = None; // Clear status on successful adjustment
            } else {
                self.status_message = Some(format!(
                    "Error: Could not parse date '{}'. Use YYYY-MM-DD format.",
                    self.add_edit_fields[0]
                ));
            }
        }
    }
    pub fn get_default_data_file_path() -> Result<PathBuf, Error> {
        const DATA_FILE_NAME: &str = "transactions.csv";
        const APP_DATA_SUBDIR: &str = "BudgetTracker";
        match dirs::data_dir() {
            Some(mut path) => {
                path.push(APP_DATA_SUBDIR);
                create_dir_all(&path)?;
                path.push(DATA_FILE_NAME);
                Ok(path)
            }
            None => Err(Error::new(
                ErrorKind::NotFound,
                "Could not find user data directory",
            )),
        }
    }
}
