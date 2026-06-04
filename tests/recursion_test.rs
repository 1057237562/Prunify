use prunify::proxy::recursion_guard::RecursionGuard;

/// Verify that "prunify" as the first token is detected as recursive.
/// This covers direct invocation and invocations with arguments.
#[test]
fn test_detect_self_invocation() {
    assert!(
        RecursionGuard::is_recursive("prunify"),
        "bare 'prunify' should be recursive"
    );
    assert!(
        RecursionGuard::is_recursive("prunify --scheme foo.json"),
        "'prunify' with args should be recursive"
    );
    assert!(
        RecursionGuard::is_recursive("prunify arg1 arg2"),
        "'prunify' with positional args should be recursive"
    );
}

/// Verify that "prunify" (debug binary name) as first token is also detected.
#[test]
fn test_detect_nested_prunify() {
    assert!(
        RecursionGuard::is_recursive("prunify"),
        "bare 'prunify' should be recursive"
    );
    assert!(
        RecursionGuard::is_recursive("prunify --scheme scheme.json"),
        "'prunify' with args should be recursive"
    );
}

/// Verify that normal commands like ls, echo, git are NOT detected as recursive.
#[test]
fn test_normal_command_not_detected() {
    assert!(
        !RecursionGuard::is_recursive("ls -la"),
        "'ls -la' should not be recursive"
    );
    assert!(
        !RecursionGuard::is_recursive("echo hello"),
        "'echo hello' should not be recursive"
    );
    assert!(
        !RecursionGuard::is_recursive("git status"),
        "'git status' should not be recursive"
    );
    assert!(
        !RecursionGuard::is_recursive("cat /etc/passwd"),
        "'cat /etc/passwd' should not be recursive"
    );
    assert!(
        !RecursionGuard::is_recursive(""),
        "empty string should not be recursive"
    );
    assert!(
        !RecursionGuard::is_recursive("   "),
        "whitespace-only string should not be recursive"
    );
}

/// Verify commands that merely CONTAIN "prunify" in arguments are NOT flagged,
/// but commands whose first token is a path to prunify ARE flagged.
#[test]
fn test_different_path_prunify() {
    // Should NOT be flagged: "prunify" appears in args, not as the first token
    assert!(
        !RecursionGuard::is_recursive("echo 'use prunify'"),
        "'prunify' in arguments should not be recursive"
    );
    assert!(
        !RecursionGuard::is_recursive("ls /path/to/prunify"),
        "'prunify' in non-first token should not be recursive"
    );

    // Should be flagged: first token is a path whose file stem is "prunify"
    assert!(
        RecursionGuard::is_recursive("./target/debug/prunify"),
        "path to prunify binary should be recursive"
    );
    assert!(
        RecursionGuard::is_recursive("/usr/local/bin/prunify --help"),
        "full path to prunify binary should be recursive"
    );
    assert!(
        RecursionGuard::is_recursive("./target/debug/prunify"),
        "path to prunify binary should be recursive"
    );
}
