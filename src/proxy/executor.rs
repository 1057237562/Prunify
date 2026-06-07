use std::process::{Command, Stdio};

use crate::error::PrunifyResult;
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
    ///
    /// On Windows, if the binary is not found directly, the command is retried
    /// through PowerShell (which resolves shell aliases like `ls`, `cat`, etc.).
    /// On Unix, the original "not found" error is returned immediately.
    pub fn execute(args: &[String]) -> PrunifyResult<ExecutionResult> {
        let binary = args.first().ok_or_else(|| {
            crate::error::PrunifyError::CommandFailed("Empty command args".to_string(), -1)
        })?;

        match Command::new(binary)
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => Ok(Self::wait_for_child(child)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Self::fallback_shell_exec(args, binary)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Wait for a child process to finish and collect its output.
    pub fn wait_for_child(child: std::process::Child) -> ExecutionResult {
        let child_pid = child.id();
        signal_handler::set_child_pid(child_pid);
        let output = child
            .wait_with_output()
            .expect("child process should be waitable");
        signal_handler::clear_child_pid();

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = output.stdout;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        ExecutionResult {
            stdout,
            stderr,
            exit_code,
        }
    }

    /// Fallback: run the command through the system shell.
    ///
    /// On Windows, this uses PowerShell (which resolves aliases like `ls`, `cat`,
    /// `grep`, etc.) falling back to `cmd.exe` if PowerShell is unavailable.
    /// On Unix, the original "not found" error is returned unchanged.
    #[allow(unused_variables)]
    fn fallback_shell_exec(args: &[String], binary: &str) -> PrunifyResult<ExecutionResult> {
        #[cfg(windows)]
        {
            let command_line = args.join(" ");
            // Try PowerShell first — it resolves aliases (ls, cat, grep, ...)
            match Command::new("powershell")
                .args(["-Command", &command_line])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => return Ok(Self::wait_for_child(child)),
                Err(_) => {
                    // PowerShell not available, try cmd.exe
                    match Command::new("cmd")
                        .args(["/C", &command_line])
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                    {
                        Ok(child) => return Ok(Self::wait_for_child(child)),
                        Err(_) => {}
                    }
                }
            }
        }

        // Reachable on Unix or when all Windows fallbacks failed
        Ok(ExecutionResult {
            stdout: Vec::new(),
            stderr: format!("command not found: {binary}"),
            exit_code: 127,
        })
    }
}
