use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::rooted::rooted_ident_for;

pub fn emit_composite_shell_command_impl(ident: &Ident) -> TokenStream {
    let rooted_ident = rooted_ident_for(ident);

    quote! {
        impl<R> ::xccute_contract::CompositeShellCommand for #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            fn program(&self) -> &::std::ffi::OsStr {
                R::program()
            }

            fn push_args(&self, out: &mut ::std::vec::Vec<::std::ffi::OsString>) {
                ::xccute_contract::CompositeArgvPart::push_argv_part(&self.inner, out);
            }
        }

    }
}
