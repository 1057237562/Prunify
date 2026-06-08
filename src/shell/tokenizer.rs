/// Shell command tokenizer — splits a command string at shell operators
/// while respecting single/double quotes and backslash escapes.
///
/// Supported operators (outside quotes):
///   `&&`  — logical AND (next runs if previous succeeds)
///   `||`  — logical OR  (next runs if previous fails)
///   `|`   — pipe (stdout → stdin of next)
///   `;`   — sequential (next runs regardless)
///   `>`   — redirect stdout to file
///   `>>`  — append stdout to file
///   `<`   — redirect stdin from file
///   `2>`  — redirect stderr to file

/// The operator that connects two command segments.
#[derive(Debug, Clone, PartialEq)]
pub enum ShellOperator {
    And,               // &&
    Or,                // ||
    Pipe,              // |
    Seq,               // ;
    RedirectStdout,    // >
    RedirectAppend,    // >>
    RedirectStderr,    // 2>
    RedirectStdin,     // <
}

/// A single command segment from a shell-pipeline command.
#[derive(Debug, Clone)]
pub struct CommandSegment {
    /// The individual arguments (first = binary, rest = args).
    pub args: Vec<String>,
    /// The operator that connects this segment to the NEXT segment,
    /// or `None` if this is the last segment.
    pub operator: Option<ShellOperator>,
    /// For redirection operators: the target file path.
    pub redirect_target: Option<String>,
}

/// Check whether a command string contains shell operators outside quotes.
pub fn has_operators(cmd: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut prev_chars: [char; 2] = ['\0', '\0']; // sliding window

    for ch in cmd.chars() {
        if escape {
            escape = false;
            prev_chars = [prev_chars[1], ch];
            continue;
        }
        if ch == '\\' && !in_single {
            escape = true;
            prev_chars = [prev_chars[1], ch];
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            prev_chars = [prev_chars[1], ch];
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            prev_chars = [prev_chars[1], ch];
            continue;
        }
        if in_single || in_double {
            prev_chars = [prev_chars[1], ch];
            continue;
        }

        // Check operators
        let window = [prev_chars[1], ch];
        match window {
            ['&', '&'] | ['|', '|'] | ['>', '>'] => return true,
            _ => {}
        }
        match ch {
            '|' | ';' | '>' | '<' => return true,
            _ => {}
        }

        // Detect `2>` as stderr redirect
        if ch == '>' && prev_chars[1] == '2' {
            return true;
        }

        prev_chars = [prev_chars[1], ch];
    }

    false
}

/// Parse a command string into segments split at shell operators.
///
/// Returns `Err` on parse errors (e.g., unterminated quotes).
pub fn parse_command(cmd: &str) -> Result<Vec<CommandSegment>, String> {
    let mut segments: Vec<CommandSegment> = Vec::new();
    let mut current_args: Vec<String> = Vec::new();
    let mut current_word = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut chars = cmd.chars().peekable();

    macro_rules! flush_word {
        () => {
            if !current_word.is_empty() {
                current_args.push(std::mem::take(&mut current_word));
            }
        };
    }

    macro_rules! push_segment {
        ($op:expr) => {{
            flush_word!();
            segments.push(CommandSegment {
                args: std::mem::take(&mut current_args),
                operator: $op,
                redirect_target: None,
            });
        }};
    }

    macro_rules! push_redirect {
        ($op:expr) => {{
            flush_word!();
            // The next word is the target file
            let target = parse_redirect_target(&mut chars, in_single, in_double)?;
            segments.push(CommandSegment {
                args: std::mem::take(&mut current_args),
                operator: Some($op),
                redirect_target: Some(target),
            });
        }};
    }

    while let Some(ch) = chars.next() {
        if escape {
            current_word.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' && !in_single {
            escape = true;
            continue;
        }

        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }

        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }

        if in_single || in_double {
            current_word.push(ch);
            continue;
        }

        // Check multi-character operators first
        let next_ch = chars.peek().copied().unwrap_or('\0');

        match (ch, next_ch) {
            ('&', '&') => {
                chars.next(); // consume second '&'
                push_segment!(Some(ShellOperator::And));
                continue;
            }
            ('|', '|') => {
                chars.next();
                push_segment!(Some(ShellOperator::Or));
                continue;
            }
            ('>', '>') => {
                chars.next();
                push_redirect!(ShellOperator::RedirectAppend);
                continue;
            }
            ('2', '>') => {
                chars.next(); // consume '>'
                push_redirect!(ShellOperator::RedirectStderr);
                continue;
            }
            _ => {}
        }

        match ch {
            '|' => {
                push_segment!(Some(ShellOperator::Pipe));
            }
            ';' => {
                push_segment!(Some(ShellOperator::Seq));
            }
            '>' => {
                push_redirect!(ShellOperator::RedirectStdout);
                continue;
            }
            '<' => {
                push_redirect!(ShellOperator::RedirectStdin);
                continue;
            }
            ' ' | '\t' => {
                flush_word!();
            }
            _ => {
                current_word.push(ch);
            }
        }
    }

    // Unterminated quote check
    if in_single {
        return Err("unterminated single quote".to_string());
    }
    if in_double {
        return Err("unterminated double quote".to_string());
    }
    if escape {
        return Err("trailing backslash".to_string());
    }

    // Flush final segment
    flush_word!();
    segments.push(CommandSegment {
        args: std::mem::take(&mut current_args),
        operator: None,
        redirect_target: None,
    });

    // Filter out empty segments (e.g. leading/trailing operators)
    segments.retain(|s| !s.args.is_empty());

    Ok(segments)
}

