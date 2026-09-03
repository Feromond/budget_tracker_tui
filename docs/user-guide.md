# User Guide

The companion to the [README](../README.md), with the longer explanations that don't belong there. You rarely need this while using the app, since the help bar at the bottom of every view shows the relevant keys and `Ctrl+H` opens the full keybindings menu for whatever mode you're in.

## The main view

The transaction list is where you land on launch, with a summary bar up top.

- `↑`/`↓` move between transactions, `PageUp`/`PageDown` jump by page, `Ctrl+↑`/`Ctrl+↓` jump to the first/last transaction
- `1`-`6` (or `F1`-`F6`) sort by Date, Description, Category, Subcategory, Type, or Amount; press again to reverse
- `a` adds a transaction, `e` edits the selected one, `d` deletes it (with a `y`/`n` confirmation), `Ctrl+C` copies it
- `f` opens the quick filter, `Ctrl+F` the advanced filter
- `r` opens recurring settings for the selected transaction
- `s`, `c`, and `b` open the monthly summary, category summary, and budget views
- `o` opens settings
- `q` or `Esc` clears any active filter, or quits the app when no filter is active

## Adding and editing transactions

`Tab`/`Shift+Tab` or `↑`/`↓` move between fields and `Enter` saves. In the date field, `+` (or `=`) moves the date forward a day, `-` moves it back, and `Shift+←`/`Shift+→` jump by month. Category and subcategory fields offer a selection list (with fuzzy search if you've enabled it in settings), and `←`/`→` toggle the income/expense type.

## Filtering

The quick filter (`f`) matches as you type across your transactions. `Enter` closes the input and keeps the filter applied; `Esc` or `Ctrl+R` clears it.

The advanced filter (`Ctrl+F`) filters on multiple fields at once: date range, description, category, type, recurring status, and amount. `Tab` or `↑`/`↓` move between fields, `Enter` applies, `Esc` cancels, and `Ctrl+R` resets everything.

The Recurring field is a `←`/`→` toggle that cycles through blank (everything), `Recurring`, and `One-Time`. Setting it to `Recurring` narrows the table and the summary totals to your recurring payments and their generated occurrences, so you can see what a cycle costs.

## Recurring transactions

Select a transaction and press `r` to make it recurring. Available frequencies:

- Daily
- Weekly
- Bi-weekly
- Semi-monthly (1st and 15th)
- Semi-monthly, weekday adjusted (shifted off weekends)
- Monthly
- Quarterly
- Yearly

Occurrences are generated automatically from the start date up to today, and an optional end date stops the series. Generated occurrences stay linked to their source transaction. Edit or delete the source to affect the series.

### Forecasting ahead

By default generation stops at today. Set *Forecast Months Ahead* in settings to project occurrences further out, up to 60 months. Forecast rows appear dimmed in the transaction table, the table title shows the active horizon, and the summary and budget views pick up the projected months so you can see where the year is heading.

Forecast occurrences are derived in memory like every other generated occurrence: nothing extra is written to the database, an end date still cuts the series off, and setting the horizon back to `0` returns to today-only generation.

## Summary views

**Monthly summary (`s`)** shows income, expenses, and net per month with an interactive chart. `↑`/`↓` move between months, `←`/`→` (or `[`/`]`) move between years. `m` toggles a multi-month line chart, and `c` toggles cumulative mode, which also draws the target budget line from settings.

**Category summary (`c`)** breaks down each month by category. `Enter` expands or collapses a month, `PageUp`/`PageDown` move between months, `←`/`→` between years.

**Budget view (`b`)** compares spending against your monthly target and any per-category budgets. `↑`/`↓` move between categories, `←`/`→` between months, `Shift+←`/`Shift+→` between years. Press `e` on a row to change that category's target budget in place. The popup starts on
the current amount; `Enter` saves, `Esc` cancels, and an empty amount clears the budget, which also
drops the category from the table since only budgeted categories are listed. Press `c` to open the
[category catalog](#the-category-catalog), which is where you set a budget on a category that
doesn't have one yet.

## The category catalog

The catalog holds your categories and subcategories. Open it from Settings (*Manage Categories*) or with `c` from the budget view. `q`/`Esc` returns to whichever view you came from.

- `↑`/`↓` move between entries, `PageUp`/`PageDown` jump by page, `Ctrl+↑`/`Ctrl+↓` jump to the first/last entry
- `f` filters the catalog as you type; `Enter` keeps the filter applied, `Esc` or `Ctrl+R` clears it
- `a` adds a category, `e` or `Enter` edits the selected one, `d` deletes it
- `1`-`5` (or `F1`-`F5`) sort by type, category, subcategory, tag, or target budget; pressing the same key again flips the direction, and the sorted column is marked in the header
- Expense categories can optionally hold a per-category target budget, used by the budget view

## Ledgers

A ledger is a self-contained set of transactions. One database can hold several of them, so you
can keep separate books for different accounts or goals, or copy of a scenario to experiment with
forecasting without touching your real data. Every ledger in a database **shares the same category
catalog**, including per-category target budgets, so a category you rename or delete updates the
transactions in all of them.

Open the list from Settings (*Ledger*, which shows the ledger currently open).

- `↑`/`↓` move between ledgers; the open one is marked with a dot
- `Enter` switches to the selected ledger and returns to settings
- `a` adds an empty ledger, `e` renames the selected one
- `Ctrl+C` copies the selected ledger, transactions and all, into a new one. Handy for trying a
  forecast or a what-if against real numbers without touching the original. You're offered a name
  like `Main (copy)`, which you can edit before saving.
- `d` deletes the selected ledger **and every transaction in it**, after a confirmation that names
  the ledger and its transaction count. The last remaining ledger can't be deleted.
- `q`/`Esc` returns to settings

The name of the open ledger is shown in the transaction list's title. The transaction list,
filters, summary views, budget view, and CSV import/export all apply to the open ledger only.

Existing databases upgrade automatically: all of your current transactions land in a ledger named
`Main`, and nothing else changes.

To move transactions between ledgers, export them to CSV, switch ledgers, then import that file.

## Settings

Press `o` to open settings. The menu is grouped into sections:

**Data Management**

- *Database Path*: where the SQLite database lives (see [Data storage](#data-storage) below).
- *Ledger*: shows the ledger currently open; opens the [ledger list](#ledgers) to switch or manage them.
- *Manage Categories*: opens the [category catalog](#the-category-catalog).
- *Import Transactions (CSV)*: merges a CSV file into the open ledger; new rows are added, exact duplicates are skipped.
- *Export Transactions (CSV)*: writes the open ledger's transactions to a CSV file for backup or use elsewhere.

**Monthly Summary View**

- *Target Budget*: your monthly spending goal, drawn as a line in the monthly summary's cumulative mode and used by the budget view.

**Transaction View**

- *Hourly Rate*: optionally enter your hourly earning rate; a *Show Costs in Hours* toggle then appears that displays amounts as hours worked.

**Recurring Transactions**

- *Forecast Months Ahead*: projects recurring occurrences this many months past today (0-60). `0` stops at today.

**Input Preferences**

- *Fuzzy Search Categories*: enables fuzzy matching when picking categories and subcategories.

**General Preferences**

- *Hide Help Bar*: hides the bottom help bar if you want the extra screen space (`Ctrl+H` still works).

## Data storage

Transactions and categories are stored together in a local SQLite database (`budget.db`). On first run with a new database, it's seeded with the default category catalog and a ledger named `Main`. Default locations:

- **Linux:** `$XDG_DATA_HOME/BudgetTracker/budget.db` (usually `~/.local/share/BudgetTracker/budget.db`)
- **macOS:** `~/Library/Application Support/BudgetTracker/budget.db`
- **Windows:** `%APPDATA%\BudgetTracker\budget.db`

The path is configurable in settings. Point it at a folder synced by iCloud, Google Drive, Dropbox, OneDrive, or similar to sync your budget across devices.

App preferences live separately in a `config.json` in your OS config directory:

- **Linux:** `~/.config/BudgetTracker/config.json`
- **macOS:** `~/Library/Application Support/BudgetTracker/config.json`
- **Windows:** `%APPDATA%\BudgetTracker\config.json`

Changes are written to the database immediately as you add, edit, or delete, so there's no separate save step. CSV files are only written when you explicitly export.

### Migrating from older versions

Versions before 1.4.0 stored transactions in a `transactions.csv` file. On first launch, the app imports that file into the database automatically and renames the original to `transactions.csv.migrated-backup`. Nothing is deleted.

The database carries a data version, and the app upgrades it in place when you install a newer release. Upgrades run in a single transaction, so a failure leaves the database exactly as it was. If you open a database that a **newer** version of the app has already upgraded (easy to do when the file is synced between machines running different versions), the older app refuses to open it and tells you to update, rather than reading or writing a schema it doesn't understand.

## CSV format

Import and export use these columns:

```
date, description, amount, transaction_type, category, subcategory
```

- **Date:** accepts `YYYY-MM-DD`, `YYYY/MM/DD`, `DD/MM/YYYY`, or `DD-MM-YYYY`
- **Transaction type:** `Income` or `Expense`, case-insensitive; `i`/`e` also work
- **Category/Subcategory:** should reference categories that exist in the category catalog (manageable in settings)

Exports additionally include the recurring columns (`is_recurring, recurrence_frequency, recurrence_end_date, is_generated_from_recurring`). These are optional on import and default to a non-recurring transaction; generated recurring rows in a file are ignored on import and re-derived from their source transaction instead.

Importing merges into the database and skips exact duplicates, so re-importing the same file is safe. Exporting writes the full set you see in the app, including generated recurring occurrences.
