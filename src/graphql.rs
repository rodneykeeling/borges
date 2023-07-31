use async_graphql::{
    http::GraphiQLSource, Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
};

use crate::repository::{BookRepository, InMemoryBookRepository};

#[derive(SimpleObject)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub image_url: Option<String>,
    pub year: u32,
    pub pages: u32,
}

pub struct Query;

#[Object]
impl Query {
    async fn book<'a>(&'a self, ctx: &Context<'a>, title: String) -> Option<&Book> {
        let repository = ctx.data_unchecked::<InMemoryBookRepository>();
        repository.get_by_title(title)
    }
}

pub async fn graphql_handler(
    schema: Extension<Schema<Query, EmptyMutation, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().finish())
}
