CREATE TABLE companies (
    id TINYINT UNSIGNED PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL
);

CREATE TABLE tickets (
    id SMALLINT UNSIGNED PRIMARY KEY,
    id_company TINYINT UNSIGNED,
    amount SMALLINT UNSIGNED NOT NULL,

    CONSTRAINT fk_ticket__companies FOREIGN KEY (id_company) REFERENCES companies(id)
);
