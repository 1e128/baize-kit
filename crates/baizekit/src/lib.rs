pub use {sea_orm, tracing};

pub mod api {
    #[cfg(feature = "http-build")]
    pub use baizekit_api::build::*;
    pub use baizekit_api::extract::*;
    pub use baizekit_api::response::*;
}

pub mod app {
    pub use baizekit_app::*;
}

pub mod log {
    pub use baizekit_log::config::LogConfig;
    pub use baizekit_log::format::LogFormat;
}

pub mod db {
    pub use baizekit_seaorm::connection::{Config, LevelFilter};
    pub use baizekit_seaorm::curd;
    #[cfg(feature = "db-migration")]
    pub use baizekit_seaorm::define_sea_orm_cli;
    #[cfg(feature = "db-migration")]
    pub use baizekit_seaorm::migration;
    #[cfg(feature = "db-partition")]
    pub use baizekit_seaorm::partition;
}

pub mod component {
    pub use baizekit_api::component::axum::{AxumComponent, AxumComponentConfig, AxumServiceInfo};
    pub use baizekit_app::component::{Component, ComponentFactory, DynComponent};
    pub use baizekit_log::component::LogComponent;
    pub use baizekit_seaorm::component::DbComponent;
}

#[cfg(feature = "derive")]
pub mod derive {
    pub use baizekit_derive::*;
    pub use baizekit_seaorm::curd::derive::*;
}

#[cfg(feature = "serde")]
pub mod serde {
    pub use baizekit_serde::prelude::*;
}

#[cfg(feature = "redis")]
pub mod redis {
    pub use baizekit_redis::*;
}

#[cfg(feature = "kafka")]
pub mod kafka {
    pub use baizekit_kafka::*;
}
