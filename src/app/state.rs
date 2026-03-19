use crate::app::update_checker;
use crate::category_store::{CategoryStore, SqliteCategoryStore};
use crate::config::{load_settings, AppSettings};
use crate::database::SqliteDatabase;
use crate::model::*;
use crate::persistence::{load_categories, load_transactions};
use chrono::{Datelike, Duration, NaiveDate};
use ratatui::widgets::{ListState, TableState};
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};
use std::fs::{copy, create_dir_all};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

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
    Budget,
    Settings,
    RecurringSettings,
    SelectingRecurrenceFrequency,
    KeybindingsInfo,
    KeybindingDetail,
    FuzzyFinding,
    CategoryCatalog,
    CategoryEditor,
    ConfirmCategoryDelete,
}

#[derive(Debug)]
pub enum CategorySummaryItem {
    Month(u32, MonthlySummary),
    Subcategory(u32, String, String, MonthlySummary),
}

#[derive(Debug, Clone)]
pub struct BudgetCategoryComparison {
    pub category: String,
    pub subcategory: String,
    pub target_budget: Decimal,
    pub actual_expense: Decimal,
}

pub struct App {
    pub(crate) transactions: Vec<Transaction>,
    pub(crate) filtered_indices: Vec<usize>,
    pub(crate) categories: Vec<CategoryInfo>,
    pub(crate) category_records: Vec<CategoryRecord>,
    pub(crate) data_file_path: PathBuf,
    pub(crate) database_path: PathBuf,
    pub(crate) should_quit: bool,
    pub(crate) table_state: TableState,
    pub(crate) mode: AppMode,
    pub(crate) simple_filter_content: String,
    pub(crate) simple_filter_cursor: usize,
    pub(crate) add_edit_fields: [String; 6],
    pub(crate) current_add_edit_field: usize,
    pub(crate) add_edit_cursor: usize,
    pub(crate) advanced_filter_fields: [String; 8],
    pub(crate) current_advanced_filter_field: usize,
    pub(crate) advanced_filter_cursor: usize,
    pub(crate) delete_index: Option<usize>,
    pub(crate) editing_index: Option<usize>,
    pub(crate) status_message: Option<String>,
    pub(crate) status_expiry: Option<std::time::Instant>,
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
    pub(crate) type_to_select: crate::app::util::TypeToSelect,
    // Category Summary State
    pub(crate) category_summary_table_state: TableState,
    pub(crate) category_summaries: HashMap<(i32, u32), HashMap<(String, String), MonthlySummary>>,
    pub(crate) category_summary_years: Vec<i32>,
    pub(crate) category_summary_year_index: usize,
    // Expansion state for hierarchical category summary
    pub(crate) expanded_category_summary_months: HashSet<u32>,
    // Flattened list of visible items for rendering and navigation
    pub(crate) cached_visible_category_items: Vec<CategorySummaryItem>,
    // Budget view state
    pub(crate) budget_years: Vec<i32>,
    pub(crate) budget_year_index: usize,
    pub(crate) selected_budget_month: Option<u32>,
    pub(crate) budget_table_state: TableState,
    // Settings form state
    pub(crate) settings_state: crate::app::settings_types::SettingsState,
    // Category catalog state
    pub(crate) category_table_state: TableState,
    pub(crate) category_edit_fields: [String; 5], // [type, category, subcategory, tag, target_budget]
    pub(crate) current_category_field: usize,
    pub(crate) category_edit_cursor: usize,
    pub(crate) editing_category_id: Option<i64>,
    pub(crate) category_delete_id: Option<i64>,
    // Budget
    pub(crate) target_budget: Option<Decimal>,
    pub(crate) hourly_rate: Option<Decimal>,
    pub(crate) show_hours: bool,
    pub(crate) fuzzy_search_mode: bool,
    pub(crate) search_query: String,
    // Recurring transaction state
    pub(crate) recurring_settings_fields: [String; 3], // [is_recurring, frequency, end_date]
    pub(crate) current_recurring_field: usize,
    pub(crate) recurring_transaction_index: Option<usize>,
    // Help/Keybindings
    pub(crate) previous_mode: Option<AppMode>,
    pub(crate) help_table_state: TableState,
    pub(crate) hide_help_bar: bool,
    // Update Check
    pub(crate) update_available_version: Option<String>,
    pub(crate) show_update_popup: bool,
    pub(crate) update_rx: mpsc::Receiver<Option<String>>,
}

