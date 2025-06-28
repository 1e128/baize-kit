use sea_orm::{ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, StreamTrait, TransactionTrait};

pub use self::find::*;
pub use self::search::*;
pub use self::upsert::*;

mod find;
mod search;
mod upsert;

pub trait RepositoryTrait<DB> {
    fn db(&self) -> &DB;
}

pub struct Repository<'db, DB, Entity, FindFilter, SearchFilter, Item, Error> {
    db: &'db DB,
    phantom_data: std::marker::PhantomData<(Entity, Item, FindFilter, SearchFilter, Error)>,
}

/// new
impl<'db, DB, Entity, Item, FindFilter, SearchFilter, Error>
    Repository<'db, DB, Entity, Item, FindFilter, SearchFilter, Error>
{
    pub fn new(db: &'db DB) -> Self {
        Self { db, phantom_data: Default::default() }
    }
}

impl<'db, DB, Entity, Item, FindFilter, SearchFilter, Error> RepositoryTrait<DB>
    for Repository<'db, DB, Entity, Item, FindFilter, SearchFilter, Error>
{
    fn db(&self) -> &DB {
        self.db
    }
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> FindTrait<DB, Entity>
    for Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
where
    DB: ConnectionTrait,
    Entity: EntityTrait,
    Data: From<Entity::Model>,
    Error: From<DbErr>,
{
    type Data = Data;
    type Error = Error;
    type Filter = FindFilter;
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> SearchTrait<'db, DB, Entity>
    for Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
where
    DB: ConnectionTrait + StreamTrait + Send + 'db,
    Entity: EntityTrait,
    Entity::Model: Send + Sync,
    Data: From<Entity::Model> + 'static,
    Error: From<DbErr> + 'static,
    SearchFilter: PaginateFilterTrait + Send,
{
    type Data = Data;
    type Error = Error;
    type Filter = SearchFilter;
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> SearchStreamTrait<'db, DB, Entity>
    for Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
where
    DB: ConnectionTrait + StreamTrait + Send + 'db,
    Entity: EntityTrait,
    Entity::Model: Send + Sync,
    Data: From<Entity::Model> + 'static,
    Error: From<DbErr> + 'static,
{
    type Data = Data;
    type Error = Error;
    type Filter = SearchFilter;
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> UpsertTrait<DB, Entity>
    for Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
where
    DB: ConnectionTrait,
    Entity: EntityTrait,
    Entity::Model: IntoActiveModel<Entity::ActiveModel> + Send + Sync,
    Entity::ActiveModel: Send,
    Data: From<Entity::Model> + Into<Entity::ActiveModel> + Send + 'static,
    Error: From<DbErr> + 'static,
{
    type Data = Data;
    type Error = Error;
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> BulkUpsertTrait<DB, Entity>
    for Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
where
    DB: ConnectionTrait + TransactionTrait + Send,
    Entity: EntityTrait,
    Entity::Model: IntoActiveModel<Entity::ActiveModel> + Send + Sync,
    Entity::ActiveModel: Send,
    Data: From<Entity::Model> + Into<Entity::ActiveModel> + Send + 'static,
    Error: From<DbErr> + 'static,
{
    type Data = Data;
    type Error = Error;
}
