use crate::database::SqliteDatabase;
use crate::model::{CategoryDraft, CategoryInfo, CategoryRecord, TransactionType};
use rusqlite::{params, Connection, Row};
use rust_decimal::Decimal;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

pub trait CategoryStore {
    fn initialize(&self, seed_categories: &[CategoryInfo]) -> Result<()>;
    fn list(&self) -> Result<Vec<CategoryRecord>>;
    fn insert(&self, draft: &CategoryDraft) -> Result<CategoryRecord>;
    fn update(&self, id: i64, draft: &CategoryDraft) -> Result<()>;
    fn delete(&self, id: i64) -> Result<()>;
}

pub struct SqliteCategoryStore {
    database: SqliteDatabase,
}

impl SqliteCategoryStore {
    pub fn new(database: SqliteDatabase) -> Self {
        Self { database }
    }

    fn open_connection(&self) -> Result<Connection> {
        self.database.open_connection("category")
    }

    fn initialize_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY,
                transaction_type TEXT NOT NULL CHECK (transaction_type IN ('Income', 'Expense')),
                category TEXT NOT NULL,
                subcategory TEXT NOT NULL DEFAULT '',
                tag TEXT NULL,
                target_budget TEXT NULL,
                UNIQUE(transaction_type, category, subcategory)
            );
            ",
        )
        .map_err(|err| Error::other(format!("Failed to initialize category schema: {}", err)))?;

        Self::ensure_target_budget_column(conn)
    }

    fn ensure_target_budget_column(conn: &Connection) -> Result<()> {
        let mut stmt = conn
            .prepare("PRAGMA table_info(categories)")
            .map_err(|err| Error::other(format!("Failed to inspect category schema: {}", err)))?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|err| Error::other(format!("Failed to read category schema: {}", err)))?;

        let has_target_budget = columns
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| Error::other(format!("Failed to collect category schema: {}", err)))?
            .into_iter()
            .any(|name| name == "target_budget");

        if !has_target_budget {
            conn.execute(
                "ALTER TABLE categories ADD COLUMN target_budget TEXT NULL",
                [],
            )
            .map_err(|err| {
                Error::other(format!(
                    "Failed to add target_budget column to categories: {}",
                    err
                ))
            })?;
        }

        Ok(())
    }

    fn seed_if_empty(&self, conn: &Connection, seed_categories: &[CategoryInfo]) -> Result<()> {
        let seeded_flag = self
            .database
            .metadata_value(conn, "category_seed_version")?;

        if seeded_flag.is_some() {
            return Ok(());
        }

        let mut stmt = conn
            .prepare(
                "
                INSERT INTO categories (
                    transaction_type,
                    category,
                    subcategory,
                    tag,
                    target_budget
                ) VALUES (?1, ?2, ?3, NULL, NULL)
                ",
            )
            .map_err(|err| {
                Error::other(format!("Failed to prepare category seed insert: {}", err))
            })?;

        for category in seed_categories {
            stmt.execute(params![
                category.transaction_type.as_str(),
                &category.category,
                &category.subcategory
            ])
            .map_err(|err| Error::other(format!("Failed to seed categories: {}", err)))?;
        }

        drop(stmt);
        self.database
            .set_metadata_value(conn, "category_seed_version", "1")?;

        Ok(())
    }

    fn load_record_by_id(conn: &Connection, id: i64) -> Result<CategoryRecord> {
        conn.query_row(
            "
            SELECT id, transaction_type, category, subcategory, tag, target_budget
            FROM categories
            WHERE id = ?1
            ",
            [id],
            Self::row_to_record,
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => Error::new(
                ErrorKind::NotFound,
                format!("Category with id {} was not found.", id),
            ),
            other => Error::other(format!("Failed to load category {}: {}", id, other)),
        })
    }

    fn row_to_record(row: &Row<'_>) -> rusqlite::Result<CategoryRecord> {
        let transaction_type_str: String = row.get(1)?;
        let target_budget_str: Option<String> = row.get(5)?;

        let transaction_type =
            TransactionType::try_from(transaction_type_str.as_str()).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Invalid transaction type '{}' in category database.",
                            transaction_type_str
                        ),
                    )),
                )
            })?;

        let target_budget = match target_budget_str {
            Some(value) if !value.trim().is_empty() => {
                Some(Decimal::from_str(value.trim()).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(Error::new(
                            ErrorKind::InvalidData,
                            format!(
                                "Invalid target budget '{}' in category database: {}",
                                value, err
                            ),
                        )),
                    )
                })?)
            }
            _ => None,
        };

        Ok(CategoryRecord {
            id: row.get(0)?,
            transaction_type,
            category: row.get(2)?,
            subcategory: row.get(3)?,
            tag: row.get(4)?,
            target_budget,
        })
    }
}

