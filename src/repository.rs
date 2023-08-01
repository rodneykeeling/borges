use std::collections::HashMap;

use crate::graphql::Book;
use axum::async_trait;

#[async_trait]
pub trait BookRepository {
    fn get_by_title(&self, title: String) -> Option<&Book>;
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

impl BookRepository for InMemoryBookRepository {
    fn get_by_title(&self, title: String) -> Option<&Book> {
        self.db.get(&title)
    }
}
