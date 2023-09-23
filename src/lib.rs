use crate::{
    graphql::{graphiql, graphql_handler, Mutation, Query},
    repository::BookRepository,
};
use anyhow::Result;
use async_graphql::{extensions::Logger, EmptySubscription, Schema};
use axum::{extract::Extension, routing::get, Router};
use dotenvy::dotenv;
use sqlx::{Pool, Postgres};
use tokio::signal;
use tower_http::trace::{self, TraceLayer};
use tracing::warn;
use tracing::Level;

pub mod graphql;
pub mod repository;

pub async fn generate_app(db_conn: Pool<Postgres>) -> Result<Router, Box<dyn std::error::Error>> {
    dotenv().ok();

    let repository = BookRepository::new(db_conn);

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(repository)
        .extension(Logger)
        .finish();

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    Ok(app)
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!("Kill signal received. Starting graceful shutdown.");
}
