use serde::Deserialize;

use crate::error::PrunifyResult;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scheme {
    pub command: String,
    pub version: u32,
    pub rules: Vec<Rule>,
}

impl Scheme {
    pub fn validate(&self) -> PrunifyResult<()> {
        // Version must be exactly 1
        if self.version != 1 {
            return Err(crate::error::PrunifyError::InvalidScheme(format!(
                "version must be 1, got {}",
                self.version
            )));
        }

        // At least one rule is required
        if self.rules.is_empty() {
            return Err(crate::error::PrunifyError::InvalidScheme(
                "scheme must have at least one rule".to_string(),
            ));
        }

        // Validate all regex patterns compile
        for rule in &self.rules {
            match &rule.match_condition {
                MatchCondition::Regex { pattern } | MatchCondition::Column { pattern, .. } => {
                    regex::Regex::new(pattern).map_err(|e| {
                        crate::error::PrunifyError::InvalidScheme(format!(
                            "invalid regex pattern '{}': {}",
                            pattern, e
                        ))
                    })?;
                }
                MatchCondition::LineNumber { .. } => {}
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub action: Action,
    pub match_condition: MatchCondition,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum Action {
    Keep,
    Discard,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum MatchCondition {
    Regex { pattern: String },
    Column { index: usize, pattern: String },
    LineNumber { lines: Vec<usize> },
}
