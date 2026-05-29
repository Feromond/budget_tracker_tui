use rusqlite::{Connection, OptionalExtension};
use std::fs::create_dir_all;
use std::io::{Error, Result};
use std::path::{Path, PathBuf};

/// The latest schema version understood by this build. Bump this and add a matching arm in
/// [`SqliteDatabase::apply_migration`] whenever the schema changes.
pub const SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Clone)]
pub struct SqliteDatabase {
    path: PathBuf,
}

impl SqliteDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn open_connection(&self, purpose: &str) -> Result<Connection> {
        self.ensure_parent_dir()?;
        Connection::open(&self.path).map_err(|err| {
            Error::other(format!(
                "Failed to open {} database '{}': {}",
                purpose,
                self.path.display(),
                err
            ))
        })
    }

    pub fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Apply any pending schema migrations, keyed on SQLite's built-in `PRAGMA user_version`.
    /// Steps run in order inside a single transaction; the version is bumped on success. This
    /// is the one place schema is created, so it is safe to call before every operation
    /// (it is a cheap version read once the database is up to date).
    pub fn run_migrations(&self, conn: &mut Connection) -> Result<()> {
        let current: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|err| Error::other(format!("Failed to read schema version: {}", err)))?;

        if current >= SCHEMA_VERSION {
            return Ok(());
        }

        let tx = conn
            .transaction()
            .map_err(|err| Error::other(format!("Failed to begin migration: {}", err)))?;

        for version in (current + 1)..=SCHEMA_VERSION {
            Self::apply_migration(&tx, version)?;
        }

        // `user_version` does not accept bound parameters, so format it into the statement.
        tx.execute_batch(&format!("PRAGMA user_version = {};", SCHEMA_VERSION))
            .map_err(|err| Error::other(format!("Failed to set schema version: {}", err)))?;
        tx.commit()
            .map_err(|err| Error::other(format!("Failed to commit migration: {}", err)))
    }

    fn apply_migration(conn: &Connection, version: i64) -> Result<()> {
        match version {
            // v1: metadata + categories (idempotent so pre-existing databases upgrade cleanly).
            1 => {
                conn.execute_batch(
                    "
                    CREATE TABLE IF NOT EXISTS database_meta (
                        key TEXT PRIMARY KEY,
                        value TEXT NOT NULL
                    );
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
                .map_err(|err| Error::other(format!("Migration v1 failed: {}", err)))?;
                // Databases created before target_budget existed need the column added.
                Self::ensure_column(conn, "categories", "target_budget", "TEXT NULL")
            }
            // v2: transactions (real rows only — generated occurrences are derived in-memory).
            2 => conn
                .execute_batch(
                    "
                    CREATE TABLE IF NOT EXISTS transactions (
                        id INTEGER PRIMARY KEY,
                        date TEXT NOT NULL,
                        description TEXT NOT NULL,
                        amount TEXT NOT NULL,
                        transaction_type TEXT NOT NULL CHECK (transaction_type IN ('Income', 'Expense')),
                        category TEXT NOT NULL DEFAULT 'Uncategorized',
                        subcategory TEXT NOT NULL DEFAULT '',
                        is_recurring INTEGER NOT NULL DEFAULT 0,
                        recurrence_frequency TEXT NULL,
                        recurrence_end_date TEXT NULL
                    );
                    CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
                    ",
                )
                .map_err(|err| Error::other(format!("Migration v2 failed: {}", err))),
            _ => Ok(()),
        }
    }

    /// Add a column to a table if it is not already present (for upgrading legacy databases).
    fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({})", table))
            .map_err(|err| Error::other(format!("Failed to inspect {} schema: {}", table, err)))?;
        let exists = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|err| Error::other(format!("Failed to read {} schema: {}", table, err)))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| Error::other(format!("Failed to collect {} schema: {}", table, err)))?
            .into_iter()
            .any(|name| name == column);

        if !exists {
            conn.execute(
                &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
                [],
            )
            .map_err(|err| Error::other(format!("Failed to add {}.{}: {}", table, column, err)))?;
        }
        Ok(())
    }

    pub fn metadata_value(&self, conn: &Connection, key: &str) -> Result<Option<String>> {
        conn.query_row(
            "SELECT value FROM database_meta WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|err| {
            Error::other(format!(
                "Failed to read database metadata '{}': {}",
                key, err
            ))
        })
    }

    pub fn set_metadata_value(&self, conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "
            INSERT INTO database_meta (key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            ",
            [key, value],
        )
        .map_err(|err| {
            Error::other(format!(
                "Failed to write database metadata '{}': {}",
                key, err
            ))
        })?;
        Ok(())
    }
}
