use std::fmt::Display;

use anyhow::Result;
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptySubscription, InputObject, Object, Schema,
    SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::Extension, response::Html};

use crate::repository::Storage;

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Book {
    /// The book ID
    pub id: i32,
    /// The title of the book
    pub title: String,
    /// The author of the book
    pub author: String,
    /// A link to an image of the book cover
    pub image_url: Option<String>,
    /// The year the book was published
    pub year: i32,
    /// The number of pages in the book
    pub pages: i32,
}

#[derive(Clone, InputObject)]
pub struct BookInput {
    /// The title of the book
    pub title: String,
    /// The author of the book
    pub author: String,
    /// A link to an image of the book cover
    pub image_url: Option<String>,
    /// The year the book was published
    pub year: i32,
    /// The number of pages in the book
    pub pages: i32,
}

#[derive(SimpleObject)]
pub struct Note {
    /// The note ID
    pub id: i32,
    /// The ID of the book this note references
    pub book_id: i32,
    /// The note content for a book
    pub note: String,
    /// An optional page number related to the number
    pub page: Option<i32>,
}

#[derive(InputObject)]
pub struct NoteInput {
    /// The ID of the book this note references
    pub book_id: i32,
    /// The note content for a book
    pub note: String,
    /// An optional page number related to the number
    pub page: Option<i32>,
}

pub struct Query;
pub struct Mutation;

#[derive(Debug)]
pub enum GraphQLError {
    BadInput(String),
}

impl Display for GraphQLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadInput(bad_input_error) => write!(f, "{bad_input_error}"),
        }
    }
}

impl std::error::Error for GraphQLError {}

#[Object]
impl Query {
    /// Fetch a book by either its ID or by its title
    async fn book(
        &self,
        ctx: &Context<'_>,
        book_id: Option<i32>,
        title: Option<String>,
    ) -> Result<Option<Book>> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        if let Some(input) = book_id {
            return Ok(repository.lock().await.get_book_by_id(input).await?);
        } else if let Some(input) = title {
            return Ok(repository.lock().await.get_book_by_title(input).await?);
        }
        Err(GraphQLError::BadInput(
            "Either `bookId` or `title` input value required for query.".to_string(),
        )
        .into())
    }
}

#[Object]
impl Mutation {
    /// Add a new book to the shelf
    async fn add_book(&self, ctx: &Context<'_>, input: BookInput) -> Result<Book> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        if input.pages < 1 {
            return Err(GraphQLError::BadInput(
                "Book page number cannot be less than 1.".to_string(),
            )
            .into());
        }

        let book_input = BookInput {
            title: input.title,
            author: input.author,
            image_url: input.image_url,
            year: input.year,
            pages: input.pages,
        };
        let book = repository.lock().await.add_book(book_input).await?;
        Ok(book)
    }

    /// Add a new note for a given book
    async fn add_note(&self, ctx: &Context<'_>, input: NoteInput) -> Result<Note> {
        let repository = ctx.data_unchecked::<Storage>().clone();

        // Fetch requested book for validation
        let book = repository
            .lock()
            .await
            .get_book_by_id(input.book_id)
            .await?;

        if book.is_none() {
            return Err(GraphQLError::BadInput(
                "Book with ID {input.book_id} not found".to_string(),
            )
            .into());
        };
        let book = book.unwrap();

        if let Some(note_page) = input.page {
            if note_page > book.pages {
                return Err(GraphQLError::BadInput(
                    "Note page number cannot be greater than the highest page count of the book."
                        .to_string(),
                )
                .into());
            } else if note_page < 1 {
                return Err(GraphQLError::BadInput(
                    "Note page number cannot be less than 1.".to_string(),
                )
                .into());
            }
        }

        let note_input = NoteInput {
            book_id: input.book_id,
            note: input.note,
            page: input.page,
        };
        let note = repository.lock().await.add_note(note_input).await?;
        Ok(note)
    }
}

#[ComplexObject]
impl Book {
    /// All notes related to the given book
    async fn notes(&self, ctx: &Context<'_>) -> Result<Option<Vec<Note>>> {
        let repository = ctx.data_unchecked::<Storage>().clone();
        let notes = repository.lock().await.get_notes_by_book(self.id).await?;
        Ok(notes)
    }
}

pub async fn graphql_handler(
    schema: Extension<Schema<Query, Mutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphiql() -> Html<String> {
    Html(GraphiQLSource::build().finish())
}
