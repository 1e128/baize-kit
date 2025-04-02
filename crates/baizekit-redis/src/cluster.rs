use std::sync::OnceLock;
use std::time::Duration;

use redis::cluster::{ClusterClient, ClusterConfig};
use redis::{cmd, RedisError};
use serde::Deserialize;

use crate::default_timeout;

static CLUSTER_CLIENT: OnceLock<ClusterClient> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct ClientConfig {
    pub nodes: Vec<String>,

    #[serde(default = "default_timeout")]
    pub timeout: Duration,
}

impl ClientConfig {
    pub fn new(nodes: Vec<String>) -> Self { Self { nodes, timeout: default_timeout() } }

    pub fn build(&self) -> Result<ClusterClient, RedisError> {
        let client = ClusterClient::new(self.nodes.clone())?;

        let cfg = ClusterConfig::new();
        let cfg = cfg.set_connection_timeout(self.timeout);

        let mut conn = client.get_connection_with_config(cfg)?;
        let _ = cmd("PING").query::<String>(&mut conn)?;

        Ok(client)
    }
}

pub fn get_or_init_cluster(cfg: &ClientConfig) -> &'static ClusterClient {
    CLUSTER_CLIENT.get_or_init(|| cfg.build().expect("Failed to build redis cluster client"))
}

pub fn get_cluster() -> Option<&'static ClusterClient> { CLUSTER_CLIENT.get() }

pub fn must_get_cluster() -> &'static ClusterClient { get_cluster().expect("Redis cluster client is not initialized") }
