SELECT 'CREATE DATABASE borges' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'borges');

DROP TABLE book CASCADE;
DROP TABLE note;

CREATE TABLE IF NOT EXISTS book(
    id SERIAL PRIMARY KEY NOT NULL,
    title VARCHAR(100) NOT NULL,
    author VARCHAR(100) NOT NULL,
    image_url VARCHAR(100),
    year INTEGER NOT NULL,
    pages INTEGER NOT NULL,
    UNIQUE(title, author)
);

CREATE TABLE IF NOT EXISTS note(
    id SERIAL PRIMARY KEY NOT NULL,
    book_id INTEGER NOT NULL,
    note TEXT NOT NULL,
    page INTEGER,
    CONSTRAINT fk_book_id
    FOREIGN KEY (book_id)
    REFERENCES book(id)
    ON DELETE NO ACTION
);

INSERT INTO book (title, author, image_url, year, pages) VALUES ('Collected Fictions', 'Jorge Luis Borges', null, 1998, 565);
INSERT INTO book (title, author, image_url, year, pages) VALUES ('Gravity''s Rainbow', 'Thomas Pynchon', null, 1973, 776);
INSERT INTO book (title, author, image_url, year, pages) VALUES ('White Teeth', 'Zadie Smith', null, 2001, 464);
INSERT INTO book (title, author, image_url, year, pages) VALUES ('Blood Meridian', 'Cormac McCarthy', null, 1985, 351);

INSERT INTO note (book_id, note, page) VALUES (1, 'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.', 100);
INSERT INTO note (book_id, note, page) VALUES (2, 'Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.', 10);
INSERT INTO note (book_id, note, page) VALUES (2, 'Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.', 420);
