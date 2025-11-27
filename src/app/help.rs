use crate::app::state::AppMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBindingInfo {
    pub key: &'static str,
    pub description: &'static str,
    pub group: &'static str,
    pub extended_description: Option<&'static str>,
}

impl KeyBindingInfo {
    pub const fn new(
        key: &'static str,
        description: &'static str,
        group: &'static str,
        extended_description: Option<&'static str>,
    ) -> Self {
        Self {
            key,
            description,
            group,
            extended_description,
        }
    }
}

pub fn get_help_for_mode(mode: AppMode) -> Vec<KeyBindingInfo> {
    match mode {
        AppMode::Normal => vec![
            KeyBindingInfo::new("↑/↓", "Navigate transactions", "Navigation", None),
            KeyBindingInfo::new("PgUp/PgDn", "Scroll page up/down", "Navigation", None),
            KeyBindingInfo::new("Ctrl+Up/Down", "Jump to First/Last", "Navigation", None),
            KeyBindingInfo::new("a", "Add new transaction", "Actions", Some("Opens the 'Add Transaction' form where you can input details like date, amount, category, etc.")),
            KeyBindingInfo::new("e", "Edit selected transaction", "Actions", Some("Opens the edit mode for the currently selected transaction.")),
            KeyBindingInfo::new("d", "Delete selected transaction", "Actions", Some("Prompts for confirmation to delete the selected transaction.")),
            KeyBindingInfo::new("r", "Manage recurring transactions", "Actions", Some("Opens the recurring transactions manager.")),
            KeyBindingInfo::new("f", "Filter transactions", "Actions", Some("Enables simple filtering mode. Type to filter by any field.")),
            KeyBindingInfo::new("s", "Monthly Summary", "Actions", Some("View a monthly breakdown of income vs expenses.")),
            KeyBindingInfo::new("c", "Category Summary", "Actions", Some("View expenses broken down by category and subcategory.")),
            KeyBindingInfo::new("o", "Settings", "Actions", Some("Open application settings (data file path, etc).")),
            KeyBindingInfo::new("1/F1", "Sort by Date", "Sorting", None),
            KeyBindingInfo::new("2/F2", "Sort by Description", "Sorting", None),
            KeyBindingInfo::new("3/F3", "Sort by Category", "Sorting", None),
            KeyBindingInfo::new("4/F4", "Sort by Subcategory", "Sorting", None),
            KeyBindingInfo::new("5/F5", "Sort by Type", "Sorting", None),
            KeyBindingInfo::new("6/F6", "Sort by Amount", "Sorting", None),
            KeyBindingInfo::new("q/Esc", "Quit Application", "System", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", Some("Shows this help menu.")),
        ],
        AppMode::Adding | AppMode::Editing => vec![
            KeyBindingInfo::new("Tab/↑/↓", "Navigate fields", "Navigation", None),
            KeyBindingInfo::new("Date Field", "Transaction Date (YYYY-MM-DD)", "Fields", Some("Enter the date of the transaction (YYYY-MM-DD). Use Arrow keys to adjust day by day. Hold Shift + Arrow keys to jump by month. Accurate dating helps with monthly filtering.")),
            KeyBindingInfo::new("Amount Field", "Transaction Amount", "Fields", Some("Numerical value of the transaction. Positive numbers are standard. The 'Type' field determines if it's income or expense. For expenses, enter the positive cost.")),
            KeyBindingInfo::new("Category", "Main Category", "Fields", Some("Primary classification (e.g., 'Food', 'Housing'). Press Enter to open a selection list of existing categories or type a new one to create it.")),
            KeyBindingInfo::new("Subcategory", "Sub Category", "Fields", Some("More specific classification (e.g., 'Groceries', 'Rent'). Useful for detailed breakdown in the Category Summary view.")),
            KeyBindingInfo::new("Type", "Expense / Income", "Fields", Some("Classifies the transaction as 'Expense' or 'Income'. This affects how totals are calculated in summaries. Use Left/Right arrows to toggle.")),
            KeyBindingInfo::new("Description", "Notes / Details", "Fields", Some("Optional details about the transaction. Add context that doesn't fit in categories.")),
            KeyBindingInfo::new("←/→", "Toggle type / Adjust date", "Input", None),
            KeyBindingInfo::new("Shift+←/→", "Jump month (Date field)", "Input", Some("In date fields, moves date by one month instead of one day.")),
            KeyBindingInfo::new("Enter", "Save transaction", "Actions", None),
            KeyBindingInfo::new("Esc", "Cancel", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::ConfirmDelete => vec![
            KeyBindingInfo::new("y", "Confirm deletion", "Actions", None),
            KeyBindingInfo::new("n/Esc", "Cancel deletion", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::Filtering => vec![
            KeyBindingInfo::new("Any Char", "Type filter text", "Input", None),
            KeyBindingInfo::new("Bksp/Del", "Delete character", "Input", None),
            KeyBindingInfo::new("←/→", "Move cursor", "Navigation", None),
            KeyBindingInfo::new("Ctrl+F", "Switch to Advanced Filter", "Actions", Some("Switch to advanced filtering mode for more specific criteria.")),
            KeyBindingInfo::new("Ctrl+R", "Clear filter", "Actions", None),
            KeyBindingInfo::new("Enter", "Apply filter", "Actions", None),
            KeyBindingInfo::new("Esc", "Exit filtering", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::AdvancedFiltering => vec![
            KeyBindingInfo::new("Tab/↑/↓", "Navigate fields", "Navigation", None),
            KeyBindingInfo::new("Start Date", "Filter From Date", "Fields", Some("Include transactions on or after this date.")),
            KeyBindingInfo::new("End Date", "Filter To Date", "Fields", Some("Include transactions on or before this date.")),
            KeyBindingInfo::new("Min Amount", "Minimum Amount", "Fields", Some("Filter transactions with amount greater than or equal to this.")),
            KeyBindingInfo::new("Max Amount", "Maximum Amount", "Fields", Some("Filter transactions with amount less than or equal to this.")),
            KeyBindingInfo::new("Category", "Filter Category", "Fields", Some("Filter by specific category.")),
            KeyBindingInfo::new("Subcategory", "Filter Subcategory", "Fields", Some("Filter by specific subcategory.")),
            KeyBindingInfo::new("Type", "Filter Type", "Fields", Some("Filter by Expense or Income.")),
            KeyBindingInfo::new("Desc", "Filter Description", "Fields", Some("Filter by text in description.")),
            KeyBindingInfo::new("Ctrl+R", "Clear all filters", "Actions", None),
            KeyBindingInfo::new("Enter", "Save & Apply", "Actions", None),
            KeyBindingInfo::new("Esc", "Cancel", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::Summary => vec![
            KeyBindingInfo::new("↑/↓", "Change Month", "Navigation", None),
            KeyBindingInfo::new("←/→", "Change Year", "Navigation", None),
            KeyBindingInfo::new("m", "Toggle Multi-Month View", "View", Some("Switch between single month view and multi-month view.")),
            KeyBindingInfo::new("c", "Toggle Cumulative View", "View", Some("Toggle cumulative (running total) view.")),
            KeyBindingInfo::new("q/Esc", "Back to Transactions", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::CategorySummary => vec![
            KeyBindingInfo::new("↑/↓", "Select Category/Subcategory", "Navigation", None),
            KeyBindingInfo::new("←/→", "Change Year", "Navigation", None),
            KeyBindingInfo::new("PgUp/PgDn", "Jump Month (if expanded)", "Navigation", None),
            KeyBindingInfo::new("Enter", "Expand/Collapse Category", "Actions", None),
            KeyBindingInfo::new("q/Esc", "Back to Transactions", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::Settings => vec![
            KeyBindingInfo::new("Tab/↑/↓", "Navigate settings", "Navigation", None),
            KeyBindingInfo::new("Data Path", "File Location", "Fields", Some("Path to the CSV file where transactions are stored. Tip: You can set this to a file in your iCloud Drive, Google Drive, or Dropbox folder to sync your budget data across multiple devices.")),
            KeyBindingInfo::new("Target Budget", "Monthly Budget Goal", "Fields", Some("Set a target monthly budget amount for reference in summaries.")),
            KeyBindingInfo::new("Hourly Rate", "Hourly Earning Rate", "Fields", Some("Optional. Enter your hourly rate to enable viewing transaction amounts as equivalent hours worked.")),
            KeyBindingInfo::new("Show Hours", "Toggle Hours View", "Fields", Some("Enable to display transaction amounts in equivalent hours based on your hourly rate.")),
            KeyBindingInfo::new("Fuzzy Search", "Toggle Fuzzy Search", "Fields", Some("Toggle to enable fuzzy searching for categories/subcategories. When enabled, selecting 'Category' opens a search bar to filter both category and subcategory at once.")),
            KeyBindingInfo::new("←/→", "Toggle Options", "Navigation", Some("Use Left/Right arrow keys to change toggle settings (e.g. Yes/No).")),
            KeyBindingInfo::new("Ctrl+D", "Reset to Default", "Actions", None),
            KeyBindingInfo::new("Ctrl+U", "Clear Field", "Actions", None),
            KeyBindingInfo::new("Enter", "Save Settings", "Actions", None),
            KeyBindingInfo::new("Esc", "Cancel / Back", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::RecurringSettings => vec![
            KeyBindingInfo::new("Tab/↑/↓", "Navigate fields", "Navigation", None),
            KeyBindingInfo::new("Active", "Enable/Disable", "Fields", Some("Toggle if this transaction will be recurring.")),
            KeyBindingInfo::new("Frequency", "Recurrence Interval", "Fields", Some("Determines the interval for the transaction. 'Daily', 'Weekly', 'Monthly', or 'Yearly'. The app will automatically generate these transactions up to the current date when you open it.")),
            KeyBindingInfo::new("End Date", "Stop Date", "Fields", Some("Optional. If set, the recurring transaction will stop being generated after this date. Leave empty for indefinite recurrence.")),
            KeyBindingInfo::new("Enter", "Select / Save", "Actions", None),
            KeyBindingInfo::new("Esc", "Cancel", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::SelectingCategory
        | AppMode::SelectingSubcategory
        | AppMode::SelectingFilterCategory
        | AppMode::SelectingFilterSubcategory
        | AppMode::SelectingRecurrenceFrequency => vec![
            KeyBindingInfo::new("↑/↓", "Navigate options", "Navigation", None),
            KeyBindingInfo::new("Enter", "Confirm Selection", "Actions", None),
            KeyBindingInfo::new("Esc", "Cancel Selection", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        AppMode::FuzzyFinding => vec![
            KeyBindingInfo::new("Any Char", "Type to search", "Input", Some("Type to filter the list of categories and subcategories.")),
            KeyBindingInfo::new("↑/↓", "Navigate results", "Navigation", None),
            KeyBindingInfo::new("Enter", "Select Category", "Actions", Some("Confirm selection. Auto-fills both Category and Subcategory fields.")),
            KeyBindingInfo::new("Esc", "Cancel", "Actions", None),
            KeyBindingInfo::new("Ctrl+H", "Show Keybindings Help", "System", None),
        ],
        _ => vec![
            KeyBindingInfo::new("Ctrl+H", "Close Help", "System", None),
            KeyBindingInfo::new("Esc", "Close Help", "System", None),
        ],
    }
}
