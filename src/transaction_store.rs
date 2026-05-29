use crate::database::SqliteDatabase;
use crate::model::{
    CategoryDraft, CategoryRecord, RecurrenceFrequency, Transaction, TransactionDraft,
    TransactionType, DATE_FORMAT,
};
use chrono::NaiveDate;
use rusqlite::{params, types::Type, Connection, Error as SqlError, Row};
use rust_decimal::Decimal;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

/// Outcome of a merge-dedupe import.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub added: usize,
    pub skipped: usize,
}

/// Persistence for transactions. Only **real** rows are stored (regular transactions and
/// recurring sources); generated occurrences are derived in-memory and never written here.
pub trait TransactionStore {
    fn list(&self) -> Result<Vec<Transaction>>;
    fn insert(&self, draft: &TransactionDraft) -> Result<i64>;
    fn update(&self, id: i64, draft: &TransactionDraft) -> Result<()>;
    fn delete(&self, id: i64) -> Result<()>;
    /// Insert every row that is not already present (matched on its natural key). Runs in a
    /// single transaction; duplicates within the batch are skipped too.
    fn import_merge(&self, rows: &[Transaction]) -> Result<ImportSummary>;
    /// Re-point all rows matching `old` onto the `new` category (used when a category is
    /// renamed/retyped in the catalog).
    fn apply_category_rename(&self, old: &CategoryRecord, new: &CategoryDraft) -> Result<()>;
    /// Clear rows that referenced a deleted category, mirroring the in-app rules: deleting a
    /// top-level category resets matches to "Uncategorized"; deleting a subcategory only
    /// clears the subcategory.
    fn apply_category_clear(&self, record: &CategoryRecord) -> Result<()>;
}

pub struct SqliteTransactionStore {
    database: SqliteDatabase,
}

impl SqliteTransactionStore {
    pub fn new(database: SqliteDatabase) -> Self {
        Self { database }
    }

    fn open_connection(&self) -> Result<Connection> {
        self.database.open_connection("transaction")
    }

    /// Open a connection with the schema guaranteed up to date.
    fn ready_connection(&self) -> Result<Connection> {
        let mut conn = self.open_connection()?;
        self.database.run_migrations(&mut conn)?;
        Ok(conn)
    }

    fn row_to_transaction(row: &Row<'_>) -> rusqlite::Result<Transaction> {
        let id: i64 = row.get(0)?;
        let date = parse_date(1, &row.get::<_, String>(1)?)?;
        let amount = parse_decimal(3, &row.get::<_, String>(3)?)?;
        let transaction_type = parse_transaction_type(4, &row.get::<_, String>(4)?)?;
        let is_recurring: i64 = row.get(7)?;
        let recurrence_frequency = row
            .get::<_, Option<String>>(8)?
            .and_then(|label| RecurrenceFrequency::from_label(&label));
        let recurrence_end_date = match row.get::<_, Option<String>>(9)? {
            Some(value) if !value.trim().is_empty() => Some(parse_date(9, value.trim())?),
            _ => None,
        };

        Ok(Transaction {
            date,
            description: row.get(2)?,
            amount,
            transaction_type,
            category: row.get(5)?,
            subcategory: row.get(6)?,
            is_recurring: is_recurring != 0,
            recurrence_frequency,
            recurrence_end_date,
            is_generated_from_recurring: false,
            id: Some(id),
            parent_id: None,
        })
    }

