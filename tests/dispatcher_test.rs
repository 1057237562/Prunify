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

/// Build a local-only trie and dispatcher (no global/fallback).
fn local_dispatcher(
    cmds: &[(&str, &str)],
    schemes: HashMap<String, Scheme>,
) -> Dispatcher {
    let mut local_trie = CommandTrie::new();
    for (cmd, id) in cmds {
        local_trie.insert(cmd, id);
    }
    Dispatcher::new(local_trie, CommandTrie::new(), schemes, HashMap::new())
}

/// Build a dispatcher with separate local and global tries + maps.
fn two_tier_dispatcher(
    local_cmds: &[(&str, &str)],
    global_cmds: &[(&str, &str)],
    project: HashMap<String, Scheme>,
    fallback: HashMap<String, Scheme>,
) -> Dispatcher {
    let mut local_trie = CommandTrie::new();
    for (cmd, id) in local_cmds {
        local_trie.insert(cmd, id);
    }
    let mut global_trie = CommandTrie::new();
    for (cmd, id) in global_cmds {
        global_trie.insert(cmd, id);
    }
    Dispatcher::new(local_trie, global_trie, project, fallback)
}

// ── Basic prefix matching (local-only) ───────────────────────────────────────

#[test]
fn test_prefix_match_short_command() {
    // "git status" in local trie → prefix match with 2 tokens
    let mut project = HashMap::new();
    project.insert(
        "git-status".to_string(),
        make_scheme("git status", vec![make_discard_regex_rule("total")]),
    );

    let d = local_dispatcher(&[("git status", "git-status")], project);

    let (pruned, mode) = d.dispatch("git status", "total 123\nreal data\n").unwrap();
    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_prefix_match_longer_command() {
    // "git status --short" also prefix-matches "git status" (2 tokens)
    let mut project = HashMap::new();
    project.insert(
        "git-status".to_string(),
        make_scheme("git status", vec![make_discard_regex_rule("total")]),
    );

    let d = local_dispatcher(&[("git status", "git-status")], project);

    let (pruned, mode) = d.dispatch("git status --short", "total 123\nreal data\n").unwrap();
    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_passthrough_when_no_match() {
    let d = local_dispatcher(&[], HashMap::new());

    let output = "some random output\nline two\n";
    let (pruned, mode) = d.dispatch("unknown command", output).unwrap();

    assert_eq!(pruned, output);
    assert_eq!(mode, DispatchMode::Passthrough);
}

#[test]
fn test_passthrough_when_scheme_id_not_in_map() {
    // local_trie has "git" → "git-base", but scheme map is empty
    let d = local_dispatcher(&[("git", "git-base")], HashMap::new());

    let output = "some data\n";
    let (pruned, mode) = d.dispatch("git", output).unwrap();

    assert_eq!(pruned, output);
    assert_eq!(mode, DispatchMode::Passthrough);
}

#[test]
fn test_deepest_prefix_match_wins() {
    // "git" (1 token) and "git status" (2 tokens) both in local trie.
    // "git status" is deepest → used.
    let mut project = HashMap::new();
    project.insert(
        "git-base".to_string(),
        make_scheme("git", vec![make_discard_regex_rule("total")]),
    );
    project.insert(
        "git-status".to_string(),
        make_scheme("git status", vec![make_discard_regex_rule("summary")]),
    );

    let d = local_dispatcher(&[("git", "git-base"), ("git status", "git-status")], project);

    let output = "total 123\nsummary line\nreal data\n";
    let (pruned, mode) = d.dispatch("git status", output).unwrap();

    // "git-status" is the deepest prefix match (2 tokens) → discards "summary"
    assert_eq!(pruned, "total 123\nreal data");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_line_parser_then_column_selector_apply_in_order() {
    let mut project = HashMap::new();
    project.insert(
        "ps-aux".to_string(),
        make_scheme(
            "ps aux",
            vec![make_discard_regex_rule("root"), make_keep_column_rule(0)],
        ),
    );

    let d = local_dispatcher(&[("ps aux", "ps-aux")], project);

    let output = "root   123  0.0  some\nuser   456  0.1  data\nroot   789  0.2  lines\n";
    let (pruned, mode) = d.dispatch("ps aux", output).unwrap();

    // LineParser discards "root" lines → "user   456  0.1  data\n"
    // ColumnSelector keeps column 0 → "user"
    assert_eq!(pruned, "user");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

// ── Two-level fallback tests (separate local + global tries) ─────────────────

#[test]
fn test_fallback_used_when_local_has_no_match() {
    // local is empty → no match. global has "git log" → used.
    let mut fallback = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );

    let d = two_tier_dispatcher(&[], &[("git log", "git-log")], HashMap::new(), fallback);

    let output = "commit abc123\nreal data\ncommit def456\n";
    let (pruned, mode) = d.dispatch("git log", output).unwrap();

    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_local_takes_priority_over_global() {
    // Same command in both. Local trie checked first → local scheme wins.
    let mut project = HashMap::new();
    project.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );
    let mut fallback = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("real")]),
    );

    let d = two_tier_dispatcher(
        &[("git log", "git-log")],
        &[("git log", "git-log")],
        project,
        fallback,
    );

    let output = "commit abc123\nreal data\ncommit def456\n";
    let (pruned, mode) = d.dispatch("git log", output).unwrap();

    // Local wins → discards "commit" lines
    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_fallback_prefix_match_when_local_empty() {
    // "git log --oneline" → local empty → global prefix match on "git log"
    let mut fallback = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );

    let d = two_tier_dispatcher(&[], &[("git log", "git-log")], HashMap::new(), fallback);

    let output = "commit abc123\nreal data\n";
    let (pruned, mode) = d.dispatch("git log --oneline", output).unwrap();

    assert_eq!(pruned, "real data");
    assert_eq!(mode, DispatchMode::PrefixMatch(2));
}

#[test]
fn test_local_prefix_match_takes_priority_over_global() {
    // Local has "git" (1-token prefix), global has "git log" (2-token).
    // "git log" is sent. Local "git" is a prefix match → local wins.
    let mut project = HashMap::new();
    project.insert(
        "git-base".to_string(),
        make_scheme("git", vec![make_discard_regex_rule("total")]),
    );
    let mut fallback = HashMap::new();
    fallback.insert(
        "git-log".to_string(),
        make_scheme("git log", vec![make_discard_regex_rule("commit")]),
    );

    let d = two_tier_dispatcher(
        &[("git", "git-base")],
        &[("git log", "git-log")],
        project,
        fallback,
    );

    let output = "total 123\ncommit abc123\nreal data\n";
    let (pruned, mode) = d.dispatch("git log", output).unwrap();

    // Local "git" prefix matches → discards "total" lines
    assert_eq!(pruned, "commit abc123\nreal data");
    assert_eq!(mode, DispatchMode::PrefixMatch(1));
}

#[test]
fn test_command_not_in_either_falls_through_to_passthrough() {
    let d = two_tier_dispatcher(&[], &[], HashMap::new(), HashMap::new());

    let output = "some random output\n";
    let (pruned, mode) = d.dispatch("unknown", output).unwrap();

    assert_eq!(pruned, output);
    assert_eq!(mode, DispatchMode::Passthrough);
}
