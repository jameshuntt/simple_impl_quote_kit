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

/// A surface entry inside a surface: the method that reaches it and its type,
/// a `CompositeSubCommand`. It takes no `init` arguments; its commands take
/// theirs.
#[derive(Debug, Clone)]
pub struct CompositeNestedSurfaceQuote {
    pub method: Ident,
    pub ty: Type,
}

impl CompositeNestedSurfaceQuote {
    pub fn new(method: Ident, ty: Type) -> Self {
        Self { method, ty }
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
    // r#type names the method `type`; the generated type is `Type...`
    syn::ext::IdentExt::unraw(ident)
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

/// The surface, holding commands and surfaces.
/// Every command, at any depth, is its leaf's nested wrapper
/// (`xccute_contract::RootableNestedCommand`), so it keeps the leaf's
/// setters, exit statuses, validation and policy; `{Surface}{Method}Command<R>`
/// names the one directly under this surface, as it did. The surface also
/// gets its nested wrapper, for when it sits under another surface, and its
/// help lines.
pub fn emit_composite_surface_impls_with(spec: &CompositeSurfaceQuote, surfaces: &[CompositeNestedSurfaceQuote]) -> TokenStream {
    let ident = &spec.ident;
    let visibility = &spec.visibility;
    let segment = &spec.segment;
    let rooted_ident = surface_rooted_ident_for(ident);

    // under a root the prefix is the surface itself; under a surface it is
    // this surface's nested wrapper
    let rooted_prefix = (quote! { self.inner }, quote! { #ident });
    let nested_prefix = (quote! { self }, quote! { Self });
    let methods = spec
        .commands
        .iter()
        .map(|command| emit_surface_command_method(command, &rooted_prefix, true))
        .chain(surfaces.iter().map(|surface| emit_surface_surface_method(surface, &rooted_prefix)));
    let nested_methods: Vec<TokenStream> = spec
        .commands
        .iter()
        .map(|command| emit_surface_command_method(command, &nested_prefix, false))
        .chain(surfaces.iter().map(|surface| emit_surface_surface_method(surface, &nested_prefix)))
        .collect();
    let wrappers = spec
        .commands
        .iter()
        .map(|command| emit_surface_command_wrapper(spec, command));
    let field_reader = crate::composite_root::emit_declared_field_reader(ident, &spec.declared_fields);
    let nested = crate::nested::emit_nested_surface_impls(ident, visibility, &nested_methods);
    let entry_types: Vec<&Type> = spec.commands.iter().map(|command| &command.ty).chain(surfaces.iter().map(|surface| &surface.ty)).collect();
    let help = crate::nested::emit_entries_help_impl(ident, segment, &entry_types);

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

        #nested

        #help
    }
}

/// A command entry's method. `prefix` is what the command sits after and its
/// type; `named` writes the return type as `{Surface}{Method}Command<R>`, the
/// name a command directly under the surface has.
fn emit_surface_command_method(
    command: &CompositeSurfaceCommandQuote,
    prefix: &(TokenStream, TokenStream),
    named: bool,
) -> TokenStream {
    let method = &command.method;
    let ty = &command.ty;
    let rooted_ident = &command.rooted_ident;
    let init_args = &command.init_args;
    let (prefix_expr, prefix_ty) = prefix;

    let inner_expr = if init_args.is_empty() {
        quote! { <#ty as ::std::default::Default>::default() }
    } else {
        quote! { #ty::new(#(#init_args),*) }
    };

    let body = quote! {
        <#ty as ::xccute_contract::RootableNestedCommand<R, #prefix_ty>>::nested(#inner_expr, #prefix_expr)
    };
    let returns = if named {
        quote! { #rooted_ident<R> }
    } else {
        quote! { <#ty as ::xccute_contract::RootableNestedCommand<R, #prefix_ty>>::Nested }
    };

    if init_args.is_empty() {
        quote! {
            pub fn #method(self) -> #returns {
                #body
            }
        }
    } else {
        let params = init_args.iter().map(|arg| {
            quote! { #arg: impl ::std::convert::Into<::std::ffi::OsString> }
        });

        quote! {
            pub fn #method(self, #(#params),*) -> #returns {
                #body
            }
        }
    }
}

/// A surface entry's method: the surface under this one, handed `prefix`.
fn emit_surface_surface_method(surface: &CompositeNestedSurfaceQuote, prefix: &(TokenStream, TokenStream)) -> TokenStream {
    let method = &surface.method;
    let ty = &surface.ty;
    let (prefix_expr, prefix_ty) = prefix;

    quote! {
        pub fn #method(self) -> <#ty as ::xccute_contract::RootableNestedSurface<R, #prefix_ty>>::NestedSurface
        where
            #ty: ::std::default::Default,
        {
            <#ty as ::xccute_contract::RootableNestedSurface<R, #prefix_ty>>::nested_surface(
                <#ty as ::std::default::Default>::default(),
                #prefix_expr,
            )
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
        /// The command directly under this surface: its leaf's nested wrapper,
        /// with the leaf's setters, exit statuses, validation and policy.
        #visibility type #rooted_ident<R> =
            <#command_ty as ::xccute_contract::RootableNestedCommand<R, #surface_ty>>::Nested;
    }
}

/// The surface as 0.3.3 wrote it: each command a `{Surface}{Method}Command<R>`
/// struct with the leaf's argv, validation and policy, but not its setters or
/// exit statuses, and no surface inside it. simple_impl_derive writes
/// [`emit_composite_surface_impls_with`] instead.
pub fn emit_composite_surface_impls(spec: &CompositeSurfaceQuote) -> TokenStream {
    let ident = &spec.ident;
    let visibility = &spec.visibility;
    let segment = &spec.segment;
    let rooted_ident = surface_rooted_ident_for(ident);

    let methods = spec
        .commands
        .iter()
        .map(emit_rooted_surface_command_method);
    let wrappers = spec
        .commands
        .iter()
        .map(|command| emit_rooted_surface_command_wrapper(spec, command));
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

fn emit_rooted_surface_command_method(command: &CompositeSurfaceCommandQuote) -> TokenStream {
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

fn emit_rooted_surface_command_wrapper(
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
