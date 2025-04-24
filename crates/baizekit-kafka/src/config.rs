use std::collections::HashMap;
use std::time::Duration;

use rdkafka::ClientConfig;
use serde::Deserialize;
use strum::{Display, EnumString};

/// Kafka Producer 的 acks 设置
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Display, EnumString, Deserialize)]
pub enum Ack {
    /// 不等待任何 broker 确认（最快，最不可靠）
    #[strum(serialize = "0")]
    #[serde(rename = "0")]
    None,

    /// 等待 leader broker 确认（默认）
    #[default]
    #[strum(serialize = "1")]
    #[serde(rename = "1")]
    Leader,

    /// 等待所有 ISR 副本确认（最安全）
    #[strum(serialize = "all")]
    #[serde(rename = "all")]
    All,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProducerConfig {
    pub bootstrap_servers: String,

    pub client_id: Option<String>,

    /// all | 1 | 0
    pub acks: Ack,

    /// 延迟发送，减少请求数
    pub linger_ms: Option<u64>,

    /// 批量发送的最大字节数
    pub batch_size: Option<usize>,

    /// gzip, snappy, lz4, zstd
    pub compression_type: Option<String>,

    /// 是否开启幂等发送
    pub enable_idempotence: bool,

    /// 自定义参数
    pub custom: HashMap<String, String>,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            client_id: Some("my-producer".to_string()),
            acks: Ack::default(),
            linger_ms: None,
            batch_size: None,
            compression_type: None,
            enable_idempotence: false,
            custom: Default::default(),
        }
    }
}

impl ProducerConfig {
    pub fn flush_timeout(&self) -> Duration {
        let flush_timeout_ms = self
            .custom
            .get("flush_timeout_ms")
            .map(|s| s.parse::<u64>().ok())
            .flatten()
            .unwrap_or(1000);
        Duration::from_millis(flush_timeout_ms)
    }

    pub fn to_rdkafka_config(&self) -> ClientConfig {
        let mut config = ClientConfig::new();
        config.set("bootstrap.servers", &self.bootstrap_servers);
        config.set("acks", self.acks.to_string());

        if let Some(ref id) = self.client_id {
            config.set("client.id", id);
        }

        if let Some(ms) = self.linger_ms {
            config.set("linger.ms", &ms.to_string());
        }

        if let Some(size) = self.batch_size {
            config.set("batch.size", &size.to_string());
        }

        if let Some(ref compression) = self.compression_type {
            config.set("compression.type", compression);
        }

        if self.enable_idempotence {
            config.set("enable.idempotence", "true");
        }

        for (k, v) in &self.custom {
            config.set(k, v);
        }

        config
    }
}
