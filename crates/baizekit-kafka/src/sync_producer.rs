use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use rdkafka::producer::{BaseProducer, BaseRecord, Producer};
use snafu::ResultExt;

use crate::error::{KafkaSnafu, RecvSnafu, Result, SendSnafu};
use crate::ProducerConfig;

enum Command {
    SendWithFlush(Message, Sender<Result<()>>),
}

pub struct Message {
    topic: String,
    key: Option<String>,
    payload: String,
}

struct ProducerActor {
    receiver: Receiver<Command>,
    producer: BaseProducer,
    flush_timeout: Duration,
}

impl ProducerActor {
    fn new(receiver: Receiver<Command>, producer: BaseProducer, flush_timeout: Duration) -> Self {
        Self { receiver, producer, flush_timeout }
    }

    fn run(&mut self) {
        tracing::info!("producer actor start");
        for msg in self.receiver.iter() {
            self.handle(msg)
        }
        tracing::info!("producer actor exit");
    }

    fn handle(&self, cmd: Command) {
        match cmd {
            Command::SendWithFlush(msg, respond_to) => {
                let result = self.send_with_flush(msg);
                if let Err(err) = respond_to.send(result) {
                    tracing::error!(?err, "send respond_to msg failed")
                }
            }
        }
    }

    fn send_with_flush(&self, msg: Message) -> Result<()> {
        let mut record = BaseRecord::to(&msg.topic).payload(&msg.payload);
        if let Some(key) = &msg.key {
            record = record.key(key);
        }

        self.producer.send(record).map_err(|e| e.0).context(KafkaSnafu)?;
        self.producer.flush(self.flush_timeout).context(KafkaSnafu)?;
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

        let producer: BaseProducer = cfg.to_rdkafka_config().create().context(KafkaSnafu)?;
        let flush_timeout = cfg.flush_timeout();

        thread::spawn(move || {
            let mut producer = ProducerActor::new(receiver, producer, flush_timeout);
            producer.run();
        });

        Ok(SyncProducer { sender: tx })
    }

    pub fn send(&self, topic: String, key: String, payload: String) -> Result<()> {
        let (resp_tx, resp_rx) = mpsc::channel();

        let msg = Message { topic, key: Some(key), payload };

        self.sender
            .send(Command::SendWithFlush(msg, resp_tx))
            .map_err(|_| SendSnafu { message: "send message failed" }.build())?;

        resp_rx.recv().map_err(|e| RecvSnafu { message: e.to_string() }.build())?
    }
}
