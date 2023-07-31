use crate::graphql::Book;
use axum::async_trait;

#[async_trait]
pub trait BookRepository {
    fn get_by_title(&self, title: String) -> Option<&Book>;
}

pub struct InMemoryBookRepository {
    pub db: Vec<Book>,
}

impl InMemoryBookRepository {
    pub fn new() -> Self {
        let db = vec![Book {
            title: "Collected Fictions".to_string(),
            author: "Jorge Luis Borges".to_string(),
            image_url: None,
            year: 1998,
            pages: 565,
        }];
        Self { db }
    }
}

impl BookRepository for InMemoryBookRepository {
    fn get_by_title(&self, title: String) -> Option<&Book> {
        self.db.iter().find(|&book| book.title == title)
    }
}
