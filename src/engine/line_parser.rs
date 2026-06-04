use crate::error::PrunifierResult;
use crate::scheme::{Action, MatchCondition, Rule};

pub struct LineParser;

impl LineParser {
    /// Apply rules to output string. Rules are applied in order sequentially.
    /// Each rule operates on the result of the previous rule.
    pub fn apply_rules(output: &str, rules: &[Rule]) -> PrunifierResult<String> {
        if output.is_empty() {
            return Ok(String::new());
        }

        let has_trailing_newline = output.ends_with('\n');
        let mut lines: Vec<&str> = output.lines().collect();

        for rule in rules {
            match &rule.match_condition {
                MatchCondition::Regex { pattern } => {
                    let re = regex::Regex::new(pattern)?;
                    match rule.action {
                        Action::Keep => {
                            lines.retain(|line| re.is_match(line));
                        }
                        Action::Discard => {
                            lines.retain(|line| !re.is_match(line));
                        }
                    }
                }
                MatchCondition::Column { index, pattern } => {
                    let re = regex::Regex::new(pattern)?;
                    match rule.action {
                        Action::Keep => {
                            lines.retain(|line| {
                                line.split_whitespace()
                                    .nth(*index)
                                    .is_some_and(|col| re.is_match(col))
                            });
                        }
                        Action::Discard => {
                            lines.retain(|line| {
                                !line
                                    .split_whitespace()
                                    .nth(*index)
                                    .is_some_and(|col| re.is_match(col))
                            });
                        }
                    }
                }
                MatchCondition::LineNumber {
                    lines: line_numbers,
                } => match rule.action {
                    Action::Keep => {
                        lines = lines
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| line_numbers.contains(&(i + 1)))
                            .map(|(_, line)| *line)
                            .collect();
                    }
                    Action::Discard => {
                        lines = lines
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| !line_numbers.contains(&(i + 1)))
                            .map(|(_, line)| *line)
                            .collect();
                    }
                },
            }
        }

        let mut result = lines.join("\n");
        if has_trailing_newline {
            result.push('\n');
        }

        Ok(result)
    }
}
