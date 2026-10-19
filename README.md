# simple_impl_quote_kit

The token emitters behind [`simple_impl_derive`](https://crates.io/crates/simple_impl_derive).

Given the semantic models from
[`simple_impl_core`](https://crates.io/crates/simple_impl_core), these
functions produce the code: a rooted setter, the argv push for a
subcommand field, the impls of a subcommand or composite command surface,
a composite shell command, a validated command. They parse no attributes
and decide no meaning; they turn decided meaning into `TokenStream`s, each
one small enough to test on its own.

- `emit_rooted_setter`, `emit_rooted_wrapper_and_rooting_impls`
- `emit_subcommand_field_argv_push` and the subcommand surface emitters
- the composite root, surface and shell emitters
- `emit_validated_command_impl` with `ValidationRuleQuote`
- `emit_simple_subcommand_nested_impls`, `emit_composite_surface_impls_with`,
  `emit_composite_shell_help_impl` and the `nested` module: a leaf and a
  surface under surfaces to any depth, and every level's help lines

## License

MIT OR Apache-2.0.
