use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

pub const DATE_FORMAT: &str = "%Y-%m-%d";

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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(with = "date_format")]
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
    use serde::{self, Serializer, Deserializer, Deserialize};

    const FORMAT: &str = super::DATE_FORMAT;

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
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