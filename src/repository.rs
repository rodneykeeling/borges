use std::sync::Arc;

use crate::graphql::{Book, BookInput, Note, NoteInput, ReadingStatus};
use anyhow::{anyhow, Result};
use sqlx::{Pool, Postgres};
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
    status: ReadingStatus,
}

impl SqlBook {
    /// Serializes a `SqlBook` database row to a Book domain model.
    fn into_book(self) -> Book {
        Book {
            id: self.id,
            title: self.title,
            author: self.author,
            image_url: self.image_url,
            year: self.year,
            pages: self.pages,
            status: self.status,
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
    /// Serializes a `SqlNote` database row to a Note domain model.
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
    pub fn new(conn: Pool<Postgres>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self { db: conn }))
    }

    pub async fn get_book_by_title(&self, title: String) -> Result<Option<Book>> {
        let row = sqlx::query_as!(
            SqlBook,
            r#"SELECT id, title, author, image_url, year, pages, status AS "status: _" FROM book WHERE title=$1"#,
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
            r#"SELECT id, title, author, image_url, year, pages, status AS "status: _" FROM book WHERE id=$1"#,
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

    pub async fn get_books(&self, status: Option<ReadingStatus>) -> Result<Option<Vec<Book>>> {
        let statuses;
        if let Some(reading_state) = status {
            statuses = vec![reading_state];
        } else {
            statuses = vec![
                ReadingStatus::Unread,
                ReadingStatus::Reading,
                ReadingStatus::Read,
            ];
        }

        let rows = sqlx::query_as!(
            SqlBook,
            r#"SELECT id, title, author, image_url, year, pages, status AS "status: _" FROM book WHERE status = ANY($1)"#,
            statuses as _,
        )
        .fetch_all(&self.db)
        .await
        .unwrap_or_default();

        Ok(Some(rows.into_iter().map(|row| row.into_book()).collect()))
    }

    pub async fn add_book(&mut self, input: BookInput) -> Result<Book> {
        let row = sqlx::query_as!(
            SqlBook,
            r#"INSERT INTO book(title, author, image_url, year, pages) VALUES ($1, $2, $3, $4, $5) RETURNING id, title, author, image_url, year, pages, status AS "status: _""#,
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

    pub async fn update_book_status(
        &mut self,
        book_id: i32,
        status: ReadingStatus,
    ) -> Result<Book> {
        let row = sqlx::query_as!(
            SqlBook,
            r#"UPDATE book SET status=$1 WHERE id=$2 RETURNING id, title, author, image_url, year, pages, status AS "status: _""#,
            status as _,
            book_id
        )
        .fetch_optional(&self.db)
        .await?;

        if row.is_none() {
            return Err(anyhow!("No book with ID {} found.", book_id));
        }

        Ok(row.unwrap().into_book())
    }

    pub async fn get_notes_by_book(&self, book_id: i32) -> Result<Option<Vec<Note>>> {
        let rows = sqlx::query_as!(
            SqlNote,
            "SELECT id, book_id, note, page FROM note WHERE book_id=$1",
            book_id,
        )
        .fetch_all(&self.db)
        .await
        .unwrap_or_default();

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
