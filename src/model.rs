use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use serde::Deserializer;
use serde::de::Error as SerdeError;

pub(crate) const DATE_FORMAT: &str = "%Y-%m-%d";

fn deserialize_flexible_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if let Ok(date) = NaiveDate::parse_from_str(&s, DATE_FORMAT) {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(&s, "%Y/%m/%d") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(&s, "%d/%m/%Y") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(&s, "%d-%m-%Y") {
        return Ok(date);
    }
    Err(SerdeError::custom(format!(
        "Invalid date format: '{}'. Expected YYYY-MM-DD, YYYY/MM/DD, DD/MM/YYYY, or DD-MM-YYYY.",
        s
    )))
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum TransactionType {
    Income,
    Expense,
}

impl TransactionType {
    pub fn as_str(self) -> &'static str {
        match self {
            TransactionType::Income => "Income",
            TransactionType::Expense => "Expense",
        }
    }
}

impl fmt::Display for TransactionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for TransactionType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "income" => Ok(TransactionType::Income),
            "expense" => Ok(TransactionType::Expense),
            t if t.starts_with('i') => Ok(TransactionType::Income),
            t if t.starts_with('e') => Ok(TransactionType::Expense),
            _ => Err(()),
        }
    }
}

impl<'de> Deserialize<'de> for TransactionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TransactionType::try_from(s.as_str()).map_err(|_| {
            SerdeError::custom(format!(
                "Invalid transaction type: '{}'. Expected 'Income', 'Expense', 'i', or 'e'.",
                s
            ))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    BiWeekly,
    SemiMonthly,
    SemiMonthlyWorkday,
    Monthly,
    Quarterly,
    Yearly,
}

impl RecurrenceFrequency {
    pub fn to_string(self) -> &'static str {
        match self {
            RecurrenceFrequency::Daily => "Daily",
            RecurrenceFrequency::Weekly => "Weekly",
            RecurrenceFrequency::BiWeekly => "Bi-Weekly",
            RecurrenceFrequency::SemiMonthly => "Semi-Monthly",
            RecurrenceFrequency::SemiMonthlyWorkday => "Semi-Monthly (Weekday Adjusted)",
            RecurrenceFrequency::Monthly => "Monthly",
            RecurrenceFrequency::Quarterly => "Quarterly",
            RecurrenceFrequency::Yearly => "Yearly",
        }
    }

    /// Parse a frequency from its display label (e.g. "Bi-Weekly"). Used for both the
    /// recurring-settings form and database round-tripping.
    pub fn from_label(label: &str) -> Option<RecurrenceFrequency> {
        match label {
            "Daily" => Some(RecurrenceFrequency::Daily),
            "Weekly" => Some(RecurrenceFrequency::Weekly),
            "Bi-Weekly" => Some(RecurrenceFrequency::BiWeekly),
            "Semi-Monthly" => Some(RecurrenceFrequency::SemiMonthly),
            "Semi-Monthly (Weekday Adjusted)" => Some(RecurrenceFrequency::SemiMonthlyWorkday),
            "Monthly" => Some(RecurrenceFrequency::Monthly),
            "Quarterly" => Some(RecurrenceFrequency::Quarterly),
            "Yearly" => Some(RecurrenceFrequency::Yearly),
            _ => None,
        }
    }

    pub fn all() -> Vec<RecurrenceFrequency> {
        vec![
            RecurrenceFrequency::Daily,
            RecurrenceFrequency::Weekly,
            RecurrenceFrequency::BiWeekly,
            RecurrenceFrequency::SemiMonthly,
            RecurrenceFrequency::SemiMonthlyWorkday,
            RecurrenceFrequency::Monthly,
            RecurrenceFrequency::Quarterly,
            RecurrenceFrequency::Yearly,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    // Apply the custom deserializer for reading dates so it can work on excel edits
    #[serde(deserialize_with = "deserialize_flexible_date")]
    #[serde(serialize_with = "date_format::serialize")]
    pub date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub subcategory: String,
    // Recurring transaction fields
    #[serde(default)]
    pub is_recurring: bool,
    #[serde(default)]
    pub recurrence_frequency: Option<RecurrenceFrequency>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_optional_date")]
    #[serde(serialize_with = "serialize_optional_date")]
    pub recurrence_end_date: Option<NaiveDate>,
    #[serde(default)]
    pub is_generated_from_recurring: bool,
    // Database identity. Excluded from CSV (import/export stay byte-compatible).
    // `id` is set for persisted (real) rows and None for in-memory-only generated rows.
    #[serde(skip)]
    pub id: Option<i64>,
    // In-memory only: the source row's id, stamped onto generated occurrences so we can
    // jump back to the source without fragile attribute matching. Never a DB column.
    #[serde(skip)]
    pub parent_id: Option<i64>,
}

impl Transaction {
    /// Build a database draft (the real-row fields stored in the `transactions` table) from a
    /// transaction. Drops `id`, the generated flag, and the in-memory `parent_id`.
    pub fn to_draft(&self) -> TransactionDraft {
        TransactionDraft {
            date: self.date,
            description: self.description.clone(),
            amount: self.amount,
            transaction_type: self.transaction_type,
            category: self.category.clone(),
            subcategory: self.subcategory.clone(),
            is_recurring: self.is_recurring,
            recurrence_frequency: self.recurrence_frequency,
            recurrence_end_date: self.recurrence_end_date,
        }
    }
}

/// Fields persisted for a real transaction row (regular transactions + recurring sources).
/// Generated occurrences are never stored, so there is no generated flag or parent link here.
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionDraft {
    pub date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    pub category: String,
    pub subcategory: String,
    pub is_recurring: bool,
    pub recurrence_frequency: Option<RecurrenceFrequency>,
    pub recurrence_end_date: Option<NaiveDate>,
}

fn default_category() -> String {
    "Uncategorized".to_string()
}

fn deserialize_optional_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) if !s.is_empty() => {
            deserialize_flexible_date(serde::de::value::StrDeserializer::new(&s)).map(Some)
        }
        _ => Ok(None),
    }
}

