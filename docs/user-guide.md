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

The advanced filter (`Ctrl+F`) filters on multiple fields at once: date range, description, category, type, and amount. `Tab` or `↑`/`↓` move between fields, `Enter` applies, `Esc` cancels, and `Ctrl+R` resets everything.

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

## Summary views

**Monthly summary (`s`)** shows income, expenses, and net per month with an interactive chart. `↑`/`↓` move between months, `←`/`→` (or `[`/`]`) move between years. `m` toggles a multi-month line chart, and `c` toggles cumulative mode, which also draws the target budget line from settings.

**Category summary (`c`)** breaks down each month by category. `Enter` expands or collapses a month, `PageUp`/`PageDown` move between months, `←`/`→` between years.

**Budget view (`b`)** compares spending against your monthly target and any per-category budgets. `↑`/`↓` move between categories, `←`/`→` between months, `Shift+←`/`Shift+→` between years. Press `c` to open the [category catalog](#the-category-catalog) and adjust per-category budgets without leaving the view.

## The category catalog

The catalog holds your categories and subcategories. Open it from Settings (*Manage Categories*) or with `c` from the budget view. `q`/`Esc` returns to whichever view you came from.

- `↑`/`↓` move between entries, `PageUp`/`PageDown` jump by page, `Ctrl+↑`/`Ctrl+↓` jump to the first/last entry
- `f` filters the catalog as you type; `Enter` keeps the filter applied, `Esc` or `Ctrl+R` clears it
- `a` adds a category, `e` or `Enter` edits the selected one, `d` deletes it
- Expense categories can optionally hold a per-category target budget, used by the budget view

## Settings

Press `o` to open settings. The menu is grouped into sections:

**Data Management**

- *Database Path*: where the SQLite database lives (see [Data storage](#data-storage) below).
- *Manage Categories*: opens the [category catalog](#the-category-catalog).
- *Import Transactions (CSV)*: merges a CSV file into your database; new rows are added, exact duplicates are skipped.
- *Export Transactions (CSV)*: writes all transactions to a CSV file for backup or use elsewhere.

**Monthly Summary View**

- *Target Budget*: your monthly spending goal, drawn as a line in the monthly summary's cumulative mode and used by the budget view.

**Transaction View**

- *Hourly Rate*: optionally enter your hourly earning rate; a *Show Costs in Hours* toggle then appears that displays amounts as hours worked.

**Input Preferences**

- *Fuzzy Search Categories*: enables fuzzy matching when picking categories and subcategories.

**General Preferences**

- *Hide Help Bar*: hides the bottom help bar if you want the extra screen space (`Ctrl+H` still works).

## Data storage

Transactions and categories are stored together in a local SQLite database (`budget.db`). On first run with a new database, it's seeded with the default category catalog. Default locations:

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
