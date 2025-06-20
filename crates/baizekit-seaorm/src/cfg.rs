use envy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// 数据库连接字符串
    #[serde(alias = "database_url", alias = "dsn")]
    pub database_url: String,
    /// 数据库schema, 仅用于 postgres
    #[serde(alias = "database_schema", alias = "schema")]
    pub database_schema: Option<String>,
    /// 是否开启sqlx的日志
    pub sqlx_logging: bool,
    /// sqlx日志级别
    pub sqlx_log_level: log::LevelFilter,
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
    pub slow_statements_log_level: log::LevelFilter,
    /// 慢查询阈值
    pub slow_statements_threshold_millis: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: Default::default(),
            database_schema: None,
            max_connections: 10,
            min_connections: 5,
            connect_timeout_seconds: 30,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 60,
            max_lifetime_seconds: 180,
            sqlx_logging: false,
            sqlx_log_level: log::LevelFilter::Info,
            slow_statements_log_level: log::LevelFilter::Info,
            slow_statements_threshold_millis: 1000,
        }
    }
}
