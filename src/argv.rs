use proc_macro2::TokenStream;
use quote::quote;

use crate::subcommand::{SubcommandArgKind, SubcommandFieldQuote};

pub fn emit_subcommand_field_argv_push(field: &SubcommandFieldQuote) -> syn::Result<TokenStream> {
    let field_ident = &field.ident;

    match &field.kind {
        SubcommandArgKind::Flag(flag) => {
            if !simple_impl_attr_kit::is_bool_ty(&field.ty) {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "#[arg(flag = ...)] currently requires bool",
                ));
            }

            Ok(quote! {
                if self.#field_ident {
                    out.push(::std::ffi::OsString::from(#flag));
                }
            })
        }
        SubcommandArgKind::KeyValue(key) => {
            if simple_impl_attr_kit::is_option_ty(&field.ty) {
                Ok(quote! {
                    if let Some(value) = &self.#field_ident {
                        out.push(::std::ffi::OsString::from(#key));
                        out.push(value.clone());
                    }
                })
            } else {
                Ok(quote! {
                    out.push(::std::ffi::OsString::from(#key));
                    out.push(self.#field_ident.clone());
                })
            }
        }
        SubcommandArgKind::Positional => {
            if simple_impl_attr_kit::is_option_ty(&field.ty) {
                Ok(quote! {
                    if let Some(value) = &self.#field_ident {
                        out.push(value.clone());
                    }
                })
            } else {
                Ok(quote! {
                    out.push(self.#field_ident.clone());
                })
            }
        }
    }
}