fn serialize_optional_date<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match date {
        Some(d) => serializer.serialize_str(&d.format(DATE_FORMAT).to_string()),
        None => serializer.serialize_str(""),
    }
}

pub mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Serializer};

    const FORMAT: &str = super::DATE_FORMAT;

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum SortColumn {
    Date,
    Description,
    Amount,
    Type,
    Category,
    Subcategory,
}

#[derive(PartialEq, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// `amount: None` clears the budget from that month on, which is not the same as having
/// no period. `category_id: None` is the ledger's monthly budget, not a category budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetPeriod {
    pub id: i64,
    pub category_id: Option<i64>,
    pub start: BudgetMonth,
    pub amount: Option<Decimal>,
}

/// Field order matters: it is what makes the earliest start sort first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BudgetMonth {
    pub year: i32,
    pub month: u32,
}

impl BudgetMonth {
    /// Sorts before every real month, so a period seeded here covers all history.
    pub const BEGINNING: Self = Self { year: 0, month: 1 };

    pub fn new(year: i32, month: u32) -> Self {
        Self { year, month }
    }

    pub fn next(self) -> Self {
        if self.month >= 12 {
            Self::new(self.year + 1, 1)
        } else {
            Self::new(self.year, self.month + 1)
        }
    }
}

/// How far a budget edit reaches. `RemoveChange` is only offered when a period starts
/// exactly at the edited month, since that is the only thing there is to undo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetEditScope {
    FromThisMonth,
    ThisMonthOnly,
    ReplaceAllMonths,
    RemoveChange,
}

/// A store operation an edit resolves to. Keeping the decision separate from the writing
/// is what lets the scope rules be tested without a database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetWrite {
    Set(BudgetMonth, Option<Decimal>),
    Remove(BudgetMonth),
    RemoveAll,
}

/// Owns the rule for reading a value out of the periods, so the UI never walks them.
#[derive(Debug, Default, Clone)]
pub struct BudgetSchedule {
    periods: Vec<BudgetPeriod>,
}

