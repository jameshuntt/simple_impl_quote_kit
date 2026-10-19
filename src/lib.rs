//! Reusable quote emitters for the `simple_impl` macro family.
//!
//! This crate owns token emission helpers. It should not parse raw attributes
//! and should not decide semantic meaning. Parsed/semantic data should arrive
//! from `simple_impl_attr_kit`, `simple_impl_core`, or `simple_impl_plan`.

pub mod argv;
pub mod composite_root;
pub mod composite_shell;
pub mod composite_surface;
pub mod nested;
pub mod rooted;
pub mod setters;
pub mod subcommand;
pub mod validation;

pub use argv::emit_subcommand_field_argv_push;
pub use composite_root::{
    emit_composite_shell_help_impl,
    emit_composite_shell_root_impls,
    CompositeShellEntryKindQuote,
    CompositeShellEntryQuote,
    CompositeShellQuote,
};
pub use composite_shell::{emit_composite_shell_command_impl, emit_composite_shell_command_impl_with};
pub use composite_surface::{
    emit_composite_surface_impls,
    nested_command_rooted_ident_for,
    emit_composite_surface_impls_with,
    surface_rooted_ident_for,
    CompositeNestedSurfaceQuote,
    CompositeSurfaceCommandQuote,
    CompositeSurfaceQuote,
};
pub use nested::{
    emit_entries_help_impl,
    emit_leaf_help_impl,
    emit_nested_command_impls,
    emit_nested_surface_impls,
    nested_ident_for,
};
pub use rooted::emit_rooted_wrapper_and_rooting_impls;
pub use setters::{emit_own_setter, emit_rooted_setter};
pub use subcommand::{
    emit_simple_subcommand_contract_impls,
    emit_simple_subcommand_contract_impls_shaped,
    emit_simple_subcommand_contract_impls_skipping,
    emit_simple_subcommand_nested_impls,
    ExitStatusQuote,
    SimpleSubcommandQuote,
    SubcommandShapeQuote,
    SubcommandArgKind,
    SubcommandFieldQuote,
};
pub use validation::{emit_validated_command_impl, emit_validated_command_impl_skipping, ValidationRuleQuote};
