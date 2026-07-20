mod db;

use axum::Router;
use tracing::{error, info};
use db::{get_db_pool, DbConfig};

fn root_router(_db_pool: sqlx::PgPool) -> Router {
    Router::new()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Use environment variable if present, otherwise use a default connection string
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    let db_pool = match get_db_pool(DbConfig { url: db_url }).await {
        Ok(pool) => pool,
        Err(err) => {
            error!(%err, "Failed to connect to the database");
            std::process::exit(1);
        }
    };
    info!("Successfully connected to DB.");

    let app = root_router(db_pool);

    let addr = "0.0.0.0:8080";
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(%err, "Failed to bind to port {}.", addr);
            std::process::exit(1);
        }
    };
    info!("Server successfully bound to {}", addr);

    info!("Starting Axum web server...");
    if let Err(err) = axum::serve(listener, app).await {
        error!(%err, "Server crashed during runtime.");
        std::process::exit(1);
    }
}
