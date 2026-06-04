use prunifier::proxy::TtyDetector;

#[test]
fn test_detect_tty_stdout() {
    // In CI/test, stdout is piped, but this test exists to verify
    // the function compiles and runs without panicking.
    let result = TtyDetector::is_tty();
    // In `cargo test` stdout is NOT a TTY (piped), so expect false.
    assert!(!result, "is_tty() should return false when stdout is piped");
}

#[test]
fn test_detect_non_tty_stdout() {
    // Even more explicit: when output is redirected (piped), isatty(1) returns 0.
    // We can't easily fake a TTY in a test, but we can verify the function
    // does not panic and returns false in the non-TTY test environment.
    assert!(!TtyDetector::is_tty());
}

#[test]
fn test_tty_passthrough_skips_pruning() {
    // Interactive commands should passthrough without processing.
    assert!(TtyDetector::should_passthrough("vim"));
    assert!(TtyDetector::should_passthrough("less file.txt"));
    assert!(TtyDetector::should_passthrough("python -m http.server"));

    // Non-interactive commands should NOT passthrough.
    assert!(!TtyDetector::should_passthrough("ls"));
    assert!(!TtyDetector::should_passthrough("grep pattern file"));
    assert!(!TtyDetector::should_passthrough("echo hello"));
}
