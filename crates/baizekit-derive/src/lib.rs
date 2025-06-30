use darling::FromDeriveInput;
use derive_with::derive_with_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use crate::derive_with::Input;

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
