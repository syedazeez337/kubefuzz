use std::path::Path;

use kuberift::config::{parse_config, Config};

#[test]
fn empty_file_returns_defaults() {
    let cfg = parse_config("", Path::new("test.toml"));
    assert!(!cfg.general.read_only);
    assert!(cfg.general.default_namespace.is_empty());
    assert!(cfg.general.default_context.is_empty());
    assert!(cfg.general.default_resource.is_empty());
    assert!(cfg.general.editor.is_empty());
    assert!(cfg.general.shell.is_empty());
    assert!(cfg.ui.show_namespace);
    assert!(cfg.ui.show_age);
    assert!(!cfg.ui.show_context);
    assert_eq!(cfg.ui.truncate_name_length, 48);
}

#[test]
fn full_config_parses_correctly() {
    let raw = r#"
        [general]
        default_namespace = "production"
        default_context = "staging-cluster"
        default_resource = "pods"
        editor = "nvim"
        shell = "/bin/zsh"
        read_only = true

        [ui]
        show_namespace = false
        show_age = false
        show_context = true
        truncate_name_length = 64
    "#;
    let cfg = parse_config(raw, Path::new("test.toml"));
    assert_eq!(cfg.general.default_namespace, "production");
    assert_eq!(cfg.general.default_context, "staging-cluster");
    assert_eq!(cfg.general.default_resource, "pods");
    assert_eq!(cfg.general.editor, "nvim");
    assert_eq!(cfg.general.shell, "/bin/zsh");
    assert!(cfg.general.read_only);
    assert!(!cfg.ui.show_namespace);
    assert!(!cfg.ui.show_age);
    assert!(cfg.ui.show_context);
    assert_eq!(cfg.ui.truncate_name_length, 64);
}

#[test]
fn partial_config_fills_defaults() {
    let raw = r#"
        [general]
        default_namespace = "kube-system"
    "#;
    let cfg = parse_config(raw, Path::new("test.toml"));
    assert_eq!(cfg.general.default_namespace, "kube-system");
    assert!(cfg.general.default_context.is_empty());
    assert!(!cfg.general.read_only);
    // UI defaults should be intact
    assert!(cfg.ui.show_namespace);
    assert_eq!(cfg.ui.truncate_name_length, 48);
}

#[test]
fn unknown_keys_warn_but_dont_fail() {
    let raw = r#"
        [general]
        default_namespace = "prod"
        some_future_key = "hello"

        [plugins]
        name = "my-plugin"
    "#;
    // toml crate with serde(default) + deny_unknown_fields NOT set → parses fine
    let cfg = parse_config(raw, Path::new("test.toml"));
    assert_eq!(cfg.general.default_namespace, "prod");
}

#[test]
fn malformed_toml_returns_defaults() {
    let raw = "this is not [valid toml @@!";
    let cfg = parse_config(raw, Path::new("bad.toml"));
    // Should return defaults, not panic
    assert!(!cfg.general.read_only);
    assert!(cfg.general.default_namespace.is_empty());
}

#[test]
fn missing_file_returns_defaults() {
    let cfg = kuberift::config::load_from_path(Path::new("/nonexistent/config.toml"));
    assert!(cfg.is_none());
}

#[test]
fn default_trait_matches_empty_parse() {
    let from_default = Config::default();
    let from_empty = parse_config("", Path::new("test.toml"));

    assert_eq!(from_default.general.read_only, from_empty.general.read_only);
    assert_eq!(
        from_default.general.default_namespace,
        from_empty.general.default_namespace
    );
    assert_eq!(
        from_default.ui.truncate_name_length,
        from_empty.ui.truncate_name_length
    );
    assert_eq!(from_default.ui.show_namespace, from_empty.ui.show_namespace);
}
