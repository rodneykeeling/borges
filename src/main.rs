use std::net::SocketAddr;

use anyhow::Result;
use axum::Server;
use borges::{generate_app, shutdown_signal};
use dotenvy_macro::dotenv;
use sqlx::{pool::PoolOptions, Pool, Postgres};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let db: Pool<Postgres> = PoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await?;

    let app = generate_app(db).await?;
    let addr: SocketAddr = "0.0.0.0:8000".parse()?;
    info!("Serving on {addr}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
