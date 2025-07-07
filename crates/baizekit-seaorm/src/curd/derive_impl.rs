use proc_macro2::{Ident, TokenStream};
use syn::Path;
use syn::__private::quote::quote;

use crate::curd::{CurdMacroOptions, FindOptions, SearchOptions};

// 生成仓库实现代码
pub fn derive_curd_impl(options: CurdMacroOptions) -> TokenStream {
    let struct_name = &options.ident;
    let db_entity_path = &options.entity;
    let domain_entity_path = &options.model;
    let error_path = &options.error;

    let mut trait_impls = Vec::new();

    // 生成 find trait impl
    if let Some(option) = options.find {
        let trait_impl = gen_find_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path, option);
        trait_impls.push(trait_impl);
    }

    // 生成 search trait impl
    if let Some(option) = options.search {
        let trait_impl = gen_search_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path, option);
        trait_impls.push(trait_impl);
    }

    if let Some(option) = options.stream_search {
        let trait_impl = gen_stream_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path, option);
        trait_impls.push(trait_impl);
    }

    // 生成 insert trait impl
    if options.insert {
        let trait_impl = gen_insert_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path);
        trait_impls.push(trait_impl);
    }

    // 生成 delete trait impl
    if options.delete {
        let trait_impl = gen_delete_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path);
        trait_impls.push(trait_impl);
    }

    // 生成 update trait impl
    if options.update {
        let trait_impl = gen_update_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path);
        trait_impls.push(trait_impl);
    }

    // 生成 upsert trait impl
    if options.save {
        let trait_impl = gen_upsert_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path);
        trait_impls.push(trait_impl);
    }

    //
    if options.bulk_insert {
        let trait_impl = gen_bulk_insert_trait_impl(struct_name, db_entity_path, domain_entity_path, error_path);
        trait_impls.push(trait_impl);
    }

    quote! {
        use baizekit_seaorm::{
            sea_orm::*,
            curd::*,
        };

        #(#trait_impls)*
    }
}

fn gen_find_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
    FindOptions { filter, select_fn }: FindOptions,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl FindTrait<#domain_entity_path, #error_path, #filter> for #struct_name {
            async fn find(&self, filter: #filter, tx: Option<&mut dyn Transaction>) -> Result<Option<#domain_entity_path>, #error_path> {
                let query: Select::<#db_entity_path> = #select_fn(filter);

                let select = match tx {
                    Some(tx) => {
                        let tx = tx
                            .as_any()
                            .downcast_mut::<SeaOrmTransaction>()
                            .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                        query.one(tx.inner()).await
                    }
                    None => query.one(&*self.db).await,
                };

                select
                    .map(|v| v.map(#domain_entity_path::from))
                    .map_err(#error_path::from)
            }
        }
    }
}

fn gen_search_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
    SearchOptions { filter, select_fn }: SearchOptions,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl SearchTrait<#domain_entity_path, #error_path, #filter> for #struct_name {
            async fn search(&self, filter: #filter) -> Result<(Vec<#domain_entity_path>, u64, bool), #error_path> {
                let paginate = filter.pagination();
                let select: Select::<#db_entity_path> = #select_fn(filter);

                let (models, num_items, has_more) = match paginate {
                    None => {
                        let stream = select.stream(&*self.db).await?;
                        let models: Vec<_> = stream.try_collect().await?;
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
                Ok((models.into_iter().map(#domain_entity_path::from).collect(), num_items, has_more))
            }
        }
    }
}

fn gen_stream_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
    SearchOptions { filter, select_fn }: SearchOptions,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl SearchStreamTrait<#domain_entity_path, #error_path, #filter> for #struct_name {
            async fn stream(&self, filter: #filter) -> Result<BoxStream<'static, Result<#domain_entity_path, #error_path>>, #error_path> {
                // let select: Select::<#db_entity_path> = #select_fn(filter);
                // Ok(Box::new(select.stream(&*self.db).await?.map(|v| v.map(#domain_entity_path::from))))
                todo!()
            }
        }
    }
}

fn gen_insert_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl InsertTrait<#domain_entity_path, #error_path> for #struct_name {
            async fn insert(&self, data: #domain_entity_path, tx: Option<&mut dyn Transaction>) -> Result<#domain_entity_path, #error_path> {
                let insert = #db_entity_path::insert(ActiveModel::from(data));
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
                .map(#domain_entity_path::from)
                .map_err(#error_path::from)
            }
        }
    }
}

fn gen_delete_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl DeleteTrait<#domain_entity_path, #error_path> for #struct_name {
            async fn delete(&self, data: #domain_entity_path, tx: Option<&mut dyn Transaction>) -> Result<#domain_entity_path, #error_path> {
                todo!()
            }
        }
    }
}

fn gen_update_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl UpdateTrait<#domain_entity_path, #error_path> for #struct_name {
            async fn update(&self, data: #domain_entity_path, tx: Option<&mut dyn Transaction>) -> Result<#domain_entity_path, #error_path> {
                todo!()
            }
        }
    }
}

fn gen_upsert_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl SaveTrait<#domain_entity_path, #error_path> for #struct_name {
            async fn save(&self, data: #domain_entity_path, tx: Option<&mut dyn Transaction>) -> Result<#domain_entity_path, #error_path> {
                todo!()
            }
        }
    }
}

fn gen_bulk_insert_trait_impl(
    struct_name: &Ident,
    db_entity_path: &Path,
    domain_entity_path: &Path,
    error_path: &Path,
) -> TokenStream {
    quote! {
        #[async_trait::async_trait]
        impl BulkInsertTrait<#domain_entity_path, #error_path> for #struct_name {
            async fn bulk_insert(
                &self,
                data: Vec<#domain_entity_path>,
                tx: Option<&mut dyn Transaction>,
            ) -> Result<Vec<#domain_entity_path>, #error_path> {
                todo!()
            }
        }
    }
}
