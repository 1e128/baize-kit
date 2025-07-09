use std::collections::HashSet;
use std::fs;

use axum::http::Method;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, Item, Lit, Meta, Token, parse_file};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct Handler {
    /// 模块名
    pub module: String,
    /// 方法名
    pub func: String,
    /// HTTP 方法
    pub http_method: Method,
    /// HTTP 路径
    pub http_path: String,
}

/// 从指定目录中提取所有 HTTP 路由
pub fn extract_handlers(handler_dir: &str) -> Vec<Handler> {
    let mut handlers = vec![];

    for entry in WalkDir::new(handler_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map(|s| s == "rs").unwrap_or(false))
    {
        let path = entry.path();
        let module = path.file_stem().unwrap().to_string_lossy().to_string();

        let content = fs::read_to_string(path).unwrap();
        let syntax = syn::parse_file(&content).unwrap();

        for item in syntax.items {
            if let Item::Fn(func) = item {
                for attr in &func.attrs {
                    if is_utoipa_path(attr) {
                        if let Some((method, path)) = parse_utoipa_path(attr) {
                            handlers.push(Handler {
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
    }

    handlers
}

/// 判断是否有 Utoipa 的 path 属性
fn is_utoipa_path(attr: &Attribute) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().map(|s| s.ident.to_string()).collect();
    // 支持 #[path(...)] 和 #[utoipa::path(...)]
    segments == ["path"] || segments == ["utoipa", "path"]
}

/// 解析 Utoipa 的 path 属性
fn parse_utoipa_path(attr: &Attribute) -> Option<(Method, String)> {
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

pub fn generate_openapi(handlers: &mut Vec<Handler>) -> String {
    handlers.sort_by(|a, b| {
        let lhs = (&a.module, &a.func);
        let rhs = (&b.module, &b.func);
        lhs.cmp(&rhs)
    });

    let mut modules = handlers
        .iter()
        .map(|v| v.module.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    modules.sort();

    let module_idents: Vec<_> = modules
        .iter()
        .map(|v| {
            let module = format_ident!("{}", v);
            quote! { use super::#module; }
        })
        .collect();

    // 构造 paths 列表
    let paths: Vec<_> = handlers
        .iter()
        .map(|handler| {
            let module = format_ident!("{}", handler.module);
            let func = format_ident!("{}", handler.func);
            quote! { #module::#func }
        })
        .collect();

    let output = quote! {
        #![cfg_attr(rustfmt, rustfmt::skip)]

        #(#module_idents)*

        /// OpenAPI 文档
        #[derive(utoipa::OpenApi)]
        #[openapi(paths(
            #(#paths),*
        ))]
        pub struct ApiDoc;
    };

    let code = output.to_string();
    match parse_file(&code) {
        Ok(parsed) => prettyplease::unparse(&parsed),
        Err(_) => code,
    }
}

pub fn generate_router(handlers: &mut Vec<Handler>) -> String {
    let mut modules = handlers
        .iter()
        .map(|v| v.module.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    modules.sort();

    let module_idents: Vec<_> = modules
        .iter()
        .map(|v| {
            let module = format_ident!("{}", v);
            quote! { use super::#module; }
        })
        .collect();

    let mut router_chain = quote! {
        Router::new()
    };

    handlers.sort_by(|a, b| {
        let lhs = (&a.http_path, a.http_method.to_string());
        let rhs = (&b.http_path, b.http_method.to_string());
        lhs.cmp(&rhs)
    });

    for handler in handlers {
        let path = &handler.http_path;
        let module = format_ident!("{}", handler.module);
        let func = format_ident!("{}", handler.func);

        let method_token = match handler.http_method {
            Method::GET => quote! { get },
            Method::POST => quote! { post },
            Method::PUT => quote! { put },
            Method::DELETE => quote! { delete },
            _ => continue, // 不支持的 method 跳过
        };

        router_chain = quote! {
            #router_chain
                .route(#path, #method_token(#module::#func))
        };
    }

    let output = quote! {
        #![cfg_attr(rustfmt, rustfmt::skip)]

        use axum::routing::*;
        use axum::Router;

        #(#module_idents)*
        use super::super::AppState;

        /// 新建路由
        pub fn new_router(state: AppState) -> Router {
            #router_chain
                .with_state(state)
        }
    };

    let code = output.to_string();
    match parse_file(&code) {
        Ok(parsed) => prettyplease::unparse(&parsed),
        Err(_) => code,
    }
}
