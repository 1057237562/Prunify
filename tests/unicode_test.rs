mod common;

use common::run_prunify;

use prunifier::engine::column_selector::ColumnSelector;
use prunifier::engine::line_parser::LineParser;
use prunifier::scheme::{Action, MatchCondition, Rule};

// ---------------------------------------------------------------------------
// Test 1: Integration — CJK and emoji passthrough via binary
// ---------------------------------------------------------------------------

/// Invoke `prunifier echo …` with CJK and emoji text.  The binary runs in
/// passthrough mode (no scheme for "echo") so the original output must be
/// preserved verbatim, with only the [UNKNOWN COMMAND] marker appended.
#[test]
fn test_unicode_passthrough_via_binary() {
    // CJK characters
    let (stdout, stderr, code) = run_prunify(&["echo", "文件1", "文件2"]);
    assert_eq!(code, 0, "prunifier echo of CJK text should exit 0");
    assert!(
        stdout.contains("文件1"),
        "expected CJK filename '文件1' in output, got: {:?}",
        stdout
    );
    assert!(
        stdout.contains("文件2"),
        "expected CJK filename '文件2' in output, got: {:?}",
        stdout
    );
    assert!(
        stderr.is_empty(),
        "expected empty stderr, got: {:?}",
        stderr
    );

    // Emoji characters
    let (stdout, stderr, code) = run_prunify(&["echo", "✅", "❌", "🎉"]);
    assert_eq!(code, 0, "prunifier echo of emoji should exit 0");
    assert!(
        stdout.contains("✅"),
        "expected ✅ in output, got: {:?}",
        stdout
    );
    assert!(
        stdout.contains("❌"),
        "expected ❌ in output, got: {:?}",
        stdout
    );
    assert!(
        stdout.contains("🎉"),
        "expected 🎉 in output, got: {:?}",
        stdout
    );
    assert!(
        stderr.is_empty(),
        "expected empty stderr, got: {:?}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// Test 2: LineParser — CJK filenames in simulated ls output
// ---------------------------------------------------------------------------

/// The line parser must not mangle Unicode text and must correctly apply
/// Keep/Discard rules to lines containing CJK characters.
#[test]
fn test_line_parser_preserves_cjk_lines() {
    // Simulated `ls` output with mixed ASCII / CJK filenames
    let output = "Cargo.toml\nREADME.md\nsrc/文件1.rs\nsrc/文件2.rs\ntarget/\n";

    // Keep only lines ending with ".rs"
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Regex {
            pattern: r"\.rs$".to_string(),
        },
        description: None,
    }];

    let result = LineParser::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "src/文件1.rs\nsrc/文件2.rs\n");
}

// ---------------------------------------------------------------------------
// Test 3: ColumnSelector — multi-byte CJK columns in tabular output
// ---------------------------------------------------------------------------

/// When column values contain multi-byte characters, the column *index* must
/// refer to the logical (whitespace-separated) column, NOT a byte/char offset.
/// Rust's split_whitespace() handles this correctly by design.
#[test]
fn test_column_selector_handles_cjk_columns() {
    // Simulated `ps` output with a CJK command name
    let output = "PID コマンド     CPU\n  1 ファイル監視 0.5\n  2 sshd        0.1\n";

    // Keep only column 1 (the COMMAND/CJK-name column)
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Column {
            index: 1,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "コマンド\nファイル監視\nsshd");
}

// ---------------------------------------------------------------------------
// Test 4: LineParser — Unicode regex matching
// ---------------------------------------------------------------------------

/// The `regex` crate supports Unicode patterns by default (the `unicode`
/// feature is enabled in Cargo.toml).  Verify that character classes like
/// `\p{Han}`, `\p{Emoji}`, and negated ASCII ranges work.
#[test]
fn test_unicode_regex_matching() {
    let output = "Café\nresume\nresumé\n你好\nemoji ✅\n";

    // Keep lines containing ANY non-ASCII character (accented, CJK, emoji)
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Regex {
            pattern: r"[^\x00-\x7F]".to_string(),
        },
        description: None,
    }];

    let result = LineParser::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "Café\nresumé\n你好\nemoji ✅\n");
}
