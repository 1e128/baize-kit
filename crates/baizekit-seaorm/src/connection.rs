use std::time::Duration;

pub use log::LevelFilter;
use sea_orm::ConnectOptions;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// 数据库连接字符串
    #[serde(alias = "database_url", alias = "dsn")]
    pub url: String,
    /// 数据库schema, 仅用于 postgres
    #[serde(alias = "database_schema", alias = "schema")]
    pub schema: Option<String>,
    /// 是否开启sqlx的日志
    pub sqlx_logging: bool,
    /// sqlx日志级别
    pub sqlx_logging_level: LevelFilter,
    /// 数据库连接池最大连接数
    pub max_connections: u32,
    /// 数据库连接池最小连接数
    pub min_connections: u32,
    /// 数据库连接池连接超时时间
    pub connect_timeout_seconds: u64,
    /// 数据库连接池获取连接超时
    pub acquire_timeout_seconds: u64,
    /// 数据库连接池空闲超时
    pub idle_timeout_seconds: u64,
    /// 数据库连接池最大生命周期
    pub max_lifetime_seconds: u64,
    /// 慢查询日志级别
    pub sqlx_slow_statements_logging_level: LevelFilter,
    /// 慢查询阈值
    pub sqlx_slow_statements_logging_threshold: u64,
    pub connect_lazy: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: Default::default(),
            schema: None,
            max_connections: 5,
            min_connections: 1,
            connect_timeout_seconds: 30,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 60,
            max_lifetime_seconds: 180,
            sqlx_logging: false,
            sqlx_logging_level: LevelFilter::Info,
            sqlx_slow_statements_logging_level: LevelFilter::Info,
            sqlx_slow_statements_logging_threshold: 1000,
            connect_lazy: false,
        }
    }
}

impl From<Config> for ConnectOptions {
    fn from(cfg: Config) -> Self {
        let mut opt = Self::new(cfg.url)
            .max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            .connect_timeout(Duration::from_secs(cfg.connect_timeout_seconds))
            .acquire_timeout(Duration::from_secs(cfg.acquire_timeout_seconds))
            .idle_timeout(Duration::from_secs(cfg.idle_timeout_seconds))
            .max_lifetime(Duration::from_secs(cfg.max_lifetime_seconds))
            .sqlx_logging(cfg.sqlx_logging)
            .sqlx_logging_level(cfg.sqlx_logging_level)
            .sqlx_slow_statements_logging_settings(
                cfg.sqlx_slow_statements_logging_level,
                Duration::from_millis(cfg.sqlx_slow_statements_logging_threshold),
            )
            .connect_lazy(cfg.connect_lazy)
            .to_owned();

        if let Some(schema) = cfg.schema {
            opt.set_schema_search_path(schema);
        }

        opt
    }
}
