use std::net::SocketAddr;

use anyhow::Result;
use axum::Server;
use borges::{generate_app, shutdown_signal};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = generate_app().await?;
    let addr: SocketAddr = "0.0.0.0:8000".parse()?;
    info!("Serving on {addr}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
