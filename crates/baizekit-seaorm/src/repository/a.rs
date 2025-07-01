use async_trait::async_trait;
use futures_util::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, IntoActiveModel, ItemsAndPagesNumber,
    Iterable, PaginatorTrait, Select, SelectModel, TransactionTrait, TryInsertResult,
};
use std::any::Any;
use std::sync::Arc;



pub struct ARepository {
    db: Arc<DatabaseConnection>,
}

impl ARepository {
    /// 创建新Repository
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// 查找单个实体
    async fn find<E, D, Er, F>(&self, param: impl Into<F>, tx: Option<&mut dyn Transaction>) -> Result<Option<D>, Er>
    where
        E: EntityTrait,
        D: From<E::Model>,
        Er: From<DbErr>,
        Select<E>: From<F>,
    {
        let filter = param.into();
        let select = Select::<E>::from(filter);

        match tx {
            Some(tx) => {
                let tx = tx
                    .as_any()
                    .downcast_mut::<SeaOrmTransaction>()
                    .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                select.one(tx.inner()).await
            }
            None => select.one(&*self.db).await,
        }
        .map(|v| v.map(D::from))
        .map_err(Er::from)
    }

    ///流式查询
    async fn stream<E, D, Er, F>(&'static self, param: impl Into<F>) -> Result<BoxStream<'static, Result<D, Er>>, Er>
    where
        E: EntityTrait,
        D: From<E::Model> + 'static,
        Er: From<DbErr>,
        Er: From<DbErr> + 'static,
        Select<E>: From<F>,
    {
        let filter = param.into();
        let select = Select::<E>::from(filter);
        let stream = select.stream(&*self.db).await?.map_ok(D::from).map_err(Er::from).boxed();
        Ok(stream)
    }

    /// 分页查询
    async fn search<'db, E, D, Er, F>(&'db self, param: impl Into<F>) -> Result<(Vec<D>, u64, bool), Er>
    where
        E: EntityTrait,
        D: From<E::Model>,
        Er: From<DbErr>,
        F: PaginatedFilter,
        Select<E>: From<F>,
        Select<E>: PaginatorTrait<'db, DatabaseConnection, Selector = SelectModel<E::Model>>,
    {
        let filter = param.into();
        let paginate = filter.pagination();
        let select = Select::<E>::from(filter);

        let (models, num_items, has_more) = match paginate {
            None => {
                let stream = select.stream(&*self.db).await?;
                let models: Vec<E::Model> = stream.try_collect().await?;
                let num_items = models.len() as u64;
                (models, num_items, false)
            }
            Some(Pagination::Offset(page, size)) => {
                let paginator = select.paginate(&*self.db, size);
                let models = paginator.fetch_page(page - 1).await?;
                let ItemsAndPagesNumber { number_of_items, number_of_pages } = paginator.num_items_and_pages().await?;
                let has_more = number_of_pages > paginator.cur_page();
                (models, number_of_items, has_more)
            }
            Some(Pagination::Cursor(size)) => {
                let paginator = select.paginate(&*self.db, size);
                let models = paginator.fetch_page(0).await?;
                let ItemsAndPagesNumber { number_of_items, number_of_pages } = paginator.num_items_and_pages().await?;
                let has_more = number_of_pages > paginator.cur_page();
                (models, number_of_items, has_more)
            }
        };

        Ok((models.into_iter().map(D::from).collect(), num_items, has_more))
    }

