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
#[command(name = "boletos", styles = STYLES)]
pub struct Cli {
    /// Specify the database's path.
    #[arg(long)]
    pub path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a ticket to the database
    Add { amount: u16, company_name: String },

    /// List tickets in the database
    List,

    /// Summary all the tickets
    Summary,

    /// Manage companies in the database
    Companies(CompaniesArgs),
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
