CREATE TABLE IF NOT EXISTS accounts (
    account_id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE CHECK (length(username) <= 16),
    password TEXT NOT NULL,
    name TEXT DEFAULT NULL CHECK (length(name) <= 32),
    email TEXT DEFAULT NULL CHECK (length(email) <= 64)
);
