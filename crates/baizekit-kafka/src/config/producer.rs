use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Kafka Producer 的 acks 设置
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Display, EnumString, Deserialize, Serialize)]
pub enum Ack {
    /// 等待所有 ISR 副本确认（最安全）
    #[strum(serialize = "all", serialize = "-1")]
    #[serde(rename = "all", alias = "-1")]
    All,

    /// 不等待任何 broker 确认（最快，最不可靠）
    #[strum(serialize = "0")]
    #[serde(rename = "0")]
    None,

    /// 等待 leader broker 确认（默认）
    #[default]
    #[strum(serialize = "1")]
    #[serde(rename = "1")]
    Leader,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ProducerConfigHighLevel {
    /// Kafka 集群地址
    #[serde(rename = "bootstrap.servers")]
    pub bootstrap_servers: String,

    /// ### acks
    ///
    /// > 设置生产者在认为写入成功前，Kafka 需要确认的副本数量。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka 的持久性依赖副本机制，每个分区可配置多个副本（replica）；
    /// - `acks` 控制了多少个副本需确认写入，才算消息“成功”；
    /// - Producer 发送消息后，只有满足 `acks` 要求的副本确认，才会收到成功响应。
    ///
    /// ---
    ///
    /// #### 可选值说明
    ///
    /// - `0`: 不等待任何确认。Producer 不会收到 Kafka 的响应，可能导致数据丢失但延迟最低
    /// - `1`: 等待 leader 副本确认写入即可，平衡了性能和可靠性
    /// - `all` 或 `-1`: 等待所有同步副本（ISR）确认写入，最安全但延迟最大
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 若对数据可靠性要求极高，建议使用 `acks = all`；
    /// - 若系统对延迟敏感，可考虑 `acks = 1`；
    /// - 不建议使用 `acks = 0`，除非你明确不关心消息是否丢失。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - `acks = all` 依赖 `min.insync.replicas`，若 ISR 不足，写入会失败；
    /// — 若 ISR 数小于 `min.insync.replicas`，即使设置了 `acks = all`，Kafka 也会拒绝写入；
    /// - 搭配 `retries` 和 `delivery.timeout.ms` 使用，可增强容错和重试控制；
    /// - 即使 `acks = all`，也无法保证消费者成功消费，只表示消息已被 Kafka 持久化。
    pub acks: Ack,

    /// ### compression.type
    ///
    /// > 指定 Kafka 生产者在发送消息时使用的压缩算法类型。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka 支持多种压缩算法：
    ///   - `none`: 无压缩，默认设置；
    /// - `gzip`: GZIP 压缩，常见的压缩方式，压缩比率较高，但压缩和解压缩速度相对较慢；
    /// - `snappy`: Snappy 压缩，压缩和解压缩速度较快，压缩比率略低；
    /// - `lz4`: LZ4 压缩，速度非常快，压缩比率略低于 gzip；
    /// - `zstd`: Zstandard 压缩，结合了高压缩比和高解压速度。
    /// - 生产者将消息压缩后发送到 Kafka，消费者解压后处理。
    /// - 各种压缩方式之间有不同的性能和资源消耗特性，选择时需要根据吞吐量、延迟和资源消耗平衡。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - `none`: 适用于对性能要求较高且压缩开销不可接受的场景；
    /// - `gzip`: 推荐用于需要较高压缩比的场景，适合批量消息；
    /// - `snappy`: 当处理速度优先，且压缩比要求不高时，可以考虑使用；
    /// - `lz4`: 推荐用于对压缩速度和解压速度要求较高的场景；
    /// - `zstd`: 推荐用于对压缩比和解压速度有较高要求的场景，适合较大数据量传输。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 启用压缩会增加生产者端 CPU 使用率，但能显著减少网络带宽的使用；
    /// - 生产者和消费者端必须支持相同的压缩类型，否则消费者无法解压消息；
    /// - 若 Kafka 集群需要存储大量数据，使用压缩可以显著减少存储成本。
    #[serde(rename = "compression.type")]
    pub compression_type: String,

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
}

impl Default for ProducerConfigHighLevel {
    fn default() -> Self {
        Self { bootstrap_servers: "".to_string(), acks: Ack::Leader, compression_type: "none".to_string(), retries: 0 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ProducerConfigMediumLevel {
    /// ### linger.ms
    ///
    /// > Producer 发送消息前等待的时间（毫秒），用于批量发送优化吞吐。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka Producer 在内部维护一个缓冲区（batch）；
    /// — 如果缓冲区未满，Producer 会等待最多 `linger.ms` 毫秒，看看是否有更多消息加入批次；
    /// - 如果在 `linger.ms` 时间内消息数量未达到 batch 大小上限，也会被发送；
    /// - 如果 buffer 满了，消息会立刻发送而不等待。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认为 0，意味着尽快发送；
    /// — 可适当设置如 5~100ms，提高吞吐量并减少请求数量；
    /// - 对吞吐量要求高、延迟容忍度高的场景，适合增加 `linger.ms`。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 设置过高会带来发送延迟，影响实时性；
    /// — `linger.ms` 与 `batch.size` 配合使用更有效；
    /// - 该参数只影响 Producer，不影响 Consumer。
    #[serde(rename = "batch.size")]
    pub batch_size: usize,

    /// ### client.id
    ///
    /// > 为 Kafka 客户端设置一个唯一的标识符，用于标识该客户端实例。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka 客户端（Producer 或 Consumer）在与 Kafka 集群通信时会附带 `client.id`；
    /// - Kafka Broker 会记录每个客户端的 `client.id`，用于日志、监控、配额控制等用途；
    /// - 不同客户端使用不同的 `client.id` 可帮助区分来源、追踪问题。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 推荐显式设置 `client.id`，特别是在多实例部署时；
    /// - 可使用服务名称 + 实例编号等形式，便于定位问题，例如 `order-consumer-01`；
    /// - 对于 Producer 和 Consumer，可使用不同前缀标识角色。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 如果不设置该字段，Kafka 客户端会自动分配一个默认值（如 `producer-1`, `consumer-1`）；
    /// — `client.id` 不影响消息处理逻辑，但对诊断问题非常重要；
    /// - 若多个客户端使用相同的 `client.id`，在 Kafka 日志和监控中可能混淆。
    #[serde(rename = "client.id")]
    pub client_id: String,

    /// ### linger.ms
    ///
    /// > Producer 发送消息前等待的时间（毫秒），用于批量发送优化吞吐。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka Producer 在内部维护一个缓冲区（batch）；
    /// — 如果缓冲区未满，Producer 会等待最多 `linger.ms` 毫秒，看看是否有更多消息加入批次；
    /// - 如果在 `linger.ms` 时间内消息数量未达到 batch 大小上限，也会被发送；
    /// - 如果 buffer 满了，消息会立刻发送而不等待。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认为 0，意味着尽快发送；
    /// — 可适当设置如 5~100ms，提高吞吐量并减少请求数量；
    /// - 对吞吐量要求高、延迟容忍度高的场景，适合增加 `linger.ms`。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 设置过高会带来发送延迟，影响实时性；
    /// — `linger.ms` 与 `batch.size` 配合使用更有效；
    /// - 该参数只影响 Producer，不影响 Consumer。
    #[serde(rename = "linger.ms")]
    pub linger_ms: i32,

    /// ### request.timeout.ms
    ///
    /// > Producer 或 Consumer 向 Kafka 发送请求后等待响应的最长时间（单位：毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 每次客户端向 Kafka 发起请求（如 produce、fetch、metadata 等）都会设置一个请求超时时间；
    /// — 如果 Kafka 在该时间内未返回响应，客户端会认为请求失败，可能会重试或抛出异常；
    /// — 对 Producer 而言，若启用了 `acks=all`，超时时间应包含等待副本确认的时间。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值通常为 `30000`（30 秒）；
    /// — 如果网络延迟较高或 Kafka 负载重，建议适当增加该值；
    /// — 对于对实时性要求高的系统，可适当调低，快速失败。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 与 `delivery.timeout.ms` 联动：Producer 的整体消息投递超时由 `delivery.timeout.ms` 控制；
    /// — 如果 `request.timeout.ms` 设置大于 `delivery.timeout.ms`，Producer 有可能在响应回来前就放弃等待；
    /// — Consumer 设置过短的超时可能会导致频繁断线重连。
    #[serde(rename = "request.timeout.ms")]
    pub request_timeout_ms: i32,
}

impl Default for ProducerConfigMediumLevel {
    fn default() -> Self {
        Self { batch_size: 16384, client_id: "".to_string(), linger_ms: 0, request_timeout_ms: 30000 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ProducerConfigLowLevel {
    /// ### enable.idempotence
    ///
    /// > 是否启用生产者的幂等性，确保在发生重试时不会导致消息重复。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 启用幂等性后，Kafka 会为每个 Producer 分配唯一的 Producer ID（PID）；
    /// - 每条消息会带上递增的序列号，Broker 端会校验序列号，防止重复写入；
    /// - 幂等性保证 **单分区、有序、无重复** 的消息写入。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为 `false`；
    /// - 推荐始终启用此选项（`true`），尤其是生产环境，以避免消息重复；
    /// - 启用后会自动设置：
    ///   - `acks=all`
    ///   - `retries` 为一个大于 0 的值（若未手动指定）
    ///   - `max.in.flight.requests.per.connection <= 5`
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 幂等性仅适用于 Kafka >= 0.11；
    /// — 幂等性 **不是事务性**，无法跨多个 partition 或 topic 提供原子性；
    /// — 如果需要事务保障，应搭配使用 `transactional.id`；
    /// — 启用幂等性对性能几乎无影响，但大幅提升可靠性。
    #[serde(rename = "enable.idempotence")]
    pub enable_idempotence: bool,

    /// ### max.in.flight.requests.per.connection
    ///
    /// > 每个连接在未收到响应前，允许同时发送的最大请求数。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka Producer 会将消息批量发送给 Broker；
    /// - 在未收到之前请求响应前，可以继续发送后续请求；
    /// — 该配置限制了这种“未完成请求”的并发数量；
    /// — 设置过高可能会导致消息乱序，尤其是在重试场景下；
    /// — 设置为 1 可强制保证消息严格有序（但吞吐下降）。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值通常为 `5`；
    /// - 若启用 `enable.idempotence=true`，此值 **必须 <= 5**，Kafka 会自动调整；
    /// - 若不启用幂等性，可视吞吐需求提高此值，但应权衡顺序性；
    /// — 推荐值范围：幂等性场景下使用 `1~5`，普通场景可适当提高。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 与消息顺序强相关：
    ///   - 单个分区 + 顺序性要求 => 建议设为 `1`；
    /// — 多分区或无严格顺序要求 => 可设为更大以提升吞吐；
    /// — 与 `retries` 一起使用时要注意：大于 1 时若发生重试，可能导致消息乱序；
    /// — 实际请求数量还受 `linger.ms`、`batch.size`、`buffer.memory` 等参数影响。
    #[serde(rename = "max.in.flight.requests.per.connection")]
    pub max_in_flight_requests_per_connection: i32,

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

impl Default for ProducerConfigLowLevel {
    fn default() -> Self {
        Self {
            enable_idempotence: false,
            max_in_flight_requests_per_connection: 5,
            reconnect_backoff_max_ms: 1000,
            reconnect_backoff_ms: 50,
            retry_backoff_ms: 100,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct ProducerConfig {
    #[serde(flatten)]
    pub high: ProducerConfigHighLevel,
    #[serde(flatten)]
    pub medium: ProducerConfigMediumLevel,
    #[serde(flatten)]
    pub low: ProducerConfigLowLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_producer_config() {
        let orig = ProducerConfig::default();
        let json = serde_json::to_string(&orig).unwrap();
        println!("{}", json);
        let new: ProducerConfig = serde_json::from_str(&json).unwrap();
        println!("{:?}", new);
        assert_eq!(orig, new);
    }
}
