mod auth;
mod db;
mod routes;

use axum::Router;
use tracing::{error, info};
use db::{get_db_pool, DbConfig};
use auth::build_webauthn;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub webauthn: Arc<webauthn_rs::Webauthn>,
    pub cookie_key: axum_extra::extract::cookie::Key,
}

impl axum::extract::FromRef<AppState> for axum_extra::extract::cookie::Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

fn root_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/auth", routes::auth_routes())
        .with_state(state)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
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

    let rp_id = std::env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin = std::env::var("RP_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let webauthn = match build_webauthn(&rp_id, &rp_origin) {
        Ok(w) => Arc::new(w),
        Err(err) => {
            error!(?err, "Failed to initialize Webauthn");
            std::process::exit(1);
        }
    };
    info!("Successfully initialized Webauthn.");

    let state = AppState {
        db: db_pool,
        webauthn,
        cookie_key: axum_extra::extract::cookie::Key::generate(),
    };

    let app = root_router(state);

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
