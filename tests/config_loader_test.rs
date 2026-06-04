use std::io::Write;
use std::path::{Path, PathBuf};

use prunify::{ConfigLoader, PrunifyConfig, PrunifyError};

/// Helper: write YAML content to a unique temp file and return its path.
fn write_temp_yaml(content: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("prunify_test_config_loader_it");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(format!("test_{}.yaml", std::process::id()));
    let mut file = std::fs::File::create(&path).expect("create temp file");
    write!(file, "{}", content).expect("write yaml content");
    path
}

/// Load a fully-specified YAML config and verify all 4 fields.
#[test]
fn test_load_yaml_config() {
    let path = write_temp_yaml(
        r#"
scheme_dir: ./my-schemes
verbose: true
no_color: false
strict: true
"#,
    );

    let config = ConfigLoader::load(Some(&path)).expect("load should succeed");

    assert_eq!(config.scheme_dir, Some(PathBuf::from("./my-schemes")));
    assert_eq!(config.verbose, Some(true));
    assert_eq!(config.no_color, Some(false));
    assert_eq!(config.strict, Some(true));
}

/// Loading a non-existent path should succeed and return PrunifyConfig::default().
#[test]
fn test_missing_config_uses_defaults() {
    let non_existent = Path::new("/tmp/does_not_exist_prunify_loader_test.yaml");
    let config =
        ConfigLoader::load(Some(non_existent)).expect("missing file should return default");

    let defaults = PrunifyConfig::default();
    assert_eq!(config.scheme_dir, defaults.scheme_dir);
    assert_eq!(config.verbose, defaults.verbose);
    assert_eq!(config.no_color, defaults.no_color);
    assert_eq!(config.strict, defaults.strict);
}

/// Invalid YAML content should produce a ConfigError.
#[test]
fn test_invalid_yaml_errors() {
    let dir = std::env::temp_dir().join("prunify_test_config_loader_it");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(format!("test_invalid_{}.yaml", std::process::id()));

    std::fs::write(&path, b"[what,ever\n").expect("write invalid yaml");

    let result = ConfigLoader::load(Some(&path));
    assert!(result.is_err(), "invalid YAML should produce an error");

    match result {
        Err(PrunifyError::ConfigError(msg)) => {
            assert!(!msg.is_empty(), "error message should not be empty");
        }
        other => panic!("expected ConfigError, got {:?}", other),
    }
}

/// Partial config (only some fields set) should merge with defaults.
#[test]
fn test_partial_config_merges_defaults() {
    let path = write_temp_yaml(
        r#"
verbose: true
"#,
    );

    let config = ConfigLoader::load(Some(&path)).expect("load should succeed");

    // Field present in YAML
    assert_eq!(config.verbose, Some(true));

    // Fields absent from YAML — filled from defaults
    assert_eq!(config.no_color, Some(false));
    assert_eq!(config.strict, Some(false));
    // scheme_dir default is None
    assert_eq!(config.scheme_dir, None);
}
