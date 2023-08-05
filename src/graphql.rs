use std::{fmt::Display, sync::Arc};

use anyhow::Result;
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptySubscription, InputObject, Object, Schema,
    SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::Extension, response::Html};
use tokio::sync::Mutex;

use crate::repository::{BookRepository, PostgresBookRepository};

#[derive(Clone, Debug, SimpleObject)]
#[graphql(complex)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub image_url: Option<String>,
    pub year: i32,
    pub pages: i32,
}

#[derive(Clone, InputObject)]
pub struct BookInput {
    pub title: String,
    pub author: String,
    pub image_url: Option<String>,
    pub year: i32,
    pub pages: i32,
}

#[derive(SimpleObject)]
pub struct Note {
    pub id: i32,
    pub note: String,
    pub book_id: i32,
}

#[derive(InputObject)]
pub struct NoteInput {
    pub book_id: i32,
    pub note: String,
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
    async fn book(
        &self,
        ctx: &Context<'_>,
        book_id: Option<i32>,
        title: Option<String>,
    ) -> Result<Option<Book>> {
        let repository = ctx
            .data_unchecked::<Arc<Mutex<PostgresBookRepository>>>()
            .clone();

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
    async fn add_book(&self, ctx: &Context<'_>, input: BookInput) -> Result<Book> {
        let repository = ctx
            .data_unchecked::<Arc<Mutex<PostgresBookRepository>>>()
            .clone();
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

    async fn add_note(&self, ctx: &Context<'_>, input: NoteInput) -> Result<Note> {
        let repository = ctx
            .data_unchecked::<Arc<Mutex<PostgresBookRepository>>>()
            .clone();
        let note_input = NoteInput {
            book_id: input.book_id,
            note: input.note,
        };
        let note = repository.lock().await.add_note(note_input).await?;
        Ok(note)
    }
}

#[ComplexObject]
impl Book {
    async fn notes(&self, ctx: &Context<'_>) -> Result<Option<Vec<Note>>> {
        let repository = ctx
            .data_unchecked::<Arc<Mutex<PostgresBookRepository>>>()
            .clone();
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
