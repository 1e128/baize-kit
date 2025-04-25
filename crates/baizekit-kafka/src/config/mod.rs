use rdkafka::ClientConfig;
use serde::Serialize;
use serde_json::Value;
use snafu::ResultExt;

use crate::error::{JsonSnafu, Result};

mod admin;
mod consumer;
mod producer;

pub use admin::*;
pub use consumer::*;
pub use producer::*;

pub trait ToRdkafkaClientConfig: Serialize {
    fn to_client_config(&self) -> Result<ClientConfig> {
        let value = serde_json::to_value(&self).context(JsonSnafu)?;

        let mut cfg = ClientConfig::new();
        let Value::Object(value) = value else {
            return Ok(cfg);
        };

        for (key, value) in value {
            match value {
                Value::Null => {}
                Value::Bool(value) => {
                    cfg.set(key, value.to_string());
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

impl ToRdkafkaClientConfig for AdminConfig {}

impl ToRdkafkaClientConfig for ProducerConfig {}

impl ToRdkafkaClientConfig for ConsumerConfig {}

#[cfg(test)]
mod tests {
    use crate::config::ToRdkafkaClientConfig;

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
            "compression.type": "gzip"
        }"#;

        let config: super::ProducerConfig = serde_json::from_str(config).unwrap();
        println!("{:?}", config);

        let kafka_conf = config.to_client_config().unwrap();
        println!("{:?}", kafka_conf.config_map());
    }
}
