mod core;

use anyhow::{Context, Result, bail};
use clap::Parser;
use core::{cli::*, *};
use directories::ProjectDirs;
use std::fs;
use thousands::Separable;

fn main() -> Result<()> {
    let args = Cli::parse();

    let dirs = ProjectDirs::from("com", "Nicolas Miranda", "tickets")
        .context("Unable to get SO directories.")?;

    let config_dir = dirs.config_dir();
    let data_dir = dirs.data_dir();

    if !config_dir.try_exists()? {
        fs::create_dir(config_dir)?;
    }

    let conf = config::read_config(config_dir.join("config.toml"))
        .context("Unable to read config file.")?;

    if let None = &args.command {
        bail!("Use a command first.")
    }

    let path = match args.database {
        Some(path) => {
            if path.try_exists()? {
                path
            } else {
                bail!("Path doesn't exists.")
            }
        }
        None => {
            if !data_dir.try_exists()? {
                fs::create_dir(data_dir)?;
            }

            data_dir.join("tickets.db")
        }
    };

    let db = Database::new(path)?;

    match args.command.unwrap() {
        Commands::Add {
            amount,
            company_name,
        } => {
            let company = match db.get_company_by_name(company_name)? {
                Some(company) => company,
                None => bail!("Error: That company is not in the database."),
            };

            let ticket = db.add_ticket(amount, company)?;

            println!("Added ticket:\n{}", ticket.fmt_block());
        }

        Commands::List => {
            let tickets = db.get_tickets()?;

            for ticket in tickets {
                println!("{}", ticket.fmt_line());
            }
        }

        Commands::Summary => {
            let count = db.count_tickets()?;
            let total = db.get_total_amount()?;

            println!("======= SUMMARY =======");
            println!("Ticket count : {}", count.separate_with_dots());
            println!("Total amount : ${}", total.separate_with_dots());

            let goal = conf.goal;

            if goal != 0 {
                println!("Goal         : ${}", goal.separate_with_dots());
                let difference = goal - total;
                println!("Difference   : ${}", difference.separate_with_dots());
                let percent: f64 = (total as f64 / goal as f64) * 100f64;
                println!("Percentage   : {:>3.2}%", percent);
            }
        }

        Commands::Companies(subcmd) => match subcmd.subcmd {
            CompaniesCommands::Add { name } => {
                let company = db.add_company(name)?;
                println!(
                    "Added company \"{}\" with ID #{}.",
                    company.get_name(),
                    company.get_id()
                );
            }
            CompaniesCommands::List => {
                let companies = db.get_companies()?;

                if companies.len() == 0 {
                    bail!("There are no companies in the database.");
                }

                println!("===== COMPANIES =====");
                for company in companies {
                    println!("[{:>2}] {}", company.get_id(), company.get_name());
                }
            }
        },

        Commands::Csv { path } => {
            let c = db.to_csv()?;

            match path {
                Some(dst) => fs::write(dst, c)?,
                None => println!("{}", c),
            }
        }
    }

    Ok(())
}
