mod core;

use anyhow::{Result, bail};
use clap::Parser;
use core::{cli::*, *};
use std::fs;
use std::path::PathBuf;
use thousands::Separable;

fn main() -> Result<()> {
    let args = Cli::parse();

    if let None = &args.command {
        bail!("Use a command first.")
    }

    let path = match args.database {
        Some(path) => path,
        None => PathBuf::from("tickets.db"),
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

            println!("Added ticket:\n{}", ticket);
        }

        Commands::List => {
            let tickets = db.get_tickets()?;

            for ticket in tickets {
                println!("{}", ticket);
            }
        }

        Commands::Summary => {
            let count = db.count_tickets()?;
            let total = db.get_total_amount()?;

            println!("===== SUMMARY =====");
            println!("Ticket count : {}", count.separate_with_dots());
            println!("Total amount : ${}", total.separate_with_dots());
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
