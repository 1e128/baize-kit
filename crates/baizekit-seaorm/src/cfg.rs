use envy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// 数据库连接字符串
    pub database_url: String,
    /// 数据库schema, 仅用于 postgres
    pub database_schema: Option<String>,
    /// 是否开启sqlx的日志
    #[serde(default)]
    pub sqlx_logging: bool,
}

impl Config {
    #[inline(always)]
    pub fn try_new_from_env() -> Result<Self, String> { envy::from_env::<Self>().map_err(|e| e.to_string()) }
}
