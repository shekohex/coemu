-- Add migration script here
-- shekohex password is 123456
INSERT INTO accounts (username, password)
VALUES (
    'shekohex',
    -- username
    '$2b$12$yrHThFrB2K2fozb4cchAke6oov7HGnGVQe0W0TJ7mdyT5i4rsd9gG'
  ) ON CONFLICT(username) DO NOTHING;
-- test1 password is 123456
INSERT INTO accounts (username, password)
VALUES (
    'test1',
    -- username
    '$2b$12$gdikPiJmesxrkUFyKQRh1ushZFv.urQhd1st8H9R5OxyQe4nzK5cq'
  ) ON CONFLICT(username) DO NOTHING;
