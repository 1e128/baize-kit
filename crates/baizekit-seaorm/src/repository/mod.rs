use sea_orm::{ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, StreamTrait, TransactionTrait};

pub use self::find::*;
pub use self::search::*;
pub use self::upsert::*;

mod find;
mod search;
mod upsert;

pub trait RepositoryTrait<'db, DB> {
    fn db(&self) -> &'db DB;
}

pub struct Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> {
    db: &'db DB,
    phantom_data: std::marker::PhantomData<(Entity, Data, FindFilter, SearchFilter, Error)>,
}

/// new
impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
    Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
{
    pub fn new(db: &'db DB) -> Self {
        Self { db, phantom_data: Default::default() }
    }
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> RepositoryTrait<'db, DB>
    for Repository<'db, DB, Entity, Data, FindFilter, SearchFilter, Error>
{
    fn db(&self) -> &'db DB {
        self.db
    }
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> FindTrait<'db, DB, Entity>
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

impl<DB, Entity, Data, FindFilter, SearchFilter, Error> SearchStreamTrait<DB, Entity>
    for Repository<'static, DB, Entity, Data, FindFilter, SearchFilter, Error>
where
    DB: ConnectionTrait + StreamTrait + Send + 'static,
    Entity: EntityTrait,
    Data: From<Entity::Model> + 'static,
    Error: From<DbErr> + 'static,
{
    type Data = Data;
    type Error = Error;
    type Filter = SearchFilter;
}

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> UpsertTrait<'db, DB, Entity>
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

impl<'db, DB, Entity, Data, FindFilter, SearchFilter, Error> BulkUpsertTrait<'db, DB, Entity>
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
