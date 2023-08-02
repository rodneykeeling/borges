use std::net::SocketAddr;

use crate::{
    graphql::{graphiql, graphql_handler, Mutation, Query},
    repository::Storage,
};
use async_graphql::{EmptySubscription, Schema};
use axum::{extract::Extension, routing::get, Router, Server};
use tower_http::trace::{self, TraceLayer};
use tracing::{info, Level};

mod graphql;
mod repository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let repository = Storage::default();

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(repository)
        .finish();

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr: SocketAddr = "0.0.0.0:8000".parse().unwrap();
    info!("Serving on {addr}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
