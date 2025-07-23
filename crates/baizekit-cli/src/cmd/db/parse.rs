use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use syn::{parse_file, Attribute, Item, Meta, Variant};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct TableInfo {
    /// 模块路径
    pub module: String,
    /// 枚举名称
    pub enum_name: String,
    /// 表名
    pub table_name: String,
}

/// 提取指定源代码目录中所有继承自DeriveIden的枚举表名
pub fn extract_table_names(src_dir: &Path) -> Result<Vec<TableInfo>> {
    let entries = WalkDir::new(src_dir)
        .min_depth(1)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() &&
            e.path().extension().map_or(false, |ext| ext == "rs"));

    let mut table_infos = Vec::new();

    for entry in entries {
        let file_path = entry.path();
        let module = file_path_to_mod_path(file_path, src_dir)
            .ok_or_else(|| anyhow!("无法解析模块路径: {:?}", file_path))?;

        let content = fs::read_to_string(file_path)
            .map_err(|e| anyhow!("无法读取文件 {:?}: {}", file_path, e))?;

        let syntax = parse_file(&content)
            .map_err(|e| anyhow!("解析文件 {:?} 失败: {}", file_path, e))?;

        process_syntax_tree(&syntax, &module, &mut table_infos);
    }

    // 排序并去重
    table_infos.sort_by_key(|t| t.table_name.clone());
    table_infos.dedup_by_key(|t| t.table_name.clone());

    Ok(table_infos)
}

/// 处理语法树，提取表信息
fn process_syntax_tree(syntax: &syn::File, module: &str, table_infos: &mut Vec<TableInfo>) {
    for item in &syntax.items {
        // 处理嵌套模块
        if let Item::Mod(mod_item) = item {
            if let Some((_, items)) = &mod_item.content {
                for nested_item in items {
                    check_and_extract_table(nested_item, module, table_infos);
                }
            }
        } else {
            check_and_extract_table(item, module, table_infos);
        }
    }
}

/// 检查是否有DeriveIden派生
fn has_derive_iden(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            // 解析derive属性内容
            if let Meta::List(list) = &attr.meta {
                // 在 syn 2.0+ 中，直接使用 parse_nested_meta
                let mut has_derive_iden = false;
                let _ = list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("DeriveIden") {
                        has_derive_iden = true;
                    }
                    Ok(())
                });
                has_derive_iden
            } else {
                false
            }
        } else {
            false
        }
    })
}


// 将驼峰式命名转换为蛇形命名（例如AccountUser → account_user）
fn camel_to_snake_case(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let mut result = String::with_capacity(s.len() + 4); // 预分配空间
    let mut chars = s.chars();
    // 处理第一个字符（转为小写）
    if let Some(c) = chars.next() {
        result.push(c.to_ascii_lowercase());
    }
    // 处理剩余字符：大写字母前加下划线并转为小写，其他字符直接添加
    for c in chars {
        if c.is_ascii_uppercase() {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// 从枚举变体中提取表名（适配syn 2.0+）
// 修改extract_table_name_from_variants函数的返回值
// 现在返回Option<Option<String>>：
// - Some(Some(name))：找到Table变体且有iden属性
// - Some(None)：找到Table变体但无iden属性
// - None：未找到Table变体
fn extract_table_name_from_variants(variants: &syn::punctuated::Punctuated<Variant, syn::Token![,]>) -> Option<Option<String>> {
    for variant in variants {
        if variant.ident == "Table" {
            // 检查是否有sea_orm(iden = "...")属性
            for attr in &variant.attrs {
                if attr.path().is_ident("sea_orm") {
                    if let Meta::List(list) = &attr.meta {
                        let mut table_name = None;
                        let _ = list.parse_nested_meta(|meta| {
                            if meta.path.is_ident("iden") {
                                if let Ok(value) = meta.value() {
                                    if let Ok(lit_str) = value.parse::<syn::LitStr>() {
                                        table_name = Some(lit_str.value());
                                    }
                                }
                            }
                            Ok(())
                        });
                        if let Some(name) = table_name {
                            return Some(Some(name)); // 有iden属性，返回指定名称
                        }
                    }
                }
            }
            return Some(None); // 找到Table变体但无iden属性
        }
    }
    None // 未找到Table变体
}

// 修改：在无iden属性时使用蛇形命名
fn check_and_extract_table(item: &Item, module: &str, table_infos: &mut Vec<TableInfo>) {
    if let Item::Enum(enum_item) = item {
        if !has_derive_iden(&enum_item.attrs) {
            return;
        }

        let enum_name = enum_item.ident.to_string(); // 枚举名称（如AccountUser）

        match extract_table_name_from_variants(&enum_item.variants) {
            Some(Some(table_name)) => {
                // 情况1：有iden属性，优先使用
                table_infos.push(TableInfo {
                    module: module.to_string(),
                    enum_name: enum_name.clone(),
                    table_name,
                });
            }
            Some(None) => {
                // 情况2：无iden属性，转换为蛇形命名
                let table_name = camel_to_snake_case(&enum_name);
                table_infos.push(TableInfo {
                    module: module.to_string(),
                    enum_name: enum_name.clone(),
                    table_name,
                });
            }
            None => {
                // 情况3：无Table变体，忽略
                return;
            }
        }
    }
}

/// 将文件路径转换为模块路径
fn file_path_to_mod_path(file_path: &Path, src_root: &Path) -> Option<String> {
    let rel_path = file_path.strip_prefix(src_root).ok()?;

    let mut components: Vec<String> = rel_path
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
        .collect();

    if let Some(last) = components.last_mut() {
        if last == "mod.rs" || last == "lib.rs" || last == "main.rs" {
            components.pop();
        } else {
            *last = last.trim_end_matches(".rs").to_string();
        }
    }

    if components.is_empty() {
        Some("crate".to_string())
    } else {
        Some(format!("crate::{}", components.join("::")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_table_name() {
        let code = r#"
            #[derive(DeriveIden)]
            pub(crate) enum Account {
                #[sea_orm(iden = "acc_account")]
                Table,
                ID,
                AccountType,
                HsFundAccount,
            }
        "#;

        let syntax = parse_file(code).unwrap();
        let mut table_infos = Vec::new();
        process_syntax_tree(&syntax, "crate::test", &mut table_infos);

        assert_eq!(table_infos.len(), 1);
        assert_eq!(table_infos[0].table_name, "acc_account");
        assert_eq!(table_infos[0].enum_name, "Account");
    }

    #[test]
    fn test_file_path_conversion() {
        let src_root = Path::new("/project/src");
        let test_file = src_root.join("models").join("account.rs");

        assert_eq!(
            file_path_to_mod_path(&test_file, &src_root),
            Some("crate::models::account".to_string())
        );
    }

    #[test]
    fn test_extract_table_name_without_iden() {
        // 测试无iden属性的情况
        let code = r#"
        #[derive(DeriveIden)]
        pub(crate) enum Account {
            Table, // 无iden属性
            ID,
            Name,
        }
    "#;

        let syntax = parse_file(code).unwrap();
        let mut table_infos = Vec::new();
        process_syntax_tree(&syntax, "crate::test", &mut table_infos);

        assert_eq!(table_infos.len(), 1);
        assert_eq!(table_infos[0].table_name, "account"); // 枚举名转小写
        assert_eq!(table_infos[0].enum_name, "Account");
    }
}