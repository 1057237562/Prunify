use regex::Regex;
use std::sync::LazyLock;

static RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("\x1b\\[[0-9;]*[a-zA-Z]").expect("valid ANSI regex"));

pub struct AnsiStripper;

impl AnsiStripper {
    /// Remove ANSI escape sequences from a string.
    /// Strips CSI sequences (\x1b[...m) and similar control sequences.
    pub fn strip(input: &str) -> String {
        RE.replace_all(input, "").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_removes_simple_color() {
        assert_eq!(AnsiStripper::strip("\x1b[31mred\x1b[0m"), "red");
    }

    #[test]
    fn strip_passthrough_plain() {
        assert_eq!(AnsiStripper::strip("hello"), "hello");
    }
}
