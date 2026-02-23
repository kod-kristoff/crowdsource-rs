-- Create User Table
CREATE TABLE users(
id uuid NOT NULL,
PRIMARY KEY (id),
email TEXT NOT NULL UNIQUE,
username TEXT NOT NULL UNIQUE,
created_at timestamptz NOT NULL
);
