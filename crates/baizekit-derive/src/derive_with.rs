use darling::{FromDeriveInput, FromField};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Generics, Ident, PathArguments, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct Input {
    ident: Ident,
    generics: Generics,
    data: darling::ast::Data<(), InputField>,
}

#[derive(Clone, Debug, FromField)]
pub struct InputField {
    ident: Option<Ident>,
    ty: Type,
}

pub fn derive_with_impl(input: Input) -> Result<TokenStream2, darling::Error> {
    let name = input.ident;
    let generics = input.generics;

    let generic_params = &generics.params;
    let (_, _, where_clause) = generics.split_for_impl();

    let data = input.data.take_struct().expect("only named structs are supported");

    let setter_methods = data.fields.iter().map(|field| {
        let field_ident = field.ident.clone().expect("darling guarantees named fields");
        let function_name = Ident::new(&format!("with_{}", field_ident), field_ident.span());
        let (is_option_type, arg_ty) = extract_arg_type(&field.ty);

        if is_option_type {
            quote! {
                pub fn #function_name(mut self, value: #arg_ty) -> Self {
                    self.#field_ident = Some(value);
                    self
                }
            }
        } else {
            quote! {
                pub fn #function_name(mut self, value: #arg_ty) -> Self {
                    self.#field_ident = value;
                    self
                }
            }
        }
    });

    Ok(if generic_params.is_empty() {
        quote! {
            impl #name {
                #( #setter_methods )*
            }
        }
    } else {
        quote! {
            impl < #generic_params > #name < #generic_params > #where_clause {
                #( #setter_methods )*
            }
        }
    })
}

fn extract_arg_type(field_ty: &Type) -> (bool, proc_macro2::TokenStream) {
    if let Type::Path(type_path) = field_ty
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == "Option"
        && let PathArguments::AngleBracketed(ab) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = ab.args.first()
    {
        (true, quote! { #inner })
    } else {
        (false, quote! { #field_ty })
    }
}
