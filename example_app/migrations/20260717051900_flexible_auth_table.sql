-- Add migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    school VARCHAR(255) UNIQUE NOT NULL, 
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user',
    metadata JSONB DEFAULT '{}' 
);

CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price INT NOT NULL
);