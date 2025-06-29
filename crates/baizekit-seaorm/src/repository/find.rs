use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, Select};

use super::RepositoryTrait;

/// find
#[async_trait]
pub trait FindTrait<'db, DB, Entity>: RepositoryTrait<'db, DB>
where
    DB: ConnectionTrait + 'db,
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
        Param: Into<Self::Filter> + Send,
        Select<Entity>: From<Self::Filter>,
    {
        let filter = param.into();
        let select = Select::<Entity>::from(filter);
        select
            .one(self.db())
            .await
            .map(|v| v.map(Self::Data::from))
            .map_err(Self::Error::from)
    }
}
