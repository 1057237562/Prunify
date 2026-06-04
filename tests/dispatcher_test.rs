use std::collections::HashMap;

use prunify::engine::CommandTrie;
use prunify::proxy::dispatcher::{DispatchMode, Dispatcher};
use prunify::scheme::{Action, MatchCondition, Rule, Scheme};

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

/// Build a dispatcher with only project schemes (no fallback).
fn dispatcher_with_project(trie: CommandTrie, project: HashMap<String, Scheme>) -> Dispatcher {
    Dispatcher::new(trie, project, HashMap::new())
}

/// Build a dispatcher with both project and fallback schemes.
fn dispatcher_with_both(
    trie: CommandTrie,
    project: HashMap<String, Scheme>,
    fallback: HashMap<String, Scheme>,
) -> Dispatcher {
    Dispatcher::new(trie, project, fallback)
}

#[test]
fn test_exact_match_applies_scheme_rules() {
    // Build trie with "git status" → "git-status"
    let mut trie = CommandTrie::new();
    trie.insert("git status", "git-status");

    // Build scheme that discards lines containing "total"
    let scheme = make_scheme("git status", vec![make_discard_regex_rule("total")]);
    let mut project: HashMap<String, Scheme> = HashMap::new();
    project.insert("git-status".to_string(), scheme);

    let dispatcher = dispatcher_with_project(trie, project);

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
    let mut project: HashMap<String, Scheme> = HashMap::new();
    project.insert("git-status".to_string(), scheme);

    let dispatcher = dispatcher_with_project(trie, project);

    let output = "total 123\nreal data\nmore output\n";
    let (pruned, mode) = dispatcher.dispatch("git status --short", output).unwrap();

    assert_eq!(pruned, "real data\nmore output");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_passthrough_returns_unmodified_output() {
    let trie = CommandTrie::new();
    let project: HashMap<String, Scheme> = HashMap::new();

    let dispatcher = dispatcher_with_project(trie, project);

    let output = "some random output\nline two\nline three\n";
    let (pruned, mode) = dispatcher.dispatch("unknown command", output).unwrap();

    assert_eq!(pruned, output);
    assert_eq!(mode, DispatchMode::Passthrough);
}

#[test]
fn test_scheme_not_in_map_falls_through_to_passthrough() {
    // Trie has "git" → "git-base", but "git-base" is NOT in any scheme map
    let mut trie = CommandTrie::new();
    trie.insert("git", "git-base");

    let project: HashMap<String, Scheme> = HashMap::new();
    let dispatcher = dispatcher_with_project(trie, project);

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

    let mut project: HashMap<String, Scheme> = HashMap::new();
    project.insert("git-base".to_string(), git_base_scheme);
    project.insert("git-status".to_string(), git_status_scheme);

    let dispatcher = dispatcher_with_project(trie, project);

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
    let scheme = make_scheme(
        "ps aux",
        vec![make_discard_regex_rule("root"), make_keep_column_rule(0)],
    );

    let mut project: HashMap<String, Scheme> = HashMap::new();
    project.insert("ps-aux".to_string(), scheme);

    let dispatcher = dispatcher_with_project(trie, project);

    let output = "root   123  0.0  some\nuser   456  0.1  data\nroot   789  0.2  lines\n";
    let (pruned, mode) = dispatcher.dispatch("ps aux", output).unwrap();

    // LineParser: discard lines containing "root" → leaves "user   456  0.1  data\n"
    // ColumnSelector: keep only column 0 → "user"
    assert_eq!(pruned, "user");
    assert_eq!(mode, DispatchMode::ExactMatch);
}

// ── Two-level fallback tests ─────────────────────────────────────────────────

#[test]
fn test_fallback_exact_match_when_project_has_no_match() {
    // Command exists only in fallback schemes, not in project.
    let mut trie = CommandTrie::new();
    trie.insert("git log", "git-log");

    let project: HashMap<String, Scheme> = HashMap::new();

    let mut fallback: HashMap<String, Scheme> = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );

    let dispatcher = dispatcher_with_both(trie, project, fallback);

    let output = "commit abc123\nreal data\ncommit def456\n";
    let (pruned, mode) = dispatcher.dispatch("git log", output).unwrap();

    // Should use fallback scheme — discard lines containing "commit"
    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::ExactMatch);
}

#[test]
fn test_project_exact_takes_priority_over_fallback() {
    // Same command exists in both project and fallback.
    // Project version should win.
    let mut trie = CommandTrie::new();
    trie.insert("git log", "git-log");

    // Project discards "commit" lines
    let mut project: HashMap<String, Scheme> = HashMap::new();
    project.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );

    // Fallback discards "real" lines
    let mut fallback: HashMap<String, Scheme> = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("real")]),
    );

    let dispatcher = dispatcher_with_both(trie, project, fallback);

    let output = "commit abc123\nreal data\ncommit def456\n";
    let (pruned, mode) = dispatcher.dispatch("git log", output).unwrap();

    // Project scheme wins → discard "commit" lines, keep "real data"
    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::ExactMatch);
}

#[test]
fn test_fallback_prefix_match_when_project_has_no_match() {
    // "git log --oneline" not in project, but "git log" is in fallback.
    let mut trie = CommandTrie::new();
    trie.insert("git log", "git-log");

    let project: HashMap<String, Scheme> = HashMap::new();

    let mut fallback: HashMap<String, Scheme> = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );

    let dispatcher = dispatcher_with_both(trie, project, fallback);

    let output = "commit abc123\nreal data\n";
    let (pruned, mode) = dispatcher.dispatch("git log --oneline", output).unwrap();

    // Should fall back to fallback's prefix match
    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_fallback_exact_then_project_prefix() {
    // "git" is in project (exact), "git log" is in fallback (exact).
    // Searching "git log" should get exact match from fallback, not prefix from project.
    let mut trie = CommandTrie::new();
    trie.insert("git", "git-base");
    trie.insert("git log", "git-log");

    let mut project: HashMap<String, Scheme> = HashMap::new();
    project.insert(
        "git-base".to_string(),
        make_scheme("git", vec![make_discard_regex_rule("total")]),
    );

    let mut fallback: HashMap<String, Scheme> = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );

    let dispatcher = dispatcher_with_both(trie, project, fallback);

    // "git log" exists only in fallback → exact match from fallback
    // Fallback discards "commit" lines → "total 123" and "real data" survive
    let output = "total 123\ncommit abc123\nreal data\n";
    let (pruned, mode) = dispatcher.dispatch("git log", output).unwrap();

    assert_eq!(pruned, "total 123\nreal data");
    assert_eq!(mode, DispatchMode::ExactMatch);
}

#[test]
fn test_command_not_in_either_falls_through_to_passthrough() {
    let trie = CommandTrie::new();
    let project: HashMap<String, Scheme> = HashMap::new();
    let fallback: HashMap<String, Scheme> = HashMap::new();

    let dispatcher = dispatcher_with_both(trie, project, fallback);

    let output = "some random output\n";
    let (pruned, mode) = dispatcher.dispatch("unknown", output).unwrap();

    assert_eq!(pruned, output);
    assert_eq!(mode, DispatchMode::Passthrough);
}
