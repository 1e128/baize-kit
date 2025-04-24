use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use snafu::ResultExt;

use crate::config::{ProducerConfig, ToRdkafkaConfig};
use crate::error::{KafkaSnafu, RecvSnafu, Result, SendSnafu};

const DEFAULT_FLUSH_TIMEOUT: Duration = Duration::from_millis(5000);

enum Command {
    SendWithFlush { message: Message, respond_to: Sender<Result<()>>, flush_timeout: Option<Duration> },
    Shutdown,
}

pub struct Message {
    topic: String,
    key: Option<String>,
    payload: String,
}

struct ProducerActor {
    receiver: Receiver<Command>,
    producer: BaseProducer,
}

impl ProducerActor {
    fn new(receiver: Receiver<Command>, producer: BaseProducer) -> Self {
        Self { receiver, producer }
    }

    fn run(&mut self) {
        tracing::info!("producer actor start");
        for cmd in self.receiver.iter() {
            match cmd {
                Command::SendWithFlush { message, respond_to, flush_timeout } => {
                    let result = self.send_with_flush(message, flush_timeout.unwrap_or(DEFAULT_FLUSH_TIMEOUT));
                    if let Err(err) = respond_to.send(result) {
                        tracing::error!(?err, "send respond_to msg failed")
                    }
                }
                Command::Shutdown => break,
            }
        }
        tracing::info!("producer actor exit");
    }

    fn send_with_flush(&self, msg: Message, flush_timeout: Duration) -> Result<()> {
        let mut record = BaseRecord::to(&msg.topic).payload(&msg.payload);
        if let Some(key) = &msg.key {
            record = record.key(key);
        }

        self.producer.send(record).map_err(|e| e.0).context(KafkaSnafu)?;
        self.producer.flush(flush_timeout).context(KafkaSnafu)?;
        Ok(())
    }
}

/// Kafka producer 类似 actor，单线程执行发送逻辑
#[derive(Clone)]
pub struct SyncProducer {
    sender: Sender<Command>,
}

impl SyncProducer {
    pub fn try_new(cfg: ProducerConfig) -> Result<Self> {
        let (tx, receiver) = mpsc::channel();

        let producer: BaseProducer = cfg.to_rdkafka_config()?.create().context(KafkaSnafu)?;

        thread::spawn(move || {
            let mut producer = ProducerActor::new(receiver, producer);
            producer.run();
        });

        Ok(SyncProducer { sender: tx })
    }

    pub fn send(&self, topic: String, key: String, payload: String) -> Result<()> {
        let (resp_tx, resp_rx) = mpsc::channel();

        let message = Message { topic, key: Some(key), payload };

        self.sender
            .send(Command::SendWithFlush { message, respond_to: resp_tx, flush_timeout: None })
            .map_err(|_| SendSnafu { message: "send message failed" }.build())?;

        resp_rx.recv().map_err(|e| RecvSnafu { message: e.to_string() }.build())?
    }

    pub fn close(&self) {
        self.sender.send(Command::Shutdown).ok();
    }
}
