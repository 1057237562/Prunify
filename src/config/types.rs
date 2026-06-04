use serde::Deserialize;
use std::path::PathBuf;

/// Top-level configuration for prunify, deserialized from `.prunify.yaml`.
///
/// All fields are `Option` so we can distinguish "not present in the file"
/// from "explicitly set to the default value".
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrunifyConfig {
    /// Custom path to scheme files. Default: "~/.prunify/schemes/"
    pub scheme_dir: Option<PathBuf>,

    /// Enable verbose logging
    pub verbose: Option<bool>,

    /// Disable colored output
    pub no_color: Option<bool>,

    /// If true, reject unknown commands with error instead of passthrough
    pub strict: Option<bool>,
}

impl Default for PrunifyConfig {
    fn default() -> Self {
        Self {
            scheme_dir: None,
            verbose: Some(false),
            no_color: Some(false),
            strict: Some(false),
        }
    }
}

/// Returns the default prunify directory: `~/.prunify/`
///
/// Falls back to `./.prunify/` if `$HOME` is not set.
pub fn default_prunify_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".prunify")
}
