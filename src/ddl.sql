-- user_version 0 was the one without version
PRAGMA user_version = 1;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS companies (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS tickets (
    id INTEGER PRIMARY KEY,
    id_company INTEGER,
    amount INTEGER NOT NULL,

    CONSTRAINT fk_ticket__companies FOREIGN KEY (id_company) REFERENCES companies(id)
);

CREATE VIEW IF NOT EXISTS companies_summary AS
    SELECT c.name AS name, SUM(t.amount) AS amount, COUNT(t.id) AS count
    FROM tickets AS t
    JOIN companies AS c ON t.id_company = c.id
    GROUP BY c.name
    ORDER BY SUM(t.amount) DESC;

CREATE VIEW IF NOT EXISTS tickets_summary AS
    SELECT t.amount AS amount, c.name AS company, COUNT(t.id) AS count
    FROM tickets AS t
    JOIN companies AS c ON t.id_company = c.id
    GROUP BY t.amount, c.name
    ORDER BY COUNT(t.id) DESC;