impl CategoryStore for SqliteCategoryStore {
    fn initialize(&self, seed_categories: &[CategoryInfo]) -> Result<()> {
        let conn = self.open_connection()?;
        self.database.ensure_metadata_table(&conn)?;
        Self::initialize_schema(&conn)?;
        self.seed_if_empty(&conn, seed_categories)
    }

    fn list(&self) -> Result<Vec<CategoryRecord>> {
        let conn = self.open_connection()?;
        self.database.ensure_metadata_table(&conn)?;
        Self::initialize_schema(&conn)?;

        let mut stmt = conn
            .prepare(
                "
                SELECT id, transaction_type, category, subcategory, tag, target_budget
                FROM categories
                ORDER BY
                    CASE transaction_type
                        WHEN 'Income' THEN 0
                        ELSE 1
                    END,
                    LOWER(category),
                    LOWER(subcategory),
                    id
                ",
            )
            .map_err(|err| Error::other(format!("Failed to prepare category query: {}", err)))?;

        let rows = stmt
            .query_map([], Self::row_to_record)
            .map_err(|err| Error::other(format!("Failed to load categories: {}", err)))?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| Error::other(format!("Failed to read categories: {}", err)))
    }

    fn insert(&self, draft: &CategoryDraft) -> Result<CategoryRecord> {
        let conn = self.open_connection()?;
        self.database.ensure_metadata_table(&conn)?;
        Self::initialize_schema(&conn)?;

        conn.execute(
            "
            INSERT INTO categories (
                transaction_type,
                category,
                subcategory,
                tag,
                target_budget
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                draft.transaction_type.as_str(),
                &draft.category,
                &draft.subcategory,
                &draft.tag,
                draft.target_budget.map(|value| value.to_string())
            ],
        )
        .map_err(|err| Error::other(format!("Failed to insert category: {}", err)))?;

        Self::load_record_by_id(&conn, conn.last_insert_rowid())
    }

    fn update(&self, id: i64, draft: &CategoryDraft) -> Result<()> {
        let conn = self.open_connection()?;
        self.database.ensure_metadata_table(&conn)?;
        Self::initialize_schema(&conn)?;

        let updated = conn
            .execute(
                "
                UPDATE categories
                SET
                    transaction_type = ?1,
                    category = ?2,
                    subcategory = ?3,
                    tag = ?4,
                    target_budget = ?5
                WHERE id = ?6
                ",
                params![
                    draft.transaction_type.as_str(),
                    &draft.category,
                    &draft.subcategory,
                    &draft.tag,
                    draft.target_budget.map(|value| value.to_string()),
                    id
                ],
            )
            .map_err(|err| Error::other(format!("Failed to update category: {}", err)))?;

        if updated == 0 {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Category with id {} was not found.", id),
            ));
        }

        Ok(())
    }

    fn delete(&self, id: i64) -> Result<()> {
        let conn = self.open_connection()?;
        self.database.ensure_metadata_table(&conn)?;
        Self::initialize_schema(&conn)?;

        let deleted = conn
            .execute("DELETE FROM categories WHERE id = ?1", [id])
            .map_err(|err| Error::other(format!("Failed to delete category: {}", err)))?;

        if deleted == 0 {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Category with id {} was not found.", id),
            ));
        }

        Ok(())
    }
}
