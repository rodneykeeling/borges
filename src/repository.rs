use std::sync::Arc;

use crate::graphql::{Book, BookInput, Note, NoteInput};
use anyhow::Result;
use dotenvy_macro::dotenv;
use sqlx::{pool::PoolOptions, Pool, Postgres};
use tokio::sync::Mutex;

pub type Storage = Arc<Mutex<BookRepository>>;

/// SQL model representing the `book` table.
struct SqlBook {
    id: i32,
    title: String,
    author: String,
    image_url: Option<String>,
    year: i32,
    pages: i32,
}

impl SqlBook {
    /// Serializes a SqlBook database row to a Book domain model.
    fn into_book(self) -> Book {
        Book {
            id: self.id,
            title: self.title,
            author: self.author,
            image_url: self.image_url,
            year: self.year,
            pages: self.pages,
        }
    }
}

/// SQL model representing the `note` table.
struct SqlNote {
    id: i32,
    book_id: i32,
    note: String,
    page: Option<i32>,
}

impl SqlNote {
    /// Serializes a SqlNote database row to a Note domain model.
    fn into_note(self) -> Note {
        Note {
            id: self.id,
            book_id: self.book_id,
            note: self.note,
            page: self.page,
        }
    }
}

pub struct BookRepository {
    pub db: Pool<Postgres>,
}

impl BookRepository {
    pub async fn new() -> Result<Arc<Mutex<Self>>> {
        let db: Pool<Postgres> = PoolOptions::new()
            .max_connections(5)
            .connect(dotenv!("DATABASE_URL"))
            .await?;
        Ok(Arc::new(Mutex::new(Self { db })))
    }

    pub async fn get_book_by_title(&self, title: String) -> Result<Option<Book>> {
        let row = sqlx::query_as!(
            SqlBook,
            "SELECT id, title, author, image_url, year, pages FROM book WHERE title=$1",
            title,
        )
        .fetch_optional(&self.db)
        .await
        .unwrap_or(None);

        if let Some(result) = row {
            return Ok(Some(result.into_book()));
        }
        Ok(None)
    }

    pub async fn get_book_by_id(&self, book_id: i32) -> Result<Option<Book>> {
        let row = sqlx::query_as!(
            SqlBook,
            "SELECT id, title, author, image_url, year, pages FROM book WHERE id=$1",
            book_id,
        )
        .fetch_optional(&self.db)
        .await
        .unwrap_or(None);

        if let Some(result) = row {
            return Ok(Some(result.into_book()));
        }
        Ok(None)
    }

    pub async fn add_book(&mut self, input: BookInput) -> Result<Book> {
        let row = sqlx::query_as!(
            SqlBook,
            "INSERT INTO book(title, author, image_url, year, pages) VALUES ($1, $2, $3, $4, $5) RETURNING id, title, author, image_url, year, pages",
            input.title,
            input.author,
            input.image_url,
            input.year,
            input.pages,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(row.into_book())
    }

    pub async fn get_notes_by_book(&self, book_id: i32) -> Result<Option<Vec<Note>>> {
        let rows = sqlx::query_as!(
            SqlNote,
            "SELECT id, book_id, note, page FROM note WHERE book_id=$1",
            book_id,
        )
        .fetch_all(&self.db)
        .await
        .unwrap_or(Vec::new());

        Ok(Some(rows.into_iter().map(|row| row.into_note()).collect()))
    }

    pub async fn add_note(&mut self, input: NoteInput) -> Result<Note> {
        let row = sqlx::query_as!(
            SqlNote,
            "INSERT INTO note(book_id, note, page) VALUES ($1, $2, $3) RETURNING id, book_id, note, page",
            input.book_id,
            input.note,
            input.page,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(row.into_note())
    }
}
