use std::path::Path;

/// Guards against recursive self-invocation by detecting when
/// `prunify` is trying to proxy itself.
///
/// Detection works by examining the first token of the proxied command
/// and checking whether it matches the binary name "prunify"
/// or the current executable's file stem.
pub struct RecursionGuard;

impl RecursionGuard {
    /// Check if the given command string would cause recursion.
    ///
    /// Returns `true` if the first token of `command` is:
    /// - Literally "prunify"
    /// - A path whose file stem is "prunify"
    /// - The file stem of the currently running executable
    ///
    /// Returns `false` for empty strings, whitespace-only strings,
    /// and normal commands (like `ls`, `echo`, etc.).
    pub fn is_recursive(command: &str) -> bool {
        let first_token = match command.split_whitespace().next() {
            Some(token) if !token.is_empty() => token,
            _ => return false,
        };

        // Direct match: first token is exactly "prunify"
        if first_token == "prunify" {
            return true;
        }

        // Extract file stem from the first token (handles paths like
        // "./target/debug/prunify" or "/usr/local/bin/prunify")
        let token_stem = Path::new(first_token)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if !token_stem.is_empty()
            && token_stem != first_token
            && token_stem == "prunify"
        {
            return true;
        }

        // Check against the current executable's file stem
        // (catches cases where the binary was renamed or is a debug build)
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_stem) = exe_path.file_stem().and_then(|s| s.to_str())
            && (first_token == exe_stem || token_stem == exe_stem)
        {
            return true;
        }

        false
    }
}
