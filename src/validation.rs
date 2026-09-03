use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path, Type};

use crate::rooted::rooted_ident_for;
use crate::subcommand::{SimpleSubcommandQuote, SubcommandFieldQuote};

#[derive(Debug, Clone)]
pub enum ValidationRuleQuote {
    Requires { field: Ident, required: Ident },
    InvalidWithout { field: Ident, required: Ident },
    OnlyPairWith { field: Ident, paired_with: Ident },
    ConflictsWith { field: Ident, conflicts_with: Ident },
    OneOf { fields: Vec<Ident> },
    AtLeastOneOf { fields: Vec<Ident> },
    CustomFunction { function_path: Path },
}

pub fn emit_validated_command_impl(spec: &SimpleSubcommandQuote) -> syn::Result<TokenStream> {
    let ident = &spec.ident;
    let rooted_ident = rooted_ident_for(ident);
    let checks = spec
        .validation_rules
        .iter()
        .map(|rule| emit_validation_rule_check(rule, &spec.fields))
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(quote! {
        impl ::xccute_contract::ValidatedCommand for #ident {
            fn validate(&self) -> ::xccute_contract::CommandValidationResult {
                #(#checks)*
                Ok(())
            }
        }

        impl<R> ::xccute_contract::ValidatedCommand for #rooted_ident<R>
        where
            R: ::xccute_contract::CompositeShellRoot,
        {
            fn validate(&self) -> ::xccute_contract::CommandValidationResult {
                ::xccute_contract::ValidatedCommand::validate(&self.inner)
            }
        }
    })
}

fn emit_validation_rule_check(
    rule: &ValidationRuleQuote,
    fields: &[SubcommandFieldQuote],
) -> syn::Result<TokenStream> {
    match rule {
        ValidationRuleQuote::Requires { field, required } => {
            let field_active = active_expr(field, fields)?;
            let required_active = active_expr(required, fields)?;
            let field_name = field.to_string();
            let required_name = required.to_string();
            Ok(quote! {
                if (#field_active) && !(#required_active) {
                    return Err(::xccute_contract::CommandValidationError::structural(
                        format!("`{}` requires `{}`", #field_name, #required_name),
                    )
                    .with_field(#field_name)
                    .with_rule("requires"));
                }
            })
        }
        ValidationRuleQuote::InvalidWithout { field, required } => {
            let field_active = active_expr(field, fields)?;
            let required_active = active_expr(required, fields)?;
            let field_name = field.to_string();
            let required_name = required.to_string();
            Ok(quote! {
                if (#field_active) && !(#required_active) {
                    return Err(::xccute_contract::CommandValidationError::structural(
                        format!("`{}` is invalid without `{}`", #field_name, #required_name),
                    )
                    .with_field(#field_name)
                    .with_rule("invalid_without"));
                }
            })
        }
        ValidationRuleQuote::OnlyPairWith { field, paired_with } => {
            let field_active = active_expr(field, fields)?;
            let paired_active = active_expr(paired_with, fields)?;
            let field_name = field.to_string();
            let paired_name = paired_with.to_string();
            Ok(quote! {
                if (#field_active) && !(#paired_active) {
                    return Err(::xccute_contract::CommandValidationError::structural(
                        format!("`{}` can only be paired with `{}`", #field_name, #paired_name),
                    )
                    .with_field(#field_name)
                    .with_rule("only_pair_with"));
                }
            })
        }
        ValidationRuleQuote::ConflictsWith { field, conflicts_with } => {
            let field_active = active_expr(field, fields)?;
            let conflict_active = active_expr(conflicts_with, fields)?;
            let field_name = field.to_string();
            let conflict_name = conflicts_with.to_string();
            Ok(quote! {
                if (#field_active) && (#conflict_active) {
                    return Err(::xccute_contract::CommandValidationError::structural(
                        format!("`{}` conflicts with `{}`", #field_name, #conflict_name),
                    )
                    .with_field(#field_name)
                    .with_rule("conflicts_with"));
                }
            })
        }
        ValidationRuleQuote::OneOf { fields: rule_fields } => {
            let active_exprs = rule_fields
                .iter()
                .map(|field| active_expr(field, fields))
                .collect::<syn::Result<Vec<_>>>()?;
            let names = rule_fields
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            Ok(quote! {
                {
                    let active_count = [#(#active_exprs),*].into_iter().filter(|active| *active).count();
                    if active_count != 1usize {
                        return Err(::xccute_contract::CommandValidationError::structural(
                            format!("exactly one of `{}` must be active", #names),
                        )
                        .with_field(#names)
                        .with_rule("one_of"));
                    }
                }
            })
        }
        ValidationRuleQuote::AtLeastOneOf { fields: rule_fields } => {
            let active_exprs = rule_fields
                .iter()
                .map(|field| active_expr(field, fields))
                .collect::<syn::Result<Vec<_>>>()?;
            let names = rule_fields
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            Ok(quote! {
                {
                    let active_count = [#(#active_exprs),*].into_iter().filter(|active| *active).count();
                    if active_count < 1usize {
                        return Err(::xccute_contract::CommandValidationError::structural(
                            format!("at least one of `{}` must be active", #names),
                        )
                        .with_field(#names)
                        .with_rule("at_least_one_of"));
                    }
                }
            })
        }
        ValidationRuleQuote::CustomFunction { function_path } => Ok(quote! {
            #function_path(self)?;
        }),
    }
}

fn active_expr(field: &Ident, fields: &[SubcommandFieldQuote]) -> syn::Result<TokenStream> {
    let field_name = field.to_string();
    let field_spec = fields
        .iter()
        .find(|candidate| candidate.ident == *field)
        .ok_or_else(|| {
            syn::Error::new_spanned(
                field,
                format!("validation rule references unknown field `{}`", field_name),
            )
        })?;

    Ok(active_expr_for_field(field, &field_spec.ty))
}

fn active_expr_for_field(field: &Ident, ty: &Type) -> TokenStream {
    if simple_impl_attr_kit::is_bool_ty(ty) {
        quote! { self.#field }
    } else if simple_impl_attr_kit::is_option_ty(ty) {
        quote! { self.#field.is_some() }
    } else {
        // Required/initialized command operands are always considered present.
        quote! { true }
    }
}
