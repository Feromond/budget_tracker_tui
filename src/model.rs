use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum TransactionType {
    Income,
    Expense,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    // Apply the custom deserializer for reading dates so it can work on excel edits
    #[serde(deserialize_with = "deserialize_flexible_date")]
    #[serde(serialize_with = "date_format::serialize")]
    pub date: NaiveDate,
    pub description: String,
    pub amount: f64,
    pub transaction_type: TransactionType,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub subcategory: String,
}

fn default_category() -> String {
    "Uncategorized".to_string()
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
    pub income: f64,
    pub expense: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryInfo {
    pub transaction_type: TransactionType,
    pub category: String,
    pub subcategory: String,
}
