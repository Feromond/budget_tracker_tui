use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use serde::de::Error as SerdeError;
use serde::Deserializer;

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
            RecurrenceFrequency::Monthly => "Monthly",
            RecurrenceFrequency::Quarterly => "Quarterly",
            RecurrenceFrequency::Yearly => "Yearly",
        }
    }

    pub fn all() -> Vec<RecurrenceFrequency> {
        vec![
            RecurrenceFrequency::Daily,
            RecurrenceFrequency::Weekly,
            RecurrenceFrequency::BiWeekly,
            RecurrenceFrequency::SemiMonthly,
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
    pub target_budget: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryRecord {
    pub id: i64,
    pub transaction_type: TransactionType,
    pub category: String,
    pub subcategory: String,
    pub tag: Option<String>,
    pub target_budget: Option<Decimal>,
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
            target_budget: self.target_budget,
        }
    }
}
