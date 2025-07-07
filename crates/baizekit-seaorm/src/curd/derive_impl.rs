use proc_macro2::TokenStream;
use syn::__private::quote::quote;
use crate::curd::CurdMacroOptions;

// 生成仓库实现代码
pub fn derive_curd_impl(options: CurdMacroOptions) -> TokenStream {
    let struct_name = &options.ident;
    let entity_path = &options.entity;
    let model_path = &options.model;
    let error_path = &options.error;

    let mut methods = Vec::new();

    // 生成find方法
    if let Some(find_options) = &options.find {
        let filter = find_options.filter.clone();
        let select_fn = &find_options.select_fn;
        methods.push(quote! {
            #[async_trait::async_trait]
            impl FindTrait<#model_path, #error_path, #filter> for #struct_name {
            async fn find(
                &self,
                filter: #filter,
                tx: Option<&mut dyn Transaction>
            ) -> Result<Option<#model_path>, #error_path> {
                let query :Select::<#entity_path> = #select_fn(filter);

                let result = match tx {
                    Some(tx) => {
                        let tx = tx
                            .as_any()
                            .downcast_mut::<SeaOrmTransaction>()
                            .ok_or_else(|| DbErr::Custom("Invalid transaction type".to_string()))?;
                        query.one(tx.inner()).await
                    }
                    None => query.one(&*self.db).await,
                };

                result
                    .map(|v| v.map(#model_path::from))
                    .map_err(#error_path::from)
            }
        }
        });
    }

    // 生成search方法
    if let Some(search_options) = &options.search {
        let filter = search_options.filter.clone();
        let select_fn = &search_options.select_fn;
        methods.push(quote! {
            #[async_trait::async_trait]
            impl SearchTrait<#model_path, #error_path, #filter> for #struct_name {
                async fn search(
                    &self,
                    filter: #filter,
                ) -> Result<(Vec<#model_path>, u64, bool), #error_path> {
                    let paginate = filter.pagination();
                    let select :Select::<#entity_path> = #select_fn(filter);
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
                    Ok((models.into_iter().map(#model_path::from).collect(), num_items, has_more))
                }
            }
        });
    }

    // 生成insert方法
    if options.insert {
        methods.push(quote! {
            #[async_trait::async_trait]
            impl InsertTrait<#model_path, #error_path> for #struct_name {
                async fn insert(
                    &self,
                    data: #model_path,
                    tx: Option<&mut dyn Transaction>
                ) -> Result<#model_path, #error_path> {
                    let insert = #entity_path::insert(data.into());
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
                    .map(#model_path::from)
                    .map_err(#error_path::from)
                }
            }
        });
    }


    quote! {
        use baizekit_seaorm::{
            sea_orm::{ConnectionTrait,DatabaseConnection, DbErr, EntityTrait, ItemsAndPagesNumber, PaginatorTrait, Select, SelectModel, StreamTrait},
            curd::{Transaction, SeaOrmTransaction, StreamExt, TryStreamExt},
        };

        #(#methods)*
        // impl #struct_name {
        //     pub fn new(db: Arc<DatabaseConnection>) -> Self {
        //         Self { db }
        //     }
        //
        //     #(#methods)*
        // }
    }
}
