use liaise_proc_macro::SynBuffer;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type, Visibility};

use crate::argv::emit_subcommand_field_argv_push;
use crate::composite_shell::emit_composite_shell_command_impl_with;
use crate::validation::{emit_validated_command_impl_skipping, ValidationRuleQuote};
use crate::rooted::{emit_rooted_wrapper_and_rooting_impls, rooted_ident_for};
use crate::setters::{emit_own_setter, emit_rooted_setter};

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

/// An exit status a leaf names: its code, its program's word for it, and
/// whether the line goes on after it. Emitted as an
/// `xccute_contract::ExitStatusWord`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitStatusQuote {
    pub code: i32,
    pub word: String,
    pub accepted: bool,
}

impl ExitStatusQuote {
    pub fn new(code: i32, word: impl Into<String>, accepted: bool) -> Self {
        Self { code, word: word.into(), accepted }
    }
}

/// What a leaf says about its line besides its fields, for
/// [`emit_simple_subcommand_contract_impls_shaped`]. The default says nothing
/// more, and emits exactly what [`emit_simple_subcommand_contract_impls_skipping`]
/// does: the segment leads, the leaf runs under a root, it names no status.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubcommandShapeQuote {
    without_segment: bool,
    program: Option<String>,
    exit_statuses: Vec<ExitStatusQuote>,
    ok_codes: Vec<i32>,
}

impl SubcommandShapeQuote {
    pub fn new() -> Self {
        Self::default()
    }

    /// No word of its own: the fields lead the line (`chown OWNER PATH`), and
    /// the spec's segment is not pushed.
    pub fn without_segment(mut self) -> Self {
        self.without_segment = true;
        self
    }

    /// The leaf is its own root, running `program` (`hostname`): it gets
    /// `CompositeShellRoot` and `CompositeShellCommand` itself, and its
    /// setters, so it runs without a `CompositeShell`.
    pub fn with_program(mut self, program: impl Into<String>) -> Self {
        self.program = Some(program.into());
        self
    }

    /// The statuses the leaf names, in order, and the codes it accepts
    /// (`ok_codes`), as the caller decided them.
    pub fn with_exit_statuses(mut self, statuses: Vec<ExitStatusQuote>, ok_codes: Vec<i32>) -> Self {
        self.exit_statuses = statuses;
        self.ok_codes = ok_codes;
        self
    }

    pub fn is_without_segment(&self) -> bool {
        self.without_segment
    }

    pub fn program(&self) -> Option<&str> {
        self.program.as_deref()
    }

    pub fn exit_statuses(&self) -> &[ExitStatusQuote] {
        &self.exit_statuses
    }

    pub fn ok_codes(&self) -> &[i32] {
        &self.ok_codes
    }
}

pub fn emit_simple_subcommand_contract_impls(
    spec: &SimpleSubcommandQuote,
) -> syn::Result<TokenStream> {
    emit_simple_subcommand_contract_impls_skipping(spec, &[], &[])
}

/// [`emit_simple_subcommand_contract_impls`] for a derive that already
/// reported the fields in `skipped` and left them out of `spec`: every other
/// field and every validation rule is still checked, and a rule naming a
/// skipped field is not reported for it again. `rule_contexts[i]`, when
/// given, is how validation rule `i` reads in errors. The tokens are only
/// meaningful when `skipped` is empty.
pub fn emit_simple_subcommand_contract_impls_skipping(
    spec: &SimpleSubcommandQuote,
    skipped: &[Ident],
    rule_contexts: &[String],
) -> syn::Result<TokenStream> {
    emit_simple_subcommand_contract_impls_shaped(spec, skipped, rule_contexts, &SubcommandShapeQuote::default())
}