    /// 插入数据, 失败或者冲突时返回错误
    async fn insert<E, D, Er>(&self, data: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>
    where
        E: EntityTrait,
        D: From<E::Model>,
        Er: From<DbErr>,
        D: Into<E::ActiveModel> + From<E::Model> + 'static,
        E::Model: IntoActiveModel<E::ActiveModel>,
    {
        let insert = E::insert(data.into());
        match tx {
            Some(tx) => {
                let tx = tx
                    .as_any()
                    .downcast_mut::<SeaOrmTransaction>()
                    .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                insert.exec_with_returning(tx.inner()).await
            }
            None => insert.exec_with_returning(&*self.db).await,
        }
        .map(D::from)
        .map_err(Er::from)
    }

    /// 更新数据, 找不到数据时返回错误
    async fn update<E, D, Er>(&self, data: D, tx: Option<&mut dyn Transaction>) -> Result<D, Er>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        D: From<E::Model>,
        D: Into<E::ActiveModel> + From<E::Model> + 'static,
        Er: From<DbErr>,
    {
        let update = E::update(data.into());
        match tx {
            Some(tx) => {
                let tx = tx
                    .as_any()
                    .downcast_mut::<SeaOrmTransaction>()
                    .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                update.exec(tx.inner()).await
            }
            None => update.exec(&*self.db).await,
        }
        .map(D::from)
        .map_err(Er::from)
    }

    /// 保存数据, 找不到数据时插入数据, 找到数据时更新数据
    async fn save<E, D, Er>(&self, data: D, on_conflict: OnConflict, tx: Option<&mut dyn Transaction>) -> Result<D, Er>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        D: From<E::Model>,
        D: Into<E::ActiveModel> + From<E::Model> + 'static,
        Er: From<DbErr>,
    {
        let insert = E::insert(data.into()).on_conflict(on_conflict);
        match tx {
            Some(tx) => {
                let tx = tx
                    .as_any()
                    .downcast_mut::<SeaOrmTransaction>()
                    .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                insert.exec_with_returning(tx.inner()).await
            }
            None => insert.exec_with_returning(&*self.db).await,
        }
        .map(D::from)
        .map_err(Er::from)
    }

    /// 插入数据, 失败或者冲突时返回错误
    async fn bulk_insert<E, D, Er>(&self, data: Vec<D>, tx: Option<&mut dyn Transaction>) -> Result<(), Er>
    where
        E: EntityTrait,
        D: From<E::Model>,
        D: Into<E::ActiveModel> + From<E::Model> + 'static,
        Er: From<DbErr>,
    {
        let chunk_size = Self::calc_chunk_size::<E>();

        let models = data.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut models_iter = models.into_iter();

        match tx {
            None => loop {
                let chunk_n: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
                if chunk_n.is_empty() {
                    break;
                }

                E::insert_many(chunk_n).on_conflict_do_nothing().exec(&*self.db).await?;
            },
            Some(tx) => {
                let tx = tx
                    .as_any()
                    .downcast_mut::<SeaOrmTransaction>()
                    .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                loop {
                    let chunk_n: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
                    if chunk_n.is_empty() {
                        break;
                    }

                    E::insert_many(chunk_n).on_conflict_do_nothing().exec(tx.inner()).await?;
                }
            }
        }

        Ok(())
    }

    /// 批量插入数据
    async fn bulk_insert_with_returning<E, D, Er>(&self, entities: Vec<D>) -> Result<Vec<D>, Er>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        D: From<E::Model>,
        D: Into<E::ActiveModel> + From<E::Model> + 'static,
        Er: From<DbErr>,
    {
        let chunk_size = Self::calc_chunk_size::<E>();

        let models = entities.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut inserted = Vec::with_capacity(models.len());
        let mut models_iter = models.into_iter();

        let tx = self.db.begin().await?;

        loop {
            let chunk_n: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
            if chunk_n.is_empty() {
                break;
            }

            let result = E::insert_many(chunk_n)
                .on_conflict_do_nothing()
                .exec_with_returning_many(&tx)
                .await?;
            if let TryInsertResult::Inserted(res) = result {
                inserted.extend(res);
            }
        }

        tx.commit().await?;

        Ok(inserted.into_iter().map(D::from).collect::<Vec<_>>())
    }

    /// 保存数据, 找不到数据时插入数据, 找到数据时更新数据
    async fn bulk_save<E, D, Er>(&self, entities: Vec<D>, on_conflict: OnConflict) -> Result<(), Er>
    where
        E: EntityTrait,
        D: From<E::Model>,
        D: Into<E::ActiveModel> + From<E::Model> + Send + 'static,
        Er: From<DbErr>,
    {
        let chunk_size = Self::calc_chunk_size::<E>();

        let models = entities.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut models_iter = models.into_iter();

        let tx = self.db.begin().await?;
        loop {
            let chunk: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
            if chunk.is_empty() {
                break;
            }

            E::insert_many(chunk).on_conflict(on_conflict.clone()).exec(&tx).await?;
        }
        tx.commit().await?;

        Ok(())
    }

    /// 保存数据, 找不到数据时插入数据, 找到数据时更新数据
    async fn bulk_save_with_returning<E, D, Er>(&self, entities: Vec<D>, on_conflict: OnConflict) -> Result<Vec<D>, Er>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        D: From<E::Model>,
        D: Into<E::ActiveModel> + From<E::Model> + Send + 'static,
        Er: From<DbErr>,
    {
        let chunk_size = Self::calc_chunk_size::<E>();

        let models = entities.into_iter().map(Into::into).collect::<Vec<_>>();
        let mut inserted = Vec::with_capacity(models.len());
        let mut models_iter = models.into_iter();

        let tx = self.db.begin().await?;

        loop {
            let chunk: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
            if chunk.is_empty() {
                break;
            }

            let res = E::insert_many(chunk)
                .on_conflict(on_conflict.clone())
                .exec_with_returning_many(&tx)
                .await?;
            inserted.extend(res);
        }

        tx.commit().await?;

        Ok(inserted.into_iter().map(D::from).collect::<Vec<_>>())
    }

    fn calc_chunk_size<E: EntityTrait>() -> usize {
        // https://github.com/launchbadge/sqlx/issues/3464
        // https://www.postgresql.org/docs/current/limits.html
        let chunk_size = (u16::MAX / E::Column::iter().len() as u16) as usize;
        assert!(chunk_size > 0, "chunk_size must be greater than 0");
        chunk_size
    }
}
