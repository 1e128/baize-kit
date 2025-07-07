#[cfg(feature = "derive")]
mod derive_impl;
#[cfg(feature = "derive")]
pub mod derive_options;

use async_trait::async_trait;
#[cfg(feature = "derive")]
pub use derive_impl::*;
#[cfg(feature = "derive")]
pub use derive_options::*;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};
use std::any::Any;
use std::sync::Arc;
pub use futures_util::stream::BoxStream;
pub use futures_util::{StreamExt, TryStreamExt};

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

#[async_trait]
pub trait FindTrait<D, Er, F>: Send {
    async fn find(&self, filter: F, tx: Option<&mut dyn Transaction>) -> Result<Option<D>, Er>;
}

#[async_trait]
pub trait SearchTrait<D, Er, F>: Send {
    async fn search(&self, filter: F) -> Result<(Vec<D>, u64, bool), Er>;
}

#[async_trait]
pub trait InsertTrait<D, Er>: Send {
    async fn insert(&self, data: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>;
}


///// 插入数据, 失败或者冲突时返回错误
//     async fn insert<E, D, Er>(&self, data: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>
//     where
//         E: EntityTrait,
//         D: From<E::Model>,
//         Er: From<DbErr>,
//         D: Into<E::ActiveModel> + From<E::Model> + 'static,
//         E::Model: IntoActiveModel<E::ActiveModel>,
//     {
//         let insert = E::insert(data.into());
//         match tx {
//             Some(tx) => {
//                 let tx = tx
//                     .as_any()
//                     .downcast_mut::<SeaOrmTransaction>()
//                     .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
//                 insert.exec_with_returning(tx.inner()).await
//             }
//             None => insert.exec_with_returning(&*self.db).await,
//         }
//         .map(D::from)
//         .map_err(Er::from)
//     }