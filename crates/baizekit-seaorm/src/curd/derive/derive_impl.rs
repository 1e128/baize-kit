use proc_macro2::TokenStream;
use syn::__private::quote::quote;

use super::{CurdMacroOptions, CurlStruct, FindOptions, SearchOptions, UpsertOptions};

// 生成仓库实现代码
pub fn derive_curd_impl(options: CurdMacroOptions) -> TokenStream {
    let curl_struct = options.to_curl_struct();

    let new_impl = gen_new_impl(&curl_struct);

    let mut trait_impls = Vec::new();

    // 生成 find trait impl
    if let Some(option) = &options.find {
        let trait_impl = gen_find_trait_impl(&curl_struct, option);
        trait_impls.push(trait_impl);
    }

    // 生成 search trait impl
    if let Some(option) = options.search {
        let trait_impl = gen_search_trait_impl(&curl_struct, option);
        trait_impls.push(trait_impl);
    }

    if let Some(option) = options.stream_search {
        let trait_impl = gen_stream_trait_impl(&curl_struct, option);
        trait_impls.push(trait_impl);
    }

    // 生成 insert trait impl
    if options.insert {
        let trait_impl = gen_insert_trait_impl(&curl_struct);
        trait_impls.push(trait_impl);
    }

    // 生成 delete trait impl
    if options.delete {
        let trait_impl = gen_delete_trait_impl(&curl_struct);
        trait_impls.push(trait_impl);
    }

    // 生成 update trait impl
    if options.update {
        let trait_impl = gen_update_trait_impl(&curl_struct);
        trait_impls.push(trait_impl);
    }

    // 生成 upsert trait impl
    if let Some(option) = options.upsert {
        let trait_impl = gen_upsert_trait_impl(&curl_struct, option);
        trait_impls.push(trait_impl);
    }

    // 生成 bulk insert trait impl
    if options.bulk_insert {
        let trait_impl = gen_bulk_insert_trait_impl(&curl_struct);
        trait_impls.push(trait_impl);
    }

    // 生产 bulk upsert trait impl
    if let Some(option) = options.bulk_upsert {
        let trait_impl = gen_bulk_upsert_trait_impl(&curl_struct, option);
        trait_impls.push(trait_impl);
    }

    quote! {
        #new_impl

        #(#trait_impls)*
    }
}

fn gen_new_impl(cs: &CurlStruct) -> TokenStream {
    // 参数列表：形如 `db: Arc<DatabaseConnection>, logger: Logger, ...`
    let mut fields = vec![cs.db_field.clone()];
    fields.extend(cs.other_fields.clone());

    let args = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        quote! { #name: #ty }
    });

    // 结构体初始化字段：形如 `db, logger, cache`
    let inits = fields.iter().map(|f| {
        let name = f.ident.as_ref().unwrap();
        quote! { #name }
    });

    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn new(#(#args),*) -> Self {
                Self {
                    #(#inits),*
                }
            }
        }
    }
}

