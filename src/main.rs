mod core;

use anyhow::{Context, Result, bail};
use clap::Parser;
use color_print::cprintln;
use core::{cli::*, *};
use directories::ProjectDirs;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
};
use tempfile::NamedTempFile;
use thousands::Separable;

/// Ask the user for confirmation before doing something.
/// ```rust
/// if confirm("Are you sure you want to do this?")? {
///     // do something good
/// } else {
///     // do something bad
/// }
/// ```
fn confirm<S: std::fmt::Display>(prompt: S) -> Result<bool> {
    let stdin = std::io::stdin();
    let mut answer = String::new();

    print!("{} [y/n]: ", prompt);
    std::io::stdout().flush()?;
    stdin.read_line(&mut answer)?;
    let answer = answer.replace("\n", "").replace("\r", "").to_lowercase();

    match answer.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        a => bail!("Invalid answer: \"{}\", please use [y/n].", a),
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let dirs = ProjectDirs::from("com", "Nicolas Miranda", "tickets")
        .context("Unable to get SO directories.")?;

    let config_dir = dirs.config_dir();
    let data_dir = dirs.data_dir();

    if !config_dir.try_exists()? {
        fs::create_dir_all(config_dir)?;
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
                fs::create_dir_all(data_dir)?;
            }

            data_dir.join("tickets.db")
        }
    };

    let db = Database::new(&path)?;

    match args.command.unwrap() {
        Commands::Add {
            amount,
            company_name,
            times,
        } => {
            let Some(company) = db.get_company_by_name(company_name)? else {
                bail!("That company is not in the database.");
            };

            if times > 1 {
                println!("Added tickets:");
            } else {
                println!("Added ticket:");
            }

            for _ in 0..times {
                let ticket = db.add_ticket(amount, company.clone())?;

                if times > 1 {
                    println!("{}", ticket.fmt_line());
                } else {
                    println!("{}", ticket.fmt_block());
                }
            }
        }

        Commands::Remove { id, to } => {
            let to = to.unwrap_or(id);

            if to < id {
                bail!("`to` must be greater than `id`.");
            }

            let range = to - id + 1;

            if range > 1 {
                println!("Removed tickets:");
            } else {
                println!("Removed ticket:");
            }

            for i in id..=to {
                let ticket = match db.get_ticket(i)? {
                    Some(ticket) => ticket,
                    None => bail!("Ticket with that ID doesn't exists."),
                };

                if !db.remove_ticket(i)? {
                    bail!("Unable to remove that ticket.");
                }

                if range > 1 {
                    println!("{}", ticket.fmt_line());
                } else {
                    println!("{}", ticket.fmt_block());
                }
            }
        }

        Commands::Show { id } => match db.get_ticket(id)? {
            Some(ticket) => println!("{}", ticket.fmt_block()),
            None => bail!("There are no ticket with that ID."),
        },

        Commands::List => {
            let tickets = db.get_tickets()?;

            for ticket in tickets {
                println!("{}", ticket.fmt_line());
            }
        }

        Commands::Summary { detailed } => {
            let count = db.count_tickets()?;

            if count == 0 {
                bail!("Database is empty.");
            }

            let total = db.get_total_amount()?;

            cprintln!("<blue,s>======== GENERAL ========</>");
            cprintln!("<green,s>Ticket count :</> {}", count.separate_with_dots());
            cprintln!("<green,s>Total amount :</> ${}", total.separate_with_dots());

            let goal = conf.goal;

            if goal != 0 {
                cprintln!("<blue,s>========= GOAL ==========</>");
                cprintln!("<green,s>Goal         :</> ${}", goal.separate_with_dots());
                let difference: isize = goal as isize - total as isize;
                cprintln!(
                    "<green,s>Difference   :</> ${:>7}",
                    difference.separate_with_dots()
                );
                let percent: f64 = (total as f64 / goal as f64) * 100f64;
                cprintln!("<green,s>Percentage   :</> {:>3.1}%", percent);
            }

            cprintln!("<blue,s>======= COMPANIES =======</>");
            println!("{}", db.summary_companies()?);

            if detailed {
                cprintln!("<blue,s>======== DETAILS ========</>");
                println!("{}", db.summary_tickets()?);
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

            CompaniesCommands::Remove { id } => {
                let company = match db.get_company(id)? {
                    Some(company) => company,
                    None => bail!("There is no company with that ID."),
                };

                if db.remove_company(id)? {
                    println!("Removed company \"{}\".", company.get_name());
                } else {
                    bail!("Unable to remove company.");
                }
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

        Commands::Drop => {
            if confirm("Are you sure you want to DROP the database?")? {
                drop(db);
                fs::remove_file(&path).context("Unable to drop the database.")?;
                println!("Dropped database.");
            } else {
                println!("Aborted.");
            }
        }

        Commands::Backup { output } => {
            let mut output = output.clone();

            if output.is_dir() {
                output.push("tickets.db.gz");
            } else {
                let has_valid_extension = output
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".db.gz"));

                if !has_valid_extension {
                    output.add_extension("db.gz");
                }
            }

            if output.try_exists()? && !confirm("Output file already exists. override?")? {
                println!("Aborted.");
                return Ok(());
            }

            {
                let file = fs::File::create(&output)?;
                let mut gz = GzEncoder::new(file, Compression::best());
                let raw = fs::read(&path)?;
                gz.write_all(&raw)?;
            }

            println!("Database backed up to '{}'.", output.display());
        }

        Commands::Restore { input } => {
            if !input.try_exists()? {
                bail!("Path doesn't exists.");
            } else if input.is_dir() {
                bail!("Path is not a file.");
            }

            let metadata = fs::metadata(&input)?;
            let mut buffer = Vec::with_capacity(metadata.len() as usize);

            // Check gzip magic number and decompress
            {
                let mut file = fs::File::open(&input)?;
                let mut gz_magic = [0u8; 2];

                file.read(&mut gz_magic)?;

                if gz_magic != [0x1f, 0x8b] {
                    bail!("File is not a valid gzip file.");
                }

                file.seek(SeekFrom::Start(0))?;

                let mut gz = GzDecoder::new(file);
                gz.read_to_end(&mut buffer)?;
            }

            // Check SQLite magic number
            if buffer.len() < 16 || &buffer[..16] != b"SQLite format 3\0" {
                bail!("File is not a valid SQLite database.");
            }

            let mut tmp = NamedTempFile::new()?;
            tmp.write(&buffer)?;
            tmp.seek(SeekFrom::Start(0))?;

            let conn = rusqlite::Connection::open(tmp.path())?;

            let status: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;

            if status != "ok" {
                bail!("Database integrity test failed.");
            }

            drop(conn);
            drop(tmp);

            if path.try_exists()? && !confirm("Database will be replaced by this backup. Proceed?")?
            {
                println!("Aborted");
                return Ok(());
            }

            drop(db);

            let old = path.with_added_extension("old");

            fs::rename(&path, &old)?;
            fs::write(&path, buffer)?;

            println!("Backup successfully restored.");
            println!("Old version was moved to '{}'.", old.display());
        }
    };

    Ok(())
}
