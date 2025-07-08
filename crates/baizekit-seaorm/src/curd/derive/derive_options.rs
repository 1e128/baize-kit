use darling::{FromDeriveInput, FromField, FromMeta};
use syn::{Generics, Ident, Path, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(curd))]
pub struct CurdMacroOptions {
    /// 结构
    pub(super) ident: Ident,
    /// 结构对应的generics
    pub(super) generics: Generics,
    /// 结构字段
    pub(super) data: darling::ast::Data<(), CurdField>,

    /// 实体类型
    pub(super) db_entity: Path,

    /// 模型类型
    pub(super) domain_entity: Path,

    /// 错误类型
    pub(super) error: Path,

    /// 查找操作配置
    #[darling(default)]
    pub(super) find: Option<FindOptions>,

    /// 搜索配置
    #[darling(default)]
    pub(super) search: Option<SearchOptions>,

    /// 流式搜索配置
    #[darling(default)]
    pub(super) stream_search: Option<SearchOptions>,

    /// 是否启用插入操作
    #[darling(default)]
    pub(super) insert: bool,

    /// 是否启用插入操作
    #[darling(default)]
    pub(super) delete: bool,

    /// 是否启用更新操作
    #[darling(default)]
    pub(super) update: bool,

    /// 是否启用保存操作
    #[darling(default)]
    pub(super) upsert: Option<UpsertOptions>,

    /// 是否启用批量插入操作
    #[darling(default)]
    pub(super) bulk_insert: bool,

    /// 是否启用批量保存操作
    pub(super) bulk_upsert: Option<UpsertOptions>,
}

impl CurdMacroOptions {
    pub(super) fn to_curl_struct(&self) -> CurlStruct {
        let ident = &self.ident;
        let generics = &self.generics;
        let db_entity_path = &self.db_entity;
        let domain_entity_path = &self.domain_entity;
        let error_path = &self.error;

        let (db_field, other_fields): (CurdField, Vec<CurdField>) = match &self.data {
            darling::ast::Data::Struct(fields) => {
                let mut db_field = None;
                let mut others = vec![];

                for f in fields.iter() {
                    if f.db {
                        if db_field.is_some() {
                            panic!("Only one #[curd(db)] field is allowed");
                        }
                        if f.ident.is_none() {
                            panic!("Anonymous fields are not supported");
                        }
                        db_field = Some(f.clone());
                    } else {
                        others.push(f.clone());
                    }
                }

                (db_field.expect("Missing #[curd(db)] field"), others)
            }
            _ => panic!("Curd macro only supports structs"),
        };

        CurlStruct {
            struct_name: ident.clone(),
            generics: generics.clone(),
            db_entity: db_entity_path.clone(),
            domain_entity: domain_entity_path.clone(),
            error: error_path.clone(),
            db_field,
            other_fields,
        }
    }
}

#[derive(Debug, FromMeta)]
pub(super) struct FindOptions {
    /// 过滤器类型
    pub(super) filter: Path,

    /// 转换函数
    pub(super) select_fn: Path,
}

#[derive(Debug, FromMeta)]
pub(super) struct SearchOptions {
    /// 过滤器类型
    pub(super) filter: Path,

    /// 转换函数
    pub(super) select_fn: Path,
}

#[derive(Debug, FromMeta)]
pub(super) struct UpsertOptions {
    pub(super) on_conflict_fn: Path,
}

pub(super) struct CurlStruct {
    pub(super) struct_name: Ident,
    pub(super) generics: Generics,
    pub(super) db_entity: Path,
    pub(super) domain_entity: Path,
    pub(super) error: Path,

    pub(super) db_field: CurdField,
    pub(super) other_fields: Vec<CurdField>,
}

#[derive(Clone, Debug, FromField)]
#[darling(attributes(curd))]
pub(super) struct CurdField {
    pub(super) ident: Option<Ident>,
    pub(super) ty: Type,

    #[darling(default)]
    pub(super) db: bool, // 识别 #[curd(db)]
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_parse_quote() {
        println!("label: before parse1");

        let input = parse_quote! {
            #[derive(Curd)]
            #[curd(
                entity = Entity,
                model = SyncJournal,
                error = DbErr,
                find(filter = SearchFilter, select_fn = trans_fun),
                // stream_search(filter = SearchFilter, transform = trans_fun),
                search(filter = SearchFilter, select_fn = trans_fun),
                insert,
                // update,
                // save,
                // bulk_insert,
            )]
            struct MyStruct;
        };

        println!("label: before parse2");
        let opts = CurdMacroOptions::from_derive_input(&input).expect(
            "Failed to parse derive \
        input",
        );
        println!("{:#?}", opts);
        // let x = derive_curd_impl(opts);
        // println!("{:#?}", x);
    }
}
