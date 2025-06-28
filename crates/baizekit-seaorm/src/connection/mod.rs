use sea_orm::{Database, DatabaseConnection, DbErr};
use tokio::sync::OnceCell;

mod cfg;
mod pg_pool;

pub use cfg::*;
pub use pg_pool::*;

static DB_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_or_init_database_connection(cfg: Config) -> Result<&'static DatabaseConnection, DbErr> {
    DB_CONNECTION.get_or_try_init(|| Database::connect(cfg)).await
}

#[inline(always)]
pub fn get_database_connection() -> Option<&'static DatabaseConnection> {
    DB_CONNECTION.get()
}

#[inline(always)]
pub fn must_get_database_connection() -> &'static DatabaseConnection {
    get_database_connection().expect("Failed to get database connection")
}
