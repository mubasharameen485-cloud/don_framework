-- Add migration script here
CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    age INT NOT NULL,
    salary INT NOT NULL,
    department VARCHAR(100) NOT NULL
);