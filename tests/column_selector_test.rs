use prunifier::engine::ColumnSelector;
use prunifier::scheme::{Action, MatchCondition, Rule};

#[test]
fn test_keep_specific_columns() {
    let output = "foo bar baz\nqux quux quuz";
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Column {
            index: 0,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    assert_eq!(result, "foo\nqux");
}

#[test]
fn test_discard_specific_columns() {
    let output = "a b c d\ne f g h";
    let rules = vec![Rule {
        action: Action::Discard,
        match_condition: MatchCondition::Column {
            index: 1,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    assert_eq!(result, "a c d\ne g h");
}

#[test]
fn test_whitespace_separator() {
    let output = "col1    col2\tcol3\n  x   y\tz  ";
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Column {
            index: 1,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    assert_eq!(result, "col2\ny");
}

#[test]
fn test_variable_column_count() {
    let output = "a b c\nd e\nf g h i";
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Column {
            index: 1,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    // Line 1: columns [a, b, c], keep index 1 → "b"
    // Line 2: columns [d, e], keep index 1 → "e"
    // Line 3: columns [f, g, h, i], keep index 1 → "g"
    assert_eq!(result, "b\ne\ng");
}

#[test]
fn test_column_index_out_of_bounds() {
    let output = "a b\nc";
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Column {
            index: 5,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    assert_eq!(result, "a b\nc");
}

#[test]
fn test_discard_out_of_bounds() {
    let output = "a b c\nd e";
    let rules = vec![Rule {
        action: Action::Discard,
        match_condition: MatchCondition::Column {
            index: 10,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    assert_eq!(result, "a b c\nd e");
}

#[test]
fn test_empty_output() {
    let output = "";
    let rules = vec![Rule {
        action: Action::Keep,
        match_condition: MatchCondition::Column {
            index: 0,
            pattern: ".*".to_string(),
        },
        description: None,
    }];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_non_column_rules_skipped() {
    let output = "keep this line";
    let rules = vec![
        Rule {
            action: Action::Discard,
            match_condition: MatchCondition::Regex {
                pattern: ".*".to_string(),
            },
            description: None,
        },
        Rule {
            action: Action::Discard,
            match_condition: MatchCondition::LineNumber { lines: vec![1] },
            description: None,
        },
    ];

    let result = ColumnSelector::apply_rules(output, &rules).unwrap();
    // Non-Column rules are skipped, so the line should be unchanged
    assert_eq!(result, "keep this line");
}