/// Parse a redirect target after a `>`, `<`, `>>`, or `2>` operator.
/// Consumes whitespace and then reads the next word (quoted or unquoted).
/// Uses peek-first to avoid consuming shell operator characters that
/// terminate the target.
fn parse_redirect_target(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    _in_single: bool,
    _in_double: bool,
) -> Result<String, String> {
    // Skip whitespace
    while let Some(&ch) = chars.peek() {
        if ch == ' ' || ch == '\t' {
            chars.next();
        } else {
            break;
        }
    }

    let mut target = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;

    while let Some(&ch) = chars.peek() {
        if escape {
            chars.next();
            target.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' && !in_single {
            chars.next();
            escape = true;
            continue;
        }
        if ch == '\'' && !in_double {
            chars.next();
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            chars.next();
            in_double = !in_double;
            continue;
        }
        if !in_single && !in_double && (ch == ' ' || ch == '\t') {
            break;
        }
        if !in_single && !in_double && is_shell_operator_char(ch) {
            // Allow `&` at the start of a redirect target for file descriptor redirects (e.g., 2>&1)
            if ch == '&' && target.is_empty() {
                chars.next();
                target.push(ch);
                continue;
            }
            break;
        }
        chars.next();
        target.push(ch);
    }

    if target.is_empty() {
        return Err("missing redirect target".to_string());
    }

    Ok(target)
}

