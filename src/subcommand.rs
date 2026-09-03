use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type, Visibility};

use crate::argv::emit_subcommand_field_argv_push;
use crate::composite_shell::emit_composite_shell_command_impl;
use crate::validation::{emit_validated_command_impl, ValidationRuleQuote};
use crate::rooted::{emit_rooted_wrapper_and_rooting_impls, rooted_ident_for};
use crate::setters::emit_rooted_setter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubcommandArgKind {
    Flag(String),
    KeyValue(String),
    Positional,
}

#[derive(Debug, Clone)]
pub struct SubcommandFieldQuote {
    pub ident: Ident,
    pub ty: Type,
    pub method: Ident,
    pub kind: SubcommandArgKind,
    pub init_required: bool,
}

impl SubcommandFieldQuote {
    pub fn new(ident: Ident, ty: Type, method: Ident, kind: SubcommandArgKind) -> Self {
        Self {
            ident,
            ty,
            method,
            kind,
            init_required: false,
        }
    }

    pub fn with_init_required(mut self, init_required: bool) -> Self {
        self.init_required = init_required;
        self
    }
}

#[derive(Debug, Clone)]
pub struct SimpleSubcommandQuote {
    pub ident: Ident,
    pub visibility: Visibility,
    pub segment: String,
    pub fields: Vec<SubcommandFieldQuote>,
    pub validation_rules: Vec<ValidationRuleQuote>,
}

impl SimpleSubcommandQuote {
    pub fn new(
        ident: Ident,
        visibility: Visibility,
        segment: impl Into<String>,
        fields: Vec<SubcommandFieldQuote>,
    ) -> Self {
        Self {
            ident,
            visibility,
            segment: segment.into(),
            fields,
            validation_rules: Vec::new(),
        }
    }

    pub fn with_validation_rules(
        mut self,
        validation_rules: Vec<ValidationRuleQuote>,
    ) -> Self {
        self.validation_rules = validation_rules;
        self
    }
}

pub fn emit_simple_subcommand_contract_impls(
    spec: &SimpleSubcommandQuote,
) -> syn::Result<TokenStream> {
    let ident = &spec.ident;
    let segment = &spec.segment;
    let rooted_ident = rooted_ident_for(ident);

    let rooted_impls = emit_rooted_wrapper_and_rooting_impls(&spec.ident, &spec.visibility);
    let setters = spec
        .fields
        .iter()
        .map(emit_rooted_setter)
        .collect::<syn::Result<Vec<_>>>()?;
    let argv_pushes = spec
        .fields
        .iter()
        .map(emit_subcommand_field_argv_push)
        .collect::<syn::Result<Vec<_>>>()?;
    let constructor = emit_simple_subcommand_constructor(spec)?;
    let shell_command_impl = emit_composite_shell_command_impl(&spec.ident);
    let validated_command_impl = emit_validated_command_impl(spec)?;

    Ok(quote! {
        #constructor

        #rooted_impls

        impl<R> #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            #(#setters)*
        }

        impl ::xccute_contract::CompositeArgvPart for #ident {
            fn push_argv_part(&self, out: &mut ::std::vec::Vec<::std::ffi::OsString>) {
                out.push(::std::ffi::OsString::from(#segment));
                #(#argv_pushes)*
            }
        }

        #shell_command_impl
        #validated_command_impl
    })
}

fn emit_simple_subcommand_constructor(spec: &SimpleSubcommandQuote) -> syn::Result<TokenStream> {
    if !spec.fields.iter().any(|field| field.init_required) {
        return Ok(TokenStream::new());
    }

    for field in spec.fields.iter().filter(|field| field.init_required) {
        if matches!(field.kind, SubcommandArgKind::Flag(_)) {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "#[builder(init_required)] cannot be used with #[arg(flag = ...)]",
            ));
        }
    }

    let ident = &spec.ident;
    let params = spec.fields.iter().filter(|field| field.init_required).map(|field| {
        let field_ident = &field.ident;
        quote! { #field_ident: impl ::std::convert::Into<::std::ffi::OsString> }
    });
    let assignments = spec.fields.iter().map(|field| {
        let field_ident = &field.ident;
        if field.init_required {
            quote! { #field_ident: #field_ident.into() }
        } else {
            quote! { #field_ident: ::std::default::Default::default() }
        }
    });

    Ok(quote! {
        impl #ident {
            pub fn new(#(#params),*) -> Self {
                Self {
                    #(#assignments),*
                }
            }
        }
    })
}
