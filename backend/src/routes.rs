use axum::{
    routing::post,
    Router,
};
use crate::AppState;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register/start", post(register_start))
        .route("/register/finish", post(register_finish))
}

async fn register_start() -> axum::response::Html<&'static str> {
    todo!("Implement register_start")
}

async fn register_finish() -> axum::response::Html<&'static str> {
    todo!("Implement register_finish")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use std::sync::Arc;
    use crate::{auth::build_webauthn, db::{DbConfig, get_db_pool}};
    use testcontainers::{runners::AsyncRunner, ImageExt};
    use testcontainers_modules::postgres::Postgres;
    use axum_extra::extract::cookie::Key;

    #[tokio::test]
    async fn test_register_start_endpoint() {
        let node = Postgres::default().with_tag("18.4-trixie").start().await.unwrap();
        let port = node.get_host_port_ipv4(5432).await.unwrap();
        let connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
        
        let pool = get_db_pool(DbConfig { url: connection_string }).await.unwrap();
        let webauthn = Arc::new(build_webauthn("localhost", "http://localhost:8080").unwrap());

        let state = AppState {
            db: pool,
            webauthn,
            cookie_key: Key::generate(),
        };

        let app = Router::new().nest("/api/auth", auth_routes()).with_state(state);

        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/register/start")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username": "testuser"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("set-cookie"));
    }

    #[tokio::test]
    async fn test_register_finish_missing_cookie() {
        let node = Postgres::default().with_tag("18.4-trixie").start().await.unwrap();
        let port = node.get_host_port_ipv4(5432).await.unwrap();
        let connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
        
        let pool = get_db_pool(DbConfig { url: connection_string }).await.unwrap();
        let webauthn = Arc::new(build_webauthn("localhost", "http://localhost:8080").unwrap());

        let state = AppState {
            db: pool,
            webauthn,
            cookie_key: Key::generate(),
        };

        let app = Router::new().nest("/api/auth", auth_routes()).with_state(state);

        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/register/finish")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"dummy": "data"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
