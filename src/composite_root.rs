use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeShellEntryKindQuote {
    Command,
    Surface,
}

#[derive(Debug, Clone)]
pub struct CompositeShellEntryQuote {
    pub kind: CompositeShellEntryKindQuote,
    pub method: Ident,
    pub ty: Type,
    pub init_args: Vec<Ident>,
}

impl CompositeShellEntryQuote {
    pub fn command(method: Ident, ty: Type) -> Self {
        Self {
            kind: CompositeShellEntryKindQuote::Command,
            method,
            ty,
            init_args: Vec::new(),
        }
    }

    pub fn surface(method: Ident, ty: Type) -> Self {
        Self {
            kind: CompositeShellEntryKindQuote::Surface,
            method,
            ty,
            init_args: Vec::new(),
        }
    }

    pub fn with_init_args(mut self, init_args: Vec<Ident>) -> Self {
        self.init_args = init_args;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CompositeShellQuote {
    pub ident: Ident,
    pub program: String,
    pub entries: Vec<CompositeShellEntryQuote>,
    pub declared_fields: Vec<Ident>,
}

impl CompositeShellQuote {
    pub fn new(
        ident: Ident,
        program: impl Into<String>,
        entries: Vec<CompositeShellEntryQuote>,
    ) -> Self {
        Self {
            ident,
            program: program.into(),
            entries,
            declared_fields: Vec::new(),
        }
    }

    pub fn with_declared_fields(mut self, fields: Vec<Ident>) -> Self {
        self.declared_fields = fields;
        self
    }
}

/// A hidden method that reads each field-style entry once, so the declaring
/// fields are not reported as never read.
pub fn emit_declared_field_reader(ident: &Ident, fields: &[Ident]) -> TokenStream {
    if fields.is_empty() {
        return TokenStream::new();
    }
    quote! {
        impl #ident {
            #[doc(hidden)]
            #[allow(dead_code)]
            fn __declared_composite_fields(&self) {
                #( let _ = &self.#fields; )*
            }
        }
    }
}

pub fn emit_composite_shell_root_impls(spec: &CompositeShellQuote) -> TokenStream {
    let ident = &spec.ident;
    let program = &spec.program;
    let methods = spec.entries.iter().map(|entry| emit_entry_method(ident, entry));
    let field_reader = emit_declared_field_reader(ident, &spec.declared_fields);

    quote! {
        #field_reader

        impl ::xccute_contract::CompositeShellRoot for #ident {
            fn program() -> &'static ::std::ffi::OsStr {
                ::std::ffi::OsStr::new(#program)
            }
        }

        impl #ident {
            #(#methods)*
        }
    }
}

fn emit_entry_method(root_ident: &Ident, entry: &CompositeShellEntryQuote) -> TokenStream {
    let method = &entry.method;
    let ty = &entry.ty;

    match entry.kind {
        CompositeShellEntryKindQuote::Command => emit_command_method(root_ident, method, ty, &entry.init_args),
        CompositeShellEntryKindQuote::Surface => quote! {
            pub fn #method() -> <#ty as ::xccute_contract::RootableCompositeSurface<#root_ident>>::RootedSurface
            where
                #ty: ::std::default::Default + ::xccute_contract::RootableCompositeSurface<#root_ident>,
            {
                <#ty as ::xccute_contract::RootableCompositeSurface<#root_ident>>::rooted_surface(
                    <#ty as ::std::default::Default>::default(),
                )
            }
        },
    }
}

fn emit_command_method(
    root_ident: &Ident,
    method: &Ident,
    ty: &Type,
    init_args: &[Ident],
) -> TokenStream {
    if init_args.is_empty() {
        quote! {
            pub fn #method() -> <#ty as ::xccute_contract::RootableSubCommand<#root_ident>>::Rooted
            where
                #ty: ::std::default::Default + ::xccute_contract::RootableSubCommand<#root_ident>,
            {
                <#ty as ::xccute_contract::RootableSubCommand<#root_ident>>::rooted(
                    <#ty as ::std::default::Default>::default(),
                )
            }
        }
    } else {
        let params = init_args.iter().map(|arg| {
            quote! { #arg: impl ::std::convert::Into<::std::ffi::OsString> }
        });

        quote! {
            pub fn #method(#(#params),*) -> <#ty as ::xccute_contract::RootableSubCommand<#root_ident>>::Rooted
            where
                #ty: ::xccute_contract::RootableSubCommand<#root_ident>,
            {
                <#ty as ::xccute_contract::RootableSubCommand<#root_ident>>::rooted(
                    #ty::new(#(#init_args),*)
                )
            }
        }
    }
}
