use std::io::Write;
use std::process::{Command, Stdio};

use crate::error::{PrunifyError, PrunifyResult};
use crate::proxy::{CommandExecutor, DispatchMode, Dispatcher, ExecutionResult, OutputMarker, RecursionGuard, TtyDetector};

use super::tokenizer::{parse_command, CommandSegment, ShellOperator};

/// Execute a command string that may contain shell operators.
///
/// Two execution paths:
/// - **Shell path**: when only redirect operators (`>`, `2>`, `>>`, `<`) are
///   present, the full command string is executed through `/bin/sh -c` so that
///   the OS shell handles redirect semantics natively. The first segment's
///   args are used as the base command for scheme matching.
/// - **Chained path**: when chaining operators (`&&`, `||`, `|`, `;`) are
///   present, each segment is executed independently and output is chained
///   per the connecting operator.
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

    let has_chaining = segments.iter().any(|s| {
        matches!(
            s.operator,
            Some(ShellOperator::And | ShellOperator::Or | ShellOperator::Pipe | ShellOperator::Seq)
        )
    });

    if has_chaining {
        execute_chained(segments, dispatcher, cli)
    } else {
        execute_redirects(cmd_str, &segments, dispatcher, cli)
    }
}

/// Execute a command whose only operators are redirects through the system
/// shell. The shell handles redirect semantics natively (e.g., `2>&1`,
/// `> file`, etc.). The base command (from the first segment) selects the
/// scheme for pruning.
fn execute_redirects(
    full_cmd: &str,
    segments: &[CommandSegment],
    dispatcher: &Dispatcher,
    cli: &crate::cli::Cli,
) -> PrunifyResult<i32> {
    let base_cmd = segments
        .first()
        .map(|s| s.args.join(" "))
        .unwrap_or_default();

    // Recursion guard — check the base command against prunify itself
    if RecursionGuard::is_recursive(&base_cmd) {
        let msg = "prunify: recursion detected — bypassing proxy\n";
        let output = Command::new("/bin/sh")
            .args(["-c", full_cmd])
            .output();
        if let Ok(out) = output {
            print!("{}", String::from_utf8_lossy(&out.stdout));
            eprint!("{}", String::from_utf8_lossy(&out.stderr));
            return Ok(out.status.code().unwrap_or(1));
        }
        eprint!("{msg}");
        return Ok(0);
    }

    let output = Command::new("/bin/sh")
        .args(["-c", full_cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| PrunifyError::CommandFailed(format!("shell execution failed: {e}"), 1))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let result = ExecutionResult {
        stdout: output.stdout,
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code,
    };

    print_pruned(&result, &base_cmd, dispatcher, cli)?;
    Ok(exit_code)
}

/// Execute a chained command (with `&&`, `||`, `|`, `;`) by running each
/// segment independently and linking them via the connecting operator.
/// Segments with redirect operators are executed through the shell so that
/// redirect semantics are handled natively.
fn execute_chained(
    segments: Vec<CommandSegment>,
    dispatcher: &Dispatcher,
    cli: &crate::cli::Cli,
) -> PrunifyResult<i32> {
    let mut last_exit_code = 0;
    let mut pipe_stdout: Option<Vec<u8>> = None;

    for (i, segment) in segments.iter().enumerate() {
        let next_op = segment.operator.as_ref();

        let should_run = match &get_prev_op(&segments, i) {
            None | Some(ShellOperator::Seq) => true,
            Some(ShellOperator::And) => last_exit_code == 0,
            Some(ShellOperator::Or) => last_exit_code != 0,
            Some(ShellOperator::Pipe) => true,
            _ => true,
        };

        if !should_run {
            pipe_stdout = None;
            continue;
        }

        let segment_cmd = segment.args.join(" ");

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

        if TtyDetector::should_passthrough(&segment.args[0]) {
            let status = if let Some(input) = pipe_stdout.take() {
                let mut child = Command::new(&segment.args[0])
                    .args(&segment.args[1..])
                    .stdin(Stdio::piped())
                    .spawn()?;
                child.stdin.take().unwrap().write_all(&input)?;
                child.wait()?
            } else {
                Command::new(&segment.args[0])
                    .args(&segment.args[1..])
                    .status()?
            };
            last_exit_code = status.code().unwrap_or(1);
            if next_op == Some(&ShellOperator::Pipe) {
                pipe_stdout = Some(Vec::new());
            }
            continue;
        }

        // Segments with redirect operators: reconstruct the full shell command
        // and execute through /bin/sh for native redirect semantics.
        if is_redirect_op(next_op) {
            let shell_cmd = build_segment_command(segment);
            let output = Command::new("/bin/sh")
                .args(["-c", &shell_cmd])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| {
                    PrunifyError::CommandFailed(format!("shell execution failed: {e}"), 1)
                })?;

            last_exit_code = output.status.code().unwrap_or(-1);
            let result = ExecutionResult {
                stdout: output.stdout,
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: last_exit_code,
            };
            print_pruned(&result, &segment_cmd, dispatcher, cli)?;
            continue;
        }

        let result = if let Some(input) = pipe_stdout.take() {
            let mut child = Command::new(&segment.args[0])
                .args(&segment.args[1..])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            child.stdin.take().unwrap().write_all(&input)?;
            drop(child.stdin.take());
            CommandExecutor::wait_for_child(child)
        } else {
            CommandExecutor::execute(&segment.args)?
        };

        last_exit_code = result.exit_code;

        match next_op {
            Some(ShellOperator::Pipe) => {
                pipe_stdout = Some(result.stdout);
                if !result.stderr.is_empty() {
                    let pruned_stderr = dispatcher.dispatch(&segment_cmd, &result.stderr)?.0;
                    if !pruned_stderr.is_empty() {
                        eprint!("{}", pruned_stderr);
                    }
                }
            }
            _ => {
                print_pruned(&result, &segment_cmd, dispatcher, cli)?;
            }
        }
    }

    Ok(last_exit_code)
}

