# Budget Tracker TUI

![Budget Tracker Logo](budget_tracker_icon.png)

A simple, fast, and efficient Terminal User Interface (TUI) application for tracking your personal budget, built with Rust and Ratatui.

## 📚 Table of Contents

- [✨ Features](#-features)
- [🚀 Getting Started](#-getting-started)
- [⚙️ Data Storage](#️-data-storage)
- [References](#references)

<!-- Placeholder for Screenshot -->

_(Screenshot coming soon!)_

## ✨ Features

- **Terminal-Based Interface:** Manage your finances directly from your terminal using an intuitive TUI.
- **Transaction Management:** Add, view, edit, and delete income and expenses.
- **Categorization:** Assign categories to your transactions for better analysis.
- **Data Persistence:** Your financial data is saved locally in a standard location for your OS.
- **Cross-Platform:** Runs on Windows, macOS, and Linux.
- **Built with Rust:** Leveraging Rust's performance and safety.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (includes `cargo`)

### Installation & Running

There are a few ways to run the application:

1.  **Build and Run Manually:**

    ```bash
    # Clone the repository
    git clone https://github.com/Feromond/budget_tracker_tui
    cd budget_tracker_tui

    # Build the project (use --release for optimized build)
    cargo build --release

    # Run the executable
    ./target/release/Budget_Tracker
    ```

2.  **Install Globally with Cargo (Recommended for Easy Access):**

    ```bash
    # Navigate to the project directory
    cd budget_tracker_tui

    # Install the binary to Cargo's bin directory
    cargo install --path .
    ```

    After installation, the `Budget_Tracker` command should be available in your terminal directly.

    _Optional Tip:_ For even quicker access, you can set up a shell alias:

    ```bash
    # Example for bash/zsh (add to your .bashrc or .zshrc)
    alias bt='Budget_Tracker'
    ```

    Then, you can just type `bt` to launch the app.

## ⚙️ Data Storage

The application stores your transaction data in a `transactions.csv` file within a dedicated application data directory. The exact location depends on your operating system:

- **Linux:** `$XDG_DATA_HOME/BudgetTracker/transactions.csv` (typically `~/.local/share/BudgetTracker/transactions.csv`)
- **macOS:** `~/Library/Application Support/BudgetTracker/transactions.csv`
- **Windows:** `%APPDATA%\BudgetTracker\transactions.csv` (e.g., `C:\Users\<YourUsername>\AppData\Roaming\BudgetTracker\transactions.csv`)

The necessary directories are created automatically if they don't exist.

## References

**[Ratatui](https://ratatui.rs)**
