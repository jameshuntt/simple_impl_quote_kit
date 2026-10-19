use proc_macro2::TokenStream;
use quote::quote;

use crate::subcommand::{SubcommandArgKind, SubcommandFieldQuote};

pub fn emit_rooted_setter(field: &SubcommandFieldQuote) -> syn::Result<TokenStream> {
    emit_setter_on(field, quote! { self.inner })
}

/// The same setter on the command itself, for a leaf that is its own root:
/// it sets the field directly instead of through the rooted wrapper.
pub fn emit_own_setter(field: &SubcommandFieldQuote) -> syn::Result<TokenStream> {
    emit_setter_on(field, quote! { self })
}

/// One setter, `place` the value that holds the field (`self.inner` on a
/// rooted wrapper, `self` on the command).
fn emit_setter_on(field: &SubcommandFieldQuote, place: TokenStream) -> syn::Result<TokenStream> {
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
                    #place.#field_ident = true;
                    self
                }
            })
        }
        SubcommandArgKind::KeyValue(_) | SubcommandArgKind::Positional => {
            if simple_impl_attr_kit::is_vec_ty(&field.ty) {
                // a repeated value: each call adds its values after the ones
                // already there
                Ok(quote! {
                    pub fn #method_ident(mut self, values: impl ::std::iter::IntoIterator<Item = impl Into<::std::ffi::OsString>>) -> Self {
                        #place.#field_ident.extend(values.into_iter().map(Into::into));
                        self
                    }
                })
            } else if simple_impl_attr_kit::is_option_ty(&field.ty) {
                Ok(quote! {
                    pub fn #method_ident(mut self, value: impl Into<::std::ffi::OsString>) -> Self {
                        #place.#field_ident = Some(value.into());
                        self
                    }
                })
            } else {
                Ok(quote! {
                    pub fn #method_ident(mut self, value: impl Into<::std::ffi::OsString>) -> Self {
                        #place.#field_ident = value.into();
                        self
                    }
                })
            }
        }
    }
}
