use prunifier::engine::line_parser::LineParser;
use prunifier::scheme::{Action, MatchCondition, Rule};

#[test]
fn test_keep_lines_matching_regex() {
    let output = "line one\nerror: something broke\nline three\nerror: critical failure\n";
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Regex {
            pattern: r"^error".to_string(),
        },
        description: None,
    }];

    let result = LineParser::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "error: something broke\nerror: critical failure\n");
}

#[test]
fn test_discard_lines_matching_regex() {
    let output = "line one\nerror: something broke\nline three\nerror: critical failure\n";
    let rules = vec![Rule {
        action: Action::Discard,
        match_condition: MatchCondition::Regex {
            pattern: r"^error".to_string(),
        },
        description: None,
    }];

    let result = LineParser::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "line one\nline three\n");
}

#[test]
fn test_multiple_rules_apply_in_order() {
    let output = "line one\nerror: A\nwarning: B\nerror: C\n";
    let rules = vec![
        Rule {
            action: Action::Keep,
            match_condition: MatchCondition::Regex {
                pattern: r"^error".to_string(),
            },
            description: None,
        },
        Rule {
            action: Action::Keep,
            match_condition: MatchCondition::Regex {
                pattern: r"C$".to_string(),
            },
            description: None,
        },
    ];

    let result = LineParser::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "error: C\n");
}

#[test]
fn test_empty_output() {
    let output = "";
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Regex {
            pattern: r"^error".to_string(),
        },
        description: None,
    }];

    let result = LineParser::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "");
}

#[test]
fn test_no_matching_rules_keeps_all() {
    let output = "line one\nline two\nline three\n";
    let rules = vec![Rule {
        action: Action::Discard,
        match_condition: MatchCondition::Regex {
            pattern: r"^error".to_string(),
        },
        description: None,
    }];

    let result = LineParser::apply_rules(output, &rules).expect("apply_rules should succeed");
    assert_eq!(result, "line one\nline two\nline three\n");
}
