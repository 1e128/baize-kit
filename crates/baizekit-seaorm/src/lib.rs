use std::time::Duration;

pub use cfg::*;
pub use cli::*;
pub use sea_orm;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::OnceCell;

mod cfg;
mod cli;

static DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_or_init_database_connection(cfg: Config) -> Result<&'static DatabaseConnection, String> {
    DB_CONNECTION
        .get_or_try_init(|| async {
            let mut opt = ConnectOptions::new(cfg.database_url)
                .max_connections(cfg.max_connections)
                .min_connections(cfg.min_connections)
                .connect_timeout(Duration::from_secs(cfg.connect_timeout_seconds))
                .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_seconds))
                .idle_timeout(Duration::from_secs(cfg.idle_timeout_seconds))
                .max_lifetime(Duration::from_secs(cfg.max_lifetime_seconds))
                .sqlx_logging(cfg.sqlx_logging)
                .sqlx_logging_level(cfg.sqlx_log_level)
                .sqlx_slow_statements_logging_settings(
                    cfg.slow_statements_log_level,
                    Duration::from_millis(cfg.slow_statements_threshold_millis),
                )
                .to_owned();

            if let Some(schema) = cfg.database_schema {
                opt.set_schema_search_path(schema);
            }

            Database::connect(opt).await.map_err(|e| e.to_string())
        })
        .await
}

#[inline(always)]
pub fn get_database_connection() -> Option<&'static DatabaseConnection> {
    DB_CONNECTION.get()
}

#[inline(always)]
pub fn must_get_database_connection() -> &'static DatabaseConnection {
    get_database_connection().expect("Failed to get database connection")
}
