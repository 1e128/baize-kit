use sea_orm::prelude::async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, Iterable, TransactionTrait, TryInsertResult};

use crate::repository::RepositoryTrait;

#[async_trait]
pub trait UpsertTrait<'db, DB, Entity>: RepositoryTrait<'db, DB>
where
    DB: ConnectionTrait + 'db,
    Entity: EntityTrait,
    Entity::Model: IntoActiveModel<Entity::ActiveModel>,
    Entity::ActiveModel: Send,
{
    type Data: Into<Entity::ActiveModel> + From<Entity::Model> + Send + 'static;
    type Error: From<DbErr>;

    /// 插入数据, 失败或者冲突时返回错误
    async fn insert(&self, entity: Self::Data) -> Result<Self::Data, Self::Error> {
        Entity::insert(entity.into())
            .exec_with_returning(self.db())
            .await
            .map(Self::Data::from)
            .map_err(Self::Error::from)
    }

    /// 更新数据, 找不到数据时返回错误
    async fn update(&self, entity: Self::Data) -> Result<Self::Data, Self::Error> {
        Entity::update(entity.into())
            .exec(self.db())
            .await
            .map(Self::Data::from)
            .map_err(Self::Error::from)
    }

    /// 保存数据, 找不到数据时插入数据, 找到数据时更新数据
    async fn save(&self, entity: Self::Data, on_conflict: OnConflict) -> Result<Self::Data, Self::Error> {
        Entity::insert(entity.into())
            .on_conflict(on_conflict)
            .exec_with_returning(self.db())
            .await
            .map(Self::Data::from)
            .map_err(Self::Error::from)
    }
}

fn calc_chunk_size<Entity: EntityTrait>() -> usize {
    // https://github.com/launchbadge/sqlx/issues/3464
    // https://www.postgresql.org/docs/current/limits.html
    let chunk_size = (u16::MAX / Entity::Column::iter().len() as u16) as usize;
    assert!(chunk_size > 0, "chunk_size must be greater than 0");
    chunk_size
}

#[async_trait]
pub trait BulkUpsertTrait<'db, DB, Entity>: RepositoryTrait<'db, DB>
where
    DB: ConnectionTrait + TransactionTrait + Send + 'db,
    Entity: EntityTrait,
    Entity::Model: IntoActiveModel<Entity::ActiveModel>,
    Entity::ActiveModel: Send,
{
    type Data: Into<Entity::ActiveModel> + From<Entity::Model> + Send + 'static;
    type Error: From<DbErr>;

    /// 插入数据, 失败或者冲突时返回错误
    async fn bulk_insert(&self, entities: Vec<Self::Data>) -> Result<(), Self::Error> {
        let chunk_size = calc_chunk_size::<Entity>();

        let models = entities.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut models_iter = models.into_iter();

        let tx = self.db().begin().await?;
        loop {
            let chunk_n: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
            if chunk_n.is_empty() {
                break;
            }

            Entity::insert_many(chunk_n).on_conflict_do_nothing().exec(&tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    /// 批量插入数据
    async fn bulk_insert_with_returning(&self, entities: Vec<Self::Data>) -> Result<Vec<Self::Data>, Self::Error> {
        let chunk_size = calc_chunk_size::<Entity>();

        let models = entities.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut inserted = Vec::with_capacity(models.len());
        let mut models_iter = models.into_iter();

        let tx = self.db().begin().await?;

        loop {
            let chunk_n: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
            if chunk_n.is_empty() {
                break;
            }

            let result = Entity::insert_many(chunk_n)
                .on_conflict_do_nothing()
                .exec_with_returning_many(&tx)
                .await?;
            if let TryInsertResult::Inserted(res) = result {
                inserted.extend(res);
            }
        }

        tx.commit().await?;

        Ok(inserted.into_iter().map(Self::Data::from).collect::<Vec<_>>())
    }

    /// 保存数据, 找不到数据时插入数据, 找到数据时更新数据
    async fn bulk_save(&self, entities: Vec<Self::Data>, on_conflict: OnConflict) -> Result<(), Self::Error> {
        let chunk_size = calc_chunk_size::<Entity>();

        let models = entities.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut models_iter = models.into_iter();

        let tx = self.db().begin().await?;
        loop {
            let chunk: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
            if chunk.is_empty() {
                break;
            }

            Entity::insert_many(chunk).on_conflict(on_conflict.clone()).exec(&tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    /// 保存数据, 找不到数据时插入数据, 找到数据时更新数据
    async fn bulk_save_with_returning(
        &self,
        entities: Vec<Self::Data>,
        on_conflict: OnConflict,
    ) -> Result<Vec<Self::Data>, Self::Error> {
        let chunk_size = calc_chunk_size::<Entity>();

        let models = entities.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut inserted = Vec::with_capacity(models.len());
        let mut models_iter = models.into_iter();

        let tx = self.db().begin().await?;

        loop {
            let chunk: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
            if chunk.is_empty() {
                break;
            }

            let res = Entity::insert_many(chunk)
                .on_conflict(on_conflict.clone())
                .exec_with_returning_many(&tx)
                .await?;
            inserted.extend(res);
        }

        tx.commit().await?;

        Ok(inserted.into_iter().map(Self::Data::from).collect::<Vec<_>>())
    }
}
