use axum::http::Method;
use quote::{format_ident, quote};
use syn::parse_file;

use crate::build::handler::HttpHandler;

pub(crate) struct CodeGenerator {
    handlers: Vec<HttpHandler>,
    state: String,
}

impl CodeGenerator {
    pub(crate) fn new(handlers: Vec<HttpHandler>, state: String) -> Self {
        Self { handlers, state }
    }

    pub(crate) fn generate_code(&mut self) -> String {
        // 生成 Router
        let mut router_chain = quote! { axum::Router::new() };
        self.handlers.sort_by(|a, b| {
            let lhs = (&a.http_path, a.http_method.to_string());
            let rhs = (&b.http_path, b.http_method.to_string());
            lhs.cmp(&rhs)
        });
        for handler in self.handlers.iter() {
            let path = &handler.http_path;

            let module: syn::Path = syn::parse_str(&handler.module).unwrap();
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

        // 构建 ApiDoc paths
        self.handlers.sort_by(|a, b| {
            let lhs = (&a.module, &a.func);
            let rhs = (&b.module, &b.func);
            lhs.cmp(&rhs)
        });
        let paths: Vec<_> = self
            .handlers
            .iter()
            .map(|h| {
                let module = syn::parse_str::<syn::Path>(&h.module).unwrap();
                let func = format_ident!("{}", h.func);
                quote! { #module::#func }
            })
            .collect();

        let state = syn::parse_str::<syn::Path>(&self.state.clone()).unwrap();
        let state = quote! { #state };

        let output = quote! {
            #[derive(utoipa::OpenApi)]
            #[openapi(paths(
                #(#paths),*
            ))]
            pub struct ApiDoc;

            pub fn new_router(state: #state) -> axum::Router {
                use axum::routing::*;

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
}
