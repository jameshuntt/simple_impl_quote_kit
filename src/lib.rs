//! Reusable quote emitters for the `simple_impl` macro family.
//!
//! This crate owns token emission helpers. It should not parse raw attributes
//! and should not decide semantic meaning. Parsed/semantic data should arrive
//! from `simple_impl_attr_kit`, `simple_impl_core`, or `simple_impl_plan`.

pub mod argv;
pub mod composite_root;
pub mod composite_shell;
pub mod composite_surface;
pub mod rooted;
pub mod setters;
pub mod subcommand;
pub mod validation;

pub use argv::emit_subcommand_field_argv_push;
pub use composite_root::{
    emit_composite_shell_root_impls,
    CompositeShellEntryKindQuote,
    CompositeShellEntryQuote,
    CompositeShellQuote,
};
pub use composite_shell::emit_composite_shell_command_impl;
pub use composite_surface::{
    emit_composite_surface_impls,
    nested_command_rooted_ident_for,
    surface_rooted_ident_for,
    CompositeSurfaceCommandQuote,
    CompositeSurfaceQuote,
};
pub use rooted::emit_rooted_wrapper_and_rooting_impls;
pub use setters::emit_rooted_setter;
pub use subcommand::{
    emit_simple_subcommand_contract_impls,
    SimpleSubcommandQuote,
    SubcommandArgKind,
    SubcommandFieldQuote,
};
pub use validation::{emit_validated_command_impl, ValidationRuleQuote};