fn is_shell_operator_char(ch: char) -> bool {
    matches!(ch, '|' | ';' | '>' | '<' | '&')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_operators_simple() {
        assert!(!has_operators("cargo build"));
        assert!(has_operators("cargo build && cargo test"));
        assert!(has_operators("ls -la || echo fail"));
        assert!(has_operators("cat foo | grep bar"));
        assert!(has_operators("make; make install"));
        assert!(has_operators("echo foo > file.txt"));
        assert!(has_operators("echo bar >> file.txt"));
        assert!(has_operators("cat < input.txt"));
        assert!(!has_operators("echo 'hello && world'"));
    }

    #[test]
    fn test_has_operators_in_quotes() {
        assert!(!has_operators("echo 'hello && world'"));
        assert!(!has_operators("echo \"hello && world\""));
        assert!(has_operators("echo 'hello' | wc"));
        assert!(has_operators("echo 'hello && world' | true"));
    }

    #[test]
    fn test_has_operators_stderr_redirect() {
        assert!(has_operators("cargo build 2> errors.log"));
    }

    #[test]
    fn test_parse_and_chain() {
        let segments = parse_command("cargo build && cargo test").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].args, ["cargo", "build"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::And));
        assert_eq!(segments[1].args, ["cargo", "test"]);
        assert_eq!(segments[1].operator, None);
    }

    #[test]
    fn test_parse_seq() {
        let segments = parse_command("clear; ls").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].args, ["clear"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::Seq));
        assert_eq!(segments[1].args, ["ls"]);
    }

    #[test]
    fn test_parse_pipe() {
        let segments = parse_command("ls -la | grep foo").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].args, ["ls", "-la"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::Pipe));
        assert_eq!(segments[1].args, ["grep", "foo"]);
    }

    #[test]
    fn test_parse_or() {
        let segments = parse_command("false || echo fallback").unwrap();
        assert_eq!(segments[0].operator, Some(ShellOperator::Or));
        assert_eq!(segments[1].args, ["echo", "fallback"]);
    }

    #[test]
    fn test_parse_redirect_stdout() {
        let segments = parse_command("echo hello > file.txt").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].args, ["echo", "hello"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectStdout));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("file.txt"));
    }

    #[test]
    fn test_parse_redirect_append() {
        let segments = parse_command("echo hello >> /tmp/log").unwrap();
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectAppend));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("/tmp/log"));
    }

    #[test]
    fn test_parse_redirect_stderr() {
        let segments = parse_command("cargo build 2> err.log").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectStderr));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("err.log"));
    }

    #[test]
    fn test_parse_with_quotes() {
        let segments = parse_command("echo 'hello world' && echo \"foo bar\"").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].args, ["echo", "hello world"]);
        assert_eq!(segments[1].args, ["echo", "foo bar"]);
    }

    #[test]
    fn test_parse_operators_in_quotes_not_split() {
        let segments = parse_command("echo 'hello && world'").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].args, ["echo", "hello && world"]);
    }

    #[test]
    fn test_parse_unterminated_single_quote() {
        assert!(parse_command("echo 'hello").is_err());
    }

    #[test]
    fn test_parse_unterminated_double_quote() {
        assert!(parse_command("echo \"hello").is_err());
    }

    #[test]
    fn test_parse_no_operators() {
        let segments = parse_command("cargo test -- --quiet").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].args, ["cargo", "test", "--", "--quiet"]);
    }

    #[test]
    fn test_parse_redirect_with_quoted_target() {
        let segments = parse_command("cat > \"output file.txt\"").unwrap();
        assert_eq!(segments[0].args, ["cat"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectStdout));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("output file.txt"));
    }

    #[test]
    fn test_parse_missing_redirect_target() {
        assert!(parse_command("echo hello >").is_err());
    }

    #[test]
    fn test_parse_chained_redirect_and_pipe() {
        let segments = parse_command("make && make install > install.log").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].args, ["make"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::And));
        assert_eq!(segments[1].args, ["make", "install"]);
        assert_eq!(segments[1].operator, Some(ShellOperator::RedirectStdout));
        assert_eq!(segments[1].redirect_target.as_deref(), Some("install.log"));
    }

    #[test]
    fn test_parse_stderr_redirect_to_fd() {
        // 2>&1: stderr redirect to file descriptor 1 (stdout)
        let segments = parse_command("cargo build 2>&1").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].args, ["cargo", "build"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectStderr));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("&1"));
    }

    #[test]
    fn test_parse_stdout_redirect_to_fd() {
        // >&2: stdout redirect to file descriptor 2 (stderr)
        let segments = parse_command("echo msg >&2").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].args, ["echo", "msg"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectStdout));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("&2"));
    }

    #[test]
    fn test_parse_stderr_redirect_to_fd_no_space() {
        // Also test variadic: 2>&1 without spaces after the operator
        let segments = parse_command("make 2>&1").unwrap();
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectStderr));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("&1"));
    }

    #[test]
    fn test_parse_redirect_with_ampersand_file() {
        // & terminates the redirect target (shell operator char);
        // only & at the start of the target is a file-descriptor redirect
        let segments = parse_command("echo data > file&1").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].args, ["echo", "data"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::RedirectStdout));
        assert_eq!(segments[0].redirect_target.as_deref(), Some("file"));
        assert_eq!(segments[1].args, ["&1"]);
        assert_eq!(segments[1].operator, None);
    }

    #[test]
    fn test_has_operators_stderr_redirect_to_fd() {
        assert!(has_operators("cargo test 2>&1"));
    }

    #[test]
    fn test_parse_triple_chain() {
        let segments = parse_command("cargo build && cargo test || echo failed").unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].args, ["cargo", "build"]);
        assert_eq!(segments[0].operator, Some(ShellOperator::And));
        assert_eq!(segments[1].args, ["cargo", "test"]);
        assert_eq!(segments[1].operator, Some(ShellOperator::Or));
        assert_eq!(segments[2].args, ["echo", "failed"]);
        assert_eq!(segments[2].operator, None);
    }
}
