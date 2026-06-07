use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::{PrunifyError, PrunifyResult};
use crate::proxy::{CommandExecutor, DispatchMode, Dispatcher, OutputMarker, RecursionGuard, TtyDetector};

use super::tokenizer::{parse_command, CommandSegment, ShellOperator};

/// Execute a command string that may contain shell operators.
///
/// Each segment is matched independently through the prunify trie,
/// executed, and chained according to the connecting operator.
pub fn execute_pipeline(
    cmd_str: &str,
    dispatcher: &Dispatcher,
    cli: &crate::cli::Cli,
) -> PrunifyResult<i32> {
    let segments = parse_command(cmd_str)
        .map_err(|e| PrunifyError::CommandFailed(format!("shell parse error: {e}"), 1))?;

    if segments.is_empty() {
        return Ok(0);
    }

    let mut last_exit_code = 0;

    // We need to iterate with index to handle pipe-chaining stdout
    let mut pipe_stdout: Option<Vec<u8>> = None;

    for (i, segment) in segments.iter().enumerate() {
        let next_op = segment.operator.as_ref();

        // Determine whether this segment should run based on chaining operator
        let should_run = match &get_prev_op(&segments, i) {
            None | Some(ShellOperator::Seq) => true,
            Some(ShellOperator::And) => last_exit_code == 0,
            Some(ShellOperator::Or) => last_exit_code != 0,
            Some(ShellOperator::Pipe) => true,
            Some(ShellOperator::RedirectStdout)
            | Some(ShellOperator::RedirectAppend)
            | Some(ShellOperator::RedirectStderr)
            | Some(ShellOperator::RedirectStdin) => true,
        };

        if !should_run {
            pipe_stdout = None;
            continue;
        }

        let segment_cmd = segment.args.join(" ");

        // Recursion guard per segment
        if RecursionGuard::is_recursive(&segment_cmd) {
            let msg = "prunify: recursion detected — bypassing proxy\n";
            if next_op == Some(&ShellOperator::Pipe) {
                pipe_stdout = Some(msg.as_bytes().to_vec());
            } else {
                eprint!("{msg}");
            }
            last_exit_code = 0;
            continue;
        }

        // TTY passthrough
        if TtyDetector::should_passthrough(&segment.args[0]) {
            let status = if let Some(input) = pipe_stdout.take() {
                let mut child = Command::new(&segment.args[0])
                    .args(&segment.args[1..])
                    .stdin(Stdio::piped())
                    .spawn()?;
                use std::io::Write;
                child.stdin.take().unwrap().write_all(&input)?;
                child.wait()?
            } else {
                Command::new(&segment.args[0])
                    .args(&segment.args[1..])
                    .status()?
            };
            last_exit_code = status.code().unwrap_or(1);
            if next_op == Some(&ShellOperator::Pipe) {
                // No stdout to pipe for TTY passthrough
                pipe_stdout = Some(Vec::new());
            }
            continue;
        }

        // Execute the segment
        let result = if let Some(input) = pipe_stdout.take() {
            // Previous segment piped its stdout to us
            let mut child = Command::new(&segment.args[0])
                .args(&segment.args[1..])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            child.stdin.take().unwrap().write_all(&input)?;
            // Drop stdin so child can finish
            drop(child.stdin.take());
            CommandExecutor::wait_for_child(child)
        } else {
            CommandExecutor::execute(&segment.args)?
        };

        last_exit_code = result.exit_code;

        // Handle the output based on the connecting operator
        match next_op {
            Some(ShellOperator::Pipe) => {
                // Pass raw stdout to next command in the pipe chain
                pipe_stdout = Some(result.stdout);
                // Still dispatch stderr through the scheme
                if !result.stderr.is_empty() {
                    let pruned_stderr = dispatcher.dispatch(&segment_cmd, &result.stderr)?.0;
                    if !pruned_stderr.is_empty() {
                        eprint!("{}", pruned_stderr);
                    }
                }
            }
            Some(ShellOperator::RedirectStdout) => {
                let path = segment.redirect_target.as_deref().unwrap_or("");
                write_redirect(&result, path, false)?;
            }
            Some(ShellOperator::RedirectAppend) => {
                let path = segment.redirect_target.as_deref().unwrap_or("");
                write_redirect(&result, path, true)?;
            }
            Some(ShellOperator::RedirectStderr) => {
                let path = segment.redirect_target.as_deref().unwrap_or("");
                write_stderr_redirect(&result, path, false)?;
            }
            Some(ShellOperator::RedirectStdin) => {
                // stdin redirect was already handled by CommandExecutor
                // Just print the output normally
                print_pruned(&result, &segment_cmd, dispatcher, cli)?;
            }
            _ => {
                // Normal output — print pruned
                print_pruned(&result, &segment_cmd, dispatcher, cli)?;
            }
        }
    }

    Ok(last_exit_code)
}

