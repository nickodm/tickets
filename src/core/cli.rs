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
#[command(name = "boletos", version = "0.1.0", styles = STYLES)]
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

    /// List tickets in the database
    List,

    /// Summary all the tickets
    Summary,

    /// Manage companies in the database
    Companies(CompaniesArgs),

    /// Export the database as CSV
    Csv { path: Option<PathBuf> },
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
}
