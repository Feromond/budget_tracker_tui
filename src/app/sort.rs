use super::state::App;
use crate::model::{SortColumn, SortOrder};

impl App {
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
}
