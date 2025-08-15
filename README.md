# Budget Tracker TUI

<p align="center">
  <img src="budget_tracker_icon.png" alt="Budget Tracker Logo" width="150"/>
</p>

A fast, modern, and efficient Terminal User Interface (TUI) application for tracking your personal budget, built with Rust and Ratatui.

## 📚 Table of Contents

- [🖼️ Screenshots](#️-screenshots)
- [✨ Features](#-features)
- [🚀 Getting Started](#-getting-started)
- [⚡ Quick Start](#-quick-start)
- [⚙️ Settings & Configuration](#️-settings--configuration)
- [📁 Data & CSV Format](#-data--csv-format)
- [References](#references)

## 🖼️ Screenshots

<p align="center">
  <img width="966" alt="main_view_tui" src="https://github.com/user-attachments/assets/26d88b97-631f-433c-95bb-4647c5bf87b9" />
  <br><i>Main transaction view with summary bar and help</i><br><br>
  <img width="966" alt="Cate Summary" src="https://github.com/user-attachments/assets/bea59559-8b34-46ac-87d8-f17e586b3cbe" />
  <br><i>Category summary with expandable/collapsible categories</i><br>
  <img width="966" alt="Main Daily Chart" src="https://github.com/user-attachments/assets/914448f2-0a10-425a-9cdc-9c388231e9c5" />
  <br><i>Monthly summary with interactive chart and budget line</i><br><br>
  
  ### SEE MORE SCREENSHOTS HERE <>
  <details>
  <summary>Click to see all monthly summary screenshots</summary>
  <p align="center">
    <img width="966" alt="Multi-month Line Chart" src="https://github.com/user-attachments/assets/d1f263b0-6a0a-4d4e-b528-57fe508a037a" />
    <br><i>Multi-Month Line chart</i><br><br>
    <img width="966" alt="Cumulative Chart" src="https://github.com/user-attachments/assets/5b01519d-cfc4-4e32-b1e6-fffefc7246e2" />
    <br><i>Cumulative chart with budget line</i><br><br>
    <img width="966" alt="Cumulative+multi_chart" src="https://github.com/user-attachments/assets/f795b663-d4dc-494e-bcb4-db1b4fefabc9" />
    <br><i>Cumulative and multi month chart</i><br><br>
  </p>
</details>
</p>

## ✨ Features

- **Intuitive Terminal UI:** Manage your finances directly from your terminal with a clean, responsive interface (TUI).
- **Transaction Management:** Add, view, edit, and delete income and expenses.
- **Recurring Transaction:** Setup transactions that automatically recur at select frequencies.
- **Advanced Filtering:** Filter transactions by date, description, category, type, and amount (including advanced multi-field filters).
- **Categorization:** Built-in, hierarchical categories and subcategories for all transactions.
- **Summaries & Charts:** Visualize your spending/income by month and by category, with interactive charts and tables.
- **Budget Tracking:** Set a monthly target budget and see your progress (including a budget line in summary charts).
- **Data Persistence:** All data is stored locally in a configurable CSV file. Settings are saved in a config file.
- **Cross-Platform:** Runs on Windows, macOS, and Linux.
- **Keyboard-Driven:** Fully operable with keyboard shortcuts for every action and mode.
- **Robust CSV Support:** Flexible date parsing, easy import/export, and Excel compatibility.
- **Built with Rust:** Safety, speed, and reliability.

## 🚀 Getting Started

### Windows Installer (Recommended for Windows Users)

If you are on **Windows**, you can download and run the latest installer for a quick and easy setup. This is the simplest way to get started on Windows—no Rust or Cargo required!

- [Download the latest Windows installer from the Releases page](https://github.com/Feromond/budget_tracker_tui/releases)

### Prerequisites (for manual/cargo install)

- [Rust](https://www.rust-lang.org/tools/install) (includes `cargo`)

### Installation & Running

**Build and Run Manually:**

```bash
# Clone the repository
git clone https://github.com/Feromond/budget_tracker_tui
cd budget_tracker_tui

# Build the project (use --release for optimized build)
cargo build --release

# Run the executable
./target/release/Budget_Tracker
```

**Install Globally with Cargo (Recommended for Linux/macOS):**

```bash
# Navigate to the project directory
cd budget_tracker_tui

# Install the binary to Cargo's bin directory
cargo install --path .
```

After installation, the `Budget_Tracker` command should be available in your terminal directly.

_Optional Tip:_ For even quicker access, set up a shell alias:

```bash
# Example for bash/zsh (add to your .bashrc or .zshrc)
alias bt='Budget_Tracker'
```

Then, you can just type `bt` to launch the app.

## ⚡ Quick Start

1. **Launch the app:** `Budget_Tracker` (or `bt` if you set up the alias)
2. **Add a transaction:** Press `a`, fill in the fields, and press `Enter` to save.
3. **Navigate:** Use `↑`/`↓` to move between transactions, `PageUp`/`PageDown` to jump by pages, `Ctrl+↑`/`Ctrl+↓` to jump to first/last transaction.
4. **Sort transactions:** Press `1-6` to sort by Date, Description, Category, Subcategory, Type, or Amount respectively.
5. **Manage transactions:** Press `e` to edit, `d` to delete, `f` to filter, `r` to manage recurring transactions.
6. **View summaries:** Press `s` for monthly summary, `c` for category summary (use `PageUp`/`PageDown` to jump between months in category view).
7. **Change settings:** Press `o` to open settings (change data file path, set target budget).
8. **Quit:** Press `q` or `Esc`.

## ⚙️ Settings & Configuration

- **Data File Path:**
  - The path to your `transactions.csv` file is configurable in-app (press `o` for settings).
  - Default locations: - **Linux:** `$XDG_DATA_HOME/BudgetTracker/transactions.csv` (usually `~/.local/share/BudgetTracker/transactions.csv`) - **macOS:** `~/Library/Application Support/BudgetTracker/transactions.csv` - **Windows:** `%APPDATA%\BudgetTracker\transactions.csv` (e.g.,
    `C:\Users\<YourUsername>\AppData\Roaming\BudgetTracker\transactions.csv`)
- **Target Budget:**
  - Set a monthly target budget in settings. This will show a budget line in summary charts.
- **Config File:**
  - The application's settings are saved in a `config.json` file, which is stored in your OS's **config directory**:
    - **Linux:** `~/.config/BudgetTracker/config.json`
    - **macOS:** `~/Library/Application Support/BudgetTracker/config.json`
    - **Windows:** `C:\Users\<YourUsername>\AppData\Roaming\BudgetTracker\config.json`
  - This is separate from the data file location, which is in your OS's data directory (see above).

## 📁 Data & CSV Format

- **CSV Columns:** `date, description, amount, transaction_type, category, subcategory`
- **Date Format:** Flexible! Accepts `YYYY-MM-DD`, `YYYY/MM/DD`, `DD/MM/YYYY`, or `DD-MM-YYYY`.
- **Transaction Type:** `Income` or `Expense` (case-insensitive, also accepts `i`/`e`)
- **Category/Subcategory:** Must match the built-in set of categories and subcategories provided by the application. Custom or arbitrary categories are not currently supported.
- **Import/Export:** You can edit the CSV in Excel/LibreOffice or import from other tools (just match the columns and use valid categories).
- **Data Safety:** The app will not overwrite your CSV unless you save a transaction, close the program, or change settings.

## References

**[Ratatui](https://ratatui.rs)**
**[Rust](https://www.rust-lang.org/)**
