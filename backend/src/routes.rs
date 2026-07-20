use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::post,
    Router,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use serde::Deserialize;
use webauthn_rs::prelude::*;
use crate::AppState;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register/start", post(register_start))
        .route("/register/finish", post(register_finish))
        .route("/login/start", post(login_start))
        .route("/login/finish", post(login_finish))
}

#[derive(Deserialize)]
struct RegisterStartRequest {
    username: String,
}

const REG_COOKIE_NAME: &str = "webauthn_reg_state";

async fn register_start(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(payload): Json<RegisterStartRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Generate a random ID for the user's registration flow
    let user_unique_id = Uuid::new_v4();
    
    let exclude_credentials = Some(vec![]);

    let (ccr, reg_state) = state
        .webauthn
        .start_passkey_registration(
            user_unique_id,
            &payload.username,
            &payload.username,
            exclude_credentials,
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let serialized_state = serde_json::to_string(&reg_state)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
    let cookie = Cookie::build((REG_COOKIE_NAME, serialized_state))
        .path("/")
        .http_only(true)
        .build();

    let updated_jar = jar.add(cookie);
    Ok((updated_jar, Json(ccr)))
}

async fn register_finish(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(payload): Json<RegisterPublicKeyCredential>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cookie = jar.get(REG_COOKIE_NAME)
        .ok_or((StatusCode::BAD_REQUEST, "Missing registration state cookie".to_string()))?;
        
    let reg_state: PasskeyRegistration = serde_json::from_str(cookie.value())
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid registration state".to_string()))?;

    let _passkey = state
        .webauthn
        .finish_passkey_registration(&payload, &reg_state)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        
    // We clear the cookie since the registration is done
    let updated_jar = jar.remove(Cookie::from(REG_COOKIE_NAME));
    
    Ok((updated_jar, StatusCode::OK))
}

#[derive(Deserialize)]
struct LoginStartRequest {
    username: String,
}

async fn login_start() -> Result<(), (StatusCode, String)> {
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented".to_string()))
}

async fn login_finish() -> Result<(), (StatusCode, String)> {
    Err((StatusCode::NOT_IMPLEMENTED, "Not implemented".to_string()))
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
            .body(Body::from(r#"{
                "id": "1234",
                "rawId": "1234",
                "type": "public-key",
                "response": {
                    "clientDataJSON": "1234",
                    "attestationObject": "1234"
                }
            }"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_login_start_endpoint() {
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
            .uri("/api/auth/login/start")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username": "testuser"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("set-cookie"));
    }

    #[tokio::test]
    async fn test_login_finish_missing_cookie() {
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
            .uri("/api/auth/login/finish")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{
                "id": "1234",
                "rawId": "1234",
                "type": "public-key",
                "response": {
                    "authenticatorData": "1234",
                    "clientDataJSON": "1234",
                    "signature": "1234",
                    "userHandle": "1234"
                }
            }"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
