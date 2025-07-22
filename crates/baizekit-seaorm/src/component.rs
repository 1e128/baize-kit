use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use baizekit_app::application::ComponentContext;
use baizekit_app::async_trait::async_trait;
use baizekit_app::component::Component;
use baizekit_app::error::{ConfigSnafu, InternalSnafu, Result, ResultExt};
use sea_orm::{Database, DatabaseConnection};
use tracing::info;

use crate::connection;

pub struct DbComponent {
    pub db: Arc<DatabaseConnection>,
    pub connections: HashMap<String, Arc<DatabaseConnection>>,
}

impl DbComponent {
    pub fn new<'a>(ctx: &'a ComponentContext<'a>) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 'a>> {
        Box::pin(async move {
            let conf = ctx.config();
            let db_conf: connection::Config = conf.get("db").context(ConfigSnafu)?;
            info!(dsn_url = db_conf.url, search_path = ?db_conf.schema, "连接数据库");
            let db = Database::connect(db_conf)
                .await
                .map(Arc::new)
                .map_err(|err| InternalSnafu { message: err.to_string() }.build())?;
            Ok(DbComponent { db, connections: Default::default() })
        })
    }

    pub fn new_multi_connections<'a>(
        ctx: &'a ComponentContext<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 'a>> {
        Box::pin(async move {
            let conf = ctx.config();

            // 默认数据库连接
            let db_conf: connection::Config = conf.get("db").context(ConfigSnafu)?;
            let db = Database::connect(db_conf)
                .await
                .map(Arc::new)
                .map_err(|err| InternalSnafu { message: err.to_string() }.build())?;

            // 带有label信息的数据库连接
            let mut connections = HashMap::new();
            let db_confs: HashMap<String, connection::Config> = conf.get("dbs").context(ConfigSnafu)?;
            for (label, db_conf) in db_confs {
                let db = Database::connect(db_conf)
                    .await
                    .map(Arc::new)
                    .map_err(|err| InternalSnafu { message: err.to_string() }.build())?;
                connections.insert(label, db);
            }

            Ok(DbComponent { db, connections })
        })
    }

    pub fn get_default_connection(&self) -> Arc<DatabaseConnection> {
        self.db.clone()
    }

    pub fn get_connection(&self, label: Option<&str>) -> Option<Arc<DatabaseConnection>> {
        let Some(label) = label else {
            return Some(self.db.clone());
        };

        self.connections.get(label).cloned()
    }
}

#[async_trait]
impl Component for DbComponent {}
