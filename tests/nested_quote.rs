//! The emitters that carry a leaf and a surface under surfaces, and every
//! level's help lines.
use simple_impl_quote_kit::{
    emit_composite_shell_help_impl, emit_composite_surface_impls, emit_composite_surface_impls_with,
    emit_simple_subcommand_contract_impls_shaped, emit_simple_subcommand_nested_impls, CompositeNestedSurfaceQuote,
    CompositeShellEntryQuote, CompositeShellQuote, CompositeSurfaceCommandQuote, CompositeSurfaceQuote, ExitStatusQuote,
    SimpleSubcommandQuote, SubcommandArgKind, SubcommandFieldQuote, SubcommandShapeQuote,
};
use syn::{parse_quote, Ident};

fn ident(name: &str) -> Ident {
    syn::parse_str(name).unwrap()
}

fn add() -> SimpleSubcommandQuote {
    SimpleSubcommandQuote::new(
        ident("Add"),
        parse_quote!(pub),
        "add",
        vec![SubcommandFieldQuote::new(ident("role"), parse_quote!(Option<::std::ffi::OsString>), ident("role"), SubcommandArgKind::KeyValue("--role".into()))],
    )
}

fn shape() -> SubcommandShapeQuote {
    SubcommandShapeQuote::new().with_exit_statuses(vec![ExitStatusQuote::new(4, "exists", false)], vec![0])
}

#[test]
fn a_leaf_under_a_surface_has_its_setters_and_statuses() {
    let tokens = emit_simple_subcommand_nested_impls(&add(), &shape()).unwrap().to_string();
    assert!(tokens.contains("pub struct AddNested < R , P >"), "{tokens}");
    assert!(tokens.contains("impl < R , P > :: xccute_contract :: RootableNestedCommand < R , P > for Add"), "{tokens}");
    assert!(tokens.contains("pub fn role (mut self , value : impl Into < :: std :: ffi :: OsString >) -> Self { self . inner . role = Some (value . into ()) ; self }"), "{tokens}");
    assert!(tokens.contains("word : \"exists\""), "{tokens}");
    // no help line in the shape, no help impl
    assert!(!tokens.contains("CommandHelp"), "{tokens}");
    let tokens = emit_simple_subcommand_nested_impls(&add(), &shape().with_help("add [--role ROLE]", "Adds.")).unwrap().to_string();
    assert!(tokens.contains(":: xccute_contract :: HelpLine :: new (\"add [--role ROLE]\" , \"Adds.\")"), "{tokens}");
    assert_eq!(shape().with_help("w", "a").help(), Some(("w", "a")));
}

#[test]
fn the_shaped_tokens_do_not_change_with_a_help_line() {
    let without = emit_simple_subcommand_contract_impls_shaped(&add(), &[], &[], &shape()).unwrap().to_string();
    let with = emit_simple_subcommand_contract_impls_shaped(&add(), &[], &[], &shape().with_help("add", "")).unwrap().to_string();
    assert_eq!(without, with);
}

fn project() -> CompositeSurfaceQuote {
    CompositeSurfaceQuote::new(
        ident("Project"),
        parse_quote!(pub),
        "project",
        vec![CompositeSurfaceCommandQuote::new(ident("add"), parse_quote!(Add), ident("ProjectAddCommand"), vec![])],
    )
}

#[test]
fn a_surface_holds_surfaces_and_its_commands_are_their_leaves_nested() {
    let member = CompositeNestedSurfaceQuote::new(ident("member"), parse_quote!(Member));
    let tokens = emit_composite_surface_impls_with(&project(), &[member]).to_string();
    assert!(tokens.contains("pub type ProjectAddCommand < R > = < Add as :: xccute_contract :: RootableNestedCommand < R , Project >> :: Nested ;"), "{tokens}");
    assert!(tokens.contains("pub fn member (self) -> < Member as :: xccute_contract :: RootableNestedSurface < R , Project >> :: NestedSurface"), "{tokens}");
    assert!(tokens.contains("pub fn member (self) -> < Member as :: xccute_contract :: RootableNestedSurface < R , Self >> :: NestedSurface"), "{tokens}");
    assert!(tokens.contains("pub struct ProjectNested < R , P >"), "{tokens}");
    assert!(tokens.contains("for < '__help > Add : :: xccute_contract :: CommandHelp , for < '__help > Member : :: xccute_contract :: CommandHelp ,"), "{tokens}");
    assert!(tokens.contains("line . under (\"project\")"), "{tokens}");
}

#[test]
fn the_old_surface_emitter_writes_what_it_did() {
    let tokens = emit_composite_surface_impls(&project()).to_string();
    assert!(tokens.contains("pub struct ProjectAddCommand < R >"), "{tokens}");
    assert!(!tokens.contains("RootableNested"), "{tokens}");
    assert!(!tokens.contains("CommandHelp"), "{tokens}");
}

#[test]
fn a_root_has_every_line_under_its_program() {
    let spec = CompositeShellQuote::new(ident("Orb"), "orb", vec![CompositeShellEntryQuote::surface(ident("project"), parse_quote!(Project))]);
    let tokens = emit_composite_shell_help_impl(&spec).to_string();
    assert!(tokens.contains("impl :: xccute_contract :: CommandHelp for Orb where for < '__help > Project : :: xccute_contract :: CommandHelp ,"), "{tokens}");
    assert!(tokens.contains("line . under (\"orb\")"), "{tokens}");
}
