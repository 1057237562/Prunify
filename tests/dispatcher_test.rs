use std::collections::HashMap;

use prunifier::engine::CommandTrie;
use prunifier::proxy::dispatcher::{DispatchMode, Dispatcher};
use prunifier::scheme::{Action, MatchCondition, Rule, Scheme};

fn make_scheme(command: &str, rules: Vec<Rule>) -> Scheme {
    Scheme {
        command: command.to_string(),
        version: 1,
        rules,
    }
}

fn make_discard_regex_rule(pattern: &str) -> Rule {
    Rule {
        action: Action::Discard,
        match_condition: MatchCondition::Regex {
            pattern: pattern.to_string(),
        },
        description: None,
    }
}

fn make_keep_column_rule(index: usize) -> Rule {
    Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Column {
            index,
            pattern: ".*".to_string(),
        },
        description: None,
    }
}

#[test]
fn test_exact_match_applies_scheme_rules() {
    // Build trie with "git status" → "git-status"
    let mut trie = CommandTrie::new();
    trie.insert("git status", "git-status");

    // Build scheme that discards lines containing "total"
    let scheme = make_scheme("git status", vec![make_discard_regex_rule("total")]);
    let mut schemes: HashMap<String, Scheme> = HashMap::new();
    schemes.insert("git-status".to_string(), scheme);

    let dispatcher = Dispatcher::new(trie, schemes);

    let output = "total 123\nreal data\nmore output\n";
    let (pruned, mode) = dispatcher.dispatch("git status", output).unwrap();

    assert_eq!(pruned, "real data\nmore output");
    assert_eq!(mode, DispatchMode::ExactMatch);
}

#[test]
fn test_prefix_match_applies_scheme_rules() {
    // Build trie with "git status" → "git-status" (no exact match for "git status --short")
    let mut trie = CommandTrie::new();
    trie.insert("git status", "git-status");

    // Build scheme that discards lines containing "total"
    let scheme = make_scheme("git status", vec![make_discard_regex_rule("total")]);
    let mut schemes: HashMap<String, Scheme> = HashMap::new();
    schemes.insert("git-status".to_string(), scheme);

    let dispatcher = Dispatcher::new(trie, schemes);

    let output = "total 123\nreal data\nmore output\n";
    let (pruned, mode) = dispatcher.dispatch("git status --short", output).unwrap();

    assert_eq!(pruned, "real data\nmore output");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_passthrough_returns_unmodified_output() {
    let trie = CommandTrie::new();
    let schemes: HashMap<String, Scheme> = HashMap::new();

    let dispatcher = Dispatcher::new(trie, schemes);

    let output = "some random output\nline two\nline three\n";
    let (pruned, mode) = dispatcher.dispatch("unknown command", output).unwrap();

    assert_eq!(pruned, output);
    assert_eq!(mode, DispatchMode::Passthrough);
}

#[test]
fn test_scheme_not_in_map_falls_through_to_passthrough() {
    // Trie has "git" → "git-base", but "git-base" is NOT in schemes map
    let mut trie = CommandTrie::new();
    trie.insert("git", "git-base");

    let schemes: HashMap<String, Scheme> = HashMap::new();

    let dispatcher = Dispatcher::new(trie, schemes);

    let output = "some data\nmore data\n";
    let (pruned, mode) = dispatcher.dispatch("git", output).unwrap();

    // Should fall through to passthrough since scheme_id "git-base" doesn't exist
    assert_eq!(pruned, output);
    assert_eq!(mode, DispatchMode::Passthrough);
}

#[test]
fn test_exact_match_takes_priority_over_prefix() {
    // Trie has "git" → "git-base" and "git status" → "git-status"
    let mut trie = CommandTrie::new();
    trie.insert("git", "git-base");
    trie.insert("git status", "git-status");

    // "git-base" discards "total" lines, "git-status" discards "summary" lines
    let git_base_scheme = make_scheme("git", vec![make_discard_regex_rule("total")]);
    let git_status_scheme = make_scheme("git status", vec![make_discard_regex_rule("summary")]);

    let mut schemes: HashMap<String, Scheme> = HashMap::new();
    schemes.insert("git-base".to_string(), git_base_scheme);
    schemes.insert("git-status".to_string(), git_status_scheme);

    let dispatcher = Dispatcher::new(trie, schemes);

    let output = "total 123\nsummary line\nreal data\n";
    let (pruned, mode) = dispatcher.dispatch("git status", output).unwrap();

    // Should use "git-status" scheme (exact match), not "git-base" (prefix match)
    // "git-status" discards "summary", so "summary line" should be removed
    assert_eq!(pruned, "total 123\nreal data");
    assert_eq!(mode, DispatchMode::ExactMatch);
}

#[test]
fn test_line_parser_then_column_selector_apply_in_order() {
    // Test that LineParser is applied first, then ColumnSelector
    let mut trie = CommandTrie::new();
    trie.insert("ps aux", "ps-aux");

    // Scheme: first discard lines matching "root", then keep only column 0 (PID)
    // But wait — the LineParser also sees the Column rule and tries to filter lines by it.
    // Use two rules: 1) Regex Discard (line parser), 2) Column Keep (column selector)
    let scheme = make_scheme(
        "ps aux",
        vec![make_discard_regex_rule("root"), make_keep_column_rule(0)],
    );

    let mut schemes: HashMap<String, Scheme> = HashMap::new();
    schemes.insert("ps-aux".to_string(), scheme);

    let dispatcher = Dispatcher::new(trie, schemes);

    let output = "root   123  0.0  some\nuser   456  0.1  data\nroot   789  0.2  lines\n";
    let (pruned, mode) = dispatcher.dispatch("ps aux", output).unwrap();

    // LineParser: discard lines containing "root" → leaves "user   456  0.1  data\n"
    // ColumnSelector: keep only column 0 → "user"
    assert_eq!(pruned, "user");
    assert_eq!(mode, DispatchMode::ExactMatch);
}
