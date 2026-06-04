use serde::Deserialize;
use std::path::PathBuf;

/// Top-level configuration for prunifier, deserialized from `.prunifier.yaml`.
///
/// All fields are `Option` so we can distinguish "not present in the file"
/// from "explicitly set to the default value".
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrunifierConfig {
    /// Custom path to scheme files. Default: ".prunifier/schemes/"
    pub scheme_dir: Option<PathBuf>,

    /// Enable verbose logging
    pub verbose: Option<bool>,

    /// Disable colored output
    pub no_color: Option<bool>,

    /// If true, reject unknown commands with error instead of passthrough
    pub strict: Option<bool>,
}

impl Default for PrunifierConfig {
    fn default() -> Self {
        Self {
            scheme_dir: None,
            verbose: Some(false),
            no_color: Some(false),
            strict: Some(false),
        }
    }
}
