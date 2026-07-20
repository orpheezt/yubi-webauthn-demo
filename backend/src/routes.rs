use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, SameSite, PrivateCookieJar};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;
use crate::AppState;
use crate::repository;

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register/start", post(register_start))
        .route("/register/finish", post(register_finish))
        .route("/login/start", post(login_start))
        .route("/login/finish", post(login_finish))
        .route("/me", get(get_me))
        .route("/logout", post(logout))
        .route("/credentials", get(list_credentials))
        .route("/credentials/{cred_id}", patch(update_credential))
        .route("/credentials/{cred_id}", delete(delete_credential))
}

#[derive(Deserialize)]
struct RegisterStartRequest {
    username: String,
    key_name: Option<String>,
}

const REG_COOKIE_NAME: &str = "webauthn_reg_state";

async fn register_start(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(payload): Json<RegisterStartRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let display_name = match &payload.key_name {
        Some(k) if !k.trim().is_empty() => format!("{} ({})", payload.username, k),
        _ => payload.username.clone(),
    };

    let (user_unique_id, exclude_credentials) = match repository::get_user_id_by_username(&state.db, &payload.username)
    .await
    .map_err(|e| {
        tracing::error!("Failed to query user: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })? {
        Some(user_id) => {
            let pks = repository::get_user_passkeys(&state.db, user_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch user credentials: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                })?;

            let descriptor_vec = pks.into_iter().map(|pk| pk.cred_id().clone()).collect();
            (user_id, Some(descriptor_vec))
        }
        None => (Uuid::new_v4(), Some(vec![])),
    };

    let (ccr, reg_state) = state
        .webauthn
        .start_passkey_registration(
            user_unique_id,
            &payload.username,
            &display_name,
            exclude_credentials,
        )
        .map_err(|e| {
            tracing::error!("start_passkey_registration error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    tracing::info!("Registration started for user: {}", payload.username);

    // We also need to store the username in the cookie so we can save it on finish
    #[derive(Serialize)]
    struct RegCookieState {
        username: String,
        state: PasskeyRegistration,
    }
    
    let cookie_state = RegCookieState {
        username: payload.username,
        state: reg_state,
    };

    let serialized_state = serde_json::to_string(&cookie_state)
        .map_err(|e| {
            tracing::error!("serde_json serialization error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
        
    let cookie = Cookie::build((REG_COOKIE_NAME, serialized_state))
        .path("/")
        .http_only(true)
        .build();

    let updated_jar = jar.add(cookie);
    Ok((updated_jar, Json(ccr)))
}

#[derive(Deserialize)]
struct RegCookieState {
    username: String,
    state: PasskeyRegistration,
}

#[derive(Deserialize)]
struct RegisterFinishPayload {
    #[serde(flatten)]
    credential: RegisterPublicKeyCredential,
    name: Option<String>,
}

async fn register_finish(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(payload): Json<RegisterFinishPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("register_finish called");
    let cookie = jar.get(REG_COOKIE_NAME)
        .ok_or_else(|| {
            tracing::error!("Missing registration state cookie!");
            (StatusCode::BAD_REQUEST, "Missing registration state cookie".to_string())
        })?;
        
    let cookie_state: RegCookieState = serde_json::from_str(cookie.value())
        .map_err(|e| {
            tracing::error!("Failed to deserialize cookie state: {:?}", e);
            (StatusCode::BAD_REQUEST, "Invalid registration state".to_string())
        })?;

    let passkey = state
        .webauthn
        .finish_passkey_registration(&payload.credential, &cookie_state.state)
        .map_err(|e| {
            tracing::error!("finish_passkey_registration error: {:?}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;
        
    // Save to DB
    let mut tx = state.db.begin().await.map_err(|e| {
        tracing::error!("Failed to begin DB transaction: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    
    let user_uuid = Uuid::new_v4();
    
    // Insert user if not exists (handling conflict)
    if let Err(e) = repository::insert_user_if_not_exists(&mut tx, user_uuid, &cookie_state.username).await {
        tracing::error!("Failed to insert user into DB: {:?}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    let user_id = match repository::get_user_id_by_username_tx(&mut tx, &cookie_state.username).await {
        Ok(Some(id)) => id,
        Ok(None) => return Err((StatusCode::INTERNAL_SERVER_ERROR, "User not found after insert".to_string())),
        Err(e) => {
            tracing::error!("Failed to fetch user record from DB: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    };

    // 2. Insert credential
    let passkey_json = match serde_json::to_value(&passkey) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to serialize passkey: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    };

    let cred_id_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, passkey.cred_id());
    let passkey_name = payload.name.unwrap_or_else(|| "Security Key".to_string());

    if let Err(e) = repository::insert_credential(&mut tx, &cred_id_b64, user_id, passkey_json, &passkey_name).await {
        tracing::error!("Failed to insert credential into DB: {:?}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    if let Err(e) = tx.commit().await {
        tracing::error!("Failed to commit DB transaction: {:?}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
    }

    tracing::info!("Successfully registered passkey for user: {}", cookie_state.username);
    let session_cookie = create_session_cookie(user_id.to_string());
    let updated_jar = jar.remove(Cookie::from(REG_COOKIE_NAME)).add(session_cookie);
    Ok((updated_jar, StatusCode::OK))
}

const SESSION_COOKIE_NAME: &str = "webauthn_session";

fn create_session_cookie(user_id: String) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, user_id))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(24))
        .build()
}

#[derive(Serialize)]
struct UserProfileResponse {
    id: Uuid,
    username: String,
    created_at: String,
    credentials_count: i64,
}

#[derive(Serialize)]
struct CredentialItem {
    cred_id: String,
    name: String,
    created_at: String,
}

#[derive(Deserialize)]
struct UpdateCredentialRequest {
    name: String,
}

async fn list_credentials(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cookie = jar.get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let user_id = Uuid::parse_str(cookie.value())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session".to_string()))?;

    let creds = repository::list_credentials_info(&state.db, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items: Vec<CredentialItem> = creds
        .into_iter()
        .map(|c| CredentialItem {
            cred_id: c.cred_id,
            name: c.name,
            created_at: c.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

async fn update_credential(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Path(cred_id): Path<String>,
    Json(payload): Json<UpdateCredentialRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cookie = jar.get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let user_id = Uuid::parse_str(cookie.value())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session".to_string()))?;

    let rows_affected = repository::update_credential_name(&state.db, &cred_id, user_id, &payload.name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Credential not found".to_string()));
    }

    Ok(StatusCode::OK)
}

async fn delete_credential(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Path(cred_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cookie = jar.get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let user_id = Uuid::parse_str(cookie.value())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session".to_string()))?;

    let rows_affected = repository::delete_credential(&state.db, &cred_id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Credential not found".to_string()));
    }

    Ok(StatusCode::OK)
}

async fn get_me(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cookie = jar.get(SESSION_COOKIE_NAME)
        .ok_or((StatusCode::UNAUTHORIZED, "Not authenticated".to_string()))?;

    let user_id = Uuid::parse_str(cookie.value())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid session".to_string()))?;

    let user = repository::get_user_info(&state.db, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    let cred_count = repository::get_credential_count(&state.db, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(UserProfileResponse {
        id: user.id,
        username: user.username,
        created_at: user.created_at.to_rfc3339(),
        credentials_count: cred_count,
    }))
}

async fn logout(
    jar: PrivateCookieJar,
) -> impl IntoResponse {
    let updated_jar = jar.remove(Cookie::from(SESSION_COOKIE_NAME));
    (updated_jar, StatusCode::OK)
}

#[derive(Deserialize)]
struct LoginStartRequest {
    username: String,
}

const AUTH_COOKIE_NAME: &str = "webauthn_auth_state";

async fn login_start(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(payload): Json<LoginStartRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("login_start called for username: '{}'", payload.username);
    
    let passkeys = if payload.username.trim().is_empty() {
        repository::get_all_passkeys(&state.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch all credentials: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?
    } else {
        let user_id = repository::get_user_id_by_username(&state.db, &payload.username)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query user: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;

        if let Some(uid) = user_id {
            repository::get_user_passkeys(&state.db, uid)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch user credentials: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                })?
        } else {
            tracing::error!("User not found: {}", payload.username);
            return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
        }
    };

    if passkeys.is_empty() {
        tracing::error!("No passkeys registered in database");
        return Err((StatusCode::NOT_FOUND, "No registered passkeys found".to_string()));
    }

    let (rcr, auth_state) = state
        .webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|e| {
            tracing::error!("start_passkey_authentication error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let serialized_state = serde_json::to_string(&auth_state)
        .map_err(|e| {
            tracing::error!("Failed to serialize auth_state: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
        
    let cookie = Cookie::build((AUTH_COOKIE_NAME, serialized_state))
        .path("/")
        .http_only(true)
        .build();

    let updated_jar = jar.add(cookie);
    tracing::info!("login_start successful");
    Ok((updated_jar, Json(rcr)))
}

async fn login_finish(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Json(payload): Json<PublicKeyCredential>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    tracing::info!("login_finish called");
    let cookie = jar.get(AUTH_COOKIE_NAME)
        .ok_or_else(|| {
            tracing::error!("Missing authentication state cookie!");
            (StatusCode::BAD_REQUEST, "Missing authentication state cookie".to_string())
        })?;
        
    let auth_state: PasskeyAuthentication = serde_json::from_str(cookie.value())
        .map_err(|e| {
            tracing::error!("Failed to deserialize auth_state: {:?}", e);
            (StatusCode::BAD_REQUEST, "Invalid authentication state".to_string())
        })?;

    let auth_result = state
        .webauthn
        .finish_passkey_authentication(&payload, &auth_state)
        .map_err(|e| {
            tracing::error!("finish_passkey_authentication error: {:?}", e);
            (StatusCode::BAD_REQUEST, e.to_string())
        })?;

    let cred_id_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, auth_result.cred_id());
    let user_id = repository::get_user_id_by_cred_id(&state.db, &cred_id_b64)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find user for credential: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User for credential not found".to_string()))?;
        
    tracing::info!("login_finish successful!");
    let session_cookie = create_session_cookie(user_id.to_string());
    let updated_jar = jar.remove(Cookie::from(AUTH_COOKIE_NAME)).add(session_cookie);
    
    Ok((updated_jar, StatusCode::OK))
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

    async fn setup_test_app() -> Result<(Router, testcontainers::ContainerAsync<Postgres>), Box<dyn std::error::Error>> {
        let pg_tag = std::env::var("TEST_POSTGRES_TAG").unwrap_or_else(|_| "18.4-trixie".to_string());
        let node = Postgres::default().with_tag(&pg_tag).start().await?;
        let port = node.get_host_port_ipv4(5432).await?;
        let connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
        
        let pool = get_db_pool(DbConfig { url: connection_string.clone(), max_connections: 5 }).await?;
        
        // Run migrations
        sqlx::migrate!("../migrations").run(&pool).await?;

        let rp_id = std::env::var("TEST_RP_ID").unwrap_or_else(|_| "localhost".to_string());
        let rp_origin = std::env::var("TEST_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let webauthn = Arc::new(build_webauthn(&rp_id, &rp_origin)?);
        let state = AppState {
            db: pool,
            webauthn,
            cookie_key: Key::generate(),
        };

        Ok((Router::new().nest("/api/auth", auth_routes()).with_state(state), node))
    }

    #[tokio::test]
    async fn test_register_start_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let (app, _node) = setup_test_app().await?;

        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/register/start")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"username": "testuser"}"#))?;

        let response = app.oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("set-cookie"));
        Ok(())
    }

    #[tokio::test]
    async fn test_register_finish_missing_cookie() -> Result<(), Box<dyn std::error::Error>> {
        let (app, _node) = setup_test_app().await?;

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
            }"#))?;

        let response = app.oneshot(request).await?;
        
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }
}
