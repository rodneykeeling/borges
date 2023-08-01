use std::net::SocketAddr;

use crate::{
    graphql::{graphiql, graphql_handler, Mutation, Query},
    repository::Storage,
};
use async_graphql::{EmptySubscription, Schema};
use axum::{extract::Extension, routing::get, Router, Server};

mod graphql;
mod repository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repository = Storage::default();
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(repository)
        .finish();
    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    let addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    println!("Serving on {addr}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
