use rdkafka::error::KafkaError;
use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source}"))]
    KafkaError {
        source: KafkaError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("SendError: {message}"))]
    SendError {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("RecvError: {message}"))]
    RecvError {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
