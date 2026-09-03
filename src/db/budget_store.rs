use crate::db::database::SqliteDatabase;
use crate::model::{BudgetMonth, BudgetPeriod};
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
    /// Drop the period beginning exactly at `start` so the month inherits the previous one.
    fn remove(&self, ledger_id: i64, category_id: Option<i64>, start: BudgetMonth) -> Result<()>;
    /// Drop every period for one scope, used when replacing a budget's whole history.
    fn remove_all(&self, ledger_id: i64, category_id: Option<i64>) -> Result<()>;
    fn copy_ledger(&self, source_ledger_id: i64, target_ledger_id: i64) -> Result<()>;
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
        let mut conn = self.ready_connection()?;
        let tx = conn
            .transaction()
            .map_err(|err| Error::other(format!("Failed to begin budget write: {}", err)))?;

        // Replace rather than upsert: the uniqueness lives in two partial indexes, which
        // ON CONFLICT cannot target as one.
        tx.execute(
            "
            DELETE FROM budget_periods
            WHERE ledger_id = ?1
              AND category_id IS ?2
              AND start_year = ?3
              AND start_month = ?4
            ",
            params![ledger_id, category_id, start.year, start.month],
        )
        .map_err(|err| Error::other(format!("Failed to replace budget period: {}", err)))?;

        tx.execute(
            "
            INSERT INTO budget_periods (ledger_id, category_id, start_year, start_month, amount)
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

        tx.commit()
            .map_err(|err| Error::other(format!("Failed to commit budget write: {}", err)))
    }

    fn remove(&self, ledger_id: i64, category_id: Option<i64>, start: BudgetMonth) -> Result<()> {
        let conn = self.ready_connection()?;
        conn.execute(
            "
            DELETE FROM budget_periods
            WHERE ledger_id = ?1
              AND category_id IS ?2
              AND start_year = ?3
              AND start_month = ?4
            ",
            params![ledger_id, category_id, start.year, start.month],
        )
        .map_err(|err| Error::other(format!("Failed to remove budget period: {}", err)))?;
        Ok(())
    }

    fn remove_all(&self, ledger_id: i64, category_id: Option<i64>) -> Result<()> {
        let conn = self.ready_connection()?;
        conn.execute(
            "DELETE FROM budget_periods WHERE ledger_id = ?1 AND category_id IS ?2",
            params![ledger_id, category_id],
        )
        .map_err(|err| Error::other(format!("Failed to clear budget history: {}", err)))?;
        Ok(())
    }

    fn copy_ledger(&self, source_ledger_id: i64, target_ledger_id: i64) -> Result<()> {
        let conn = self.ready_connection()?;
        conn.execute(
            "
            INSERT INTO budget_periods (ledger_id, category_id, start_year, start_month, amount)
            SELECT ?1, category_id, start_year, start_month, amount
            FROM budget_periods
            WHERE ledger_id = ?2
            ",
            params![target_ledger_id, source_ledger_id],
        )
        .map_err(|err| Error::other(format!("Failed to copy budgets: {}", err)))?;
        Ok(())
    }
}
