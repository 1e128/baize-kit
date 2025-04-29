use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoOffsetReset {
    Earliest,
    #[default]
    Latest,
    None,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    #[default]
    ReadUncommitted,
    ReadCommitted,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsumerConfigHighLevel {
    /// Kafka 集群地址
    #[serde(rename = "bootstrap.servers")]
    pub bootstrap_servers: String,

    /// ### fetch.min.bytes
    ///
    /// > 指定服务器在响应 fetch 请求前，至少要准备好多少字节的数据。
    ///
    /// ---
    ///
    /// #### 工作原理
    ///
    /// 当消费者发出 fetch 请求时，Kafka 会：
    ///
    /// 1. 检查对应分区中是否有新消息；
    /// 2. 如果新消息的数据大小 **小于 `fetch.min.bytes`**，Kafka 会等待一段时间（由 `fetch.max.wait.ms` 控制）；
    /// 3. 如果数据大小满足或等到时间到了，就返回消息。
    ///
    /// ---
    ///
    /// #### 使用建议
    ///
    /// - 实时系统或低延迟消费场景（如交易撮合、监控预警）：保留默认值 `1`；
    /// - 高吞吐场景（如日志处理、批量消费）：建议设置为 `1024` ～ `65536`；
    /// - 分布式流处理系统（如 Flink、Spark）：可设置为 `65536` 或更高，以提高吞吐。
    ///
    /// ---
    ///
    /// #### 注意事项
    ///
    /// - `fetch.min.bytes` 越大，吞吐可能提升，但延迟也会随之增加；
    /// - 需与 `fetch.max.wait.ms` 搭配使用，控制 **等待时间与吞吐** 的平衡；
    /// - 不要与 `max.partition.fetch.bytes` 混淆，后者是单个分区一次 fetch 返回的最大数据量。
    #[serde(rename = "fetch.min.bytes")]
    pub fetch_min_bytes: i32,

    /// ### group.id
    ///
    /// > 唯一标识消费者所属消费者组的字符串。
    ///
    /// ---
    ///
    /// #### 工作原理
    ///
    /// 当使用 `subscribe(topic)` 或 Kafka 的偏移量管理功能时，必须设置 `group.id`，Kafka 会根据该 ID：
    ///
    /// 1. 将消费者划分进一个消费者组；
    /// 2. 协调组内成员，分配分区（由 Group Coordinator 完成）；
    /// 3. 使用 Kafka 存储偏移量（而非手动管理）；
    /// 4. 使同组消费者实现分区消费负载均衡与容错。
    ///
    /// ---
    ///
    /// #### 使用建议
    ///
    /// - 若使用 `subscribe` 方法，必须设置 `group.id`；
    /// - 每个独立消费的业务逻辑建议用不同的 `group.id`；
    /// - 若希望多个消费者共享消费进度，则它们应使用相同的 `group.id`。
    ///
    /// ---
    ///
    /// #### 注意事项
    ///
    /// - 若未设置 `group.id`，且使用了 `subscribe()`，会报错；
    /// - 同一个 `group.id` 下，**每个消费者必须有唯一的 `client.id` 或 host**，否则可能互相抢占分区；
    /// - 修改 `group.id` 会视为“新组”，之前的消费进度不会保留；
    /// - 若只使用 `assign()` 手动分配分区，可不设置 `group.id`。
    #[serde(rename = "group.id")]
    pub group_id: String,

    /// ### heartbeat.interval.ms
    ///
    /// > 消费者向 Kafka 群组协调器发送心跳的时间间隔（毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 心跳用于向协调器表明消费者仍然存活；
    /// - Kafka 使用心跳来检测消费者是否宕机或失联，以便及时触发 rebalance；
    /// - 心跳由后台线程自动发送，与 poll() 无关。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值一般为 3000 毫秒（3 秒）；
    /// - 通常应保持 `heartbeat.interval.ms` < `session.timeout.ms` 的 1/3；
    /// - 设置过小会带来网络和协调器负担，设置过大则可能导致误判消费者失联。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 心跳间隔不影响消费者拉取消息的频率；
    /// - 如果消费者在 `session.timeout.ms` 期间未发送足够的心跳，将被视为失效；
    /// - 建议不要将该值调得太低，除非你有非常严格的容错延迟要求。
    #[serde(rename = "heartbeat.interval.ms")]
    pub heartbeat_interval_ms: i32,

    /// ### max.partition.fetch.bytes
    ///
    /// > 指定每个分区单次 fetch 请求能返回的最大数据量（单位：字节）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 消费者从 Kafka 拉取数据时，针对每个分区都有一个最大数据量限制；
    /// - Kafka 会尽量返回尽可能多的消息，但不会超过此配置设置的上限；
    /// - 如果消息本身超过该值，Kafka 仍会返回整条消息，避免消息截断。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为 1048576 字节（1MB）；
    /// - 如果你预期消息体较大（如压缩后的批量消息），可适当调高此值以减少 fetch 次数；
    /// - 如果消费者内存资源受限，可适当调低以控制单次数据量和内存占用。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 此参数作用于**每个分区**，如果消费者一次 fetch 多个分区，总体数据量会更大；
    /// - 不要与 `fetch.max.bytes` 混淆，后者是整个 fetch 请求级别的总大小限制；
    /// - 该参数不会限制最大消息大小，仅影响返回数据量的“期望值”，Kafka 会确保返回完整消息。
    #[serde(rename = "max.partition.fetch.bytes")]
    pub max_partition_fetch_bytes: i32,

    /// ### session.timeout.ms
    ///
    /// > 指定消费者与 Kafka 群组协调器之间的心跳超时值（毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 如果消费者在 `session.timeout.ms` 时间内未向 Kafka 协调器发送心跳或处理分配的任务，协调器会认为消费者已经失效；
    /// - 如果在超时之前消费者未成功发送心跳，Kafka 会将其视为失效并触发 rebalance；
    /// - 心跳机制由消费者自动处理，通过后台线程定时向协调器发送心跳。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值通常为 10000 毫秒（10 秒）；
    /// - 根据网络环境和容错需求设置，通常应保证心跳间隔（`heartbeat.interval.ms`）小于 `session.timeout.ms` 的 1/3；
    /// - 设置过低的值可能导致在网络延迟较大的情况下，消费者频繁地被误判为失效，触发不必要的 rebalance。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 设置过短的超时时间可能会导致频繁的消费者重新平衡，影响性能；
    /// - 与 `heartbeat.interval.ms` 配合使用，避免在网络波动时消费者被错误地认为失效；
    #[serde(rename = "session.timeout.ms")]
    pub session_timeout_ms: i32,
}

impl Default for ConsumerConfigHighLevel {
    fn default() -> Self {
        Self {
            bootstrap_servers: "".to_string(),
            fetch_min_bytes: 1,
            group_id: "".to_string(),
            heartbeat_interval_ms: 3000,
            max_partition_fetch_bytes: 1048576,
            session_timeout_ms: 10000,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsumerConfigMediumLevel {
    /// ### auto.offset.reset
    ///
    /// > 当 Kafka 中不存在初始偏移量，或当前偏移量已过期（例如数据被删除），消费者该如何处理。
    ///
    /// ---
    ///
    /// #### 工作原理
    ///
    /// 当消费者属于某个 `group.id` 并使用 Kafka 自动管理偏移量时，Kafka 会尝试从内部保存的偏移位置开始拉取数据。如果遇到以下情况：
    ///
    /// 1. **该消费者组之前从未消费过某个分区**；
    /// 2. **之前的偏移位置对应的数据已被 Kafka 清除（因过期或保留策略）**；
    ///
    /// 则 Kafka 会依据 `auto.offset.reset` 的值，决定如何恢复消费位置。
    ///
    /// ---
    ///
    /// #### 可选值说明
    ///
    /// - `earliest`：自动重置偏移到最早的可用数据；
    /// - `latest`：自动重置偏移到最新的数据，仅消费“之后”产生的数据；
    /// - `none`：如果找不到偏移，则抛出异常；
    /// - 其他值：同样抛出异常，提示非法配置。
    ///
    /// ---
    ///
    /// #### 使用建议
    ///
    /// - **流式处理推荐使用 `earliest`**：保证首次消费可以拿到所有历史数据；
    /// - **实时消费推荐使用 `latest`**：只关心“从现在开始”产生的数据；
    /// - 若希望严格控制消费起点，请使用 `assign()` 并设置具体偏移，或使用 `none` 强制开发者处理。
    ///
    /// ---
    ///
    /// #### 注意事项
    ///
    /// - 此参数只在 **初次消费** 或 **偏移丢失** 时起作用；
    /// - 若 offset 存在，则不会触发该参数的逻辑；
    /// - 不影响手动设置 offset 的消费者（如使用 `seek()`）；
    /// - 若配置错误（非上述三者），会直接抛出异常。
    #[serde(rename = "auto.offset.reset")]
    pub auto_offset_reset: AutoOffsetReset,

    /// ### enable.auto.commit
    ///
    /// > 指定是否启用自动提交消费偏移量。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 如果设置为 `true`，消费者会自动提交已消费消息的偏移量；
    /// - 默认情况下，Kafka 会每隔一段时间（由 `auto.commit.interval.ms` 控制）提交一次消费者的偏移量；
    /// - 如果设置为 `false`，消费者必须手动提交偏移量（通过调用 `commitSync()` 或 `commitAsync()` 方法）。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 如果需要精确控制偏移量提交，建议将该参数设置为 `false`，并在应用中显式地调用手动提交；
    /// - 如果你不需要精确的偏移量管理，或者希望简化代码，可以将其设置为 `true`；
    /// - 自动提交适用于消息消费不要求严格一致性的场景，但可能导致部分消息丢失或重复消费。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 自动提交的频率由 `auto.commit.interval.ms` 参数控制；
    /// - 如果消费者未能及时提交偏移量，可能导致重新消费已经处理的消息；
    /// - 若使用手动提交，需特别注意确保提交的正确性与一致性。
    #[serde(rename = "enable.auto.commit")]
    pub enable_auto_commit: bool,

    /// ### fetch.max.bytes
    ///
    /// > 指定每次 fetch 请求可以从 Kafka 中获取的最大数据量（字节数）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 当消费者向 Kafka 发起 fetch 请求时，Kafka 会根据此参数限制返回的数据大小；
    /// - 如果返回的数据超过此值，Kafka 会截断数据，保证不会超过指定的最大字节数；
    /// - 该参数可以用来控制消费者单次拉取的数据量，以避免消费者处理过大消息时出现内存问题。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为 50MB，可以根据实际需求调整；
    /// - 如果消费者需要处理大量数据，可以适当调高此值；
    /// - 设置过小可能导致多次 fetch 请求，增加网络开销；
    /// - 设置过大可能导致消费者内存消耗过大，影响性能。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - `fetch.max.bytes` 不包括消息的头部和元数据，只限制消息体部分的大小；
    /// - 与 `max.partition.fetch.bytes` 配合使用时，`max.partition.fetch.bytes` 会限制单个分区的最大数据量，而 `fetch.max.bytes` 会限制整个消费者的最大数据量；
    /// - 如果在消费者配置中设置了较大的 `fetch.max.bytes`，请确保消费者能够处理大规模的数据量，避免内存溢出或崩溃。
    #[serde(rename = "fetch.max.bytes")]
    pub fetch_max_bytes: i32,

    /// ### isolation.level
    ///
    /// > 定义消费者如何读取 Kafka 中的消息，影响消息的可见性和处理的一致性。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - `isolation.level` 参数控制消费者读取消息的方式，主要有两种模式：
    ///   - `read_uncommitted`：消费者可以读取已提交和未提交的消息；
    ///   - `read_committed`：消费者只会读取已提交的消息，忽略正在进行事务的未提交消息。
    /// - 当 Kafka 使用事务时，`read_committed` 模式能确保消费者只读取已完成的事务数据，避免读取到中途失败的事务。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为 `read_uncommitted`，适用于绝大多数应用，性能上通常优于 `read_committed`；
    /// - 如果需要确保数据一致性并避免读取到未完成的事务数据，应使用 `read_committed`；
    /// - 对于需要确保消费者读取已提交事务消息的应用，如金融交易系统或高一致性要求的系统，推荐使用 `read_committed`。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - `read_committed` 会带来一定的性能开销，因为 Kafka 必须确保消费者读取的是已提交的数据；
    /// - 使用 `read_committed` 时，必须确保 Kafka 配置和事务管理正确，避免消费者读取到不一致的数据；
    /// - 如果系统对事务一致性要求较高，推荐使用 `read_committed`，否则可以选择 `read_uncommitted` 来提高性能。
    #[serde(rename = "isolation.level")]
    pub isolation_level: IsolationLevel,

    /// ### max.poll.interval.ms
    ///
    /// > 定义消费者调用 `poll()` 方法之间允许的最大时间间隔（毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - `max.poll.interval.ms` 控制的是消费者在两次 `poll()` 调用之间的最大时间间隔；
    /// - 如果消费者在规定时间内没有调用 `poll()`，消费者会被认为失效，Kafka 会触发重新平衡（rebalance）；
    /// - 该参数用于防止消费者长时间无响应，确保群组协调器能够及时发现并处理失效消费者。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为 `300000` 毫秒（5 分钟）；
    /// - 通常情况下，`max.poll.interval.ms` 不应设置得过小，因为这会导致消费者在长时间处理任务时被误认为失效；
    /// - 设置较大的值可以防止长时间处理任务的消费者被强制重新平衡，但也可能影响群组的整体性能。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 该参数的设置应考虑消费者的处理能力，尤其是当消费者处理复杂任务或需要较长时间来处理数据时；
    /// - 如果消费者的 `poll()` 调用之间时间间隔过长，可能会影响 Kafka 群组的稳定性，导致频繁的重新平衡；
    /// - 应避免将该值设置得过大，以免导致失效的消费者长时间不被检测到。
    #[serde(rename = "max.poll.interval.ms")]
    pub max_poll_interval_ms: i32,

    /// ### partition.assignment.strategy
    ///
    /// > 消费者用于分配分区的策略。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - 消费者需要从 Kafka 集群中分配分区，以便从中拉取消息；
    /// - `partition.assignment.strategy` 控制消费者使用的分区分配策略；
    /// - Kafka 支持多种分配策略，常见的包括 `range` 和 `roundrobin`：
    ///   - `range`: 按照分区范围进行分配，通常适用于每个消费者有较均衡的数据负载的场景；
    ///   - `roundrobin`: 按照轮询方式进行分配，通常用于有较多消费者的场景，确保负载均衡。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值通常为 `range`，适合多数常见场景；
    /// - 如果消费者数目变化频繁，或者存在不均衡的负载，建议使用 `roundrobin`，以避免分配过程中产生较大的不平衡；
    /// - 若希望使用自定义策略，可以通过实现 `PartitionAssignor` 接口来扩展分配策略。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 分配策略仅在消费者组内的分区重新分配时生效，例如消费者加入或离开时；
    /// - 不同策略的性能和行为差异较大，选择时应考虑消费负载的均衡性；
    /// - 适当的策略选择可以减少 rebalance 的频率，避免因频繁的分区分配导致消费者性能下降。
    #[serde(rename = "partition.assignment.strategy")]
    pub partition_assignment_strategy: String,
}

impl Default for ConsumerConfigMediumLevel {
    fn default() -> Self {
        Self {
            auto_offset_reset: AutoOffsetReset::Latest,
            enable_auto_commit: true,
            fetch_max_bytes: 52428800,
            isolation_level: IsolationLevel::ReadUncommitted,
            max_poll_interval_ms: 300000,
            partition_assignment_strategy: "range".to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsumerConfigLowLevel {
    /// ### auto.commit.interval.ms
    ///
    /// > 自动提交偏移量的时间间隔（毫秒）。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka 消费者可以自动提交消费的偏移量；
    /// - `auto.commit.interval.ms` 定义了自动提交偏移量的间隔时间，默认值通常为 `5000` 毫秒（5 秒）；
    /// - 如果启用了自动提交（`enable.auto.commit=true`），消费者将定期提交偏移量。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为 `5000` 毫秒（5 秒）；
    /// - 如果消费者处理速度较慢，可以适当增加该值，以避免频繁的偏移量提交；
    /// - 如果消费者需要严格的消息顺序性，可以适当减少该值，以便及时提交偏移量。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 提交偏移量的频率过高可能会增加 Kafka 集群的负载，尤其是在消费者处理大量消息时；
    /// - 提交偏移量的频率过低可能导致消费者重新处理已经消费的消息，尤其是在消费者崩溃后；
    /// - `auto.commit.interval.ms` 与 `enable.auto.commit` 配合使用时生效，如果禁用自动提交，则该参数不会起作用。
    #[serde(rename = "auto.commit.interval.ms")]
    pub auto_commit_interval_ms: i32,

    /// ### check.crcs
    ///
    /// > 启用或禁用消息的 CRC 校验。
    ///
    /// ---
    ///
    /// #### 工作原理
    /// - Kafka 客户端在接收到消息时，会进行 CRC 校验，确保消息在传输过程中没有损坏；
    /// - 如果启用此选项，消费者将在每次获取消息时验证其校验和；
    /// - 启用此选项可以增加数据的完整性验证，但也会带来额外的性能开销。
    ///
    /// ---
    ///
    /// #### 使用建议
    /// - 默认值为 `true`，表示启用 CRC 校验；
    /// - 如果对数据完整性有较高要求，可以保持默认启用；
    /// - 如果性能是首要考虑，可以考虑禁用此选项（`check.crcs=false`），但会牺牲部分数据验证功能。
    ///
    /// ---
    ///
    /// #### 注意事项
    /// - 禁用 CRC 校验可能会导致数据损坏或传输错误时无法及时发现；
    /// - 启用 CRC 校验会稍微增加消息的处理延迟，因为每条消息都需要进行校验；
    /// - 如果禁用 CRC 校验，仍然可以依赖 Kafka 内部的其他机制来处理数据完整性问题。
    #[serde(rename = "check.crcs")]
    pub check_crcs: bool,

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
}

impl Default for ConsumerConfigLowLevel {
    fn default() -> Self {
        Self {
            auto_commit_interval_ms: 5000,
            check_crcs: true,
            client_id: "".to_string(),
            reconnect_backoff_max_ms: 1000,
            reconnect_backoff_ms: 50,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsumerConfig {
    #[serde(flatten)]
    pub high: ConsumerConfigHighLevel,
    #[serde(flatten)]
    pub medium: ConsumerConfigMediumLevel,
    #[serde(flatten)]
    pub low: ConsumerConfigLowLevel,
}

#[cfg(test)]
mod tests {
    use super::ConsumerConfig;

    #[test]
    fn serde_consumer_config() {
        let orig = ConsumerConfig::default();
        let json = serde_json::to_string_pretty(&orig).unwrap();
        println!("{}", json);

        let new: ConsumerConfig = serde_json::from_str(&json).unwrap();
        println!("{:?}", new);
        assert_eq!(orig, new);
    }
}
