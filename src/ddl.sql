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
