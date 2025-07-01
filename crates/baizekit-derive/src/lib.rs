use darling::{FromDeriveInput};
use derive_with::derive_with_impl;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

use crate::derive_with::Input;

mod derive_with;
use baizekit_seaorm::curd::CurdMacroOptions;
use baizekit_seaorm::curd::derive_curd_impl;

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
pub fn curd_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let options = match CurdMacroOptions::from_derive_input(&input) {
        Ok(c) => c,
        Err(e) => return e.write_errors().into(),
    };

    derive_curd_impl(options).into()
}