/// True if the operator is a redirect (not a chain operator).
fn is_redirect_op(op: Option<&ShellOperator>) -> bool {
    matches!(
        op,
        Some(ShellOperator::RedirectStdout)
            | Some(ShellOperator::RedirectAppend)
            | Some(ShellOperator::RedirectStderr)
            | Some(ShellOperator::RedirectStdin)
    )
}

/// Reconstruct the full shell command string for a segment that has a
/// redirect operator, e.g., `echo ok 2>&1` or `echo hello > file.txt`.
fn build_segment_command(segment: &CommandSegment) -> String {
    let mut cmd = segment.args.join(" ");
    if let Some(ref op) = segment.operator {
        match op {
            ShellOperator::RedirectStdout => {
                if let Some(ref t) = segment.redirect_target {
                    cmd.push_str(&format!(" >{}", t));
                }
            }
            ShellOperator::RedirectAppend => {
                if let Some(ref t) = segment.redirect_target {
                    cmd.push_str(&format!(" >>{}", t));
                }
            }
            ShellOperator::RedirectStderr => {
                if let Some(ref t) = segment.redirect_target {
                    cmd.push_str(&format!(" 2>{}", t));
                }
            }
            ShellOperator::RedirectStdin => {
                if let Some(ref t) = segment.redirect_target {
                    cmd.push_str(&format!(" <{}", t));
                }
            }
            _ => {}
        }
    }
    cmd
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

    #[test]
    fn test_pipeline_stderr_redirect_to_fd() {
        // 2>&1 should parse and execute without error — stderr merges into stdout
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "echo hello 2>&1"]).unwrap();
        // This should not fail with "missing redirect target"
        let exit = execute_pipeline("echo hello 2>&1", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_pipeline_stdout_redirect_to_fd() {
        // 1>&2 should parse and execute without error — stdout merges into stderr
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "echo hello 1>&2"]).unwrap();
        let exit = execute_pipeline("echo hello 1>&2", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_pipeline_stderr_redirect_to_fd_compound() {
        // Compound: command with && and 2>&1
        let dispatcher = empty_dispatcher();
        let cli = crate::cli::Cli::try_parse_from(["prunify", "-c", "true && echo ok 2>&1"]).unwrap();
        let exit = execute_pipeline("true && echo ok 2>&1", &dispatcher, &cli).unwrap();
        assert_eq!(exit, 0);
    }
}