    fn insert_with_conn(conn: &Connection, draft: &TransactionDraft) -> Result<i64> {
        conn.execute(
            "
            INSERT INTO transactions (
                date,
                description,
                amount,
                transaction_type,
                category,
                subcategory,
                is_recurring,
                recurrence_frequency,
                recurrence_end_date
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ",
            params![
                draft.date.format(DATE_FORMAT).to_string(),
                &draft.description,
                draft.amount.normalize().to_string(),
                draft.transaction_type.as_str(),
                &draft.category,
                &draft.subcategory,
                draft.is_recurring as i64,
                draft.recurrence_frequency.map(|freq| freq.to_string()),
                draft
                    .recurrence_end_date
                    .map(|date| date.format(DATE_FORMAT).to_string()),
            ],
        )
        .map_err(|err| Error::other(format!("Failed to insert transaction: {}", err)))?;

        Ok(conn.last_insert_rowid())
    }

    /// Does a row with the same natural key already exist? Amounts are compared in their
    /// canonical `Decimal` string form so "10" and "10.00" are treated as equal.
    fn natural_key_exists(conn: &Connection, tx: &Transaction) -> Result<bool> {
        conn.query_row(
            "
            SELECT 1 FROM transactions
            WHERE date = ?1
              AND description = ?2
              AND amount = ?3
              AND transaction_type = ?4
              AND category = ?5
              AND subcategory = ?6
            LIMIT 1
            ",
            params![
                tx.date.format(DATE_FORMAT).to_string(),
                &tx.description,
                tx.amount.normalize().to_string(),
                tx.transaction_type.as_str(),
                &tx.category,
                &tx.subcategory,
            ],
            |_| Ok(()),
        )
        .map(|_| true)
        .or_else(|err| match err {
            SqlError::QueryReturnedNoRows => Ok(false),
            other => Err(Error::other(format!(
                "Failed to check for existing transaction: {}",
                other
            ))),
        })
    }
}

impl TransactionStore for SqliteTransactionStore {
    fn list(&self) -> Result<Vec<Transaction>> {
        let conn = self.ready_connection()?;
        let mut stmt = conn
            .prepare(
                "
                SELECT id, date, description, amount, transaction_type, category, subcategory,
                       is_recurring, recurrence_frequency, recurrence_end_date
                FROM transactions
                ORDER BY date, id
                ",
            )
            .map_err(|err| Error::other(format!("Failed to prepare transaction query: {}", err)))?;

        let rows = stmt
            .query_map([], Self::row_to_transaction)
            .map_err(|err| Error::other(format!("Failed to load transactions: {}", err)))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| Error::other(format!("Failed to read transactions: {}", err)))
    }

    fn insert(&self, draft: &TransactionDraft) -> Result<i64> {
        let conn = self.ready_connection()?;
        Self::insert_with_conn(&conn, draft)
    }

    fn update(&self, id: i64, draft: &TransactionDraft) -> Result<()> {
        let conn = self.ready_connection()?;
        let updated = conn
            .execute(
                "
                UPDATE transactions
                SET
                    date = ?1,
                    description = ?2,
                    amount = ?3,
                    transaction_type = ?4,
                    category = ?5,
                    subcategory = ?6,
                    is_recurring = ?7,
                    recurrence_frequency = ?8,
                    recurrence_end_date = ?9
                WHERE id = ?10
                ",
                params![
                    draft.date.format(DATE_FORMAT).to_string(),
                    &draft.description,
                    draft.amount.normalize().to_string(),
                    draft.transaction_type.as_str(),
                    &draft.category,
                    &draft.subcategory,
                    draft.is_recurring as i64,
                    draft.recurrence_frequency.map(|freq| freq.to_string()),
                    draft
                        .recurrence_end_date
                        .map(|date| date.format(DATE_FORMAT).to_string()),
                    id,
                ],
            )
            .map_err(|err| Error::other(format!("Failed to update transaction: {}", err)))?;

        if updated == 0 {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Transaction with id {} was not found.", id),
            ));
        }
        Ok(())
    }

    fn delete(&self, id: i64) -> Result<()> {
        let conn = self.ready_connection()?;
        let deleted = conn
            .execute("DELETE FROM transactions WHERE id = ?1", [id])
            .map_err(|err| Error::other(format!("Failed to delete transaction: {}", err)))?;

        if deleted == 0 {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Transaction with id {} was not found.", id),
            ));
        }
        Ok(())
    }

    fn import_merge(&self, rows: &[Transaction]) -> Result<ImportSummary> {
        let mut conn = self.ready_connection()?;
        let tx = conn
            .transaction()
            .map_err(|err| Error::other(format!("Failed to begin import: {}", err)))?;

        // Insert oldest first so auto-increment ids line up with chronological order
        // (otherwise a newest-first CSV would give the most recent row the lowest id).
        let mut ordered: Vec<&Transaction> = rows.iter().collect();
        ordered.sort_by_key(|row| row.date);

        let mut summary = ImportSummary::default();
        for row in ordered {
            if Self::natural_key_exists(&tx, row)? {
                summary.skipped += 1;
            } else {
                Self::insert_with_conn(&tx, &row.to_draft())?;
                summary.added += 1;
            }
        }

        tx.commit()
            .map_err(|err| Error::other(format!("Failed to commit import: {}", err)))?;
        Ok(summary)
    }

    fn apply_category_rename(&self, old: &CategoryRecord, new: &CategoryDraft) -> Result<()> {
        let conn = self.ready_connection()?;
        conn.execute(
            "
            UPDATE transactions
            SET transaction_type = ?1, category = ?2, subcategory = ?3
            WHERE transaction_type = ?4
              AND LOWER(category) = LOWER(?5)
              AND LOWER(subcategory) = LOWER(?6)
            ",
            params![
                new.transaction_type.as_str(),
                &new.category,
                &new.subcategory,
                old.transaction_type.as_str(),
                &old.category,
                &old.subcategory,
            ],
        )
        .map_err(|err| {
            Error::other(format!(
                "Failed to update transactions for category: {}",
                err
            ))
        })?;
        Ok(())
    }

    fn apply_category_clear(&self, record: &CategoryRecord) -> Result<()> {
        let conn = self.ready_connection()?;
        // Deleting a top-level category (no subcategory) resets matches to Uncategorized;
        // deleting a subcategory only clears the subcategory field.
        let set_clause = if record.subcategory.is_empty() {
            "category = 'Uncategorized', subcategory = ''"
        } else {
            "subcategory = ''"
        };
        conn.execute(
            &format!(
                "
                UPDATE transactions
                SET {}
                WHERE transaction_type = ?1
                  AND LOWER(category) = LOWER(?2)
                  AND LOWER(subcategory) = LOWER(?3)
                ",
                set_clause
            ),
            params![
                record.transaction_type.as_str(),
                &record.category,
                &record.subcategory,
            ],
        )
        .map_err(|err| {
            Error::other(format!(
                "Failed to clear transactions for category: {}",
                err
            ))
        })?;
        Ok(())
    }
}

