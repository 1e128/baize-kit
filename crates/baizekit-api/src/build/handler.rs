use std::fs;

use axum::http::Method;
use globset::GlobMatcher;
use syn::punctuated::Punctuated;
use syn::{parse_file, Attribute, Expr, Item, Lit, Meta, Token};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub(crate) struct HttpHandler {
    /// 模块名
    pub module: String,
    /// 函数名
    pub func: String,
    /// HTTP 方法
    pub http_method: Method,
    /// HTTP 路径
    pub http_path: String,
}

impl HttpHandler {
    pub(crate) fn parse(
        crate_project_path: &String,
        http_handlers_dir: &String,
        file_glob_matcher: &Option<GlobMatcher>,
    ) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let cargo_src_path = std::path::Path::new(&crate_project_path).join("src");

        // 获取目标文件
        let entries = WalkDir::new(&http_handlers_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file() && e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
            .filter(|e| if let Some(m) = &file_glob_matcher { m.is_match(e.path()) } else { true });

        let mut handlers = vec![];
        for entry in entries {
            let Some(module) = file_path_to_mod_path(entry.path(), &cargo_src_path) else { continue };
            let Ok(content) = fs::read_to_string(entry.path()) else { continue };
            let Ok(syntax) = parse_file(&content) else { continue };

            for item in syntax.items {
                if let Item::Fn(func) = item {
                    for attr in &func.attrs {
                        if let Some((method, path)) = parse_utoipa_path(attr) {
                            handlers.push(HttpHandler {
                                module: module.clone(),
                                func: func.sig.ident.to_string(),
                                http_method: method,
                                http_path: path,
                            });
                        }
                    }
                }
            }
        }

        Ok(handlers)
    }
}

/// 判断是否有 Utoipa 的 path 属性
fn is_utoipa_path(attr: &Attribute) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().map(|s| s.ident.to_string()).collect();
    // 支持 #[path(...)] 和 #[utoipa::path(...)]
    segments == ["path"] || segments == ["utoipa", "path"]
}

/// 解析 Utoipa 的 path 属性
fn parse_utoipa_path(attr: &Attribute) -> Option<(Method, String)> {
    if !is_utoipa_path(attr) {
        return None;
    }

    // 把整个 Attribute 的内容解析为一组 Meta
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = attr.parse_args_with(parser).ok()?;

    let mut method: Option<Method> = None;
    let mut path: Option<String> = None;

    for meta in metas {
        match meta {
            Meta::Path(path_ident) => {
                let ident_str = path_ident.segments.last()?.ident.to_string().to_lowercase();
                method = match ident_str.as_str() {
                    "get" => Some(Method::GET),
                    "post" => Some(Method::POST),
                    "put" => Some(Method::PUT),
                    "delete" => Some(Method::DELETE),
                    "patch" => Some(Method::PATCH),
                    _ => method,
                };
            }

            Meta::NameValue(nv) => {
                if nv.path.is_ident("path") {
                    if let Expr::Lit(expr_lit) = nv.value {
                        if let Lit::Str(lit_str) = expr_lit.lit {
                            path = Some(lit_str.value());
                        }
                    }
                }
            }

            Meta::List(_) => {
                // 忽略如 `responses(...)` 的内容
            }
        }
    }

    method.zip(path)
}

/// 将文件路径（例如 ./src/foo/bar.rs）转换为对应的 Rust 模块路径，例如 foo::bar
/// - `crate_src_root`: 项目中 src 目录的路径，例如 /project_root/src
/// - `file_path`: 要转换的 .rs 文件路径
fn file_path_to_mod_path(file_path: &std::path::Path, crate_src_root: &std::path::Path) -> Option<String> {
    let rel_path = file_path.strip_prefix(crate_src_root).ok()?;

    let mut components: Vec<String> = rel_path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    if let Some(last) = components.last_mut() {
        if last == "mod.rs" || last == "lib.rs" || last == "main.rs" {
            components.pop(); // crate 根
        } else {
            *last = last.trim_end_matches(".rs").to_string();
        }
    }

    if components.is_empty() { None } else { Some(format!("crate::{}", components.join("::"))) }
}
