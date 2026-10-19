//! What carries a leaf through surfaces to any depth, and the help lines of
//! every level.
//!
//! A leaf gets `{Leaf}Nested<R, P>`: the leaf after the words of `P`, the
//! surface it sits under, with the same setters, exit statuses, validation
//! and policy its rooted wrapper has. A surface gets `{Surface}Nested<R, P>`,
//! itself a prefix for what is under it. The surface finds each one through
//! `xccute_contract::RootableNestedCommand` and `RootableNestedSurface`, as a
//! root finds a leaf through `RootableSubCommand`.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type, Visibility};

/// `{Leaf}Nested` / `{Surface}Nested`.
pub fn nested_ident_for(ident: &Ident) -> Ident {
    format_ident!("{}Nested", ident)
}

/// A leaf's nested wrapper and its `RootableNestedCommand` impl. `setters`
/// are the leaf's rooted setters (they set `self.inner`), `status_methods`
/// its `ok_codes` and `exit_statuses` (nothing when it names none).
pub fn emit_nested_command_impls(
    ident: &Ident,
    visibility: &Visibility,
    setters: &[TokenStream],
    status_methods: &TokenStream,
) -> TokenStream {
    let nested_ident = nested_ident_for(ident);

    quote! {
        #[derive(Debug, Clone)]
        #visibility struct #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            prefix: P,
            inner: #ident,
            _root: ::std::marker::PhantomData<R>,
        }

        impl<R, P> ::xccute_contract::RootableNestedCommand<R, P> for #ident
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart,
        {
            type Nested = #nested_ident<R, P>;

            fn nested(self, prefix: P) -> Self::Nested {
                #nested_ident {
                    prefix,
                    inner: self,
                    _root: ::std::marker::PhantomData,
                }
            }
        }

        impl<R, P> #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart,
        {
            #(#setters)*
        }

        impl<R, P> ::xccute_contract::CompositeShellCommand for #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart,
        {
            fn program(&self) -> &::std::ffi::OsStr {
                R::program()
            }

            fn push_args(&self, out: &mut ::std::vec::Vec<::std::ffi::OsString>) {
                ::xccute_contract::CompositeArgvPart::push_argv_part(&self.prefix, out);
                ::xccute_contract::CompositeArgvPart::push_argv_part(&self.inner, out);
            }
            #status_methods
        }

        impl<R, P> ::xccute_contract::ValidatedCommand for #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart,
            #ident: ::xccute_contract::ValidatedCommand,
        {
            fn validate(&self) -> ::xccute_contract::CommandValidationResult {
                ::xccute_contract::ValidatedCommand::validate(&self.inner)
            }
        }

        impl<R, P> ::xccute_contract::XccutePolicyMetadata for #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart + ::xccute_contract::XccutePolicyMetadata,
            #ident: ::xccute_contract::XccutePolicyMetadata,
        {
            fn xccute_policy(&self) -> ::xccute_contract::XccuteCommandPolicyMetadata {
                ::xccute_contract::XccutePolicyMetadata::xccute_policy(&self.prefix)
                    .merge(::xccute_contract::XccutePolicyMetadata::xccute_policy(&self.inner))
            }
        }
    }
}

/// A surface's nested wrapper: the surface after the words of `P`, a prefix
/// itself for the commands and surfaces under it. `methods` are its entries'
/// methods, written with `self` as the prefix they hand down.
pub fn emit_nested_surface_impls(ident: &Ident, visibility: &Visibility, methods: &[TokenStream]) -> TokenStream {
    let nested_ident = nested_ident_for(ident);

    quote! {
        #[derive(Debug, Clone)]
        #visibility struct #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            prefix: P,
            inner: #ident,
            _root: ::std::marker::PhantomData<R>,
        }

        impl<R, P> ::xccute_contract::RootableNestedSurface<R, P> for #ident
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart,
        {
            type NestedSurface = #nested_ident<R, P>;

            fn nested_surface(self, prefix: P) -> Self::NestedSurface {
                #nested_ident {
                    prefix,
                    inner: self,
                    _root: ::std::marker::PhantomData,
                }
            }
        }

        impl<R, P> ::xccute_contract::CompositeArgvPart for #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart,
        {
            fn push_argv_part(&self, out: &mut ::std::vec::Vec<::std::ffi::OsString>) {
                ::xccute_contract::CompositeArgvPart::push_argv_part(&self.prefix, out);
                ::xccute_contract::CompositeArgvPart::push_argv_part(&self.inner, out);
            }
        }

        impl<R, P> ::xccute_contract::XccutePolicyMetadata for #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart + ::xccute_contract::XccutePolicyMetadata,
            #ident: ::xccute_contract::XccutePolicyMetadata,
        {
            fn xccute_policy(&self) -> ::xccute_contract::XccuteCommandPolicyMetadata {
                ::xccute_contract::XccutePolicyMetadata::xccute_policy(&self.prefix)
                    .merge(::xccute_contract::XccutePolicyMetadata::xccute_policy(&self.inner))
            }
        }

        impl<R, P> #nested_ident<R, P>
        where
            R: ::xccute_contract::CompositeShellRoot,
            P: ::xccute_contract::CompositeArgvPart,
        {
            #(#methods)*
        }
    }
}

/// A leaf's one help line: `words` as its declaration writes them, `about`
/// its doc.
pub fn emit_leaf_help_impl(ident: &Ident, words: &str, about: &str) -> TokenStream {
    quote! {
        impl ::xccute_contract::CommandHelp for #ident {
            fn help_lines() -> ::std::vec::Vec<::xccute_contract::HelpLine> {
                ::std::vec![::xccute_contract::HelpLine::new(#words, #about)]
            }
        }
    }
}

/// The help lines of a surface or a root: every line of every entry, each led
/// by `word` (the surface's segment, the root's program). The impl holds only
/// when every entry has help lines; the bounds are written `for<'__help>` so
/// an entry without them leaves the impl out instead of failing the build.
pub fn emit_entries_help_impl(ident: &Ident, word: &str, entries: &[&Type]) -> TokenStream {
    quote! {
        impl ::xccute_contract::CommandHelp for #ident
        where
            #(for<'__help> #entries: ::xccute_contract::CommandHelp,)*
        {
            fn help_lines() -> ::std::vec::Vec<::xccute_contract::HelpLine> {
                let mut lines = ::std::vec::Vec::new();
                #(lines.extend(<#entries as ::xccute_contract::CommandHelp>::help_lines());)*
                lines.into_iter().map(|line| line.under(#word)).collect()
            }
        }
    }
}
