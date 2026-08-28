use anyhow::Result;
use rusqlite::{Connection, params};
use std::fmt::Display;
use std::path::PathBuf;
use thousands::Separable;

pub mod cli;
pub mod config;

#[derive(Debug)]
pub struct Company {
    id: u8,
    name: String,
}

impl Display for Company {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Company {
    fn new<S: Into<String>>(id: u8, name: S) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub fn get_name(self: &Self) -> &str {
        &self.name
    }

    pub fn get_id(self: &Self) -> u8 {
        self.id
    }
}

#[derive(Debug)]
pub struct Ticket {
    id: u16,
    company: Company,
    amount: u16,
}

impl Ticket {
    pub fn new(id: u16, company: Company, amount: u16) -> Self {
        Self {
            id,
            company,
            amount,
        }
    }

    pub fn fmt_line(self: &Self) -> String {
        format!(
            "[{:>3}] {} ${}",
            self.id,
            &self.company.name,
            self.amount.separate_with_dots()
        )
    }

    pub fn fmt_block(self: &Self) -> String {
        let mut buf = String::new();

        buf.push_str("===== TICKET =====\n");
        buf.push_str(&format!("ID      : {}\n", self.id));
        buf.push_str(&format!("Amount  : {}\n", self.amount.separate_with_dots()));
        buf.push_str(&format!("Company : {}", self.company.name));

        buf
    }
}

const INITIALIZE_SQL_QUERY: &str = include_str!("../ddl.sql");

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Self> {
        let existed = path.exists();

        let s = Self {
            conn: Connection::open(&path)?,
        };

        if !existed {
            s.initialize()?;
        }

        Ok(s)
    }

    fn initialize(self: &Self) -> Result<()> {
        self.conn.execute_batch(INITIALIZE_SQL_QUERY)?;
        Ok(())
    }

    /// Add a ticket to the database and return it.
    pub fn add_ticket(self: &Self, amount: u16, company: Company) -> Result<Ticket> {
        let id = self.conn.query_row(
            "INSERT INTO tickets (amount, id_company) VALUES (?1, ?2) RETURNING id",
            params![amount, company.id],
            |row| row.get(0),
        )?;

        let ticket = Ticket::new(id, company, amount);
        Ok(ticket)
    }

    /// Count all the tickets in the database.
    pub fn count_tickets(self: &Self) -> Result<u16> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM tickets")?;
        let count: u16 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_tickets(self: &Self) -> Result<Vec<Ticket>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.amount, c.id, c.name FROM tickets AS t
            INNER JOIN companies AS c ON t.id_company = c.id",
        )?;

        let mut rows = stmt.query([])?;

        let mut tickets: Vec<Ticket> = Vec::new();

        while let Some(row) = rows.next()? {
            let ticket_id: u16 = row.get(0)?;
            let ticket_amount: u16 = row.get(1)?;
            let company_id: u8 = row.get(2)?;
            let company_name: String = row.get(3)?;

            let company = Company::new(company_id, company_name);
            let ticket = Ticket::new(ticket_id, company, ticket_amount);

            tickets.push(ticket);
        }

        Ok(tickets)
    }

    pub fn get_total_amount(self: &Self) -> Result<u32> {
        let mut stmt = self.conn.prepare("SELECT SUM(amount) FROM tickets")?;
        let total: u32 = stmt.query_row([], |row| row.get(0))?;
        Ok(total)
    }

    pub fn get_companies(self: &Self) -> Result<Vec<Company>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM companies")?;
        let mut rows = stmt.query([])?;
        let mut companies: Vec<Company> = Vec::new();

        while let Some(row) = rows.next()? {
            let id: u8 = row.get(0)?;
            let name: String = row.get(1)?;

            companies.push(Company::new(id, name));
        }

        Ok(companies)
    }

    pub fn add_company<S: Into<String>>(self: &Self, name: S) -> Result<Company> {
        let name = name.into();

        let id = self.conn.query_row(
            "INSERT INTO companies (name) VALUES (?1) RETURNING id",
            params![&name],
            |row| row.get(0),
        )?;

        Ok(Company::new(id, name))
    }

    pub fn get_company_by_name<S: Into<String>>(self: &Self, name: S) -> Result<Option<Company>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM companies WHERE name = ?1")?;
        let mut query = stmt.query(params![name.into()])?;

        match query.next()? {
            Some(row) => {
                let id: u8 = row.get(0)?;
                let name: String = row.get(1)?;

                Ok(Some(Company::new(id, name)))
            }
            None => Ok(None),
        }
    }

    pub fn to_csv(self: &Self) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        wtr.write_record(["id", "monto", "empresa"])?;

        let tickets = self.get_tickets()?;

        for ticket in tickets {
            let id = ticket.id.to_string();
            let amount = ticket.amount.to_string();
            let company_name = ticket.company.name.clone();

            wtr.write_record(&[id, amount, company_name])?;
        }

        let buf = wtr.into_inner()?;
        Ok(String::from_utf8(buf)?)
    }
}
