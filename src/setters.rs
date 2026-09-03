use proc_macro2::TokenStream;
use quote::quote;

use crate::subcommand::{SubcommandArgKind, SubcommandFieldQuote};

pub fn emit_rooted_setter(field: &SubcommandFieldQuote) -> syn::Result<TokenStream> {
    let field_ident = &field.ident;
    let method_ident = &field.method;

    match &field.kind {
        SubcommandArgKind::Flag(_) => {
            if !simple_impl_attr_kit::is_bool_ty(&field.ty) {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "#[arg(flag = ...)] currently requires bool",
                ));
            }

            Ok(quote! {
                pub fn #method_ident(mut self) -> Self {
                    self.inner.#field_ident = true;
                    self
                }
            })
        }
        SubcommandArgKind::KeyValue(_) | SubcommandArgKind::Positional => {
            if simple_impl_attr_kit::is_option_ty(&field.ty) {
                Ok(quote! {
                    pub fn #method_ident(mut self, value: impl Into<::std::ffi::OsString>) -> Self {
                        self.inner.#field_ident = Some(value.into());
                        self
                    }
                })
            } else {
                Ok(quote! {
                    pub fn #method_ident(mut self, value: impl Into<::std::ffi::OsString>) -> Self {
                        self.inner.#field_ident = value.into();
                        self
                    }
                })
            }
        }
    }
}
