use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::OnceCell;

mod cfg;
mod cli;

pub use cfg::Config;
pub use sea_orm;
pub use cli::*;

static DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_or_init_database_connection(cfg: Config) -> Result<&'static DatabaseConnection, String> {
    DB_CONNECTION
        .get_or_try_init(|| async {
            let mut opt = ConnectOptions::new(cfg.database_url)
                .max_connections(100)
                .min_connections(5)
                .connect_timeout(Duration::from_secs(5))
                .acquire_timeout(Duration::from_secs(5))
                .idle_timeout(Duration::from_secs(100))
                .max_lifetime(Duration::from_secs(100))
                .sqlx_logging(cfg.sqlx_logging)
                .to_owned();

            if let Some(schema) = cfg.database_schema {
                opt.set_schema_search_path(schema);
            }

            Database::connect(opt).await.map_err(|e| e.to_string())
        })
        .await
}

pub async fn try_get_database_connection() -> Result<&'static DatabaseConnection, String> {
    get_or_init_database_connection(Config::try_new_from_env()?).await
}

pub fn get_database_connection() -> Option<&'static DatabaseConnection> { DB_CONNECTION.get() }

#[inline(always)]
pub fn must_get_database_connection() -> &'static DatabaseConnection {
    get_database_connection().expect("Failed to get database connection")
}
