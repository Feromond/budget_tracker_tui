use crate::db::database::SqliteDatabase;
use crate::model::{BudgetMonth, BudgetPeriod, BudgetWrite};
use rusqlite::{Connection, Row, params};
use rust_decimal::Decimal;
use std::io::{Error, ErrorKind, Result};
use std::str::FromStr;

pub trait BudgetStore {
    fn list(&self, ledger_id: i64) -> Result<Vec<BudgetPeriod>>;
    /// Apply `amount` from `start` on. `None` clears the budget from that month.
    fn set(
        &self,
        ledger_id: i64,
        category_id: Option<i64>,
        start: BudgetMonth,
        amount: Option<Decimal>,
    ) -> Result<()>;
    /// One transaction, since a plan can erase history before writing its replacement.
    fn apply(&self, ledger_id: i64, category_id: Option<i64>, writes: &[BudgetWrite])
    -> Result<()>;
}

pub struct SqliteBudgetStore {
    database: SqliteDatabase,
}

impl SqliteBudgetStore {
    pub fn new(database: SqliteDatabase) -> Self {
        Self { database }
    }

    fn ready_connection(&self) -> Result<Connection> {
        self.database.ready_connection("budget")
    }

    fn row_to_period(row: &Row<'_>) -> rusqlite::Result<BudgetPeriod> {
        let amount_str: Option<String> = row.get(4)?;
        let amount = match amount_str {
            Some(value) if !value.trim().is_empty() => {
                Some(Decimal::from_str(value.trim()).map_err(|err| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(Error::new(
                            ErrorKind::InvalidData,
                            format!("Invalid budget amount '{}' in database: {}", value, err),
                        )),
                    )
                })?)
            }
            _ => None,
        };

        Ok(BudgetPeriod {
            id: row.get(0)?,
            category_id: row.get(1)?,
            start: BudgetMonth::new(row.get(2)?, row.get(3)?),
            amount,
        })
    }

    /// `category_id IS ?` rather than `=`, so the NULL key of a monthly budget matches.
    fn delete_period(
        conn: &Connection,
        ledger_id: i64,
        category_id: Option<i64>,
        start: BudgetMonth,
    ) -> Result<()> {
        conn.execute(
            "
            DELETE FROM budget_periods
            WHERE ledger_id = ?1 AND category_id IS ?2 AND start_year = ?3 AND start_month = ?4
            ",
            params![ledger_id, category_id, start.year, start.month],
        )
        .map_err(|err| Error::other(format!("Failed to remove budget period: {}", err)))?;
        Ok(())
    }

    fn apply_write(
        conn: &Connection,
        ledger_id: i64,
        category_id: Option<i64>,
        write: &BudgetWrite,
    ) -> Result<()> {
        match write {
            BudgetWrite::Set(start, amount) => {
                // Replace rather than upsert: the uniqueness lives in two partial indexes,
                // which ON CONFLICT cannot target as one.
                Self::delete_period(conn, ledger_id, category_id, *start)?;
                conn.execute(
                    "
                    INSERT INTO budget_periods
                        (ledger_id, category_id, start_year, start_month, amount)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    ",
                    params![
                        ledger_id,
                        category_id,
                        start.year,
                        start.month,
                        amount.map(|value| value.to_string())
                    ],
                )
                .map_err(|err| Error::other(format!("Failed to save budget period: {}", err)))?;
            }
            BudgetWrite::Remove(start) => {
                Self::delete_period(conn, ledger_id, category_id, *start)?;
            }
            BudgetWrite::RemoveAll => {
                conn.execute(
                    "DELETE FROM budget_periods WHERE ledger_id = ?1 AND category_id IS ?2",
                    params![ledger_id, category_id],
                )
                .map_err(|err| Error::other(format!("Failed to clear budget history: {}", err)))?;
            }
        }
        Ok(())
    }
}

impl BudgetStore for SqliteBudgetStore {
    fn list(&self, ledger_id: i64) -> Result<Vec<BudgetPeriod>> {
        let conn = self.ready_connection()?;
        let mut stmt = conn
            .prepare(
                "
                SELECT id, category_id, start_year, start_month, amount
                FROM budget_periods
                WHERE ledger_id = ?1
                ORDER BY start_year, start_month, id
                ",
            )
            .map_err(|err| Error::other(format!("Failed to prepare budget query: {}", err)))?;

        stmt.query_map([ledger_id], Self::row_to_period)
            .map_err(|err| Error::other(format!("Failed to load budgets: {}", err)))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|err| Error::other(format!("Failed to read budgets: {}", err)))
    }

    fn set(
        &self,
        ledger_id: i64,
        category_id: Option<i64>,
        start: BudgetMonth,
        amount: Option<Decimal>,
    ) -> Result<()> {
        self.apply(ledger_id, category_id, &[BudgetWrite::Set(start, amount)])
    }

    fn apply(
        &self,
        ledger_id: i64,
        category_id: Option<i64>,
        writes: &[BudgetWrite],
    ) -> Result<()> {
        let mut conn = self.ready_connection()?;
        let tx = conn
            .transaction()
            .map_err(|err| Error::other(format!("Failed to begin budget write: {}", err)))?;
        for write in writes {
            Self::apply_write(&tx, ledger_id, category_id, write)?;
        }
        tx.commit()
            .map_err(|err| Error::other(format!("Failed to commit budget write: {}", err)))
    }
}
