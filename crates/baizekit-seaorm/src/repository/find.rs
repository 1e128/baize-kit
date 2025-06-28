use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, Select};

use super::RepositoryTrait;

/// find
#[async_trait]
pub trait FindTrait<DB, Entity>: RepositoryTrait<DB>
where
    DB: ConnectionTrait,
    Entity: EntityTrait,
{
    /// 查询结果
    type Data: From<Entity::Model>;
    /// 查询错误
    type Error: From<DbErr>;
    /// 查询器
    type Filter;

    async fn find<Param>(&self, param: Param) -> Result<Option<Self::Data>, Self::Error>
    where
        Param: Send,
        Self::Filter: From<Param>,
        Select<Entity>: From<Self::Filter>,
    {
        let filter: Self::Filter = Self::Filter::from(param);
        let select = Select::<Entity>::from(filter);
        select
            .one(self.db())
            .await
            .map(|v| v.map(Self::Data::from))
            .map_err(Self::Error::from)
    }
}
