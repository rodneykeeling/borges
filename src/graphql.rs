use std::sync::{Arc, Mutex};

use async_graphql::{
    http::GraphiQLSource, Context, EmptySubscription, InputObject, Object, Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
};

use crate::repository::{BookRepository, InMemoryBookRepository};

#[derive(Clone, Debug, SimpleObject)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub image_url: Option<String>,
    pub year: u32,
    pub pages: u32,
}

#[derive(Clone, InputObject)]
pub struct BookInput {
    pub title: String,
    pub author: String,
    pub image_url: Option<String>,
    pub year: u32,
    pub pages: u32,
}

pub struct Query;
pub struct Mutation;

#[Object]
impl Query {
    async fn book(&self, ctx: &Context<'_>, title: String) -> Option<Book> {
        let repository = ctx
            .data_unchecked::<Arc<Mutex<InMemoryBookRepository>>>()
            .clone();
        let book = repository.lock().unwrap().get_by_title(title);
        book
    }
}

#[Object]
impl Mutation {
    async fn add_book(&self, ctx: &Context<'_>, input: BookInput) -> Book {
        let repository = ctx
            .data_unchecked::<Arc<Mutex<InMemoryBookRepository>>>()
            .clone();
        let book = Book {
            title: input.title,
            author: input.author,
            image_url: input.image_url,
            year: input.year,
            pages: input.pages,
        };
        let result = repository.lock().unwrap().add(book);
        result
    }
}

pub async fn graphql_handler(
    schema: Extension<Schema<Query, Mutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().finish())
}
