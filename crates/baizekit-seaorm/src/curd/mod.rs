use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
pub use futures_util::stream::BoxStream;
pub use futures_util::{StreamExt, TryStreamExt};
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

#[cfg(feature = "derive")]
mod derive_impl;
#[cfg(feature = "derive")]
mod derive_options;

#[cfg(feature = "derive")]
pub use derive_impl::*;
#[cfg(feature = "derive")]
pub use derive_options::*;

pub trait Transaction: Send + Sync {
    fn as_any(&mut self) -> &mut dyn Any;
}

#[async_trait]
pub trait TransactionManager {
    type Tx: Transaction + Send + Sync;

    async fn begin_transaction(&self) -> Result<Self::Tx, DbErr>;

    async fn commit(&self, tx: Self::Tx) -> Result<(), DbErr>;

    async fn rollback(&self, tx: Self::Tx) -> Result<(), DbErr>;
}

pub struct SeaOrmTransaction {
    tx: DatabaseTransaction,
}

impl SeaOrmTransaction {
    pub fn inner(&mut self) -> &mut DatabaseTransaction {
        &mut self.tx
    }
}

impl Transaction for SeaOrmTransaction {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct SeaOrmTransactionManager {
    db: Arc<DatabaseConnection>,
}

#[async_trait]
impl TransactionManager for SeaOrmTransactionManager {
    type Tx = SeaOrmTransaction;

    async fn begin_transaction(&self) -> Result<Self::Tx, DbErr> {
        let tx = self.db.begin().await?;
        Ok(SeaOrmTransaction { tx })
    }

    async fn commit(&self, tx: Self::Tx) -> Result<(), DbErr> {
        // 直接消耗整个 SeaOrmTransaction
        tx.tx.commit().await
    }

    async fn rollback(&self, tx: Self::Tx) -> Result<(), DbErr> {
        // 直接消耗整个 SeaOrmTransaction
        tx.tx.rollback().await
    }
}

/// 分页参数
#[derive(Debug, Clone, Copy)]
pub enum Pagination {
    /// 偏移分页 (页码，每页大小)
    Offset(u64, u64),
    /// 游标分页 (每页大小)
    Cursor(u64),
}

/// 可分页的过滤器trait
pub trait PaginatedFilter {
    /// 获取分页参数
    fn pagination(&self) -> Option<Pagination>;
}

#[async_trait::async_trait]
pub trait FindTrait<D, Er, F>: Send {
    async fn find(&self, filter: F, tx: Option<&mut dyn Transaction>) -> Result<Option<D>, Er>;
}

#[async_trait::async_trait]
pub trait SearchTrait<D, Er, F>: Send {
    async fn search(&self, filter: F) -> Result<(Vec<D>, u64, bool), Er>;
}

#[async_trait::async_trait]
pub trait SearchStreamTrait<D, Er, F>: Send {
    async fn stream(&self, filter: F) -> Result<BoxStream<'static, Result<D, Er>>, Er>;
}

#[async_trait::async_trait]
pub trait InsertTrait<D, Er>: Send {
    async fn insert(&self, domain_entity: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>;
}

#[async_trait::async_trait]
pub trait DeleteTrait<D, Er>: Send {
    async fn delete(&self, domain_entity: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>;
}

#[async_trait::async_trait]
pub trait UpdateTrait<D, Er>: Send {
    async fn update(&self, domain_entity: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>;
}

#[async_trait::async_trait]
pub trait SaveTrait<D, Er>: Send {
    async fn save(&self, domain_entity: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>;
}

#[async_trait::async_trait]
pub trait BulkInsertTrait<D, Er>: Send {
    async fn bulk_insert(&self, domain_entities: Vec<D>, tx: Option<&mut dyn Transaction>) -> Result<Vec<D>, Er>;
}
