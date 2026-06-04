use std::process::{Command, Stdio};

use crate::error::PrunifierResult;
use crate::proxy::signal_handler;

/// Captured output from a proxied command.
pub struct ExecutionResult {
    /// Raw stdout bytes from the child process (not decoded to String).
    /// Use `String::from_utf8_lossy` when string operations are needed.
    pub stdout: Vec<u8>,
    pub stderr: String,
    pub exit_code: i32,
}

/// Wraps [`std::process::Command`] to capture stdout, stderr, and exit code.
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a command from its individual args, capturing stdout + stderr + exit code.
    ///
    /// The first element is the binary, the rest are passed as arguments directly —
    /// no splitting on whitespace, so quoted arguments are preserved.
    /// Command-not-found errors are returned as `Ok` with `exit_code: 127`.
    pub fn execute(args: &[String]) -> PrunifierResult<ExecutionResult> {
        let binary = args.first().ok_or_else(|| {
            crate::error::PrunifierError::CommandFailed("Empty command args".to_string(), -1)
        })?;

        let child = match Command::new(binary)
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ExecutionResult {
                    stdout: Vec::new(),
                    stderr: format!("command not found: {binary}"),
                    exit_code: 127,
                });
            }
            Err(e) => return Err(e.into()),
        };

        let child_pid = child.id();
        signal_handler::set_child_pid(child_pid);
        let output = child.wait_with_output()?;
        signal_handler::clear_child_pid();

        let exit_code = output.status.code().unwrap_or(-1);
        // Keep raw stdout bytes to preserve binary passthrough.
        // String operations (dispatch, scheme matching) use from_utf8_lossy at the call site.
        let stdout = output.stdout;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(ExecutionResult {
            stdout,
            stderr,
            exit_code,
        })
    }
}
