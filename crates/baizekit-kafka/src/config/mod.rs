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
    use rdkafka::admin::AdminClient;
    use rdkafka::client::DefaultClientContext;
    use rdkafka::consumer::BaseConsumer;
    use rdkafka::producer::BaseProducer;

    use crate::config::{AdminConfig, ConsumerConfig, ProducerConfig, ToRdkafkaClientConfig};

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

    #[test]
    fn test_new_producer() {
        let mut cfg = ProducerConfig::default();
        cfg.high.bootstrap_servers = "127.0.0.1:9092".to_string();
        let _producer: BaseProducer = cfg.to_client_config().unwrap().create().unwrap();
    }

    #[test]
    fn test_new_consumer() {
        let mut cfg = ConsumerConfig::default();
        cfg.high.bootstrap_servers = "127.0.0.1:9092".to_string();
        cfg.high.group_id = "test".to_string();
        let _consumer: BaseConsumer = cfg.to_client_config().unwrap().create().unwrap();
    }

    #[test]
    fn test_new_admin() {
        let mut cfg = AdminConfig::default();
        cfg.high.bootstrap_servers = "127.0.0.1:9092".to_string();
        let _admin: AdminClient<DefaultClientContext> = cfg.to_client_config().unwrap().create().unwrap();
    }
}
