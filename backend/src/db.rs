use sqlx::{postgres::PgPoolOptions, PgPool};

pub struct DbConfig {
    pub url: String,
}

pub async fn get_db_pool(config: DbConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.url)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::{runners::AsyncRunner, ImageExt};
    use testcontainers_modules::postgres::Postgres;

    #[tokio::test]
    async fn test_db_connection() -> Result<(), String> {
        let node = match Postgres::default().with_tag("18.4-trixie").start().await {
            Ok(n) => n,
            Err(err) => return Err(format!("Failed to start test container: {}", err)),
        };

        let port = match node.get_host_port_ipv4(5432).await {
            Ok(p) => p,
            Err(err) => return Err(format!("Failed to get port: {}", err)),
        };

        let connection_string = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

        let pool = match get_db_pool(DbConfig { url: connection_string }).await {
            Ok(p) => p,
            Err(err) => return Err(format!("Failed to connect to db: {}", err)),
        };

        let row: (i32,) = match sqlx::query_as("SELECT 1").fetch_one(&pool).await {
            Ok(r) => r,
            Err(err) => return Err(format!("Query failed: {}", err)),
        };

        assert_eq!(row.0, 1);
        Ok(())
    }
}
