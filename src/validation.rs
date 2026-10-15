use liaise_proc_macro::collect_all;
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
    emit_validated_command_impl_skipping(spec, &[], &[])
}

/// [`emit_validated_command_impl`] for a derive that already reported the
/// fields in `skipped` and left them out of `spec`: a rule naming one of them
/// is still checked for its other names, and the skipped name is not reported
/// again. `rule_contexts[i]`, when given, is how rule `i` reads in errors (the
/// rule as written, before the caller dropped any bad names). The tokens are
/// only meaningful when `skipped` is empty.
pub fn emit_validated_command_impl_skipping(
    spec: &SimpleSubcommandQuote,
    skipped: &[Ident],
    rule_contexts: &[String],
) -> syn::Result<TokenStream> {
    let ident = &spec.ident;
    let rooted_ident = rooted_ident_for(ident);
    let checks = collect_all(spec.validation_rules.iter().enumerate().map(|(i, rule)| {
        let context = rule_contexts.get(i).cloned().unwrap_or_else(|| rule_context(rule));
        emit_validation_rule_check(rule, &spec.fields, skipped, &context)
    }))?;

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

/// One rule's check. Its errors (names with no place of their own) end with
/// the rule, so two different rules never print the same message.
fn emit_validation_rule_check(
    rule: &ValidationRuleQuote,
    fields: &[SubcommandFieldQuote],
    skipped: &[Ident],
    context: &str,
) -> syn::Result<TokenStream> {
    rule_check(rule, fields, skipped).map_err(|e| {
        let parts = e.clone().into_iter().map(|part| syn::Error::new(part.span(), format!("{part}: in {context}")));
        parts.reduce(|mut all, part| {
            all.combine(part);
            all
        })
        .unwrap_or(e)
    })
}

fn rule_context(rule: &ValidationRuleQuote) -> String {
    let list = |names: &[Ident]| names.iter().map(|n| format!("\"{n}\"")).collect::<Vec<_>>().join(", ");
    match rule {
        ValidationRuleQuote::Requires { field, .. } => format!("#[requires] on `{field}`"),
        ValidationRuleQuote::InvalidWithout { field, .. } => format!("#[invalid_without] on `{field}`"),
        ValidationRuleQuote::OnlyPairWith { field, .. } => format!("#[only_pair_with] on `{field}`"),
        ValidationRuleQuote::ConflictsWith { field, .. } => format!("#[conflicts_with] on `{field}`"),
        ValidationRuleQuote::OneOf { fields } => format!("#[one_of({})]", list(fields)),
        ValidationRuleQuote::AtLeastOneOf { fields } => format!("#[at_least_one_of({})]", list(fields)),
        ValidationRuleQuote::CustomFunction { .. } => "#[validate(with)]".to_owned(),
    }
}

fn rule_check(
    rule: &ValidationRuleQuote,
    fields: &[SubcommandFieldQuote],
    skipped: &[Ident],
) -> syn::Result<TokenStream> {
    match rule {
        ValidationRuleQuote::Requires { field, required } => {
            let [field_active, required_active]: [TokenStream; 2] = collect_all([active_expr(field, fields, skipped), active_expr(required, fields, skipped)])?
                .try_into()
                .expect("two rule fields");
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
            let [field_active, required_active]: [TokenStream; 2] = collect_all([active_expr(field, fields, skipped), active_expr(required, fields, skipped)])?
                .try_into()
                .expect("two rule fields");
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
            let [field_active, paired_active]: [TokenStream; 2] = collect_all([active_expr(field, fields, skipped), active_expr(paired_with, fields, skipped)])?
                .try_into()
                .expect("two rule fields");
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
            let [field_active, conflict_active]: [TokenStream; 2] = collect_all([active_expr(field, fields, skipped), active_expr(conflicts_with, fields, skipped)])?
                .try_into()
                .expect("two rule fields");
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
            let active_exprs = collect_all(rule_fields.iter().map(|field| active_expr(field, fields, skipped)))?;
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
            let active_exprs = collect_all(rule_fields.iter().map(|field| active_expr(field, fields, skipped)))?;
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

fn active_expr(field: &Ident, fields: &[SubcommandFieldQuote], skipped: &[Ident]) -> syn::Result<TokenStream> {
    if skipped.contains(field) {
        // already reported by the caller; nothing is generated from this
        return Ok(quote! { false });
    }
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
