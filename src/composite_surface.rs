use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type, Visibility};

#[derive(Debug, Clone)]
pub struct CompositeSurfaceCommandQuote {
    pub method: Ident,
    pub ty: Type,
    pub rooted_ident: Ident,
    pub init_args: Vec<Ident>,
}

impl CompositeSurfaceCommandQuote {
    pub fn new(method: Ident, ty: Type, rooted_ident: Ident, init_args: Vec<Ident>) -> Self {
        Self {
            method,
            ty,
            rooted_ident,
            init_args,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompositeSurfaceQuote {
    pub ident: Ident,
    pub visibility: Visibility,
    pub segment: String,
    pub commands: Vec<CompositeSurfaceCommandQuote>,
    pub declared_fields: Vec<Ident>,
}

impl CompositeSurfaceQuote {
    pub fn new(
        ident: Ident,
        visibility: Visibility,
        segment: impl Into<String>,
        commands: Vec<CompositeSurfaceCommandQuote>,
    ) -> Self {
        Self {
            ident,
            visibility,
            segment: segment.into(),
            commands,
            declared_fields: Vec::new(),
        }
    }

    pub fn with_declared_fields(mut self, fields: Vec<Ident>) -> Self {
        self.declared_fields = fields;
        self
    }
}

pub fn surface_rooted_ident_for(ident: &Ident) -> Ident {
    format_ident!("{}Rooted", ident)
}

pub fn nested_command_rooted_ident_for(surface_ident: &Ident, method_ident: &Ident) -> Ident {
    format_ident!("{}{}Command", surface_ident, pascal_case_ident(method_ident))
}

fn pascal_case_ident(ident: &Ident) -> String {
    ident
        .to_string()
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<String>()
}

pub fn emit_composite_surface_impls(spec: &CompositeSurfaceQuote) -> TokenStream {
    let ident = &spec.ident;
    let visibility = &spec.visibility;
    let segment = &spec.segment;
    let rooted_ident = surface_rooted_ident_for(ident);

    let methods = spec
        .commands
        .iter()
        .map(emit_surface_command_method);
    let wrappers = spec
        .commands
        .iter()
        .map(|command| emit_surface_command_wrapper(spec, command));
    let field_reader = crate::composite_root::emit_declared_field_reader(ident, &spec.declared_fields);

    quote! {
        #field_reader

        #[derive(Debug, Clone)]
        #visibility struct #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            inner: #ident,
            _root: ::std::marker::PhantomData<R>,
        }

        impl<R> ::xccute_contract::RootableCompositeSurface<R> for #ident
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            type RootedSurface = #rooted_ident<R>;

            fn rooted_surface(self) -> Self::RootedSurface {
                #rooted_ident {
                    inner: self,
                    _root: ::std::marker::PhantomData,
                }
            }
        }

        impl ::xccute_contract::CompositeArgvPart for #ident {
            fn push_argv_part(&self, out: &mut ::std::vec::Vec<::std::ffi::OsString>) {
                out.push(::std::ffi::OsString::from(#segment));
            }
        }

        impl<R> #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            #(#methods)*
        }

        #(#wrappers)*
    }
}

fn emit_surface_command_method(command: &CompositeSurfaceCommandQuote) -> TokenStream {
    let method = &command.method;
    let ty = &command.ty;
    let rooted_ident = &command.rooted_ident;
    let init_args = &command.init_args;

    let inner_expr = if init_args.is_empty() {
        quote! { <#ty as ::std::default::Default>::default() }
    } else {
        quote! { #ty::new(#(#init_args),*) }
    };

    let body = quote! {
        #rooted_ident {
            prefix: self.inner,
            inner: #inner_expr,
            _root: ::std::marker::PhantomData,
        }
    };

    if init_args.is_empty() {
        quote! {
            pub fn #method(self) -> #rooted_ident<R> {
                #body
            }
        }
    } else {
        let params = init_args.iter().map(|arg| {
            quote! { #arg: impl ::std::convert::Into<::std::ffi::OsString> }
        });

        quote! {
            pub fn #method(self, #(#params),*) -> #rooted_ident<R> {
                #body
            }
        }
    }
}

fn emit_surface_command_wrapper(
    spec: &CompositeSurfaceQuote,
    command: &CompositeSurfaceCommandQuote,
) -> TokenStream {
    let visibility = &spec.visibility;
    let surface_ty = &spec.ident;
    let rooted_ident = &command.rooted_ident;
    let command_ty = &command.ty;

    quote! {
        #[derive(Debug, Clone)]
        #visibility struct #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            prefix: #surface_ty,
            inner: #command_ty,
            _root: ::std::marker::PhantomData<R>,
        }

        impl<R> ::xccute_contract::CompositeShellCommand for #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            fn program(&self) -> &::std::ffi::OsStr {
                R::program()
            }

            fn push_args(&self, out: &mut ::std::vec::Vec<::std::ffi::OsString>) {
                ::xccute_contract::CompositeArgvPart::push_argv_part(&self.prefix, out);
                ::xccute_contract::CompositeArgvPart::push_argv_part(&self.inner, out);
            }
        }

        impl<R> ::xccute_contract::ValidatedCommand for #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
            #command_ty: ::xccute_contract::ValidatedCommand,
        {
            fn validate(&self) -> ::xccute_contract::CommandValidationResult {
                ::xccute_contract::ValidatedCommand::validate(&self.inner)
            }
        }

        impl<R> ::xccute_contract::XccutePolicyMetadata for #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
            #surface_ty: ::xccute_contract::XccutePolicyMetadata,
            #command_ty: ::xccute_contract::XccutePolicyMetadata,
        {
            fn xccute_policy(&self) -> ::xccute_contract::XccuteCommandPolicyMetadata {
                ::xccute_contract::XccutePolicyMetadata::xccute_policy(&self.prefix)
                    .merge(::xccute_contract::XccutePolicyMetadata::xccute_policy(&self.inner))
            }
        }
    }
}
