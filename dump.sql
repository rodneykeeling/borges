SELECT 'CREATE DATABASE borges' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'borges');

DROP TABLE book;

CREATE TABLE IF NOT EXISTS book(
    id SERIAL PRIMARY KEY NOT NULL,
    title VARCHAR(100) NOT NULL,
    author VARCHAR(100) NOT NULL,
    image_url VARCHAR(100),
    year INTEGER NOT NULL,
    pages INTEGER NOT NULL,
    UNIQUE(title, author)
);
