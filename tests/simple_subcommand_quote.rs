use simple_impl_quote_kit::{
    emit_composite_shell_root_impls,
    emit_composite_surface_impls,
    emit_simple_subcommand_contract_impls,
    CompositeShellEntryQuote,
    CompositeShellQuote,
    CompositeSurfaceCommandQuote,
    CompositeSurfaceQuote,
    SimpleSubcommandQuote,
    SubcommandArgKind,
    SubcommandFieldQuote,
    ValidationRuleQuote,
};
use syn::{parse_quote, Ident, Visibility};

fn ident(name: &str) -> Ident {
    syn::parse_str(name).unwrap()
}

#[test]
fn emits_simple_subcommand_contract_tokens() {
    let spec = SimpleSubcommandQuote::new(
        ident("GitCommit"),
        parse_quote!(pub),
        "commit",
        vec![
            SubcommandFieldQuote::new(
                ident("message"),
                parse_quote!(Option<::std::ffi::OsString>),
                ident("message"),
                SubcommandArgKind::KeyValue("-m".into()),
            ),
            SubcommandFieldQuote::new(
                ident("all"),
                parse_quote!(bool),
                ident("all"),
                SubcommandArgKind::Flag("-a".into()),
            ),
        ],
    );

    let tokens = emit_simple_subcommand_contract_impls(&spec).unwrap().to_string();

    assert!(tokens.contains("GitCommitRooted"));
    assert!(tokens.contains("CompositeArgvPart"));
    assert!(tokens.contains("CompositeShellCommand"));
    assert!(tokens.contains("ValidatedCommand"));
    assert!(tokens.contains("commit"));
}

#[test]
fn flag_setter_rejects_non_bool_fields() {
    let spec = SimpleSubcommandQuote::new(
        ident("Bad"),
        Visibility::Inherited,
        "bad",
        vec![SubcommandFieldQuote::new(
            ident("flag"),
            parse_quote!(::std::ffi::OsString),
            ident("flag"),
            SubcommandArgKind::Flag("--flag".into()),
        )],
    );

    let err = emit_simple_subcommand_contract_impls(&spec).unwrap_err();
    assert!(err.to_string().contains("requires bool"));
}

#[test]
fn positional_option_fields_are_supported() {
    let spec = SimpleSubcommandQuote::new(
        ident("Add"),
        Visibility::Inherited,
        "add",
        vec![SubcommandFieldQuote::new(
            ident("path"),
            parse_quote!(Option<::std::ffi::OsString>),
            ident("path"),
            SubcommandArgKind::Positional,
        )],
    );

    let tokens = emit_simple_subcommand_contract_impls(&spec).unwrap().to_string();
    assert!(tokens.contains("if let Some"));
    assert!(tokens.contains("path"));
}


#[test]
fn emits_composite_shell_root_command_method() {
    let spec = CompositeShellQuote::new(
        ident("Git"),
        "git",
        vec![CompositeShellEntryQuote::command(
            ident("commit"),
            parse_quote!(GitCommit),
        )],
    );

    let tokens = emit_composite_shell_root_impls(&spec).to_string();

    assert!(tokens.contains("CompositeShellRoot"));
    assert!(tokens.contains("RootableSubCommand"));
    assert!(tokens.contains("program"));
    assert!(tokens.contains("commit"));
    assert!(tokens.contains("git"));
}


#[test]
fn emits_init_required_constructor_for_simple_subcommand() {
    let spec = SimpleSubcommandQuote::new(
        ident("GitRemoteAdd"),
        Visibility::Inherited,
        "add",
        vec![
            SubcommandFieldQuote::new(
                ident("name"),
                parse_quote!(::std::ffi::OsString),
                ident("name"),
                SubcommandArgKind::Positional,
            )
            .with_init_required(true),
            SubcommandFieldQuote::new(
                ident("url"),
                parse_quote!(::std::ffi::OsString),
                ident("url"),
                SubcommandArgKind::Positional,
            )
            .with_init_required(true),
        ],
    );

    let tokens = emit_simple_subcommand_contract_impls(&spec).unwrap().to_string();
    assert!(tokens.contains("pub fn new"));
    assert!(tokens.contains("name"));
    assert!(tokens.contains("url"));
}

#[test]
fn emits_composite_surface_command_method() {
    let spec = CompositeSurfaceQuote::new(
        ident("GitRemote"),
        Visibility::Inherited,
        "remote",
        vec![CompositeSurfaceCommandQuote::new(
            ident("add"),
            parse_quote!(GitRemoteAdd),
            ident("GitRemoteAddCommand"),
            vec![ident("name"), ident("url")],
        )],
    );

    let tokens = emit_composite_surface_impls(&spec).to_string();
    assert!(tokens.contains("RootableCompositeSurface"));
    assert!(tokens.contains("CompositeArgvPart"));
    assert!(tokens.contains("GitRemoteAddCommand"));
    assert!(tokens.contains("pub fn add"));
    assert!(tokens.contains("remote"));
}

#[test]
fn emits_composite_shell_root_command_method_with_init_args() {
    let spec = CompositeShellQuote::new(
        ident("Git"),
        "git",
        vec![CompositeShellEntryQuote::command(
            ident("clone_repo"),
            parse_quote!(GitClone),
        )
        .with_init_args(vec![ident("url")])],
    );

    let tokens = emit_composite_shell_root_impls(&spec).to_string();

    assert!(tokens.contains("pub fn clone_repo"));
    assert!(tokens.contains("url"));
    assert!(tokens.contains("GitClone :: new"));
    assert!(tokens.contains("RootableSubCommand"));
}


#[test]
fn emits_structural_validation_checks_for_simple_subcommand() {
    let spec = SimpleSubcommandQuote::new(
        ident("GitBranchDelete"),
        Visibility::Inherited,
        "branch",
        vec![
            SubcommandFieldQuote::new(
                ident("delete"),
                parse_quote!(bool),
                ident("delete"),
                SubcommandArgKind::Flag("-d".into()),
            ),
            SubcommandFieldQuote::new(
                ident("force"),
                parse_quote!(bool),
                ident("force"),
                SubcommandArgKind::Flag("-f".into()),
            ),
        ],
    )
    .with_validation_rules(vec![ValidationRuleQuote::InvalidWithout {
        field: ident("delete"),
        required: ident("force"),
    }]);

    let tokens = emit_simple_subcommand_contract_impls(&spec).unwrap().to_string();
    assert!(tokens.contains("fn validate"));
    assert!(tokens.contains("invalid_without"));
    assert!(tokens.contains("CommandValidationError"));
    assert!(tokens.contains("delete"));
    assert!(tokens.contains("force"));
}

#[test]
fn emits_custom_validation_hook_for_simple_subcommand() {
    let spec = SimpleSubcommandQuote::new(
        ident("GitCloneChecked"),
        Visibility::Inherited,
        "clone",
        vec![SubcommandFieldQuote::new(
            ident("url"),
            parse_quote!(::std::ffi::OsString),
            ident("url"),
            SubcommandArgKind::Positional,
        )],
    )
    .with_validation_rules(vec![ValidationRuleQuote::CustomFunction {
        function_path: parse_quote!(validate_clone_url_preflight),
    }]);

    let tokens = emit_simple_subcommand_contract_impls(&spec).unwrap().to_string();
    assert!(tokens.contains("validate_clone_url_preflight"));
    assert!(tokens.contains("fn validate"));
}
