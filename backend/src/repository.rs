use sqlx::{Error, PgPool, Postgres, Transaction};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

pub async fn get_user_id_by_username(pool: &PgPool, username: &str) -> Result<Option<Uuid>, Error> {
    let rec = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
        .fetch_optional(pool)
        .await?;
    Ok(rec.map(|r| r.id))
}

pub async fn get_user_id_by_username_tx(tx: &mut Transaction<'_, Postgres>, username: &str) -> Result<Option<Uuid>, Error> {
    let rec = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(rec.map(|r| r.id))
}

pub async fn get_user_passkeys(pool: &PgPool, user_id: Uuid) -> Result<Vec<Passkey>, Box<dyn std::error::Error>> {
    let creds = sqlx::query!("SELECT passkey_json FROM credentials WHERE user_id = $1", user_id)
        .fetch_all(pool)
        .await?;
    
    let mut pks = Vec::new();
    for c in creds {
        let pk: Passkey = serde_json::from_value(c.passkey_json)?;
        pks.push(pk);
    }
    Ok(pks)
}

pub async fn get_all_passkeys(pool: &PgPool) -> Result<Vec<Passkey>, Box<dyn std::error::Error>> {
    let creds = sqlx::query!("SELECT passkey_json FROM credentials")
        .fetch_all(pool)
        .await?;
    
    let mut pks = Vec::new();
    for c in creds {
        let pk: Passkey = serde_json::from_value(c.passkey_json)?;
        pks.push(pk);
    }
    Ok(pks)
}

pub async fn insert_user_if_not_exists(tx: &mut Transaction<'_, Postgres>, id: Uuid, username: &str) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO users (id, username) VALUES ($1, $2) ON CONFLICT (username) DO NOTHING",
        id, username
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn insert_credential(
    tx: &mut Transaction<'_, Postgres>, 
    cred_id_b64: &str, 
    user_id: Uuid, 
    passkey_json: serde_json::Value, 
    name: &str
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO credentials (cred_id, user_id, passkey_json, name) VALUES ($1, $2, $3, $4)",
        cred_id_b64, user_id, passkey_json, name
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn get_user_id_by_cred_id(pool: &PgPool, cred_id_b64: &str) -> Result<Option<Uuid>, Error> {
    let rec = sqlx::query!("SELECT user_id FROM credentials WHERE cred_id = $1", cred_id_b64)
        .fetch_optional(pool)
        .await?;
    Ok(rec.map(|r| r.user_id))
}
