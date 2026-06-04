use std::collections::HashMap;
use std::path::PathBuf;

use prunifier::config::PrunifierConfig;
use prunifier::scheme::loader::SchemeLoader;
use prunifier::scheme::{Action, Scheme};

const FIXTURES_DIR: &str = "tests/fixtures/scheme_loader";

/// Helper: a non-existent directory used to prevent picking up real project schemes.
fn absent_dir() -> PathBuf {
    PathBuf::from(FIXTURES_DIR).join("__absent__")
}

#[test]
fn test_load_defaults() {
    let default_dir = PathBuf::from(FIXTURES_DIR).join("defaults");
    let loader = SchemeLoader::new(default_dir);

    // Explicitly set scheme_dir to an absent dir so we don't accidentally
    // load real schemes from .prunifier/schemes/.
    let mut config = PrunifierConfig::default();
    config.scheme_dir = Some(absent_dir());

    let schemes: HashMap<String, Scheme> = loader.load(&config).expect("should load defaults");
    assert_eq!(schemes.len(), 2, "should load 2 default schemes");
    assert!(
        schemes.contains_key("git status"),
        "should contain 'git status'"
    );
    assert!(
        schemes.contains_key("git diff"),
        "should contain 'git diff'"
    );

    // Verify the git status scheme has the default rules (keep action)
    let git_status = schemes.get("git status").unwrap();
    assert_eq!(git_status.rules.len(), 1);
    assert!(matches!(git_status.rules[0].action, Action::Keep));
}

#[test]
fn test_project_override_replaces_default() {
    let default_dir = PathBuf::from(FIXTURES_DIR).join("defaults");
    let project_dir = PathBuf::from(FIXTURES_DIR).join("overrides");
    let loader = SchemeLoader::new(default_dir);

    let mut config = PrunifierConfig::default();
    config.scheme_dir = Some(project_dir);

    let schemes: HashMap<String, Scheme> =
        loader.load(&config).expect("should load with overrides");
    assert_eq!(schemes.len(), 2, "should still have 2 total schemes");

    // git status should come from the override (discard action, not keep)
    let git_status = schemes.get("git status").unwrap();
    assert_eq!(git_status.rules.len(), 1);
    assert!(
        matches!(git_status.rules[0].action, Action::Discard),
        "override should replace git_status rules with Discard action"
    );

    // git diff should still come from defaults (not overridden)
    let git_diff = schemes.get("git diff").unwrap();
    assert_eq!(git_diff.rules.len(), 1);
    assert!(matches!(git_diff.rules[0].action, Action::Keep));
}

#[test]
fn test_no_project_config_uses_defaults() {
    let default_dir = PathBuf::from(FIXTURES_DIR).join("defaults");
    let loader = SchemeLoader::new(default_dir);

    // Set scheme_dir to absent dir to isolate from real project schemes
    let mut config = PrunifierConfig::default();
    config.scheme_dir = Some(absent_dir());

    let schemes: HashMap<String, Scheme> = loader.load(&config).expect("should load defaults only");
    assert_eq!(schemes.len(), 2, "should load 2 default schemes");
    assert!(schemes.contains_key("git status"));
    assert!(schemes.contains_key("git diff"));
}

#[test]
fn test_empty_scheme_dir() {
    // Both default and project directories are absent — should return empty map
    let default_dir = PathBuf::from(FIXTURES_DIR).join("nonexistent");
    let loader = SchemeLoader::new(default_dir);

    let mut config = PrunifierConfig::default();
    config.scheme_dir = Some(absent_dir());

    let schemes: HashMap<String, Scheme> =
        loader.load(&config).expect("empty dir should not error");
    assert!(schemes.is_empty(), "no scheme dirs should give empty map");
}
