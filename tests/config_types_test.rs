use prunify::PrunifyConfig;
use std::path::PathBuf;

/// Deserialize a basic YAML with scheme_dir and verify all fields.
#[test]
fn test_deserialize_basic_yaml() {
    let yaml = r#"
scheme_dir: ./my-schemes
verbose: true
no_color: false
strict: true
"#;

    let config: PrunifyConfig =
        serde_yaml::from_str(yaml).expect("valid YAML should deserialize");

    assert_eq!(config.scheme_dir, Some(PathBuf::from("./my-schemes")));
    assert_eq!(config.verbose, Some(true));
    assert_eq!(config.no_color, Some(false));
    assert_eq!(config.strict, Some(true));
}

/// Empty YAML should deserialize with all fields as None
/// (callers use `.unwrap_or(default)` to apply defaults).
#[test]
fn test_empty_yaml_uses_defaults() {
    let yaml = "{}";

    let config: PrunifyConfig =
        serde_yaml::from_str(yaml).expect("empty YAML should deserialize");

    // All fields are Option; absent from YAML → None
    assert_eq!(config.scheme_dir, None);
    assert_eq!(config.verbose, None);
    assert_eq!(config.no_color, None);
    assert_eq!(config.strict, None);
}

/// `#[serde(deny_unknown_fields)]` causes unknown fields to be rejected.
#[test]
fn test_unknown_field_ignored() {
    let yaml = r#"
scheme_dir: ./my-schemes
unknown_field: something
"#;

    let result: Result<PrunifyConfig, _> = serde_yaml::from_str(yaml);
    assert!(
        result.is_err(),
        "expected unknown field to cause a deserialization error, got {:?}",
        result
    );
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("unknown_field") || msg.contains("unknown field"),
        "error should mention the unknown field, got: {msg}"
    );
}

/// Verify scheme_dir resolves to a proper PathBuf.
#[test]
fn test_scheme_dir_path() {
    let yaml = r#"
scheme_dir: /etc/prunify/schemes
"#;

    let config: PrunifyConfig =
        serde_yaml::from_str(yaml).expect("valid YAML should deserialize");

    let path = config.scheme_dir.expect("scheme_dir should be Some");
    assert!(path.is_absolute(), "expected absolute path, got: {path:?}");
    assert_eq!(path, PathBuf::from("/etc/prunify/schemes"));

    // Also test a relative path
    let yaml_rel = r#"
scheme_dir: ./custom/schemes
"#;
    let config_rel: PrunifyConfig =
        serde_yaml::from_str(yaml_rel).expect("relative path YAML should deserialize");
    let rel_path = config_rel.scheme_dir.expect("scheme_dir should be Some");
    assert!(
        rel_path.is_relative(),
        "expected relative path, got: {rel_path:?}"
    );
    assert_eq!(rel_path, PathBuf::from("./custom/schemes"));
}
