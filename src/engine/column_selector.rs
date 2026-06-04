use crate::error::PrunifyResult;
use crate::scheme::{Action, MatchCondition, Rule};

pub struct ColumnSelector;

impl ColumnSelector {
    /// Apply column-based rules to tabular output.
    /// Each line is split on whitespace, then column rules are applied.
    ///
    /// Processing order:
    /// 1. All Column Keep rules are collected. Their indices are merged so that
    ///    multiple Keep rules retain multiple columns (e.g. Keep(1) + Keep(10)
    ///    keeps both PID and COMMAND).
    /// 2. If all Keep indices are valid for a line, only those columns are kept.
    ///    If any is out of bounds, the full line is preserved (backward compat).
    /// 3. Then Column Discard rules remove columns by their current (possibly
    ///    shifted) index.
    ///
    /// Non-Column rules (Regex, LineNumber) are skipped entirely.
    pub fn apply_rules(output: &str, rules: &[Rule]) -> PrunifyResult<String> {
        let mut result_lines: Vec<String> = Vec::new();

        let keep_indices: Vec<usize> = rules
            .iter()
            .filter_map(|rule| {
                if matches!(rule.action, Action::Keep)
                    && let MatchCondition::Column { index, pattern: _ } = &rule.match_condition
                {
                    return Some(*index);
                }
                None
            })
            .collect();

        for line in output.lines() {
            let columns: Vec<&str> = line.split_whitespace().collect();

            // Empty or whitespace-only lines: preserve as-is
            if columns.is_empty() {
                result_lines.push(line.to_string());
                continue;
            }

            let mut current: Vec<String> = if keep_indices.is_empty() {
                columns.iter().map(|s| s.to_string()).collect()
            } else {
                let all_keep_valid = keep_indices.iter().all(|i| *i < columns.len());
                if all_keep_valid {
                    keep_indices
                        .iter()
                        .map(|i| columns[*i].to_string())
                        .collect()
                } else {
                    columns.iter().map(|s| s.to_string()).collect()
                }
            };

            for rule in rules {
                if let MatchCondition::Column { index, pattern: _ } = &rule.match_condition
                    && matches!(rule.action, Action::Discard)
                    && *index < current.len()
                {
                    current.remove(*index);
                }
            }

            // If discarding all columns left nothing, skip the line entirely
            if !current.is_empty() {
                result_lines.push(current.join(" "));
            }
        }

        Ok(result_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_keep_multiple_columns() {
        let output = "a b c d e f g h i j k\nl m n o p q r s t u v";
        let rules = vec![
            Rule {
                action: Action::Keep,
                match_condition: MatchCondition::Column {
                    index: 1,
                    pattern: ".*".to_string(),
                },
                description: None,
            },
            Rule {
                action: Action::Keep,
                match_condition: MatchCondition::Column {
                    index: 10,
                    pattern: ".*".to_string(),
                },
                description: None,
            },
        ];
        let result = ColumnSelector::apply_rules(output, &rules).unwrap();
        // Keep columns 1 and 10 → "b k" and "m v"
        assert_eq!(result, "b k\nm v");
    }

    #[test]
    fn test_internal_keep_first_column() {
        let output = "foo bar\nbaz qux";
        let rules = vec![Rule {
            action: Action::Keep,
            match_condition: MatchCondition::Column {
                index: 0,
                pattern: ".*".to_string(),
            },
            description: None,
        }];
        assert_eq!(
            ColumnSelector::apply_rules(output, &rules).unwrap(),
            "foo\nbaz"
        );
    }

    #[test]
    fn test_internal_discard_middle_column() {
        let output = "a b c\nd e f";
        let rules = vec![Rule {
            action: Action::Discard,
            match_condition: MatchCondition::Column {
                index: 1,
                pattern: ".*".to_string(),
            },
            description: None,
        }];
        assert_eq!(
            ColumnSelector::apply_rules(output, &rules).unwrap(),
            "a c\nd f"
        );
    }
}
