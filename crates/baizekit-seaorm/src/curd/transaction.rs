use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

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
