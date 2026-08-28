use anyhow::Result;
use rusqlite::{Connection, OptionalExtension, params};
use std::fmt::Display;
use std::path::Path;
use tabular::{Row, Table};
use thousands::Separable;

pub mod cli;
pub mod config;

#[derive(Debug, Clone)]
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
        buf.push_str(&format!(
            "Amount  : ${}\n",
            self.amount.separate_with_dots()
        ));
        buf.push_str(&format!("Company : {}", self.company.name));

        buf
    }
}

/// SQL script to initialize the database
const INITIALIZE_SQL_QUERY: &str = include_str!("../ddl.sql");

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let s = Self {
            conn: Connection::open(path)?,
        };

        s.conn.execute_batch(INITIALIZE_SQL_QUERY)?;

        Ok(s)
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

    pub fn get_ticket(self: &Self, id: u16) -> Result<Option<Ticket>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.amount, c.id, c.name FROM tickets AS t
            INNER JOIN companies AS c ON c.id = t.id_company WHERE t.id = ?1",
        )?;

        let result = stmt
            .query_row(params![id], |row| {
                let amount: u16 = row.get(0)?;
                let company_id: u8 = row.get(1)?;
                let company_name: String = row.get(2)?;

                let company = Company::new(company_id, company_name);
                Ok(Ticket::new(id, company, amount))
            })
            .optional()?;

        Ok(result)
    }

    pub fn remove_ticket(self: &Self, id: u16) -> Result<bool> {
        let mut stmt = self.conn.prepare("DELETE FROM tickets WHERE id = ?1")?;
        let rows = stmt.execute(params![id])?;
        Ok(rows >= 1)
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

    pub fn get_company(self: &Self, id: u8) -> Result<Option<Company>> {
        Ok(self
            .conn
            .query_row(
                "SELECT name FROM companies WHERE id = ?1",
                params![id],
                |row| {
                    let name: String = row.get(0)?;
                    Ok(Company::new(id, name))
                },
            )
            .optional()?)
    }

    pub fn remove_company(self: &Self, id: u8) -> Result<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM companies WHERE id = ?1", params![id])?;

        Ok(rows >= 1)
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

    pub fn pivot_companies(self: &Self) -> Result<String> {
        let mut table = Table::new("{:<}  {:>}  {:^}");

        // TODO: Implement colors
        table.add_row(
            Row::new()
                .with_cell("COMPANY")
                .with_cell("AMOUNT")
                .with_cell("COUNT"),
        );

        let mut stmt = self.conn.prepare(
            "
            SELECT c.name, SUM(t.amount), COUNT(t.id) FROM tickets AS t
                INNER JOIN companies AS c ON c.id = t.id_company
                GROUP BY c.name
                ORDER BY sum(t.amount) DESC;
            ",
        )?;

        let mut rows = stmt.query(params![])?;

        while let Some(row) = rows.next()? {
            let company_name: String = row.get(0)?;
            let amount: isize = row.get(1)?;
            let count: isize = row.get(2)?;

            table.add_row(
                Row::new()
                    .with_cell(company_name)
                    .with_cell(amount.separate_with_dots())
                    .with_cell(count),
            );
        }

        Ok(table.to_string())
    }

    pub fn detail_tickets(self: &Self) -> Result<String> {
        let mut table = Table::new("{:>}  {:<}  {:^}");

        // TODO: Implement colors
        table.add_row(
            Row::new()
                .with_cell("AMOUNT")
                .with_cell("COMPANY")
                .with_cell("COUNT"),
        );

        let mut stmt = self.conn.prepare(
            "
            SELECT t.amount, c.name, COUNT(t.id) FROM tickets AS t
                INNER JOIN companies AS c ON c.id = t.id_company
                GROUP BY t.amount, c.name ORDER BY COUNT(t.id) DESC
            ",
        )?;

        let mut rows = stmt.query(params![])?;

        while let Some(row) = rows.next()? {
            let amount: isize = row.get(0)?;
            let name: String = row.get(1)?;
            let count: isize = row.get(2)?;

            table.add_row(
                Row::new()
                    .with_cell(amount.separate_with_dots())
                    .with_cell(name)
                    .with_cell(count),
            );
        }

        Ok(table.to_string())
    }
}
