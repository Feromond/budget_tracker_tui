# Budget Tracker TUI

<p align="center">
  <img src="budget_tracker_icon.png" alt="Budget Tracker Logo" width="150"/>
</p>

A simple, fast, and efficient Terminal User Interface (TUI) application for tracking your personal budget, built with Rust and Ratatui.

## 📚 Table of Contents

- [✨ Features](#-features)
- [🚀 Getting Started](#-getting-started)
- [⚙️ Data Storage](#️-data-storage)
- [References](#references)

<!-- Placeholder for Screenshot -->


<img width="928" alt="Screenshot 2025-04-15 at 11 44 08 PM" src="https://github.com/user-attachments/assets/ab47b560-f3ab-4f95-919f-5de58a828d0c" style="border: 2px solid #ccc; margin: 20px;" />
<br><br>
<img width="926" alt="Screenshot 2025-04-15 at 11 44 30 PM" src="https://github.com/user-attachments/assets/71aa516e-4917-41d8-9257-e837491a3bd9" style="border: 2px solid #ccc; margin: 20px;" />
<br><br>
<img width="926" alt="Screenshot 2025-04-15 at 11 44 45 PM" src="https://github.com/user-attachments/assets/c230ee05-b69b-457f-b108-4afe6ee5aaa1" style="border: 2px solid #ccc; margin: 20px;" />

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
