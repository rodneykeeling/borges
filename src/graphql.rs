use std::sync::Arc;

use anyhow::Result;
use async_graphql::{
    http::GraphiQLSource, Context, EmptySubscription, InputObject, Object, Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::Extension, response::Html};
use tokio::sync::Mutex;

use crate::repository::{BookRepository, PostgresBookRepository};

#[derive(Clone, Debug, SimpleObject)]
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

pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    async fn book(&self, ctx: &Context<'_>, title: String) -> Result<Option<Book>> {
        let repository = ctx
            .data_unchecked::<Arc<Mutex<PostgresBookRepository>>>()
            .clone();
        let book = repository.lock().await.get_by_title(title).await?;
        Ok(book)
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
        let book = repository.lock().await.add(book_input).await?;
        Ok(book)
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
