mod adapter;
mod postgres;

pub use adapter::*;
pub use postgres::*;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sea_orm::Database;

    use crate::partition::{PartitionAdapter, PartitionOptions, PartitionStrategy, PostgresPartitionAdapter};

    #[tokio::test]
    async fn test_create_partition() -> Result<(), Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").unwrap();
        let pg_pool = Arc::new(Database::connect(&url).await?);

        let adapter = PostgresPartitionAdapter::new(pg_pool);

        let table_name = "xxx".to_string();
        let partition_name = format!("{}_{}", &table_name, "202001");
        println!("table_name: {}", table_name);
        println!("partition_name: {}", partition_name);
        let options = PartitionOptions {
            table_name: table_name.to_string(),
            partition_name: partition_name.to_string(),
            strategy: PartitionStrategy::Range { start: "20200101".to_string(), end: "20200201".to_string() },
        };
        adapter.create(options).await.unwrap();

        let partitions = adapter.query(table_name).await.unwrap();
        println!("{:?}", partitions);

        assert!(partitions.iter().any(|partition| partition == &partition_name));
        Ok(())
    }

    #[tokio::test]
    async fn test_drop_partition() -> Result<(), Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").unwrap();
        let pg_pool = Arc::new(Database::connect(&url).await?);
        let adapter = PostgresPartitionAdapter::new(pg_pool);

        let table_name = "xxx".to_string();
        let partition_name = format!("{}_{}", &table_name, "202001");
        adapter.drop(partition_name.clone()).await.unwrap();

        let partitions = adapter.query(table_name).await.unwrap();
        println!("{:?}", partitions);

        assert!(partitions.is_empty());
        Ok(())
    }
}
