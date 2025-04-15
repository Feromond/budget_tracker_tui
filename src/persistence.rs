use crate::model::{Transaction, CategoryInfo, TransactionType};
use std::fs::{File, create_dir_all};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::result::Result as StdResult;

pub(crate) const DATA_FILE_NAME: &str = "transactions.csv";
const APP_DATA_SUBDIR: &str = "BudgetTracker";

// Helper function to get the application-specific data directory path
fn get_data_file_path() -> StdResult<PathBuf, Error> {
    match dirs::data_dir() {
        Some(mut path) => {
            path.push(APP_DATA_SUBDIR);
            // Ensure the directory exists
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

pub(crate) fn load_transactions() -> StdResult<Vec<Transaction>, Error> {
    let data_path = get_data_file_path()?;

    if !data_path.exists() {
        return Ok(vec![]);
    }

    let file = File::open(&data_path)?;
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_reader(file);
    let mut transactions = Vec::new();
    let headers = rdr.headers().map_err(|e| Error::new(ErrorKind::InvalidData, format!("Failed to read headers from {}: {}", data_path.display(), e)))?.clone();

    for (index, result) in rdr.deserialize().enumerate() {
        let transaction: Transaction = result.map_err(|e| {
            Error::new(ErrorKind::InvalidData, format!("Failed to parse transaction at row {} in {}: {}. Headers: {:?}", index + 2, data_path.display(), e, headers))
        })?;
        transactions.push(transaction);
    }
    Ok(transactions)
}

pub(crate) fn save_transactions(transactions: &[Transaction]) -> StdResult<(), Error> {
    let data_path = get_data_file_path()?;
    let file = File::create(&data_path)?;
    let mut wtr = csv::Writer::from_writer(file);
    for transaction in transactions {
        wtr.serialize(transaction).map_err(|e| {
            Error::new(ErrorKind::Other, format!("Failed to serialize transaction {:?} to {}: {}", transaction, data_path.display(), e))
        })?;
    }
    wtr.flush()?;
    Ok(())
}

// Function to load categories from embedded data
pub(crate) fn load_categories() -> StdResult<Vec<CategoryInfo>, Error> {
    let embedded_csv_data = include_str!("../budget_categories.csv");

    let mut rdr = csv::Reader::from_reader(embedded_csv_data.as_bytes());
    let mut categories = Vec::new();
    let mut header_error = None;

    let headers = rdr.headers()?.clone();
    let type_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("Type"));
    let cat_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("Category"));
    let subcat_idx = headers.iter().position(|h| h.eq_ignore_ascii_case("Subcategory"));

    if type_idx.is_none() || cat_idx.is_none() || subcat_idx.is_none() {
        return Err(Error::new(ErrorKind::InvalidData, "Embedded category data missing required headers: Type, Category, Subcategory"));
    }
    let type_idx = type_idx.unwrap();
    let cat_idx = cat_idx.unwrap();
    let subcat_idx = subcat_idx.unwrap();

    for (index, result) in rdr.records().enumerate() {
        let record = result.map_err(|e| Error::new(ErrorKind::InvalidData, format!("Failed to read record at row {} from embedded category data: {}", index+1, e)))?;

        let type_str = record.get(type_idx).ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Missing Type at row {} in embedded data", index + 1)))?.trim();
        let cat_str = record.get(cat_idx).ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Missing Category at row {} in embedded data", index + 1)))?.trim();
        let subcat_str = record.get(subcat_idx).ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Missing Subcategory at row {} in embedded data", index + 1)))?.trim();

        let transaction_type = match type_str {
            t if t.eq_ignore_ascii_case("Income") => TransactionType::Income,
            t if t.eq_ignore_ascii_case("Expense") => TransactionType::Expense,
            _ => {
                 if header_error.is_none() {
                     header_error = Some(Error::new(ErrorKind::InvalidData, format!("Invalid Type '{}' at row {} in embedded data", type_str, index + 1)));
                 }
                 continue;
             }
        };

        if cat_str.is_empty() {
             continue;
         }

        categories.push(CategoryInfo {
            transaction_type,
            category: cat_str.to_string(),
            subcategory: subcat_str.to_string(),
        });
    }

    if let Some(err) = header_error {
         Err(err)
     } else {
         Ok(categories)
     }
} 