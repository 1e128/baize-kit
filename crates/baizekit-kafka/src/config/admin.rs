use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct AdminConfigHighLevel {
    /// Kafka 集群地址
    #[serde(rename = "bootstrap.servers")]
    pub bootstrap_servers: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct AdminConfigMediumLevel {
    /// ### client.id
    ///
    /// > 客户端的唯一标识符，用于在 Kafka 服务端日志中标识客户端实例。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka 使用 `client.id` 来标识来自不同客户端的请求；
    /// - 该标识符会出现在 Kafka 服务端的日志中，帮助管理员追踪特定客户端的行为；
    /// - `client.id` 也可以在调试时帮助区分不同的消费者或生产者客户端。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为空字符串，表示不设置 `client.id`；
    /// - 如果需要通过日志区分不同的客户端实例，可以为每个实例设置不同的 `client.id`；
    /// - 设置合理的 `client.id` 有助于更好地跟踪性能和故障分析。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - `client.id` 在集群中的所有客户端都应唯一，避免混淆；
    /// - 对于具有多个消费者或生产者的应用程序，确保为每个客户端指定唯一的 `client.id` 以便于日志的区分；
    /// - `client.id` 不会影响 Kafka 的消息传递，只是用于日志和调试。
    #[serde(rename = "client.id")]
    pub client_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct AdminConfigLowLevel {
    /// ### reconnect.backoff.max.ms
    ///
    /// > 消费者与 Kafka 集群断开连接后，重试连接的最大等待时间（毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 当消费者与 Kafka 集群失去连接时，消费者会尝试重新连接；
    /// - `reconnect.backoff.max.ms` 配置项定义了消费者尝试重新连接时，等待的最大时间间隔；
    /// - 如果重连失败，Kafka 会遵循指数退避策略，在重试之间逐渐增加等待时间，直到达到 `reconnect.backoff.max.ms` 的上限。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值通常为 10000 毫秒（10 秒）；
    /// - 设置较小的值可以使消费者更快地恢复连接，但也可能增加网络负担；
    /// - 设置较大的值可以减少重试次数，但可能延迟恢复连接的时间；
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 该配置项适用于消费者失去与 Kafka 集群的连接时，控制重试间隔；
    /// - 配合 `reconnect.backoff.ms` 使用，前者是重试间隔的起始值，后者是最大值；
    /// - 如果消费者需要高可用性，可以适当调整该值来平衡恢复时间和网络负担。
    #[serde(rename = "reconnect.backoff.max.ms")]
    pub reconnect_backoff_max_ms: i32,

    /// ### reconnect.backoff.ms
    ///
    /// > 消费者与 Kafka 集群断开连接后，首次重试连接的等待时间（毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 当消费者与 Kafka 集群失去连接时，消费者会尝试重新连接；
    /// - `reconnect.backoff.ms` 配置项定义了首次重试连接时的等待时间；
    /// - 如果连接仍然失败，Kafka 会继续根据指数退避策略逐步增加等待时间，直到达到 `reconnect.backoff.max.ms` 的上限。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值通常为 50 毫秒；
    /// - 设置较小的值可以让消费者更快地尝试恢复连接，但可能增加网络负担；
    /// - 设置较大的值可能会减少重试频率，但恢复连接的时间会更长。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 该配置项适用于消费者失去与 Kafka 集群的连接时，控制首次重试间隔；
    /// - 配合 `reconnect.backoff.max.ms` 使用，后者定义了重试的最大等待时间；
    /// - 在高可用性要求较高的场景下，可以调整该值来平衡恢复连接的速度和系统的稳定性。
    #[serde(rename = "reconnect.backoff.ms")]
    pub reconnect_backoff_ms: i32,

    /// ### retries
    ///
    /// > 设置生产者在发送失败时重试的最大次数。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Producer 在发送消息失败时（如网络错误、Leader 不可用），会根据 `retries` 配置进行自动重试；
    /// - 每次重试之间会等待一个 `retry.backoff.ms` 所定义的时间间隔；
    /// — 重试只适用于可重试的错误（如临时网络故障），不会对非可恢复错误（如消息过大）生效；
    /// - Kafka 的 Producer 是幂等性的（需开启 `enable.idempotence`），可以保证多次重试不会造成重复写入。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 推荐设置为一个大于 0 的合理值，例如 `retries = 3`；
    /// - 若要求高可用，可设置更高的值，但要注意整体写入延迟可能增加；
    /// - 配合 `acks=all` 和 `enable.idempotence=true` 可实现可靠、幂等的消息写入。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 仅设置 `retries` 不会无限重试，超时仍会失败（受 `delivery.timeout.ms` 控制）；
    /// - 不同异常类型的重试策略可能不同，部分错误不会被重试；
    /// - 若开启了事务模式，也应确保 `retries` 设置合理，以避免事务失败。
    pub retries: i32,

    /// ### retry.backoff.ms
    ///
    /// > 消费者在重试请求失败时的等待时间（毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 当消费者发送请求（例如：拉取消息）失败时，Kafka 会根据 `retry.backoff.ms` 设置的等待时间来延迟下一次重试；
    /// - 该配置项控制消费者在失败后重新尝试的时间间隔；
    /// - 重试次数和最大重试间隔由其他配置项控制，如 `retries` 和 `retry.backoff.max.ms`。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值通常为 100 毫秒；
    /// - 设置较小的值可以让消费者更快地重试，但可能导致请求压力增加；
    /// - 设置较大的值可以减少请求压力，但可能会延迟消费者的响应时间。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 该配置项仅在请求失败时起作用，若请求成功则不使用该等待时间；
    /// - 在高可用性和容错性要求较高的系统中，可以根据负载调节此参数；
    /// - 配合 `retry.backoff.max.ms` 使用，后者定义了最大重试间隔。
    #[serde(rename = "retry.backoff.ms")]
    pub retry_backoff_ms: i32,
}

impl Default for AdminConfigLowLevel {
    fn default() -> Self {
        Self { reconnect_backoff_max_ms: 1000, reconnect_backoff_ms: 50, retries: 5, retry_backoff_ms: 100 }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct AdminConfig {
    #[serde(flatten)]
    pub high: AdminConfigHighLevel,
    #[serde(flatten)]
    pub medium: AdminConfigMediumLevel,
    #[serde(flatten)]
    pub low: AdminConfigLowLevel,
}