impl App {
    pub fn new() -> Self {
        // --- Start Update Check ---
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = update_checker::check_for_updates();
            let _ = tx.send(result);
        });

        // --- Load Settings ---
        let (loaded_settings, load_settings_error_msg) = match load_settings() {
            Ok(settings) => (settings, None),
            Err(e) => (
                AppSettings::default(),
                Some(format!("Config Load Error: {}. Using defaults.", e)),
            ),
        };

        let (initial_data_file_path, data_path_error_msg) = Self::resolve_configured_path(
            loaded_settings.data_file_path.clone(),
            Self::get_default_data_file_path,
            "transactions.csv",
            "Data file",
        );
        let (initial_database_path, database_path_error_msg) =
            match loaded_settings.database_path.clone() {
                Some(path) => Self::resolve_configured_path(
                    Some(path),
                    Self::get_default_database_file_path,
                    "budget.db",
                    "Database",
                ),
                None => Self::resolve_default_database_path(&initial_data_file_path),
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

        let (seed_categories, load_seed_error_msg) = match load_categories() {
            Ok(cats) => (cats, None),
            Err(e) => (vec![], Some(format!("Embedded Category Seed Error: {}", e))),
        };
        let (category_records, load_cat_error_msg) =
            match Self::load_category_records(&initial_database_path, &seed_categories) {
                Ok(records) => (records, None),
                Err(e) => (
                    vec![],
                    Some(format!(
                        "Category DB Error [{}]: {}",
                        initial_database_path.display(),
                        e
                    )),
                ),
            };
        let categories = if category_records.is_empty() {
            seed_categories.clone()
        } else {
            Self::project_categories(&category_records)
        };

        // Combine potential errors (settings, paths, tx load, category load)
        let load_tx_error_msg = [
            load_settings_error_msg,
            data_path_error_msg,
            load_tx_specific_error_msg,
            database_path_error_msg,
            load_seed_error_msg,
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
            category_records,
            data_file_path: initial_data_file_path,
            database_path: initial_database_path,
            should_quit: false,
            table_state: TableState::default(),
            mode: AppMode::Normal,
            simple_filter_content: String::new(),
            simple_filter_cursor: 0,
            add_edit_fields: Default::default(),
            current_add_edit_field: 0,
            add_edit_cursor: 0,
            advanced_filter_fields: Default::default(),
            current_advanced_filter_field: 0,
            advanced_filter_cursor: 0,
            delete_index: None,
            editing_index: None,
            status_message: load_error_msg,
            status_expiry: None,
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
            type_to_select: crate::app::util::TypeToSelect::new(),
            category_summaries: HashMap::new(),
            category_summary_years: Vec::new(),
            category_summary_year_index: 0,
            category_summary_table_state: TableState::default(),
            expanded_category_summary_months: HashSet::new(),
            cached_visible_category_items: Vec::new(),
            budget_years: Vec::new(),
            budget_year_index: 0,
            selected_budget_month: None,
            budget_table_state: TableState::default(),
            settings_state: crate::app::settings_types::SettingsState::default(),
            category_table_state: TableState::default(),
            category_edit_fields: Default::default(),
            current_category_field: 0,
            category_edit_cursor: 0,
            editing_category_id: None,
            category_delete_id: None,
            target_budget: loaded_settings.target_budget,
            hourly_rate: loaded_settings.hourly_rate,
            show_hours: loaded_settings.show_hours.unwrap_or(false),
            fuzzy_search_mode: loaded_settings.fuzzy_search_mode.unwrap_or(false),
            search_query: String::new(),
            recurring_settings_fields: Default::default(),
            current_recurring_field: 0,
            recurring_transaction_index: None,
            previous_mode: None,
            help_table_state: TableState::default(),
            hide_help_bar: loaded_settings.hide_help_bar.unwrap_or(false),
            update_available_version: None,
            show_update_popup: false,
            update_rx: rx,
        };
        app.calculate_monthly_summaries();
        app.calculate_category_summaries();
        app.refresh_budget_years();
        if !app.summary_years.is_empty() {
            app.selected_summary_year_index = app.summary_years.len() - 1;
        }
        if !app.transactions.is_empty() {
            app.table_state.select(Some(0));
        }

        // Generate recurring transactions up to today
        app.generate_recurring_transactions();

        app
    }

    fn resolve_configured_path(
        configured_path: Option<String>,
        default_path_fn: fn() -> Result<PathBuf, Error>,
        fallback_name: &str,
        label: &str,
    ) -> (PathBuf, Option<String>) {
        match configured_path {
            Some(path_str) => {
                let path = PathBuf::from(path_str);
                if let Some(parent) = path.parent() {
                    if let Err(err) = create_dir_all(parent) {
                        let fallback =
                            default_path_fn().unwrap_or_else(|_| PathBuf::from(fallback_name));
                        return (
                            fallback,
                            Some(format!(
                                "{} path error: Could not create parent dir for {}: {}. Using default.",
                                label,
                                path.display(),
                                err
                            )),
                        );
                    }
                }

                (path, None)
            }
            None => match default_path_fn() {
                Ok(path) => (path, None),
                Err(err) => (
                    PathBuf::from(fallback_name),
                    Some(format!(
                        "{} default path error: {}. Using local '{}'.",
                        label, err, fallback_name
                    )),
                ),
            },
        }
    }

    fn project_categories(records: &[CategoryRecord]) -> Vec<CategoryInfo> {
        records
            .iter()
            .map(CategoryRecord::to_category_info)
            .collect()
    }

    fn resolve_default_database_path(data_file_path: &Path) -> (PathBuf, Option<String>) {
        let default_path = Self::default_database_path_for_data_path(data_file_path);

        if let Some(parent) = default_path.parent() {
            if let Err(err) = create_dir_all(parent) {
                return (
                    PathBuf::from("budget.db"),
                    Some(format!(
                        "Database default path error: Could not create parent dir for {}: {}. Using local 'budget.db'.",
                        default_path.display(),
                        err
                    )),
                );
            }
        }

        (default_path, None)
    }

    fn category_store_for_path(database_path: &Path) -> SqliteCategoryStore {
        SqliteCategoryStore::new(SqliteDatabase::new(database_path))
    }

    pub(crate) fn category_store(&self) -> SqliteCategoryStore {
        Self::category_store_for_path(&self.database_path)
    }

    pub(crate) fn initialize_category_database(
        database_path: &Path,
        seed_categories: &[CategoryInfo],
    ) -> Result<(), Error> {
        let store = Self::category_store_for_path(database_path);
        store.initialize(seed_categories)?;
        Ok(())
    }

    pub(crate) fn prepare_category_database_for_path_change(
        current_database_path: &Path,
        new_database_path: &Path,
        seed_categories: &[CategoryInfo],
    ) -> Result<(), Error> {
        if current_database_path != new_database_path
            && !new_database_path.exists()
            && current_database_path.exists()
        {
            let destination_database = SqliteDatabase::new(new_database_path);
            destination_database.ensure_parent_dir()?;
            copy(current_database_path, new_database_path).map_err(|err| {
                Error::other(format!(
                    "Failed to copy database from '{}' to '{}': {}",
                    current_database_path.display(),
                    new_database_path.display(),
                    err
                ))
            })?;
        }

        Self::initialize_category_database(new_database_path, seed_categories)
    }

    fn load_category_records(
        database_path: &Path,
        seed_categories: &[CategoryInfo],
    ) -> Result<Vec<CategoryRecord>, Error> {
        let store = Self::category_store_for_path(database_path);
        store.initialize(seed_categories)?;
        store.list()
    }

    pub(crate) fn refresh_category_state(&mut self, records: Vec<CategoryRecord>) {
        self.category_records = records;
        self.categories = Self::project_categories(&self.category_records);
    }

    pub(crate) fn refresh_categories_from_database(&mut self) -> Result<(), Error> {
        let store = self.category_store();
        let records = store.list()?;
        self.refresh_category_state(records);
        Ok(())
    }

    pub(crate) fn reload_categories_from_store(&mut self) -> Result<(), Error> {
        self.refresh_categories_from_database()
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
    pub fn next_item(&mut self) {
        let list_len = match self.mode {
            AppMode::Normal | AppMode::Filtering => self.filtered_indices.len(),
            AppMode::Summary => 12,
            AppMode::CategorySummary => self.cached_visible_category_items.len(),
            AppMode::Budget => 12,
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
            AppMode::Budget => 12,
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

    /// Jump to the very first item in the transaction list
    /// Only works in Normal and Filtering modes
    pub fn jump_to_first(&mut self) {
        match self.mode {
            AppMode::Normal | AppMode::Filtering => {
                let list_len = self.filtered_indices.len();
                if list_len > 0 {
                    self.table_state.select(Some(0));
                }
            }
            _ => {}
        }
    }

    /// Jump to the very last item in the transaction list
    /// Only works in Normal and Filtering modes
    pub fn jump_to_last(&mut self) {
        match self.mode {
            AppMode::Normal | AppMode::Filtering => {
                let list_len = self.filtered_indices.len();
                if list_len > 0 {
                    let last_index = list_len - 1;
                    self.table_state.select(Some(last_index));
                }
            }
            _ => {}
        }
    }

    /// Page size for transaction navigation (PageUp/PageDown)
    const TRANSACTION_PAGE_SIZE: usize = 20;

    /// Jump up by approximately one page worth of transactions
    /// Only works in Normal and Filtering modes
    pub fn page_up(&mut self) {
        match self.mode {
            AppMode::Normal | AppMode::Filtering => {
                let list_len = self.filtered_indices.len();
                if list_len == 0 {
                    return;
                }

                let page_size = Self::TRANSACTION_PAGE_SIZE;
                let current_selection = self.table_state.selected().unwrap_or(0);
                let new_selection = current_selection.saturating_sub(page_size);

                self.table_state.select(Some(new_selection));
            }
            _ => {}
        }
    }

    /// Jump down by approximately one page worth of transactions
    /// Only works in Normal and Filtering modes
    pub fn page_down(&mut self) {
        match self.mode {
            AppMode::Normal | AppMode::Filtering => {
                let list_len = self.filtered_indices.len();
                if list_len == 0 {
                    return;
                }

                let page_size = Self::TRANSACTION_PAGE_SIZE;
                let current_selection = self.table_state.selected().unwrap_or(0);
                let new_selection = (current_selection + page_size).min(list_len - 1);

                self.table_state.select(Some(new_selection));
            }
            _ => {}
        }
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
        self.refresh_budget_years();
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
                        // Use centralized month arithmetic
                        crate::validation::add_months(current_date, amount as i32)
                    }
                };
                self.add_edit_fields[0] = new_date.format(crate::model::DATE_FORMAT).to_string();
                self.add_edit_cursor = self.add_edit_fields[0].len();
                self.clear_status_message() // Clear status on successful adjustment
            } else {
                self.set_status_message(
                    format!(
                        "Error: Could not parse date '{}'. Use YYYY-MM-DD format.",
                        self.add_edit_fields[0]
                    ),
                    None,
                );
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

    pub fn default_database_path_for_data_path(data_file_path: &Path) -> PathBuf {
        const DATABASE_FILE_NAME: &str = "budget.db";

        match data_file_path.parent() {
            Some(parent) => parent.join(DATABASE_FILE_NAME),
            None => PathBuf::from(DATABASE_FILE_NAME),
        }
    }

    pub fn get_default_database_file_path() -> Result<PathBuf, Error> {
        Self::get_default_data_file_path()
            .map(|data_path| Self::default_database_path_for_data_path(&data_path))
    }
    // --- Sorting Logic ---
    pub(crate) fn set_sort_column(&mut self, column: SortColumn) {
        if self.sort_by == column {
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            self.sort_by = column;
            self.sort_order = SortOrder::Ascending;
        }

        // Preserve the current filter type when sorting
        // Check if any advanced filter fields are active
        let has_advanced_filters = self
            .advanced_filter_fields
            .iter()
            .any(|field| !field.is_empty());

        if has_advanced_filters {
            self.apply_advanced_filter();
        } else {
            self.apply_filter();
        }
    }

    pub fn set_status_message<S: Into<String>>(&mut self, message: S, duration: Option<Duration>) {
        self.status_message = Some(message.into());
        self.status_expiry = duration.map(|d| {
            std::time::Instant::now() + std::time::Duration::from_secs(d.num_seconds() as u64)
        });
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
        self.status_expiry = None;
    }
}
