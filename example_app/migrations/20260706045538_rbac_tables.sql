-- Add migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user', -- NAYA: Role column
    metadata JSONB DEFAULT '{}'
);