impl BudgetSchedule {
    pub fn new(mut periods: Vec<BudgetPeriod>) -> Self {
        periods.sort_by_key(|period| period.start);
        Self { periods }
    }

    /// The latest period starting on or before `month`. `None` means no budget, whether
    /// none was ever set or one cleared it.
    pub fn amount_for(&self, category_id: Option<i64>, month: BudgetMonth) -> Option<Decimal> {
        self.periods
            .iter()
            .rfind(|period| period.category_id == category_id && period.start <= month)
            .and_then(|period| period.amount)
    }

    pub fn monthly_budget(&self, month: BudgetMonth) -> Option<Decimal> {
        self.amount_for(None, month)
    }

    pub fn category_budget(&self, category_id: i64, month: BudgetMonth) -> Option<Decimal> {
        self.amount_for(Some(category_id), month)
    }

    /// Resolve an edit into the writes that carry it out. Pure, so the ordering rule that
    /// makes `ThisMonthOnly` work is checked by tests rather than by inspection.
    pub fn plan_edit(
        &self,
        category_id: Option<i64>,
        start: BudgetMonth,
        amount: Option<Decimal>,
        scope: BudgetEditScope,
    ) -> Vec<BudgetWrite> {
        match scope {
            BudgetEditScope::FromThisMonth => vec![BudgetWrite::Set(start, amount)],
            BudgetEditScope::ThisMonthOnly => {
                // Read what the next month inherits now; after the first write it would
                // just report the value being set.
                let inherited = self.amount_for(category_id, start.next());
                vec![
                    BudgetWrite::Set(start, amount),
                    BudgetWrite::Set(start.next(), inherited),
                ]
            }
            BudgetEditScope::ReplaceAllMonths => {
                vec![
                    BudgetWrite::RemoveAll,
                    BudgetWrite::Set(BudgetMonth::BEGINNING, amount),
                ]
            }
            BudgetEditScope::RemoveChange => vec![BudgetWrite::Remove(start)],
        }
    }

    pub fn years(&self) -> Vec<i32> {
        let mut years: Vec<i32> = self
            .periods
            .iter()
            .map(|period| period.start.year)
            .filter(|year| *year > BudgetMonth::BEGINNING.year)
            .collect();
        years.sort_unstable();
        years.dedup();
        years
    }

    /// Lets a destructive edit report how much it replaced.
    pub fn period_count(&self, category_id: Option<i64>) -> usize {
        self.periods
            .iter()
            .filter(|period| period.category_id == category_id)
            .count()
    }

    /// Only a period starting exactly here can be removed.
    pub fn starts_at(&self, category_id: Option<i64>, month: BudgetMonth) -> bool {
        self.periods
            .iter()
            .any(|period| period.category_id == category_id && period.start == month)
    }

    pub fn budgeted_categories(&self, month: BudgetMonth) -> Vec<(i64, Decimal)> {
        let mut ids: Vec<i64> = self
            .periods
            .iter()
            .filter_map(|period| period.category_id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids.into_iter()
            .filter_map(|id| self.category_budget(id, month).map(|amount| (id, amount)))
            .collect()
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum CategorySortColumn {
    Type,
    Category,
    Subcategory,
    Tag,
    TargetBudget,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MonthlySummary {
    pub income: Decimal,
    pub expense: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryInfo {
    pub transaction_type: TransactionType,
    pub category: String,
    pub subcategory: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryDraft {
    pub transaction_type: TransactionType,
    pub category: String,
    pub subcategory: String,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryRecord {
    pub id: i64,
    pub transaction_type: TransactionType,
    pub category: String,
    pub subcategory: String,
    pub tag: Option<String>,
}

impl CategoryRecord {
    pub fn to_category_info(&self) -> CategoryInfo {
        CategoryInfo {
            transaction_type: self.transaction_type,
            category: self.category.clone(),
            subcategory: self.subcategory.clone(),
        }
    }

    pub fn to_draft(&self) -> CategoryDraft {
        CategoryDraft {
            transaction_type: self.transaction_type,
            category: self.category.clone(),
            subcategory: self.subcategory.clone(),
            tag: self.tag.clone(),
        }
    }
}
