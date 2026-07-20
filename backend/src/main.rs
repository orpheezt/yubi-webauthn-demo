mod auth;
mod db;
pub mod repository;
mod routes;

use auth::build_webauthn;
use axum::Router;
use db::{DbConfig, get_db_pool};
use std::sync::Arc;
use tracing::{error, info};

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

async fn healthz() -> impl axum::response::IntoResponse {
    axum::http::StatusCode::OK
}

async fn readyz(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<impl axum::response::IntoResponse, (axum::http::StatusCode, String)> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|e| (axum::http::StatusCode::SERVICE_UNAVAILABLE, e.to_string()))?;
    Ok(axum::http::StatusCode::OK)
}

fn root_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", axum::routing::get(healthz))
        .route("/readyz", axum::routing::get(readyz))
        .nest("/api/auth", routes::auth_routes())
        .with_state(state)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Configurable database connection parameters
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    let max_connections = std::env::var("MAX_DB_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(10);

    let db_pool = match get_db_pool(DbConfig {
        url: db_url,
        max_connections,
    })
    .await
    {
        Ok(pool) => pool,
        Err(err) => {
            error!(%err, "Failed to connect to the database");
            std::process::exit(1);
        }
    };
    info!(
        "Successfully connected to DB with max_connections={}.",
        max_connections
    );

    let rp_id = std::env::var("RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin =
        std::env::var("RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let webauthn = match build_webauthn(&rp_id, &rp_origin) {
        Ok(w) => Arc::new(w),
        Err(err) => {
            error!(?err, "Failed to initialize Webauthn");
            std::process::exit(1);
        }
    };
    info!(
        "Successfully initialized Webauthn for RP_ID='{}', RP_ORIGIN='{}'.",
        rp_id, rp_origin
    );

    // Persistent Cookie Encryption Key if COOKIE_SECRET is provided, otherwise generate key
    let cookie_key = match std::env::var("COOKIE_SECRET") {
        Ok(secret) => {
            let bytes = secret.as_bytes();
            if bytes.len() >= 64 {
                axum_extra::extract::cookie::Key::from(&bytes[..64])
            } else {
                let mut padded = [0u8; 64];
                padded[..bytes.len()].copy_from_slice(bytes);
                axum_extra::extract::cookie::Key::from(&padded)
            }
        }
        Err(_) => axum_extra::extract::cookie::Key::generate(),
    };

    let state = AppState {
        db: db_pool,
        webauthn,
        cookie_key,
    };

    let app = root_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| format!("0.0.0.0:{}", port));
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(%err, "Failed to bind to {}.", bind_addr);
            std::process::exit(1);
        }
    };
    info!("Server successfully bound to {}", bind_addr);

    info!("Starting Axum web server...");
    if let Err(err) = axum::serve(listener, app).await {
        error!(%err, "Server crashed during runtime.");
        std::process::exit(1);
    }
}
