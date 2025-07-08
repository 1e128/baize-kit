use futures_util::stream::BoxStream;

use crate::curd::transaction::Transaction;

/// 分页参数
#[derive(Copy, Clone, Debug)]
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
pub trait FindTrait<Entity, Err, Filter>: Send + Sync
where
    Filter: Send + 'static,
{
    #[inline(always)]
    async fn find(&self, filter: Filter) -> Result<Option<Entity>, Err> {
        self.find_with_tx(filter, None).await
    }

    async fn find_with_tx(&self, filter: Filter, tx: Option<&mut dyn Transaction>) -> Result<Option<Entity>, Err>;
}

#[async_trait::async_trait]
pub trait SearchTrait<Entity, Err, Filter>: Send + Sync
where
    Filter: PaginatedFilter + Send + 'static,
{
    async fn search(&self, filter: Filter) -> Result<(Vec<Entity>, u64, bool), Err>;
}

#[async_trait::async_trait]
pub trait SearchStreamTrait<D, Er, F>: Send + Sync {
    async fn stream(&self, filter: F) -> BoxStream<'static, Result<D, Er>>;
}

#[async_trait::async_trait]
pub trait InsertTrait<Entity, Err>: Send + Sync
where
    Entity: Send + 'static,
{
    #[inline(always)]
    async fn insert(&self, domain_entity: Entity) -> Result<Entity, Err> {
        self.insert_with_tx(domain_entity, None).await
    }

    async fn insert_with_tx(&self, domain_entity: Entity, tx: Option<&mut dyn Transaction>) -> Result<Entity, Err>;
}

#[async_trait::async_trait]
pub trait DeleteTrait<Entity, Error>: Send + Sync
where
    Entity: Send + 'static,
{
    #[inline(always)]
    async fn delete(&self, domain_entity: Entity) -> Result<Entity, Error> {
        self.delete_with_tx(domain_entity, None).await
    }

    async fn delete_with_tx(&self, _domain_entity: Entity, _tx: Option<&mut dyn Transaction>) -> Result<Entity, Error>;
}

#[async_trait::async_trait]
pub trait UpdateTrait<Entity, Err>: Send + Sync
where
    Entity: Send + 'static,
{
    #[inline(always)]
    async fn update(&self, domain_entity: Entity) -> Result<Entity, Err> {
        self.update_with_tx(domain_entity, None).await
    }

    async fn update_with_tx(&self, domain_entity: Entity, tx: Option<&mut dyn Transaction>) -> Result<Entity, Err>;
}

#[async_trait::async_trait]
pub trait UpsertTrait<Entity, Err>: Send + Sync
where
    Entity: Send + 'static,
{
    #[inline(always)]
    async fn upsert(&self, domain_entity: Entity) -> Result<Entity, Err> {
        self.upsert_with_tx(domain_entity, None).await
    }

    async fn upsert_with_tx(&self, domain_entity: Entity, tx: Option<&mut dyn Transaction>) -> Result<Entity, Err>;
}

#[async_trait::async_trait]
pub trait BulkInsertTrait<Entity, Err>: Send + Sync
where
    Entity: Send + 'static,
{
    #[inline(always)]
    async fn bulk_insert(&self, entities: Vec<Entity>) -> Result<(), Err> {
        self.bulk_insert_with_tx(entities, None).await
    }

    async fn bulk_insert_with_tx(&self, entities: Vec<Entity>, tx: Option<&mut dyn Transaction>) -> Result<(), Err>;
}

#[async_trait::async_trait]
pub trait BulkUpsertTrait<Entity, Err>: Send + Sync
where
    Entity: Send + 'static,
{
    #[inline(always)]
    async fn bulk_upsert(&self, entities: Vec<Entity>) -> Result<(), Err> {
        self.bulk_upsert_with_tx(entities, None).await
    }

    async fn bulk_upsert_with_tx(&self, entities: Vec<Entity>, tx: Option<&mut dyn Transaction>) -> Result<(), Err>;
}
