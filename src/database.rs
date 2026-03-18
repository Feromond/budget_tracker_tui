use rusqlite::{Connection, OptionalExtension};
use std::fs::create_dir_all;
use std::io::{Error, Result};
use std::path::{Path, PathBuf};

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

    pub fn ensure_metadata_table(&self, conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS database_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )
        .map_err(|err| Error::other(format!("Failed to initialize database metadata: {}", err)))
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
