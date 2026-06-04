use std::collections::HashMap;
use std::path::PathBuf;

use prunify::config::PrunifyConfig;
use prunify::scheme::loader::SchemeLoader;
use prunify::scheme::{Action, Scheme};

const FIXTURES_DIR: &str = "tests/fixtures/scheme_loader";

/// Helper: a non-existent directory used to prevent picking up real project schemes.
fn absent_dir() -> PathBuf {
    PathBuf::from(FIXTURES_DIR).join("__absent__")
}

#[test]
fn test_load_defaults() {
    let fallback_dir = PathBuf::from(FIXTURES_DIR).join("defaults");
    let loader = SchemeLoader::new(fallback_dir);

    // Explicitly set scheme_dir to an absent dir so we don't accidentally
    // load real schemes from .prunify/schemes/.
    let mut config = PrunifyConfig::default();
    config.scheme_dir = Some(absent_dir());

    let (project, fallback): (HashMap<String, Scheme>, HashMap<String, Scheme>) =
        loader.load(&config).expect("should load defaults");

    // project_dir = absent_dir() → no project schemes
    assert!(project.is_empty(), "absent project dir should be empty");

    // fallback_dir = defaults/ → 2 schemes
    assert_eq!(fallback.len(), 2, "should load 2 fallback schemes");
    assert!(
        fallback.contains_key("git status"),
        "should contain 'git status'"
    );
    assert!(
        fallback.contains_key("git diff"),
        "should contain 'git diff'"
    );

    // Verify the git status scheme has the default rules (keep action)
    let git_status = fallback.get("git status").unwrap();
    assert_eq!(git_status.rules.len(), 1);
    assert!(matches!(git_status.rules[0].action, Action::Keep));
}

#[test]
fn test_project_override_replaces_default() {
    let fallback_dir = PathBuf::from(FIXTURES_DIR).join("defaults");
    let project_dir = PathBuf::from(FIXTURES_DIR).join("overrides");
    let loader = SchemeLoader::new(fallback_dir);

    let mut config = PrunifyConfig::default();
    config.scheme_dir = Some(project_dir);

    let (project, fallback): (HashMap<String, Scheme>, HashMap<String, Scheme>) =
        loader.load(&config).expect("should load with overrides");

    // project (overrides/) has 1 scheme: git status with Discard
    assert_eq!(project.len(), 1, "project should have 1 scheme");
    let git_status = project.get("git status").unwrap();
    assert_eq!(git_status.rules.len(), 1);
    assert!(
        matches!(git_status.rules[0].action, Action::Discard),
        "override should replace git_status rules with Discard action"
    );

    // fallback (defaults/) has 2 schemes: git status (Keep) + git diff (Keep)
    assert_eq!(fallback.len(), 2, "fallback should have 2 schemes");
    assert!(fallback.contains_key("git status"));
    assert!(fallback.contains_key("git diff"));

    // The dispatcher will prefer project (Discard) over fallback (Keep)
    // for "git status", and fall back to fallback (Keep) for "git diff".
}

#[test]
fn test_no_project_config_uses_defaults() {
    let fallback_dir = PathBuf::from(FIXTURES_DIR).join("defaults");
    let loader = SchemeLoader::new(fallback_dir);

    // Set scheme_dir to absent dir to isolate from real project schemes
    let mut config = PrunifyConfig::default();
    config.scheme_dir = Some(absent_dir());

    let (project, fallback): (HashMap<String, Scheme>, HashMap<String, Scheme>) =
        loader.load(&config).expect("should load defaults only");

    assert!(project.is_empty(), "absent project dir should be empty");
    assert_eq!(fallback.len(), 2, "should load 2 fallback schemes");
    assert!(fallback.contains_key("git status"));
    assert!(fallback.contains_key("git diff"));
}

#[test]
fn test_empty_scheme_dir() {
    // Both fallback and project directories are absent — should return empty maps
    let fallback_dir = PathBuf::from(FIXTURES_DIR).join("nonexistent");
    let loader = SchemeLoader::new(fallback_dir);

    let mut config = PrunifyConfig::default();
    config.scheme_dir = Some(absent_dir());

    let (project, fallback): (HashMap<String, Scheme>, HashMap<String, Scheme>) =
        loader.load(&config).expect("empty dir should not error");
    assert!(project.is_empty(), "absent project dir should be empty");
    assert!(fallback.is_empty(), "absent fallback dir should be empty");
}
