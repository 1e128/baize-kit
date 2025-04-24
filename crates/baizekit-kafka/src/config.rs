use std::collections::HashMap;

use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::ResultExt;
use strum::{Display, EnumString};

use crate::error::{JsonSnafu, Result};

pub trait ToRdkafkaConfig: Serialize {
    fn to_rdkafka_config(&self) -> Result<ClientConfig> {
        let value = serde_json::to_value(&self).context(JsonSnafu)?;

        let mut cfg = ClientConfig::new();
        let Value::Object(value) = value else {
            return Ok(cfg);
        };

        for (key, value) in value {
            match value {
                Value::Null => {}
                Value::Bool(value) => {
                    cfg.set(key, if value { "true" } else { "false" });
                }
                Value::Number(value) => {
                    cfg.set(key, value.to_string());
                }
                Value::String(value) => {
                    cfg.set(key, value);
                }
                Value::Array(_) => {}
                Value::Object(_) => {}
            }
        }

        Ok(cfg)
    }
}

/// Kafka Producer 的 acks 设置
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Display, EnumString, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ProducerConfig {
    #[serde(rename = "bootstrap.servers")]
    pub bootstrap_servers: String,

    #[serde(rename = "client.id")]
    pub client_id: Option<String>,

    /// all | 1 | 0
    #[serde(rename = "acks")]
    pub acks: Ack,

    /// 重试次数
    #[serde(rename = "retries")]
    pub retries: i64,

    /// 在尝试将一个失败的请求重试到给定的 topic 分区之前需要等待的时间。这避免在某些失败场景下在紧凑的循环中重复发送请求。
    #[serde(rename = "retry.backoff.ms")]
    pub retry_backoff_ms: Option<u64>,

    /// 在发生阻塞之前，客户端的一个连接上允许出现未确认请求的最大数量。
    ///
    /// 注意，如果这个设置大于1，并且有失败的发送，则消息可能会由于重试而导致重新排序(如果重试是启用的话)
    #[serde(rename = "max.in.flight.requests.per.connection")]
    pub max_in_flight_requests_per_connection: Option<i64>,

    /// 是否启用幂等性
    #[serde(rename = "enable.idempotence")]
    pub enable_idempotence: bool,

    /// 客户端等待请求响应的最大时长。
    /// 如果超时未收到响应，则客户端将在必要时重新发送请求，如果重试的次数达到允许的最大重试次数，则请求失败。
    /// 这个参数应该比 replica.lag.time.max.ms （Broker 的一个参数）更大，以降低由于不必要的重试而导致的消息重复的可能性。
    #[serde(rename = "request.timeout.ms")]
    pub request_timeout_ms: Option<u64>,

    /// gzip, snappy, lz4, zstd
    #[serde(rename = "compression.type")]
    pub compression_type: Option<String>,

    #[serde(flatten)]
    pub other: HashMap<String, String>,
}

impl ToRdkafkaConfig for ProducerConfig {}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsumerConfig {
    #[serde(rename = "bootstrap.servers")]
    pub bootstrap_servers: String,

    #[serde(rename = "group.id")]
    pub group_id: String,

    #[serde(rename = "enable.auto.commit")]
    pub enable_auto_commit: bool,

    #[serde(rename = "auto.commit.interval.ms")]
    pub auto_commit_interval_ms: i32,

    #[serde(rename = "session.timeout.ms")]
    pub session_timeout_ms: i32,

    #[serde(rename = "auto.offset.reset")]
    pub auto_offset_reset: AutoOffsetReset,

    #[serde(rename = "max.poll.records")]
    pub max_poll_records: i32,

    #[serde(rename = "fetch.min.bytes")]
    pub fetch_min_bytes: i32,

    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoOffsetReset {
    Earliest,
    #[default]
    Latest,
    None,
}

impl ToRdkafkaConfig for ConsumerConfig {}

#[cfg(test)]
mod tests {
    use crate::config::ToRdkafkaConfig;

    #[test]
    fn serde_producer_config() {
        let config = r#"
        {
            "bootstrap.servers": "localhost:9092",
            "client.id": "test",
            "acks": "all",
            "retries": 5,
            "retry.backoff.ms": 1000,
            "max.in.flight.requests.per.connection": 5,
            "enable.idempotence": true,
            "request.timeout.ms": 5000,
            "compression.type": "gzip",
            "key": "value"
        }"#;

        let config: super::ProducerConfig = serde_json::from_str(config).unwrap();

        println!("{:?}", config);

        let kafka_conf = config.to_rdkafka_config().unwrap();
        println!("{:?}", kafka_conf.config_map());

        assert!(false);
    }
}
