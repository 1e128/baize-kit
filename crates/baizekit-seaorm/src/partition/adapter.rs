use sea_orm::prelude::async_trait::async_trait;
use sea_orm::DbErr;

/// 分区类型
#[derive(Debug, Clone)]
pub enum PartitionStrategy {
    Range { start: String, end: String },
    List { values: Vec<String> },
    Hash { modulus: u32, remainder: u32 },
}

pub struct PartitionOptions {
    /// 表名
    pub table_name: String,
    /// 分区名称
    pub partition_name: String,
    /// 分区策略
    pub strategy: PartitionStrategy,
}

#[async_trait]
pub trait PartitionAdapter {
    /// 查询分区列表
    async fn query(&self, table_name: String) -> Result<Vec<String>, DbErr>;

    /// 创建分区
    async fn create(&self, partition: PartitionOptions) -> Result<(), DbErr>;

    /// 删除分区
    async fn drop(&self, partition_name: String) -> Result<(), DbErr>;
}
