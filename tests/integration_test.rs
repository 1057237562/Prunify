mod common;

use common::{run_prunify, scheme_fixture, temp_dir};

/// TDD RED phase: this test expects prunify to reject invalid JSON.
///
/// The binary currently prints "Hello, world!" and exits 0, so this test
/// will fail until the scheme validator is implemented.
#[test]
fn test_scheme_validator_rejects_invalid_json() {
    let dir = temp_dir("reject_invalid_json");
    let file = scheme_fixture(&dir, "bad_scheme", r#"{"name": "test" "version": 1}"#);

    let (_stdout, _stderr, exit_code) =
        run_prunify(&["--scheme", file.to_str().expect("valid utf-8 path")]);

    assert_ne!(
        exit_code, 0,
        "Expected non-zero exit code for invalid scheme JSON, but got {}",
        exit_code
    );
}