fn gen_find_trait_impl(cs: &CurlStruct, FindOptions { filter, select_fn }: &FindOptions) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics FindTrait<#domain_entity, #error, #filter> for #struct_name #ty_generics #where_clause {
            async fn find_with_tx(&self, filter: #filter, tx: Option<&mut dyn Transaction>) -> Result<Option<#domain_entity>, #error> {
                let query: Select::<#db_entity> = #select_fn(filter);

                let select = match tx {
                    None => query.one(&*self.#db_field_name).await,
                    Some(tx) => {
                        let tx = tx
                            .as_any()
                            .downcast_mut::<SeaOrmTransaction>()
                            .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                        query.one(tx.inner()).await
                    }
                };

                select
                    .map(|v| v.map(#domain_entity::from))
                    .map_err(#error::from)
            }
        }
    }
}

fn gen_search_trait_impl(cs: &CurlStruct, SearchOptions { filter, select_fn }: SearchOptions) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics SearchTrait<#domain_entity, #error, #filter> for #struct_name #ty_generics #where_clause {
            async fn search(&self, filter: #filter) -> Result<(Vec<#domain_entity>, u64, bool), #error> {
                let paginate = filter.pagination();
                let select: Select::<#db_entity> = #select_fn(filter);

                let (models, num_items, has_more) = match paginate {
                    None => {
                        let stream = select.stream(&*self.#db_field_name).await?;
                        let models: Vec<_> = stream.try_collect().await?;
                        let num_items = models.len() as u64;
                        (models, num_items, false)
                    }
                    Some(Pagination::Offset(page, size)) => {
                        let paginator = select.paginate(&*self.#db_field_name, size);
                        let models = paginator.fetch_page(page - 1).await?;
                        let page_info = paginator.num_items_and_pages().await?;
                        let has_more = page_info.number_of_pages > paginator.cur_page();
                        (models, page_info.number_of_items, has_more)
                    }
                    Some(Pagination::Cursor(size)) => {
                        let paginator = select.paginate(&*self.#db_field_name, size);
                        let models = paginator.fetch_page(0).await?;
                        let page_info = paginator.num_items_and_pages().await?;
                        let has_more = page_info.number_of_pages > paginator.cur_page();
                        (models, page_info.number_of_items, has_more)
                    }
                };
                Ok((models.into_iter().map(#domain_entity::from).collect(), num_items, has_more))
            }
        }
    }
}

fn gen_stream_trait_impl(cs: &CurlStruct, SearchOptions { filter, select_fn }: SearchOptions) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics SearchStreamTrait<#domain_entity, #error, #filter> for #struct_name #ty_generics #where_clause {
            async fn stream(&self, filter: #filter) -> BoxStream<'static, Result<#domain_entity, #error >> {
                let db = self.#db_field_name.clone(); // 克隆 Arc 保证不依赖外部引用
                let select: Select::<#db_entity> = #select_fn(filter);

                async_stream::try_stream! {
                    let mut stream = select.stream(&*db).await?.map_ok(#domain_entity::from).map_err(#error::from);
                    while let Some(data) = stream.try_next().await? {
                        yield data;
                    }
                }.boxed()
            }
        }
    }
}

fn gen_insert_trait_impl(cs: &CurlStruct) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics InsertTrait<#domain_entity, #error> for #struct_name #ty_generics #where_clause {
            async fn insert_with_tx(&self, data: #domain_entity, tx: Option<&mut dyn Transaction>) -> Result<#domain_entity, #error> {
                let insert = #db_entity::insert(ActiveModel::from(data));
                match tx {
                    None => insert.exec_with_returning(&*self.#db_field_name).await,
                    Some(tx) => {
                        let tx = tx
                            .as_any()
                            .downcast_mut::<SeaOrmTransaction>()
                            .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                        insert.exec_with_returning(tx.inner()).await
                    }
                }
                .map(#domain_entity::from)
                .map_err(#error::from)
            }
        }
    }
}

fn gen_delete_trait_impl(cs: &CurlStruct) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics DeleteTrait<#domain_entity, #error> for #struct_name #ty_generics #where_clause {
            async fn delete_with_tx(&self, data: #domain_entity, tx: Option<&mut dyn Transaction>) -> Result<#domain_entity, #error> {
                unimplemented!()
            }
        }
    }
}

fn gen_update_trait_impl(cs: &CurlStruct) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics UpdateTrait<#domain_entity, #error> for #struct_name #ty_generics #where_clause {
            async fn update_with_tx(&self, data: #domain_entity, tx: Option<&mut dyn Transaction>) -> Result<#domain_entity, #error> {
                let update = #db_entity::update(ActiveModel::from(data));
                match tx {
                    None => update.exec(&*self.#db_field_name).await,
                    Some(tx) => {
                        let tx = tx
                            .as_any()
                            .downcast_mut::<SeaOrmTransaction>()
                            .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                        update.exec(tx.inner()).await
                    }
                }
                .map(#domain_entity::from)
                .map_err(#error::from)

            }
        }
    }
}

