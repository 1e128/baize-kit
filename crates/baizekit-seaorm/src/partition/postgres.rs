use std::sync::Arc;

use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, FromQueryResult, Statement, Value};

use super::{PartitionAdapter, PartitionOptions, PartitionStrategy};

#[derive(Clone)]
pub struct PostgresPartitionAdapter {
    pub db: Arc<DatabaseConnection>,
}

impl PostgresPartitionAdapter {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PartitionAdapter for PostgresPartitionAdapter {
    async fn query(&self, table_name: String) -> Result<Vec<String>, DbErr> {
        #[derive(Debug, FromQueryResult)]
        struct PartitionRow {
            partition_name: String,
        }

        let sql = r#"
            SELECT child.relname AS partition_name
            FROM pg_inherits
            JOIN pg_class parent ON pg_inherits.inhparent = parent.oid
            JOIN pg_class child ON pg_inherits.inhrelid = child.oid
            WHERE parent.relname = $1
            ORDER BY child.relname;
        "#;

        let rows: Vec<PartitionRow> = PartitionRow::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            sql,
            vec![Value::String(Some(Box::new(table_name)))],
        ))
        .all(self.db.as_ref())
        .await?;

        Ok(rows.into_iter().map(|r| r.partition_name).collect())
    }

    async fn create(&self, partition: PartitionOptions) -> Result<(), DbErr> {
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

        self.db.execute_unprepared(&sql).await?;
        Ok(())
    }

    async fn drop(&self, partition_name: String) -> Result<(), DbErr> {
        let sql = format!("DROP TABLE IF EXISTS {partition_name} CASCADE;");
        self.db.execute_unprepared(&sql).await?;
        Ok(())
    }
}
