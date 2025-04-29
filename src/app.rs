use crate::config::{load_settings, save_settings, AppSettings};
use crate::model::*;
use crate::persistence::{load_categories, load_transactions, save_transactions};
use chrono::{Datelike, Duration, Local, NaiveDate};
use ratatui::widgets::{ListState, TableState};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::create_dir_all;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub(crate) enum DateUnit {
    Day,
    Month,
}

// Define application modes
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
    categories: Vec<CategoryInfo>,
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
}

impl App {
    pub(crate) fn new() -> Self {
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
            None => {
                // No path in config, use default logic
                match Self::get_default_data_file_path() {
                    Ok(path) => (path, None),
                    Err(e) => (
                        PathBuf::from("transactions.csv"),
                        Some(format!("Data Dir Error: {}. Using local.", e)),
                    ),
                }
            }
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
        sort_transactions_impl(&mut transactions, initial_sort_by, initial_sort_order);

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
            selecting_field_index: None,
            current_selection_list: Vec::new(),
            selection_list_state: ListState::default(),
            category_summaries: HashMap::new(),
            category_summary_years: Vec::new(),
            category_summary_year_index: 0,
            category_summary_table_state: TableState::default(),
            expanded_category_summary_months: HashSet::new(),
            cached_visible_category_items: Vec::new(),
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

    pub(crate) fn quit(&mut self) {
        self.should_quit = true;
    }

    // --- Index Mapping ---
    fn get_original_index(&self, filtered_view_index: usize) -> Option<usize> {
        self.filtered_indices.get(filtered_view_index).copied()
    }

    // --- List Navigation (using filtered indices) ---
    pub(crate) fn next_item(&mut self) {
        let list_len = match self.mode {
            AppMode::Normal | AppMode::Filtering => self.filtered_indices.len(),
            AppMode::Summary => 12, // Always 12 months in the view
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

    pub(crate) fn previous_item(&mut self) {
        let list_len = match self.mode {
            AppMode::Normal | AppMode::Filtering => self.filtered_indices.len(),
            AppMode::Summary => 12, // Always 12 months
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

    // --- Adding Logic ---
    pub(crate) fn start_adding(&mut self) {
        self.mode = AppMode::Adding;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default();

        // Default Date to today
        let today = Local::now().date_naive();
        self.add_edit_fields[0] = today.format(DATE_FORMAT).to_string();

        // Default Type to Expense
        self.add_edit_fields[3] = "Expense".to_string();
        self.status_message = None;
    }

    pub(crate) fn exit_adding(&mut self) {
        self.mode = AppMode::Normal;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default();
        self.status_message = Some("Select a transaction to edit first".to_string());
    }

    // --- Date Adjustment Logic ---
    fn adjust_date(&mut self, amount: i64, unit: DateUnit) {
        if self.current_add_edit_field == 0 {
            if let Ok(current_date) =
                NaiveDate::parse_from_str(&self.add_edit_fields[0], DATE_FORMAT)
            {
                let new_date = match unit {
                    DateUnit::Day => {
                        if amount > 0 {
                            current_date + Duration::days(amount)
                        } else {
                            current_date - Duration::days(-amount)
                        }
                    },  
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
                                    NaiveDate::from_ymd_opt(target_year, target_month as u32 + 1, 1).unwrap()
                                };
                                last_day - Duration::days(1)
                            })
                    }
                };
                
                self.add_edit_fields[0] = new_date.format(DATE_FORMAT).to_string();
                self.status_message = None; // Clear status on successful adjustment
            } else {
                self.status_message = Some(format!(
                    "Error: Could not parse date '{}'. Use YYYY-MM-DD format.",
                    self.add_edit_fields[0]
                ));
            }
        }
    }

    pub(crate) fn increment_date(&mut self) {
        self.adjust_date(1, DateUnit::Day);
    }

    pub(crate) fn decrement_date(&mut self) {
        self.adjust_date(-1, DateUnit::Day);
    }

    pub(crate) fn increment_month(&mut self) {
        self.adjust_date(1, DateUnit::Month);
    }

    pub(crate) fn decrement_month(&mut self) {
        self.adjust_date(-1, DateUnit::Month);
    }

    // --- Validation Helper ---
    fn validate_category(
        &self,
        transaction_type: TransactionType,
        category: &str,
        subcategory: &str,
    ) -> Result<(), String> {
        // Allow "Uncategorized" or empty category
        if category.is_empty() || category.eq_ignore_ascii_case("Uncategorized") {
            return Ok(());
        }

        let category_lower = category.to_lowercase();
        let subcategory_lower = subcategory.to_lowercase();

        // Find matching category info
        let is_valid = self.categories.iter().any(|cat_info| {
            cat_info.transaction_type == transaction_type
                && cat_info.category.to_lowercase() == category_lower
                && (cat_info.subcategory.to_lowercase() == subcategory_lower
                    || cat_info.subcategory.is_empty())
        });

        if is_valid {
            Ok(())
        } else {
            Err(format!(
                "Invalid Category/Subcategory: '{}' / '{}' for {:?}",
                category, subcategory, transaction_type
            ))
        }
    }

    pub(crate) fn add_transaction(&mut self) {
        let date_res = NaiveDate::parse_from_str(&self.add_edit_fields[0], DATE_FORMAT);
        let description = self.add_edit_fields[1].trim();
        let amount_res = self.add_edit_fields[2].parse::<f64>();
        let type_str = self.add_edit_fields[3].trim().to_lowercase();
        let category = self.add_edit_fields[4].trim();
        let subcategory = self.add_edit_fields[5].trim();

        let transaction_type = if type_str.starts_with('i') {
            TransactionType::Income
        } else {
            TransactionType::Expense
        };

        if let Err(cat_err) = self.validate_category(transaction_type, category, subcategory) {
            self.status_message = Some(format!("Error: {}", cat_err));
            return;
        }

        match (date_res, description, amount_res) {
            (Ok(date), desc, Ok(amount)) if !desc.is_empty() && amount > 0.0 => {
                let new_transaction = Transaction {
                    date,
                    description: desc.to_string(),
                    amount,
                    transaction_type,
                    category: category.to_string(),
                    subcategory: subcategory.to_string(),
                };
                self.transactions.push(new_transaction);
                self.sort_transactions();
                self.apply_filter();
                self.calculate_monthly_summaries();

                match save_transactions(&self.transactions, &self.data_file_path) {
                    Ok(_) => {
                        self.status_message = Some("Transaction added successfully.".to_string());
                        self.exit_adding();
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving transaction: {}", e));
                    }
                }
            }
            (Err(_), _, _) => {
                self.status_message = Some(format!(
                    "Error: Invalid Date Format (Expected {})",
                    DATE_FORMAT
                ));
            }
            (_, _, Err(_)) => {
                self.status_message = Some("Error: Invalid Amount (Must be a number)".to_string());
            }
            (_, "", _) => {
                self.status_message = Some("Error: Description cannot be empty".to_string());
            }
            (_, _, Ok(amount)) if amount <= 0.0 => {
                self.status_message = Some("Error: Amount must be positive".to_string());
            }
            _ => {
                self.status_message = Some("Error: Could not add transaction".to_string());
            }
        }
    }

    // --- Editing Logic ---
    pub(crate) fn start_editing(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                let tx = &self.transactions[original_index];
                self.mode = AppMode::Editing;
                self.editing_index = Some(original_index);
                self.current_add_edit_field = 0;
                // Populate all 6 fields
                self.add_edit_fields = [
                    tx.date.format(DATE_FORMAT).to_string(),
                    tx.description.clone(),
                    format!("{:.2}", tx.amount),
                    if tx.transaction_type == TransactionType::Income {
                        "Income".to_string()
                    } else {
                        "Expense".to_string()
                    },
                    tx.category.clone(),
                    tx.subcategory.clone(),
                ];
                self.status_message = None;
            } else {
                self.status_message =
                    Some("Error: Could not map view index to transaction".to_string());
            }
        } else {
            self.status_message = Some("Select a transaction to edit first".to_string());
        }
    }

    pub(crate) fn exit_editing(&mut self) {
        self.mode = AppMode::Normal;
        self.editing_index = None;
        self.current_add_edit_field = 0;
        self.add_edit_fields = Default::default();
        self.status_message = None;
    }

    pub(crate) fn update_transaction(&mut self) {
        if let Some(index) = self.editing_index {
            let date_res = NaiveDate::parse_from_str(&self.add_edit_fields[0], DATE_FORMAT);
            let description = self.add_edit_fields[1].trim();
            let amount_res = self.add_edit_fields[2].parse::<f64>();
            let type_str = self.add_edit_fields[3].trim().to_lowercase();
            let category = self.add_edit_fields[4].trim();
            let subcategory = self.add_edit_fields[5].trim();

            // Determine transaction type first for validation
            let transaction_type = if type_str.starts_with('i') {
                TransactionType::Income
            } else {
                TransactionType::Expense
            };

            if let Err(cat_err) = self.validate_category(transaction_type, category, subcategory) {
                self.status_message = Some(format!("Error: {}", cat_err));
                return;
            }

            match (date_res, description, amount_res) {
                (Ok(date), desc, Ok(amount)) if !desc.is_empty() && amount > 0.0 => {
                    if index < self.transactions.len() {
                        self.transactions[index] = Transaction {
                            date,
                            description: desc.to_string(),
                            amount,
                            transaction_type,
                            category: category.to_string(),
                            subcategory: subcategory.to_string(),
                        };
                        // Remove sort_transactions() and apply_filter() from here
                        // self.sort_transactions(); // Redundant
                        // self.apply_filter();

                        match save_transactions(&self.transactions, &self.data_file_path) {
                            Ok(_) => {
                                // Apply filter *after* successful save
                                self.status_message =
                                    Some("Transaction updated successfully.".to_string());
                                self.apply_filter();
                                self.calculate_monthly_summaries();
                                self.exit_editing();
                            }
                            Err(e) => {
                                self.status_message =
                                    Some(format!("Error saving updated transaction: {}", e));
                                // UI state (sorting/filtering) will not be updated on error
                            }
                        }
                    } else {
                        self.status_message = Some("Error: Invalid index during edit".to_string());
                        self.exit_editing();
                    }
                }
                (Err(_), _, _) => {
                    self.status_message = Some(format!(
                        "Error: Invalid Date Format (Expected {})",
                        DATE_FORMAT
                    ));
                }
                (_, _, Err(_)) => {
                    self.status_message =
                        Some("Error: Invalid Amount (Must be a number)".to_string());
                }
                (_, "", _) => {
                    self.status_message = Some("Error: Description cannot be empty".to_string());
                }
                (_, _, Ok(amount)) if amount <= 0.0 => {
                    self.status_message = Some("Error: Amount must be positive".to_string());
                }
                _ => {
                    self.status_message = Some("Error: Could not update transaction".to_string());
                }
            }
        } else {
            self.status_message = Some("Error: No transaction selected for editing".to_string());
            self.exit_editing();
        }
    }

    // --- Toggle Transaction Type ---
    pub(crate) fn toggle_transaction_type(&mut self) {
        if self.current_add_edit_field == 3 {
            self.add_edit_fields[3] = if self.add_edit_fields[3].eq_ignore_ascii_case("income") {
                "Expense".to_string()
            } else {
                "Income".to_string()
            };
            // Clear category/subcategory as they might be invalid for the new type
            self.add_edit_fields[4] = String::new();
            self.add_edit_fields[5] = String::new();
        }
    }

    // --- Deleting Logic ---
    pub(crate) fn prepare_for_delete(&mut self) {
        if let Some(view_index) = self.table_state.selected() {
            if let Some(original_index) = self.get_original_index(view_index) {
                self.delete_index = Some(original_index);
                self.mode = AppMode::ConfirmDelete;
                self.status_message = Some("Confirm Delete? (y/n)".to_string());
            } else {
                self.status_message =
                    Some("Error: Could not map view index to transaction".to_string());
            }
        } else {
            self.status_message = Some("Select a transaction to delete first".to_string());
        }
    }

    pub(crate) fn confirm_delete(&mut self) {
        if let Some(original_index) = self.delete_index {
            self.transactions.remove(original_index);
            self.apply_filter();

            if let Some(selected) = self.table_state.selected() {
                if selected >= self.filtered_indices.len() && !self.filtered_indices.is_empty() {
                    self.table_state
                        .select(Some(self.filtered_indices.len() - 1));
                }
            }

            match save_transactions(&self.transactions, &self.data_file_path) {
                Ok(_) => {
                    self.status_message = Some("Transaction deleted successfully.".to_string());
                    self.delete_index = None;
                    self.mode = AppMode::Normal;
                    self.calculate_monthly_summaries();
                    self.calculate_category_summaries();
                }
                Err(e) => {
                    self.status_message = Some(format!("Error saving after delete: {}", e));
                }
            }
        } else {
            self.cancel_delete();
        }
    }

    pub(crate) fn cancel_delete(&mut self) {
        self.mode = AppMode::Normal;
        self.delete_index = None;
        self.status_message = None;
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
        self.apply_filter();
    }

    fn sort_transactions(&mut self) {
        sort_transactions_impl(&mut self.transactions, self.sort_by, self.sort_order);
    }

    // --- Filtering Logic ---
    pub(crate) fn start_filtering(&mut self) {
        self.mode = AppMode::Filtering;
        self.input_field_cursor = self.input_field_content.len();
        self.status_message = None;
    }

    pub(crate) fn exit_filtering(&mut self) {
        self.mode = AppMode::Normal;
        self.status_message = None;
    }

    pub(crate) fn apply_filter(&mut self) {
        sort_transactions_impl(&mut self.transactions, self.sort_by, self.sort_order);

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
                    // TODO: Extend filtering (e.g., category, date range)
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

    // --- Input Handling ---
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
            if let Some(new_content) = validate_and_insert_date_char(field_content, c) {
                *field_content = new_content;
            }
        } 
        // Special handling for the Amount field (index 2)
        else if current_field == 2 {
            // Only allow digits and one decimal point
            if c.is_ascii_digit() || (c == '.' && !field_content.contains('.')) {
                field_content.push(c);
            }
        }
        else {
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
                field_content.pop(); // Standard backspace
            }
        } else if !field_content.is_empty() {
            // Default behavior for other fields
            field_content.pop();
        }
    }
    pub(crate) fn next_add_edit_field(&mut self) {
        let next_field = (self.current_add_edit_field + 1) % self.add_edit_fields.len();
        self.current_add_edit_field = next_field;
    }

    pub(crate) fn previous_add_edit_field(&mut self) {
        if self.current_add_edit_field == 0 {
            self.current_add_edit_field = self.add_edit_fields.len() - 1;
        } else {
            self.current_add_edit_field -= 1;
        }
    }

    // --- Summary Logic ---
    fn calculate_monthly_summaries(&mut self) {
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

    pub(crate) fn enter_summary_mode(&mut self) {
        self.mode = AppMode::Summary;
        self.calculate_monthly_summaries();
        if !self.summary_years.is_empty() {
            self.selected_summary_year_index = self.summary_years.len() - 1;
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
    fn calculate_category_summaries(&mut self) {
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

        // Reset selection based on the potentially new list for the current year/month
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

    pub(crate) fn enter_category_summary_mode(&mut self) {
        self.mode = AppMode::CategorySummary;
        self.calculate_category_summaries();
        self.cached_visible_category_items = self.get_visible_category_summary_items();
        if !self.category_summary_years.is_empty() {
            self.category_summary_year_index = self.category_summary_years.len() - 1;
        }
        // Select first visible item (or none)
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
            // Refresh cache and select first visible row
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
            // Refresh cache and select first visible row
            self.cached_visible_category_items = self.get_visible_category_summary_items();
            let len = self.cached_visible_category_items.len();
            self.category_summary_table_state
                .select(if len > 0 { Some(0) } else { None });
        }
    }

    // --- Category/Subcategory Selection Logic ---
    pub(crate) fn start_category_selection(&mut self) {
        self.selecting_field_index = Some(4); // Index of Category field
        self.mode = AppMode::SelectingCategory;

        let current_type_str = self.add_edit_fields[3].trim();
        let Ok(current_type) = TransactionType::try_from(current_type_str) else {
            self.status_message =
                Some("Error: Invalid transaction type for category lookup.".to_string());
            self.mode = if self.editing_index.is_some() {
                AppMode::Editing
            } else {
                AppMode::Adding
            };
            return;
        };

        let mut unique_categories: HashSet<String> = self
            .categories
            .iter()
            .filter(|cat_info| cat_info.transaction_type == current_type)
            .map(|cat_info| cat_info.category.clone())
            .collect();

        let mut options: Vec<String> = unique_categories.drain().collect();
        options.sort_unstable();

        self.current_selection_list = options;
        self.selection_list_state = ListState::default();
        if !self.current_selection_list.is_empty() {
            self.selection_list_state.select(Some(0));
        }
    }

    pub(crate) fn start_subcategory_selection(&mut self) {
        self.selecting_field_index = Some(5); // Index of Subcategory field
        self.mode = AppMode::SelectingSubcategory;

        let current_type_str = self.add_edit_fields[3].trim();
        let current_category = self.add_edit_fields[4].trim();

        let Ok(current_type) = TransactionType::try_from(current_type_str) else {
            self.status_message =
                Some("Error: Invalid transaction type for subcategory lookup.".to_string());
            self.mode = if self.editing_index.is_some() {
                AppMode::Editing
            } else {
                AppMode::Adding
            };
            return;
        };

        if current_category.is_empty() || current_category.eq_ignore_ascii_case("Uncategorized") {
            self.current_selection_list = vec!["(None)".to_string()];
        } else {
            let mut unique_subcategories: HashSet<String> = self
                .categories
                .iter()
                .filter(|cat_info| {
                    cat_info.transaction_type == current_type
                        && cat_info.category.eq_ignore_ascii_case(current_category)
                        && !cat_info.subcategory.is_empty()
                })
                .map(|cat_info| cat_info.subcategory.clone())
                .collect();

            let mut options: Vec<String> = unique_subcategories.drain().collect();
            options.sort_unstable();
            options.insert(0, "(None)".to_string());
            self.current_selection_list = options;
        }

        self.selection_list_state = ListState::default();
        if !self.current_selection_list.is_empty() {
            self.selection_list_state.select(Some(0));
        }
    }

    pub(crate) fn confirm_selection(&mut self) {
        if let Some(selected_index) = self.selection_list_state.selected() {
            if let Some(field_index) = self.selecting_field_index {
                if let Some(selected_value) = self.current_selection_list.get(selected_index) {
                    let value_to_set = if field_index == 5 && selected_value == "(None)" {
                        ""
                    } else {
                        selected_value.as_str()
                    };
                    self.add_edit_fields[field_index] = value_to_set.to_string();

                    if field_index == 4 {
                        self.current_add_edit_field = 5;
                        self.start_subcategory_selection();
                        return;
                    } else if field_index == 5 {
                        self.current_add_edit_field = 0;
                    }
                }
            }
        }
        self.mode = if self.editing_index.is_some() {
            AppMode::Editing
        } else {
            AppMode::Adding
        };
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }

    pub(crate) fn cancel_selection(&mut self) {
        self.mode = if self.editing_index.is_some() {
            AppMode::Editing
        } else {
            AppMode::Adding
        };
        if let Some(field_index) = self.selecting_field_index {
            self.current_add_edit_field = field_index;
        }
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }

    pub(crate) fn select_next_list_item(&mut self) {
        let list_len = self.current_selection_list.len();
        if list_len == 0 {
            return;
        }
        let i = match self.selection_list_state.selected() {
            Some(i) => {
                if i >= list_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selection_list_state.select(Some(i));
    }

    pub(crate) fn select_previous_list_item(&mut self) {
        let list_len = self.current_selection_list.len();
        if list_len == 0 {
            return;
        }
        let i = match self.selection_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    list_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selection_list_state.select(Some(i));
    }

    // --- Settings Mode Logic ---
    pub(crate) fn enter_settings_mode(&mut self) {
        self.mode = AppMode::Settings;
        self.input_field_content = self.data_file_path.to_string_lossy().to_string();
        self.input_field_cursor = self.input_field_content.len(); // Cursor at end
        self.status_message = None;
    }

    pub(crate) fn exit_settings_mode(&mut self) {
        self.mode = AppMode::Normal;
        self.input_field_content.clear();
        self.input_field_cursor = 0;
        self.status_message = None;
    }

    pub(crate) fn save_settings(&mut self) {
        let new_path_str = self.input_field_content.trim();
        if new_path_str.is_empty() {
            self.status_message = Some("Error: Path cannot be empty.".to_string());
            return;
        }
        let new_path = PathBuf::from(new_path_str);

        if !new_path.exists() {
            if let Err(e) = save_transactions(&self.transactions, &new_path) {
                self.status_message = Some(format!(
                    "Error creating transactions file '{}': {}. Check path and permissions.",
                    new_path.display(),
                    e
                ));
                return;
            }
        }

        // Save the new data file path in config
        let settings = AppSettings {
            data_file_path: Some(new_path_str.to_string()),
        };
        if let Err(e) = save_settings(&settings) {
            self.status_message = Some(format!("Error saving config file: {}", e));
            return;
        }

        // Update app state to use new path
        self.data_file_path = new_path.clone();

        let txs = match load_transactions(&self.data_file_path) {
            Ok(tx) => tx,
            Err(e) => {
                self.status_message = Some(format!(
                    "Error loading transactions from '{}': {}. Check file format and permissions.",
                    self.data_file_path.display(),
                    e
                ));
                return;
            }
        };
        self.transactions = txs;

        // Exit settings UI and refresh the table and summaries
        self.exit_settings_mode();
        self.sort_transactions();
        self.filtered_indices = (0..self.transactions.len()).collect();
        if !self.filtered_indices.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
        self.calculate_monthly_summaries();
        self.calculate_category_summaries();

        self.status_message = Some(format!(
            "Settings saved. Data file set to: {}",
            self.data_file_path.display()
        ));
    }

    pub(crate) fn reset_settings_path_to_default(&mut self) {
        match Self::get_default_data_file_path() {
            Ok(default_path) => {
                let path_str = default_path.to_string_lossy().to_string();
                self.input_field_content = path_str;
                self.input_field_cursor = self.input_field_content.len();
                self.status_message =
                    Some("Path reset to default. Press Enter to save.".to_string());
            }
            Err(e) => {
                let fallback_path = "transactions.csv";
                self.input_field_content = fallback_path.to_string();
                self.input_field_cursor = self.input_field_content.len();
                self.status_message = Some(format!(
                    "Error getting default path ({}). Reset to local '{}'. Press Enter to save.",
                    e, fallback_path
                ));
            }
        }
    }

    fn get_default_data_file_path() -> Result<PathBuf, Error> {
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

    // Helper to clear the generic input field
    pub(crate) fn clear_input_field(&mut self) {
        self.input_field_content.clear();
        self.input_field_cursor = 0;
    }

    // Helper to get visible hierarchical items for the category summary view
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

    // --- Advanced Filtering Logic ---
    pub(crate) fn start_advanced_filtering(&mut self) {
        self.mode = AppMode::AdvancedFiltering;
        self.advanced_filter_fields = Default::default();
        self.current_advanced_filter_field = 0;
        self.status_message = None;
    }

    pub(crate) fn cancel_advanced_filtering(&mut self) {
        self.mode = AppMode::Filtering;
        self.status_message = None;
    }

    pub(crate) fn finish_advanced_filtering(&mut self) {
        self.apply_advanced_filter();
        self.mode = AppMode::Normal;
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
                // Date fields
                if let Some(new_content) = validate_and_insert_date_char(field, c) {
                    *field = new_content;
                }
            }
            5 => { /* Type field: toggle only via arrows/enter */ }
            6 | 7 => {
                // Amount fields: only digits and one decimal point
                if c.is_ascii_digit() || (c == '.' && !field.contains('.')) {
                    field.push(c);
                }
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
                // Date fields: special backspace
                let len = field.len();
                if (len == 5 || len == 8) && field.ends_with('-') {
                    if field
                        .chars()
                        .nth(len - 2)
                        .map(|ch| ch.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        field.pop();
                        field.pop();
                    } else {
                        field.pop();
                    }
                } else if !field.is_empty() {
                    field.pop();
                }
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
        self.mode = AppMode::SelectingFilterCategory;
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
        self.mode = AppMode::SelectingFilterSubcategory;
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
        self.mode = AppMode::AdvancedFiltering;
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }

    pub(crate) fn cancel_advanced_selection(&mut self) {
        self.mode = AppMode::AdvancedFiltering;
        if let Some(fi) = self.selecting_field_index {
            self.current_advanced_filter_field = fi;
        }
        self.selecting_field_index = None;
        self.current_selection_list.clear();
    }

    pub(crate) fn apply_advanced_filter(&mut self) {
        sort_transactions_impl(&mut self.transactions, self.sort_by, self.sort_order);
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
                        // Simple Duration-based day adjustment
                        if amount > 0 {
                            current_date + Duration::days(amount)
                        } else {
                            current_date - Duration::days(-amount)
                        }
                    },
                    DateUnit::Month => {
                        // Use Chrono's date arithmetic methods
                        let mut year = current_date.year();
                        let mut month = current_date.month() as i32 + amount as i32;
                        
                        // Adjust year if month goes out of range (1-12)
                        while month > 12 {
                            month -= 12;
                            year += 1;
                        }
                        while month < 1 {
                            month += 12;
                            year -= 1;
                        }
                        
                        let day = current_date.day();
                        
                        // Try to create date with same day, or last day of month if that would be invalid
                        NaiveDate::from_ymd_opt(year, month as u32, day)
                            .unwrap_or_else(|| {
                                // If creating with the same day fails, use the last day of the month
                                let last_day = if month == 12 {
                                    NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                                } else {
                                    NaiveDate::from_ymd_opt(year, month as u32 + 1, 1).unwrap()
                                };
                                last_day - Duration::days(1)
                            })
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

// Helper function for date input validation
fn validate_and_insert_date_char(field: &str, c: char) -> Option<String> {
    if !c.is_ascii_digit() {
        return None;
    }
    
    let len = field.len();
    
    if len >= 10 {
        return None;
    }
    
    let mut result = field.to_string();
    
    // Validate month digits as they're entered
    if len == 5 {
        let month_digit = c.to_digit(10).unwrap_or(0);
        if month_digit > 1 { // First digit of month can only be 0 or 1
            // Allow direct entry of single-digit months
            result.push(c);
            result.push('-');
            return Some(result);
        }
    } else if len == 6 {
        let first_digit = field.chars().nth(5)
            .and_then(|ch| ch.to_digit(10))
            .unwrap_or(0);
        let month = first_digit * 10 + c.to_digit(10).unwrap_or(0);
        
        if month == 0 || month > 12 {
            return None;
        }
    }
    
    // Validate day digits
    if len == 8 {
        let day_digit = c.to_digit(10).unwrap_or(0);
        if day_digit > 3 { // First digit of day can only be 0, 1, 2, or 3
            return None;
        }
    } else if len == 9 {
        if let (Ok(year), Ok(month)) = (
            field[0..4].parse::<i32>(),
            field[5..7].parse::<u32>()
        ) {
            let first_digit = field.chars().nth(8)
                .and_then(|ch| ch.to_digit(10))
                .unwrap_or(0);
            let day = first_digit * 10 + c.to_digit(10).unwrap_or(0);
            
            let last_day = match month {
                2 => if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                    29 // Leap year February
                } else {
                    28 // Regular February
                },
                4 | 6 | 9 | 11 => 30, // 30-day months
                _ => 31, // 31-day months
            };
            
            if day == 0 || day > last_day {
                return None;
            }
        }
    }
    
    // Add the digit
    result.push(c);
    
    // Auto-insert hyphens
    if result.len() == 4 {
        // Validate year
        if let Ok(year) = result.parse::<i32>() {
            if year < 1900 || year > 2100 {
                return None; // Reject invalid year
            }
        }
        result.push('-');
    } else if result.len() == 7 {
        result.push('-');
    }
    
    Some(result)
}

fn sort_transactions_impl(
    transactions: &mut [Transaction],
    sort_by: SortColumn,
    sort_order: SortOrder,
) {
    transactions.sort_by(|a, b| {
        let ordering = match sort_by {
            SortColumn::Date => a.date.cmp(&b.date),
            SortColumn::Description => a.description.cmp(&b.description),
            SortColumn::Amount => a.amount.partial_cmp(&b.amount).unwrap_or(Ordering::Equal),
            SortColumn::Type => a.transaction_type.cmp(&b.transaction_type),
            SortColumn::Category => a.category.cmp(&b.category),
            SortColumn::Subcategory => a.subcategory.cmp(&b.subcategory),
        };
        if sort_order == SortOrder::Descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}