fn gen_upsert_trait_impl(cs: &CurlStruct, UpsertOptions { on_conflict_fn }: UpsertOptions) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics UpsertTrait<#domain_entity, #error> for #struct_name #ty_generics #where_clause {
            async fn upsert_with_tx(&self, data: #domain_entity, tx: Option<&mut dyn Transaction>) -> Result<Option<#domain_entity>, #error> {
                let insert = #db_entity::insert(ActiveModel::from(data)).on_conflict(#on_conflict_fn());
                let result = match tx {
                    None => insert.exec_with_returning(&*self.#db_field_name).await,
                    Some(tx) => {
                        let tx = tx
                            .as_any()
                            .downcast_mut::<SeaOrmTransaction>()
                            .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                        insert.exec_with_returning(tx.inner()).await
                    }
                };

                match result {
                    Ok(model) => Ok(Some(#domain_entity::from(model))),
                    Err(DbErr::RecordNotInserted) => Ok(None),
                    Err(err) => Err(#error::from(err)),
                }
            }
        }
    }
}

fn gen_bulk_insert_trait_impl(cs: &CurlStruct) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics BulkInsertTrait<#domain_entity, #error> for #struct_name #ty_generics #where_clause {
            async fn bulk_insert_with_tx(
                &self,
                entities: Vec<#domain_entity>,
                tx: Option<&mut dyn Transaction>,
            ) -> Result<(), #error> {
                use sea_orm::entity::EntityTrait;
                // https://github.com/launchbadge/sqlx/issues/3464
                // https://www.postgresql.org/docs/current/limits.html
                let column_count = <#db_entity as EntityTrait>::Column::iter().count() as u16;
                assert!(column_count > 0, "entity must have at least one column");

                // https://github.com/launchbadge/sqlx/issues/3464
                // https://www.postgresql.org/docs/current/limits.html
                let chunk_size = (u16::MAX / column_count) as usize;
                assert!(chunk_size > 0, "chunk_size must be greater than 0");

                let models = entities
                    .into_iter()
                    .map(<#db_entity as EntityTrait>::ActiveModel::from)
                    .collect::<Vec<_>>();
                let mut models_iter = models.into_iter();

                match tx {
                    None => {
                        let tx = self.#db_field_name.begin().await?;
                        loop {
                            let chunk_n: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
                            if chunk_n.is_empty() {
                                break;
                            }

                            Entity::insert_many(chunk_n).on_conflict_do_nothing().exec(&tx).await?;
                        }
                        tx.commit().await?;
                    }
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
                            Entity::insert_many(chunk_n).on_conflict_do_nothing().exec(tx.inner()).await?;
                        }
                    }
                }

                Ok(())
            }
        }
    }
}

fn gen_bulk_upsert_trait_impl(cs: &CurlStruct, UpsertOptions { on_conflict_fn }: UpsertOptions) -> TokenStream {
    let struct_name = &cs.struct_name;
    let generics = &cs.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let db_entity = &cs.db_entity;
    let domain_entity = &cs.domain_entity;
    let error = &cs.error;
    let db_field_name = cs.db_field.ident.as_ref().unwrap();

    quote! {
        #[async_trait::async_trait]

        impl #impl_generics BulkUpsertTrait<#domain_entity, #error> for #struct_name #ty_generics #where_clause {
            async fn bulk_upsert_with_tx(
                &self,
                entities: Vec<#domain_entity>,
                tx: Option<&mut dyn Transaction>,
            ) -> Result<(), #error> {
                use sea_orm::entity::EntityTrait;
                // https://github.com/launchbadge/sqlx/issues/3464
                // https://www.postgresql.org/docs/current/limits.html
                let column_count = <#db_entity as EntityTrait>::Column::iter().count() as u16;
                assert!(column_count > 0, "entity must have at least one column");

                // https://github.com/launchbadge/sqlx/issues/3464
                // https://www.postgresql.org/docs/current/limits.html
                let chunk_size = (u16::MAX / column_count) as usize;
                assert!(chunk_size > 0, "chunk_size must be greater than 0");

                let models = entities
                    .into_iter()
                    .map(<#db_entity as EntityTrait>::ActiveModel::from)
                    .collect::<Vec<_>>();
                let mut models_iter = models.into_iter();

                match tx {
                    None => {
                        let tx = self.#db_field_name.begin().await?;
                        loop {
                            let chunk_n: Vec<_> = models_iter.by_ref().take(chunk_size).collect();
                            if chunk_n.is_empty() {
                                break;
                            }
                            Entity::insert_many(chunk_n).on_conflict(#on_conflict_fn()).exec(&tx).await?;
                        }
                        tx.commit().await?;
                    }
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
                            Entity::insert_many(chunk_n).on_conflict(#on_conflict_fn()).exec(tx.inner()).await?;
                        }
                    }
                }

                Ok(())
            }
        }
    }
}
