pub mod client;

#[cfg(feature = "cluster")]
pub mod cluster;

pub use redis;

fn default_timeout() -> std::time::Duration { std::time::Duration::from_secs(10) }
