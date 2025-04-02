pub mod client;

#[cfg(feature = "cluster")]
pub mod cluster;

fn default_timeout() -> std::time::Duration { std::time::Duration::from_secs(10) }