/// [`emit_simple_subcommand_contract_impls_skipping`] for a leaf whose line
/// has more to it ([`SubcommandShapeQuote`]): no segment, its own program, the
/// exit statuses it names. With the default shape it emits exactly what
/// `_skipping` does.
pub fn emit_simple_subcommand_contract_impls_shaped(
    spec: &SimpleSubcommandQuote,
    skipped: &[Ident],
    rule_contexts: &[String],
    shape: &SubcommandShapeQuote,
) -> syn::Result<TokenStream> {
    let ident = &spec.ident;
    let segment = &spec.segment;
    let rooted_ident = rooted_ident_for(ident);

    let rooted_impls = emit_rooted_wrapper_and_rooting_impls(&spec.ident, &spec.visibility);
    let mut errs = SynBuffer::new();
    let mut setters = Vec::new();
    let mut own_setters = Vec::new();
    let mut argv_pushes = Vec::new();
    for field in &spec.fields {
        // the setter and the argv push check the same field type; ask once
        if let Some(setter) = errs.absorb(emit_rooted_setter(field)) {
            setters.push(setter);
            argv_pushes.extend(errs.absorb(emit_subcommand_field_argv_push(field)));
            if shape.program.is_some() {
                own_setters.extend(errs.absorb(emit_own_setter(field)));
            }
        }
    }
    let constructor = errs.absorb(emit_simple_subcommand_constructor(spec));
    let status_methods = emit_exit_status_methods(shape);
    let shell_command_impl = emit_composite_shell_command_impl_with(&spec.ident, &status_methods);
    let validated_command_impl = errs.absorb(emit_validated_command_impl_skipping(spec, skipped, rule_contexts));
    errs.finish()?;
    let segment_push = if shape.without_segment {
        TokenStream::new()
    } else {
        quote! { out.push(::std::ffi::OsString::from(#segment)); }
    };
    let own_root = shape.program.as_ref().map(|program| emit_own_root_impls(ident, program, &status_methods, &own_setters));

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
                #segment_push
                #(#argv_pushes)*
            }
        }

        #shell_command_impl
        #validated_command_impl
        #own_root
    })
}

/// A leaf's `ok_codes` and `exit_statuses`, for its `CompositeShellCommand`
/// impls; nothing when it names no status.
fn emit_exit_status_methods(shape: &SubcommandShapeQuote) -> TokenStream {
    if shape.exit_statuses.is_empty() {
        return TokenStream::new();
    }
    let ok_codes = &shape.ok_codes;
    let statuses = shape.exit_statuses.iter().map(|status| {
        let (code, word, accepted) = (status.code, &status.word, status.accepted);
        quote! { ::xccute_contract::ExitStatusWord { code: #code, word: #word, accepted: #accepted } }
    });
    quote! {
        fn ok_codes(&self) -> &'static [i32] {
            &[#(#ok_codes),*]
        }

        fn exit_statuses(&self) -> &'static [::xccute_contract::ExitStatusWord] {
            &[#(#statuses),*]
        }
    }
}

/// A leaf that is its own root: it names its program, runs as itself, and has
/// its setters, the same ones its rooted wrapper has.
fn emit_own_root_impls(ident: &Ident, program: &str, status_methods: &TokenStream, own_setters: &[TokenStream]) -> TokenStream {
    quote! {
        impl ::xccute_contract::CompositeShellRoot for #ident {
            fn program() -> &'static ::std::ffi::OsStr {
                ::std::ffi::OsStr::new(#program)
            }
        }

        impl ::xccute_contract::CompositeShellCommand for #ident {
            fn program(&self) -> &::std::ffi::OsStr {
                <Self as ::xccute_contract::CompositeShellRoot>::program()
            }

            fn push_args(&self, out: &mut ::std::vec::Vec<::std::ffi::OsString>) {
                ::xccute_contract::CompositeArgvPart::push_argv_part(self, out);
            }
            #status_methods
        }

        impl #ident {
            #(#own_setters)*
        }
    }
}

fn emit_simple_subcommand_constructor(spec: &SimpleSubcommandQuote) -> syn::Result<TokenStream> {
    if !spec.fields.iter().any(|field| field.init_required) {
        return Ok(TokenStream::new());
    }

    let mut errs = SynBuffer::new();
    for field in spec.fields.iter().filter(|field| field.init_required) {
        if matches!(field.kind, SubcommandArgKind::Flag(_)) {
            errs.push(syn::Error::new_spanned(
                &field.ty,
                "#[builder(init_required)] cannot be used with #[arg(flag = ...)]",
            ));
        }
    }
    errs.finish()?;

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
