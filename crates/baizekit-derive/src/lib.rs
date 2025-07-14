use baizekit_seaorm::curd::derive::{CurdMacroOptions, derive_curd_impl};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

use crate::derive_with::{Input, derive_with_impl};

mod derive_with;

#[proc_macro_derive(With)]
pub fn derive_with(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let input = match Input::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    match derive_with_impl(input) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(e) => TokenStream::from(e.write_errors()),
    }
}

#[proc_macro_derive(Curd, attributes(curd))]
pub fn derive_curd(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let options = match CurdMacroOptions::from_derive_input(&input) {
        Ok(c) => c,
        Err(e) => return e.write_errors().into(),
    };

    derive_curd_impl(options).into()
}

#[proc_macro_derive(PaginatedFilter, attributes(paginate))]
pub fn derive_paginated_filter(input: TokenStream) -> TokenStream {
    let opts = parse_macro_input!(input as DeriveInput);

    let struct_name = &opts.ident;

    // 查找被 #[paginate] 标注的字段
    let field_name = if let Data::Struct(data_struct) = &opts.data {
        data_struct
            .fields
            .iter()
            .find(|f| f.attrs.iter().any(|attr| attr.path().is_ident("paginate")))
            .map(|f| f.ident.as_ref().unwrap())
    } else {
        None
    };

    let Some(field_name) = field_name else {
        return syn::Error::new_spanned(struct_name, "Expected one named field with #[paginate] attribute")
            .to_compile_error()
            .into();
    };

    let expanded = quote! {
        impl baizekit_seaorm::curd::PaginatedFilter for #struct_name {
            fn pagination(&self) -> Option<baizekit_seaorm::curd::Pagination> {
                self.#field_name.clone()
            }
        }
    };

    expanded.into()
}
