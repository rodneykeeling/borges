use std::{collections::HashMap, sync::Arc};

use crate::graphql::{Book, BookInput};
use anyhow::Result;
use axum::async_trait;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::Mutex;

struct SqlBook {
    id: i32,
    title: String,
    author: String,
    image_url: Option<String>,
    year: i32,
    pages: i32,
}

impl SqlBook {
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

#[async_trait]
pub trait BookRepository {
    async fn get_by_title(&self, title: String) -> Result<Option<Book>>;
    async fn add(&mut self, input: BookInput) -> Result<Book>;
}

pub struct PostgresBookRepository {
    pub db: Pool<Postgres>,
}

impl PostgresBookRepository {
    pub async fn new() -> Result<Arc<Mutex<Self>>> {
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://root:pass@localhost/borges")
            .await?;
        Ok(Arc::new(Mutex::new(Self { db })))
    }
}

#[async_trait]
impl BookRepository for PostgresBookRepository {
    async fn get_by_title(&self, title: String) -> Result<Option<Book>> {
        let row = sqlx::query_as!(
            SqlBook,
            "SELECT id, title, author, image_url, year, pages FROM book WHERE title=$1",
            title,
        )
        .fetch_optional(&self.db)
        .await
        .unwrap_or(None)
        .unwrap();

        return Ok(Some(row.into_book()));
    }

    async fn add(&mut self, input: BookInput) -> Result<Book> {
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

        return Ok(row.into_book());
    }
}

// In-memory datastore used for tests/testing. Uses a HashMap instead of Vec for O(1) gets and inserts
pub struct InMemoryBookRepository {
    pub db: HashMap<String, Book>,
}

impl InMemoryBookRepository {
    pub fn new() -> Self {
        let mut db = HashMap::new();
        let title = "Collected Fictions".to_string();
        db.insert(
            title.clone(),
            Book {
                id: 0,
                title,
                author: "Jorge Luis Borges".to_string(),
                image_url: None,
                year: 1998,
                pages: 565,
            },
        );
        Self { db }
    }
}

#[async_trait]
impl BookRepository for InMemoryBookRepository {
    async fn get_by_title(&self, title: String) -> Result<Option<Book>> {
        Ok(self.db.get(&title).cloned())
    }

    async fn add(&mut self, input: BookInput) -> Result<Book> {
        let book = Book {
            id: 0,
            title: input.title,
            author: input.author,
            image_url: input.image_url,
            year: input.year,
            pages: input.pages,
        };
        self.db.insert(book.title.clone(), book.clone());
        Ok(book)
    }
}
