use darling::{FromDeriveInput, FromMeta};
use syn::{Ident, Path};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(curd))]
pub struct CurdMacroOptions {
    pub ident: Ident,

    /// 实体类型
    pub entity: Path,

    /// 模型类型
    pub model: Path,

    /// 错误类型
    pub error: Path,

    /// 查找操作配置
    #[darling(default)]
    pub find: Option<FindOptions>,

    /// 搜索配置
    #[darling(default)]
    pub search: Option<SearchOptions>,

    /// 流式搜索配置
    #[darling(default)]
    pub stream_search: Option<SearchOptions>,

    /// 是否启用插入操作
    #[darling(default)]
    pub insert: bool,

    /// 是否启用插入操作
    #[darling(default)]
    pub delete: bool,

    /// 是否启用更新操作
    #[darling(default)]
    pub update: bool,

    /// 是否启用保存操作
    #[darling(default)]
    pub save: bool,

    /// 是否启用批量插入操作
    #[darling(default)]
    pub bulk_insert: bool,
}

#[derive(Debug, FromMeta)]
pub struct FindOptions {
    /// 过滤器类型
    pub filter: Path,

    /// 转换函数
    pub select_fn: Path,
}

#[derive(Debug, FromMeta)]
pub struct SearchOptions {
    /// 过滤器类型
    pub filter: Path,

    /// 转换函数
    pub select_fn: Path,
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
