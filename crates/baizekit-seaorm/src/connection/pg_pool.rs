use std::fmt::Write;

use log::LevelFilter;
use sqlx::postgres::PgConnectOptions;
use sqlx::PgPool;

use crate::connection::Config;

pub async fn try_new_pg_pool(c: Config) -> Result<PgPool, sqlx::Error> {
    let schema = c.schema.clone();
    let options = sea_orm::ConnectOptions::from(c);
    let mut opt = options.get_url().parse::<PgConnectOptions>()?;

    use sqlx::ConnectOptions;
    if !options.get_sqlx_logging() {
        opt = opt.disable_statement_logging();
    } else {
        opt = opt.log_statements(options.get_sqlx_logging_level());
        let (level, duration) = options.get_sqlx_slow_statements_logging_settings();
        if level != LevelFilter::Off {
            opt = opt.log_slow_statements(level, duration);
        }
    }
    let set_search_path_sql = schema.as_ref().map(|schema| {
        let mut string = "SET search_path = ".to_owned();
        if schema.starts_with('"') {
            write!(&mut string, "{schema}").unwrap();
        } else {
            for (i, schema) in schema.split(',').enumerate() {
                if i > 0 {
                    write!(&mut string, ",").unwrap();
                }
                if schema.starts_with('"') {
                    write!(&mut string, "{schema}").unwrap();
                } else {
                    write!(&mut string, "\"{schema}\"").unwrap();
                }
            }
        }
        string
    });

    let lazy = options.get_connect_lazy();
    let mut pool_options = options.sqlx_pool_options();
    if let Some(sql) = set_search_path_sql {
        pool_options = pool_options.after_connect(move |conn, _| {
            let sql = sql.clone();
            Box::pin(async move { sqlx::Executor::execute(conn, sql.as_str()).await.map(|_| ()) })
        });
    }
    if lazy { Ok(pool_options.connect_lazy_with(opt)) } else { pool_options.connect_with(opt).await }
}
