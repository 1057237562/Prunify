use std::path::Path;

use prunifier::scheme::SchemeStorage;

const FIXTURES_DIR: &str = "tests/fixtures";

#[test]
fn test_load_valid_scheme() {
    let path = Path::new(FIXTURES_DIR).join("valid-scheme.json");
    let scheme = SchemeStorage::load(&path).expect("should load valid scheme");
    assert_eq!(scheme.command, "git status");
    assert_eq!(scheme.version, 1);
    assert!(!scheme.rules.is_empty());
}

#[test]
fn test_load_missing_file() {
    let path = Path::new(FIXTURES_DIR).join("does-not-exist.json");
    let result = SchemeStorage::load(&path);
    assert!(result.is_err(), "loading a missing file should fail");
}

#[test]
fn test_load_invalid_json() {
    let path = Path::new(FIXTURES_DIR).join("invalid-schema.json");
    let result = SchemeStorage::load(&path);
    assert!(result.is_err(), "loading invalid JSON should fail");
}

#[test]
fn test_load_all_skips_non_json() {
    let dir = Path::new(FIXTURES_DIR);
    let schemes = SchemeStorage::load_all(dir).expect("load_all should not fail");
    // fixtures/ contains valid-scheme.json and invalid-schema.json (non-parseable).
    // Only the valid one should load successfully.
    assert_eq!(schemes.len(), 1, "only valid-scheme.json should be loaded");
    assert_eq!(schemes[0].command, "git status");
}
