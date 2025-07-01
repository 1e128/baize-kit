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
                param: #filter,
                tx: Option<&mut dyn Transaction>
            ) -> Result<Option<#model_path>, #error_path> {
                let filter = param.into();
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

    // 生成update方法
    if options.update {
        methods.push(quote! {
            pub async fn update(
                &self,
                data: #model_path,
                tx: Option<&DatabaseTransaction>
            ) -> Result<#model_path, #error_path> {
                let active_model = data.into_active_model();
                let result = match tx {
                    Some(tx) => active_model.update(tx).await,
                    None => active_model.update(&self.db).await,
                };

                result.map(Into::into).map_err(Into::into)
            }
        });
    }

    // 生成delete方法
    if options.delete {
        methods.push(quote! {
            pub async fn delete(
                &self,
                tx: Option<&DatabaseTransaction>
            ) -> Result<(), #error_path> {
                let result = match tx {
                    Some(tx) => <#entity_path as EntityTrait>::delete_by_id(id)
                        .exec(tx)
                        .await,
                    None => <#entity_path as EntityTrait>::delete_by_id(id)
                        .exec(&self.db)
                        .await,
                };

                result.map(|_| ()).map_err(Into::into)
            }
        });
    }

    quote! {
        use baizekit_seaorm::{
            sea_orm::{ConnectionTrait,DatabaseConnection},
            curd::{Transaction, SeaOrmTransaction},
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
