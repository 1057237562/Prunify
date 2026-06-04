use prunify::scheme::{Action, MatchCondition, Scheme};

#[test]
fn test_deserialize_valid_scheme() {
    let json = r#"{
        "command": "ls -la",
        "version": 1,
        "rules": [{
            "action": "discard",
            "match_condition": {
                "type": "Regex",
                "pattern": "^total"
            }
        }]
    }"#;

    let scheme: Scheme = serde_json::from_str(json).expect("Should deserialize valid scheme");
    assert_eq!(scheme.command, "ls -la");
    assert_eq!(scheme.version, 1);
    assert_eq!(scheme.rules.len(), 1);
    assert!(matches!(scheme.rules[0].action, Action::Discard));
    assert!(matches!(
        scheme.rules[0].match_condition,
        MatchCondition::Regex { .. }
    ));
    assert!(scheme.rules[0].description.is_none());
    assert!(
        scheme.validate().is_ok(),
        "validate() should return Ok for valid scheme"
    );
}

#[test]
fn test_reject_invalid_action() {
    let json = r#"{
        "command": "ls",
        "version": 1,
        "rules": [{
            "action": "delete",
            "match_condition": {
                "type": "Regex",
                "pattern": "."
            }
        }]
    }"#;

    let result: Result<Scheme, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Should reject invalid action 'delete'");
}

#[test]
fn test_reject_missing_command() {
    let json = r#"{
        "version": 1,
        "rules": [{
            "action": "keep",
            "match_condition": {
                "type": "Regex",
                "pattern": "."
            }
        }]
    }"#;

    let result: Result<Scheme, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Should reject scheme missing 'command'");
}

#[test]
fn test_column_rule_requires_index() {
    let json = r#"{
        "command": "ls",
        "version": 1,
        "rules": [{
            "action": "keep",
            "match_condition": {
                "type": "Column",
                "pattern": "^d"
            }
        }]
    }"#;

    let result: Result<Scheme, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "Should reject Column match_condition missing 'index'"
    );
}
