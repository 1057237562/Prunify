#[cfg(unix)]
use libc;

/// Detects TTY output and identifies interactive commands that should passthrough unprocessed.
pub struct TtyDetector;

/// Known interactive commands that should always passthrough without pruning.
const INTERACTIVE_COMMANDS: &[&str] = &[
    "vim", "nano", "htop", "top", "less", "more", "emacs", "screen", "tmux", "man", "irb",
    "python", "node",
];

impl TtyDetector {
    /// Check if stdout is a TTY.
    /// On Unix, uses libc::isatty. On Windows, returns false (no winapi dependency).
    #[cfg(unix)]
    pub fn is_tty() -> bool {
        unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
    }

    /// Windows stub: no TTY detection available without winapi.
    #[cfg(windows)]
    pub fn is_tty() -> bool {
        false
    }

    /// Check if command should always passthrough (interactive programs).
    /// Extracts the first word of the command and checks against known interactive binaries.
    pub fn should_passthrough(command: &str) -> bool {
        let first_word = command.split_whitespace().next().unwrap_or("");
        INTERACTIVE_COMMANDS.contains(&first_word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_passthrough_known_interactive() {
        assert!(TtyDetector::should_passthrough("vim"));
        assert!(TtyDetector::should_passthrough("vim /etc/hosts"));
        assert!(TtyDetector::should_passthrough("top -u root"));
        assert!(TtyDetector::should_passthrough(
            "python -c \"print('hello')\""
        ));
    }

    #[test]
    fn test_should_passthrough_unknown_command() {
        assert!(!TtyDetector::should_passthrough("ls -la"));
        assert!(!TtyDetector::should_passthrough("grep foo bar"));
        assert!(!TtyDetector::should_passthrough("cat /etc/passwd"));
    }

    #[test]
    fn test_should_passthrough_empty_string() {
        assert!(!TtyDetector::should_passthrough(""));
        assert!(!TtyDetector::should_passthrough("   "));
    }
}
