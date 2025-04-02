use std::sync::OnceLock;
use std::time::Duration;

use redis::{cmd, Client, RedisError};
use serde::Deserialize;

use super::default_timeout;

static REDIS: OnceLock<Client> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub dsn: String,

    #[serde(default = "default_timeout")]
    pub timeout: Duration,
}

impl ClientConfig {
    pub fn new(dsn: String) -> Self { Self { dsn, timeout: default_timeout() } }

    pub fn build(&self) -> Result<Client, RedisError> {
        let client = Client::open(self.dsn.clone())?;

        let mut conn = client.get_connection_with_timeout(self.timeout)?;
        let _ = cmd("PING").query::<String>(&mut conn)?;

        Ok(client)
    }
}

/// 初始化 Redis 连接
pub fn get_or_init_client(cfg: &ClientConfig) -> &'static Client {
    REDIS.get_or_init(move || cfg.build().unwrap_or_else(|err| panic!("Redis 初始化失败. err: {}", err)))
}

/// 获取 Redis 连接
pub fn get_client() -> Option<&'static Client> { REDIS.get() }

/// 获取 Redis 连接
pub fn must_get_client() -> &'static Client { get_client().unwrap_or_else(|| panic!("Redis连接未初始化")) }