/// Get the operator that connects segment `i-1` to segment `i`.
fn get_prev_op(segments: &[CommandSegment], i: usize) -> Option<ShellOperator> {
    if i == 0 {
        return None;
    }
    segments.get(i - 1)?.operator.clone()
}

/// Print a segment's output after dispatching through the scheme.
fn print_pruned(
    result: &crate::proxy::ExecutionResult,
    command_str: &str,
    dispatcher: &Dispatcher,
    cli: &crate::cli::Cli,
) -> PrunifyResult<()> {
    let stdout_str = String::from_utf8_lossy(&result.stdout);
    let (pruned, mode) = dispatcher.dispatch(command_str, &stdout_str)?;

    let pruned_stderr = dispatcher.dispatch(command_str, &result.stderr)?.0;

    let tokens = match &mode {
        DispatchMode::PrefixMatch(n) => *n,
        _ => 0,
    };
    let output = OutputMarker::mark_pruned(&pruned, &mode, tokens, cli.no_mark);

    let formatted = if cli.no_mark && mode == DispatchMode::Passthrough {
        String::from_utf8_lossy(&result.stdout).to_string()
    } else {
        output
    };

    print!("{}", formatted);
    if !pruned_stderr.is_empty() {
        eprint!("{}", pruned_stderr);
    }

    Ok(())
}

/// Write stdout to a file (create or append).
fn write_redirect(result: &crate::proxy::ExecutionResult, path: &str, append: bool) -> PrunifyResult<()> {
    let file_path = std::path::Path::new(path);

    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = if append {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    } else {
        std::fs::File::create(path)
    };

    match file {
        Ok(mut f) => {
            f.write_all(&result.stdout).map_err(|e| {
                PrunifyError::CommandFailed(format!("failed to write to {path}: {e}"), 1)
            })?;
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            Ok(())
        }
        Err(e) => Err(PrunifyError::CommandFailed(
            format!("failed to open {path} for writing: {e}"),
            1,
        )),
    }
}

/// Write stderr to a file (create or append).
fn write_stderr_redirect(result: &crate::proxy::ExecutionResult, path: &str, _append: bool) -> PrunifyResult<()> {
    let file_path = std::path::Path::new(path);

    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = std::fs::File::create(path);
    match file {
        Ok(mut f) => {
            f.write_all(result.stderr.as_bytes()).map_err(|e| {
                PrunifyError::CommandFailed(format!("failed to write to {path}: {e}"), 1)
            })?;
            if !result.stdout.is_empty() {
                let stdout_str = String::from_utf8_lossy(&result.stdout);
                print!("{stdout_str}");
            }
            Ok(())
        }
        Err(e) => Err(PrunifyError::CommandFailed(
            format!("failed to open {path} for writing: {e}"),
            1,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::collections::HashMap;

    /// Minimal harness: a dispatcher with no schemes (all passthrough).
    fn empty_dispatcher() -> Dispatcher {
        Dispatcher::new(
            crate::engine::CommandTrie::new(),
            crate::engine::CommandTrie::new(),
            HashMap::new(),
            HashMap::new(),
        )
    }

    #[test]
    fn test_pipeline_single_no_op() {
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "echo hello"]).unwrap();
        let exit = execute_pipeline("echo hello", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_pipeline_and_chain() {
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "true && echo ok"]).unwrap();
        let exit = execute_pipeline("true && echo ok", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_pipeline_and_short_circuit() {
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "false && echo should_not_run"]).unwrap();
        let exit = execute_pipeline("false && echo should_not_run", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 1);
    }

    #[test]
    fn test_pipeline_or_chain() {
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "false || echo ok"]).unwrap();
        let exit = execute_pipeline("false || echo ok", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_pipeline_or_short_circuit() {
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "true || echo should_not_run"]).unwrap();
        let exit = execute_pipeline("true || echo should_not_run", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 0);
    }
}
