use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Visibility};

pub fn rooted_ident_for(ident: &Ident) -> Ident {
    format_ident!("{}Rooted", ident)
}

pub fn emit_rooted_wrapper_and_rooting_impls(ident: &Ident, visibility: &Visibility) -> TokenStream {
    let rooted_ident = rooted_ident_for(ident);

    quote! {
        #[derive(Debug, Clone)]
        #visibility struct #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            inner: #ident,
            _root: ::std::marker::PhantomData<R>,
        }

        impl #ident {
            pub fn rooted<R>(self) -> #rooted_ident<R>
            where
                R: ::xccute_contract::CompositeShellRoot,
            {
                #rooted_ident {
                    inner: self,
                    _root: ::std::marker::PhantomData,
                }
            }
        }

        impl<R> ::xccute_contract::RootableSubCommand<R> for #ident
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            type Rooted = #rooted_ident<R>;

            fn rooted(self) -> Self::Rooted {
                #rooted_ident {
                    inner: self,
                    _root: ::std::marker::PhantomData,
                }
            }
        }

        impl<R> ::xccute_contract::XccutePolicyMetadata for #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
            #ident: ::xccute_contract::XccutePolicyMetadata,
        {
            fn xccute_policy(&self) -> ::xccute_contract::XccuteCommandPolicyMetadata {
                ::xccute_contract::XccutePolicyMetadata::xccute_policy(&self.inner)
            }
        }
    }
}
