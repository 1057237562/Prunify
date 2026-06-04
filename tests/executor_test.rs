use prunifier::proxy::CommandExecutor;

#[test]
fn test_execute_simple_command() {
    let result = CommandExecutor::execute(&["echo".to_string(), "hello".to_string()]).unwrap();
    assert_eq!(result.stdout, b"hello\n");
    assert_eq!(result.stderr, "");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_capture_stdout() {
    let result = CommandExecutor::execute(&[
        "echo".to_string(),
        "hello".to_string(),
        "stdout".to_string(),
        "capture".to_string(),
    ])
    .unwrap();
    assert_eq!(result.stdout, b"hello stdout capture\n");
    assert_eq!(result.stderr, "");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_capture_stderr() {
    // ls writes error messages to stderr when the path doesn't exist
    let result = CommandExecutor::execute(&[
        "ls".to_string(),
        "/nonexistent_test_path_xyz_123".to_string(),
    ])
    .unwrap();
    assert!(
        !result.stderr.is_empty(),
        "stderr should contain error message"
    );
    assert_eq!(result.stdout, b"");
    assert_eq!(result.exit_code, 2);
}

#[test]
fn test_exit_code_propagation() {
    // false exits with code 1
    let result = CommandExecutor::execute(&["false".to_string()]).unwrap();
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_quoted_args_exit_code() {
    // sh -c "exit 42" must return 42 (regression: quoting was destroyed by join/split round-trip)
    let result =
        CommandExecutor::execute(&["sh".to_string(), "-c".to_string(), "exit 42".to_string()])
            .unwrap();
    assert_eq!(result.exit_code, 42);
}

#[test]
fn test_command_not_found() {
    let result = CommandExecutor::execute(&["nonexistent_cmd_xyz_123_test".to_string()]).unwrap();
    assert_eq!(result.exit_code, 127);
    assert_eq!(result.stdout, b"");
    assert!(!result.stderr.is_empty());
}

#[test]
fn test_output_with_no_newline() {
    // printf without \n in format string produces no trailing newline
    let result = CommandExecutor::execute(&["printf".to_string(), "hello".to_string()]).unwrap();
    assert_eq!(result.stdout, b"hello");
    assert_eq!(result.stderr, "");
    assert_eq!(result.exit_code, 0);
}
