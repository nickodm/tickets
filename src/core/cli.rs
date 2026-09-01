use clap::{
    Args, Parser, Subcommand,
    builder::styling::{AnsiColor, Effects, Styles},
};
use std::path::PathBuf;

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default());

#[derive(Parser)]
#[command(name = "tickets", version = "0.2.0", styles = STYLES)]
pub struct Cli {
    /// Specify the database's path.
    #[arg(long)]
    pub database: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a ticket to the database
    Add {
        /// Ticket's amount.
        #[arg(short, long)]
        amount: u16,
        /// Ticket's company's name.
        #[arg(short, long)]
        company_name: String,
        /// How many tickets add with this fields.
        #[arg(short, long, default_value_t = 1)]
        times: usize,
    },

    /// Remove a ticket from the database.
    Remove {
        /// ID of the ticket to remove.
        id: u16,

        /// When provided, remove all the tickets from `id` to `to`.
        #[arg(long)]
        to: Option<u16>,
    },

    /// Show a ticket in the database.
    Show {
        /// ID of the ticket to show.
        id: u16,
    },

    /// List tickets in the database
    List,

    /// Summary all the tickets
    Summary {
        /// Whether to show detailed information about tickets.
        #[arg(short, long, default_value_t = false)]
        detailed: bool,
    },

    /// Manage companies in the database
    Companies(CompaniesArgs),

    /// Export the database as CSV
    Csv { path: Option<PathBuf> },

    /// Drop the database. CANNOT BE UNDONE!
    Drop,

    /// Create a database's backup.
    Backup {
        /// Path to write the output file
        output: PathBuf,
    },

    /// Restore a database's backup.
    Restore {
        /// Path of the backup file.
        input: PathBuf,
    },
}

#[derive(Args)]
pub struct CompaniesArgs {
    #[command(subcommand)]
    pub subcmd: CompaniesCommands,
}

#[derive(Subcommand, Clone)]
pub enum CompaniesCommands {
    /// Add a company to the database
    Add { name: String },
    /// List all the companies in the database
    List,

    /// Remove a company from the database
    Remove {
        /// The ID of the company to remove
        id: u8,
    },
}
