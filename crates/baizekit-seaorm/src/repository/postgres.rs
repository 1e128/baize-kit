use std::marker::PhantomData;

use futures_util::{Stream, TryStreamExt};
use sea_orm::{
    ConnectionTrait, DbErr, EntityTrait, Insert, IntoActiveModel, ItemsAndPagesNumber, Iterable, PaginatorTrait,
    Select, SelectModel, StreamTrait, TransactionTrait,
};

pub struct Repository<'db, DB, DBEntity, Entity, Error> {
    db: &'db DB,
    phantom_data: PhantomData<(DBEntity, Entity, Error)>,
}

/// new
impl<'db, DB, DBEntity, Entity, Error> Repository<'db, DB, DBEntity, Entity, Error> {
    pub fn new(db: &'db DB) -> Self {
        Self { db, phantom_data: Default::default() }
    }
}

/// find
impl<'db, DB, DBEntity, Entity, Error> Repository<'db, DB, DBEntity, Entity, Error>
where
    DB: ConnectionTrait,
    DBEntity: EntityTrait,
    Entity: From<DBEntity::Model>,
    Error: From<DbErr>,
{
    pub async fn find<FindParam, FindFilter>(&self, filter: FindParam) -> Result<Option<Entity>, Error>
    where
        FindParam: Into<FindFilter>,
        Select<DBEntity>: From<FindFilter>,
    {
        let filter: FindFilter = filter.into();
        let select = Select::<DBEntity>::from(filter);
        let data = select.one(self.db).await.map(|v| v.map(Entity::from))?;
        Ok(data)
    }
}

#[derive(Clone)]
pub enum Paginate {
    Offset((u64, u64)),
    Cursor(u64),
}

pub trait PaginateFilterTrait {
    fn paginate(&self) -> Option<Paginate>;
}

/// search and stream
impl<'db, DB, DBEntity, Entity, Error> Repository<'db, DB, DBEntity, Entity, Error>
where
    DB: ConnectionTrait + StreamTrait + Send,
    DBEntity: EntityTrait,
    Entity: From<DBEntity::Model>,
    Error: From<DbErr>,
    Select<DBEntity>: PaginatorTrait<'db, DB, Selector = SelectModel<DBEntity::Model>>,
{
    pub async fn search<SearchParam, SearchFilter>(
        &self,
        filter: SearchParam,
    ) -> Result<(Vec<Entity>, u64, bool), Error>
    where
        SearchParam: Into<SearchFilter>,
        SearchFilter: PaginateFilterTrait,
        Select<DBEntity>: From<SearchFilter>,
    {
        let filter: SearchFilter = filter.into();
        let paginate = filter.paginate();
        let select = Select::<DBEntity>::from(filter);

        let (models, num_items, has_more) = match paginate {
            None => {
                let stream = select.stream(self.db).await?;
                let models: Vec<DBEntity::Model> = stream.try_collect().await?;
                let num_items = models.len() as u64;
                (models, num_items, false)
            }
            Some(Paginate::Offset((page, size))) => {
                let paginator = select.paginate(self.db, size);
                let models = paginator.fetch_page(page - 1).await?;
                let ItemsAndPagesNumber { number_of_items, number_of_pages } = paginator.num_items_and_pages().await?;
                let has_more = number_of_pages > paginator.cur_page();
                (models, number_of_items, has_more)
            }
            Some(Paginate::Cursor(size)) => {
                let paginator = select.paginate(self.db, size);
                let models = paginator.fetch_page(0).await?;
                let num_items = paginator.num_items().await?;
                let has_more = num_items > models.len() as u64;
                (models, num_items, has_more)
            }
        };

        Ok((models.into_iter().map(Entity::from).collect(), num_items, has_more))
    }
}

/// stream
impl<'db, DB, DBEntity, Entity, Error> Repository<'db, DB, DBEntity, Entity, Error>
where
    DB: ConnectionTrait + StreamTrait + Send,
    DBEntity: EntityTrait,
    Entity: From<DBEntity::Model> + 'static,
    Error: From<DbErr> + 'static,
{
    pub async fn stream<SearchParam, SearchFilter>(
        &self,
        param: SearchParam,
    ) -> Result<impl Stream<Item = Result<Entity, Error>> + Send + 'db, Error>
    where
        SearchParam: Into<SearchFilter>,
        Select<DBEntity>: From<SearchFilter>,
    {
        let filter = param.into();
        let select = Select::<DBEntity>::from(filter);
        let stream = select.stream(self.db).await?.map_ok(Entity::from).map_err(Error::from);
        Ok(stream)
    }
}

/// insert and update
impl<'a, DB, DBEntity, Entity, Error> Repository<'a, DB, DBEntity, Entity, Error>
where
    DB: ConnectionTrait,
    DBEntity: EntityTrait,
    DBEntity::Model: IntoActiveModel<DBEntity::ActiveModel>,
    DBEntity::ActiveModel: From<Entity>,
    Entity: From<DBEntity::Model>,
    Error: From<DbErr>,
{
    pub async fn insert(&self, entity: Entity) -> Result<Entity, Error> {
        let active_model = DBEntity::ActiveModel::from(entity);
        let inserted = DBEntity::insert(active_model)
            .exec_with_returning(self.db)
            .await
            .map(Entity::from)?;
        Ok(inserted)
    }

    pub async fn update(&self, entity: Entity) -> Result<Entity, Error> {
        let active_model = DBEntity::ActiveModel::from(entity);
        let updated = DBEntity::update(active_model).exec(self.db).await.map(Entity::from)?;
        Ok(updated)
    }
}

/// upsert
impl<'a, DB, DBEntity, Entity, Error> Repository<'a, DB, DBEntity, Entity, Error>
where
    DB: ConnectionTrait,
    DBEntity: EntityTrait,
    DBEntity::Model: IntoActiveModel<DBEntity::ActiveModel>,
    Insert<DBEntity::ActiveModel>: From<Entity>,
    Entity: From<DBEntity::Model>,
    Error: From<DbErr>,
{
    pub async fn upsert(&self, entity: Entity) -> Result<Entity, Error> {
        let upserted = Insert::<DBEntity::ActiveModel>::from(entity)
            .exec_with_returning(self.db)
            .await
            .map(Entity::from)?;
        Ok(upserted)
    }
}

impl<'db, DB, DBEntity, Entity, Error> Repository<'db, DB, DBEntity, Entity, Error>
where
    DB: ConnectionTrait + TransactionTrait,
    DBEntity: EntityTrait,
    DBEntity::Model: IntoActiveModel<DBEntity::ActiveModel>,
    DBEntity::ActiveModel: From<Entity>,
    for<'a> Insert<DBEntity::ActiveModel>: From<&'a [Entity]>,
    Entity: From<DBEntity::Model>,
    Error: From<DbErr>,
{
    pub async fn bulk_insert(&self, entities: Vec<Entity>) -> Result<(), Error> {
        // https://github.com/launchbadge/sqlx/issues/3464
        // https://www.postgresql.org/docs/current/limits.html
        let chunk_size = (u16::MAX / DBEntity::Column::iter().len() as u16) as usize;

        let tx = self.db.begin().await?;
        for chunk in entities.chunks(chunk_size) {
            Insert::<DBEntity::ActiveModel>::from(chunk).exec(&tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
