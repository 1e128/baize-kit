use std::path::PathBuf;

use config::ConfigError;
pub use snafu::ResultExt;
use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum FrameworkError {
    #[snafu(display("{}", message))]
    InternalError {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("{}", message))]
    ComponentError {
        message: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(""))]
    ConfigError {
        source: ConfigError,
        #[snafu(implicit)]
        location: Location,
    },

    /// I/O operation failed
    #[snafu(display(""))]
    IoError {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(""))]
    ParseError {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display(""))]
    PathError {
        name: Option<PathBuf>,
        #[snafu(implicit)]
        location: Location,
    },
}

pub type Result<T, E = FrameworkError> = std::result::Result<T, E>;
