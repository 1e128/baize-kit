use sea_orm::prelude::async_trait::async_trait;
use sqlx::{PgPool, Row};

use super::{PartitionAdapter, PartitionOptions, PartitionStrategy};
use crate::connection::{try_new_pg_pool, Config};

#[derive(Clone)]
pub struct PostgresPartitionAdapter {
    pub pool: PgPool,
}

impl PostgresPartitionAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn try_new_from_config(cfg: Config) -> Result<Self, sqlx::Error> {
        let pool = try_new_pg_pool(cfg).await?;
        Ok(Self::new(pool))
    }
}

#[async_trait]
impl PartitionAdapter for PostgresPartitionAdapter {
    async fn query(&self, table_name: String) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT child.relname AS partition_name
            FROM pg_inherits
            JOIN pg_class parent ON pg_inherits.inhparent = parent.oid
            JOIN pg_class child ON pg_inherits.inhrelid = child.oid
            WHERE parent.relname = $1
            ORDER BY child.relname;
            "#,
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let partitions = rows
            .into_iter()
            .filter_map(|row| row.try_get::<String, _>("partition_name").ok())
            .collect();
        Ok(partitions)
    }

    async fn create(&self, partition: PartitionOptions) -> Result<(), sqlx::Error> {
        let strategy_sql = match partition.strategy {
            PartitionStrategy::Range { start, end } => {
                format!("FOR VALUES FROM ('{start}') TO ('{end}')")
            }
            PartitionStrategy::List { values } => {
                let value_list = values.into_iter().map(|v| format!("('{}')", v)).collect::<Vec<_>>().join(", ");
                format!("FOR VALUES IN ({value_list})")
            }
            PartitionStrategy::Hash { modulus, remainder } => {
                format!("FOR VALUES WITH (MODULUS {modulus}, REMAINDER {remainder})")
            }
        };

        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {partition_name}
            PARTITION OF {table_name}
            {strategy};
            "#,
            partition_name = partition.partition_name,
            table_name = partition.table_name,
            strategy = strategy_sql,
        );

        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    async fn drop(&self, partition_name: String) -> Result<(), sqlx::Error> {
        let sql = format!("DROP TABLE IF EXISTS {partition_name} CASCADE;");
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }
}
