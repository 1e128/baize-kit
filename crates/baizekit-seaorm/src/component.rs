use std::collections::HashMap;
use std::sync::Arc;

use crate::connection;
use baizekit_app::application::ApplicationInner;
use baizekit_app::async_trait::async_trait;
use baizekit_app::component::Component;
use sea_orm::{Database, DatabaseConnection};
use tracing::info;

pub struct DbComponent {
    pub db: Arc<DatabaseConnection>,
    pub connections: HashMap<String, Arc<DatabaseConnection>>,
}

impl DbComponent {
    pub async fn new(inner: Arc<ApplicationInner>, _label: String) -> baizekit_app::anyhow::Result<Self> {
        let conf = inner.config().await;
        let db_conf: connection::Config = conf.get("db")?;
        info!(dsn_url = db_conf.url, search_path = ?db_conf.schema, "连接数据库");
        let db = Database::connect(db_conf).await.map(Arc::new)?;
        Ok(DbComponent { db, connections: Default::default() })
    }

    pub async fn new_multi_connections(
        inner: Arc<ApplicationInner>,
        _label: String,
    ) -> baizekit_app::anyhow::Result<Self> {
        let conf = inner.config().await;

        // 默认数据库连接
        let db_conf: connection::Config = conf.get("db")?;
        let db = Database::connect(db_conf).await.map(Arc::new)?;

        // 带有label信息的数据库连接
        let mut connections = HashMap::new();
        let db_confs: HashMap<String, connection::Config> = conf.get("dbs")?;
        for (label, db_conf) in db_confs {
            let db = Database::connect(db_conf).await.map(Arc::new)?;
            connections.insert(label, db);
        }

        Ok(DbComponent { db, connections })
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
