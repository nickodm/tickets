<!-- Written by Gemini -->

# Tickets

A fast, lightweight CLI ticket and expense/revenue tracker written in Rust, powered by SQLite.

---

## Features

- **Company & Ticket Management**: Associate tickets with specific companies with relational integrity.
- **Batch Ticket Insertion**: Quickly add multiple identical tickets in a single command (`--times`).
- **Flexible Ticket Removal**: Remove a single ticket or a range of IDs (`--to`).
- **Visual Summaries & Analytics**: View total counts, aggregated sums, breakdown by company, and itemized frequency tables using colored ASCII tables.
- **Goal Tracking**: Set a target financial goal in `config.toml` to track differences and completion percentage.
- **Database Backup**: Create quick backups of your SQLite database with override protection.
- **CSV Export**: Export all database records to CSV via stdout or directly to a file.
- **SQLite Storage**: Zero-configuration, local database created automatically under standard OS data directories or a custom path.

---

## Installation

### Prerequisites

- [Rust & Cargo](https://www.rust-lang.org/tools/install) (2024 edition or newer)
- SQLite development libraries (if linking dynamically)

### Build from Source

```bash
git clone https://github.com/nickodm/tickets.git
cd tickets
cargo build --release
```

The compiled binary will be available at `target/release/tickets`.

You can also install it to your Cargo bin path:

```bash
cargo install --path .
```

---

## Usage

```text
Usage: tickets [OPTIONS] [COMMAND]

Commands:
  add        Add a ticket to the database
  remove     Remove a ticket from the database
  show       Show a ticket in the database
  list       List tickets in the database
  summary    Summary all the tickets
  companies  Manage companies in the database
  csv        Export the database as CSV
  drop       Drop the database. CANNOT BE UNDONE!
  backup     Create a backup of the database
  help       Print this message or the help of the given subcommand(s)

Options:
      --database <DATABASE>  Specify the database's path
  -h, --help                 Print help
  -V, --version              Print version
```

---

## Command Reference & Examples

### 1. Companies Management

Before adding tickets, register companies into the database:

```bash
# Add companies
tickets companies add "Acme Corp"
tickets companies add "Stark Industries"

# List registered companies
tickets companies list

# Remove a company by ID
tickets companies remove 2
```

### 2. Adding Tickets

Add one or multiple tickets associated with a company:

```bash
# Add a single ticket
tickets add -a 5000 -c "Acme Corp"

# Add multiple tickets at once (e.g. 5 tickets of $2,500)
tickets add -a 2500 -c "Acme Corp" -t 5
```

### 3. Listing & Inspecting Tickets

```bash
# List all tickets
tickets list

# Show detailed information for a specific ticket ID
tickets show 1
```

### 4. Removing Tickets

```bash
# Remove a single ticket by ID
tickets remove 3

# Remove a range of tickets from ID 5 to 10
tickets remove 5 --to 10
```

### 5. Summary & Goal Tracking

Display general metrics, company breakdown pivot tables, and goal progress:

```bash
# Basic summary
tickets summary

# Detailed summary (includes ticket frequency grouped by amount and company)
tickets summary --detailed
# or
tickets summary -d
```

### 6. CSV Export

Export data to standard output or save to a file:

```bash
# Output CSV to stdout
tickets csv

# Save directly to a file
tickets csv output.csv
```

### 7. Custom Database Path

Use a custom SQLite database file:

```bash
tickets --database /path/to/custom.db list
```

### 8. Backing Up the Database

Create a backup copy of the current database file:

```bash
tickets backup backup_tickets.db
```

> If the destination file already exists, you will be prompted for confirmation before overwriting.

### 9. Dropping the Database

Delete the database file with confirmation prompt:

```bash
tickets drop
```

---

## Configuration

`tickets` automatically generates a configuration file on its first run in your operating system's standard configuration directory:

- **Linux**: `~/.config/tickets/config.toml`
- **macOS**: `~/Library/Application Support/com.Nicolas-Miranda.tickets/config.toml`
- **Windows**: `C:\Users\<User>\AppData\Roaming\Nicolas Miranda\tickets\config.toml`

### Example `config.toml`

```toml
# Set a financial goal to track against the total sum in `tickets summary`
goal = 100000
```

When `goal` is set to a non-zero number, `tickets summary` will display:
- **Goal**: Total target amount.
- **Difference**: Remaining or exceeded balance.
- **Percentage**: Progress toward the goal.

---

## Database Storage

Unless overridden by `--database <PATH>`, the SQLite database file (`tickets.db`) is stored in standard OS data directories:

- **Linux**: `~/.local/share/tickets/tickets.db`
- **macOS**: `~/Library/Application Support/com.Nicolas-Miranda.tickets/tickets.db`
- **Windows**: `C:\Users\<User>\AppData\Local\Nicolas Miranda\tickets\tickets.db`

---

## License

This project is open source.