fn parse_date(index: usize, value: &str) -> rusqlite::Result<NaiveDate> {
    NaiveDate::parse_from_str(value, DATE_FORMAT).map_err(|err| {
        SqlError::FromSqlConversionFailure(
            index,
            Type::Text,
            Box::new(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid date '{}' in transaction database: {}", value, err),
            )),
        )
    })
}

fn parse_decimal(index: usize, value: &str) -> rusqlite::Result<Decimal> {
    Decimal::from_str(value.trim()).map_err(|err| {
        SqlError::FromSqlConversionFailure(
            index,
            Type::Text,
            Box::new(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Invalid amount '{}' in transaction database: {}",
                    value, err
                ),
            )),
        )
    })
}

fn parse_transaction_type(index: usize, value: &str) -> rusqlite::Result<TransactionType> {
    TransactionType::try_from(value).map_err(|_| {
        SqlError::FromSqlConversionFailure(
            index,
            Type::Text,
            Box::new(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Invalid transaction type '{}' in transaction database.",
                    value
                ),
            )),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::SCHEMA_VERSION;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A temporary on-disk database that deletes itself (and its sidecar files) when dropped.
    struct TempDb {
        path: PathBuf,
    }

    impl TempDb {
        fn new() -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "budget_tracker_test_{}_{}_{}.db",
                std::process::id(),
                nanos,
                unique
            ));
            Self { path }
        }

        fn store(&self) -> SqliteTransactionStore {
            SqliteTransactionStore::new(SqliteDatabase::new(&self.path))
        }
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
            let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
            let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
        }
    }

    fn draft(date: &str, description: &str, amount: &str, category: &str) -> TransactionDraft {
        TransactionDraft {
            date: NaiveDate::parse_from_str(date, DATE_FORMAT).unwrap(),
            description: description.to_string(),
            amount: Decimal::from_str(amount).unwrap(),
            transaction_type: TransactionType::Expense,
            category: category.to_string(),
            subcategory: String::new(),
            is_recurring: false,
            recurrence_frequency: None,
            recurrence_end_date: None,
        }
    }

    #[test]
    fn migration_creates_schema_at_latest_version() {
        let temp = TempDb::new();
        // Listing forces the schema/migrations to run.
        assert!(temp.store().list().unwrap().is_empty());

        let conn = SqliteDatabase::new(&temp.path)
            .open_connection("test")
            .unwrap();
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn insert_list_update_delete_roundtrip() {
        let temp = TempDb::new();
        let store = temp.store();

        let id = store
            .insert(&draft("2026-01-05", "Coffee", "4.50", "Food"))
            .unwrap();

        let rows = store.list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, Some(id));
        assert_eq!(rows[0].description, "Coffee");
        assert_eq!(rows[0].amount, Decimal::from_str("4.50").unwrap());
        assert!(!rows[0].is_recurring);

        let mut updated = draft("2026-01-06", "Latte", "5.25", "Food");
        updated.is_recurring = true;
        updated.recurrence_frequency = Some(RecurrenceFrequency::Monthly);
        store.update(id, &updated).unwrap();

        let rows = store.list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].description, "Latte");
        assert!(rows[0].is_recurring);
        assert_eq!(
            rows[0].recurrence_frequency,
            Some(RecurrenceFrequency::Monthly)
        );

        store.delete(id).unwrap();
        assert!(store.list().unwrap().is_empty());
    }

    #[test]
    fn import_merge_skips_duplicates() {
        let temp = TempDb::new();
        let store = temp.store();
        store
            .insert(&draft("2026-01-05", "Coffee", "4.50", "Food"))
            .unwrap();

        // One duplicate (note "4.5" vs stored "4.50", which canonicalize equal) and one new row.
        let dup = draft("2026-01-05", "Coffee", "4.5", "Food").into_transaction();
        let fresh = draft("2026-02-01", "Books", "20", "Education").into_transaction();

        let summary = store.import_merge(&[dup, fresh]).unwrap();
        assert_eq!(summary.added, 1);
        assert_eq!(summary.skipped, 1);
        assert_eq!(store.list().unwrap().len(), 2);
    }

    #[test]
    fn importing_a_csv_drops_generated_rows() {
        let temp = TempDb::new();
        let csv_path = temp.path.with_extension("csv");
        std::fs::write(
            &csv_path,
            "date,description,amount,transaction_type,category,subcategory,is_recurring,recurrence_frequency,recurrence_end_date,is_generated_from_recurring\n\
             2026-01-01,Rent,1000,Expense,Housing,Rent,true,Monthly,,false\n\
             2026-02-01,Rent,1000,Expense,Housing,Rent,true,Monthly,,true\n\
             2026-03-01,Rent,1000,Expense,Housing,Rent,true,Monthly,,true\n\
             2026-01-15,Coffee,4.50,Expense,Food,Coffee,false,,,false\n",
        )
        .unwrap();

        let rows = crate::persistence::load_transactions(&csv_path).unwrap();
        assert_eq!(rows.len(), 4, "all CSV rows parse");

        // The import path drops generated occurrences, keeping only real rows (source + normal).
        let real_rows: Vec<_> = rows
            .into_iter()
            .filter(|tx| !tx.is_generated_from_recurring)
            .collect();
        let summary = temp.store().import_merge(&real_rows).unwrap();
        assert_eq!(summary.added, 2);

        let stored = temp.store().list().unwrap();
        assert_eq!(stored.len(), 2);
        assert!(stored.iter().all(|tx| !tx.is_generated_from_recurring));
        // The recurring source survived with its rule intact.
        assert!(stored
            .iter()
            .any(|tx| tx.is_recurring
                && tx.recurrence_frequency == Some(RecurrenceFrequency::Monthly)));

        let _ = std::fs::remove_file(&csv_path);
    }

    // Small helper to turn a draft into a Transaction for import tests.
    impl TransactionDraft {
        fn into_transaction(self) -> Transaction {
            Transaction {
                date: self.date,
                description: self.description,
                amount: self.amount,
                transaction_type: self.transaction_type,
                category: self.category,
                subcategory: self.subcategory,
                is_recurring: self.is_recurring,
                recurrence_frequency: self.recurrence_frequency,
                recurrence_end_date: self.recurrence_end_date,
                is_generated_from_recurring: false,
                id: None,
                parent_id: None,
            }
        }
    }
}
