use futures_util::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::{
    ConnectionTrait, DbErr, EntityTrait, ItemsAndPagesNumber, PaginatorTrait, Select, SelectModel, StreamTrait,
};

use super::RepositoryTrait;

#[derive(derive_more::From, Clone)]
pub enum Paginate {
    Offset((u64, u64)),
    Cursor(u64),
}

impl Paginate {
    pub fn offset(offset: u64, limit: u64) -> Self {
        Self::Offset((offset, limit))
    }

    pub fn cursor(cursor: u64) -> Self {
        Self::Cursor(cursor)
    }
}

pub trait PaginateFilterTrait {
    fn paginate(&self) -> Option<Paginate>;
}

#[async_trait]
pub trait SearchTrait<'db, DB, Entity>: RepositoryTrait<'db, DB>
where
    DB: ConnectionTrait + StreamTrait + Send + 'db,
    Entity: EntityTrait,
    Entity::Model: Send + Sync,
{
    type Data: From<Entity::Model>;
    type Error: From<DbErr>;
    type Filter: PaginateFilterTrait + Send;

    async fn search<Param>(&'db self, param: Param) -> Result<(Vec<Self::Data>, u64, bool), Self::Error>
    where
        Param: Into<Self::Filter> + Send,
        Select<Entity>: From<Self::Filter>,
        Select<Entity>: PaginatorTrait<'db, DB, Selector = SelectModel<Entity::Model>>,
    {
        let filter = param.into();
        let paginate = filter.paginate();
        let select = Select::<Entity>::from(filter);

        let (models, num_items, has_more) = match paginate {
            None => {
                let stream = select.stream(self.db()).await?;
                let models: Vec<Entity::Model> = stream.try_collect().await?;
                let num_items = models.len() as u64;
                (models, num_items, false)
            }
            Some(Paginate::Offset((page, size))) => {
                let paginator = select.paginate(&self.db(), size);
                let models = paginator.fetch_page(page - 1).await?;
                let ItemsAndPagesNumber { number_of_items, number_of_pages } = paginator.num_items_and_pages().await?;
                let has_more = number_of_pages > paginator.cur_page();
                (models, number_of_items, has_more)
            }
            Some(Paginate::Cursor(size)) => {
                let paginator = select.paginate(&self.db(), size);
                let models = paginator.fetch_page(0).await?;
                let ItemsAndPagesNumber { number_of_items, number_of_pages } = paginator.num_items_and_pages().await?;
                let has_more = number_of_pages > paginator.cur_page();
                (models, number_of_items, has_more)
            }
        };

        Ok((models.into_iter().map(Self::Data::from).collect(), num_items, has_more))
    }
}

#[async_trait]
pub trait SearchStreamTrait<DB, Entity>: RepositoryTrait<'static, DB>
where
    DB: ConnectionTrait + StreamTrait + Send + 'static,
    Entity: EntityTrait,
{
    type Data: From<Entity::Model> + 'static;
    type Error: From<DbErr> + 'static;
    type Filter;

    async fn stream<Param>(
        &self,
        param: Param,
    ) -> Result<BoxStream<'static, Result<Self::Data, Self::Error>>, Self::Error>
    where
        Param: Into<Self::Filter> + Send,
        Select<Entity>: From<Self::Filter>,
    {
        let filter = param.into();
        let select = Select::<Entity>::from(filter);
        let stream = select
            .stream(self.db())
            .await?
            .map_ok(Self::Data::from)
            .map_err(Self::Error::from)
            .boxed();
        Ok(stream)
    }
}
