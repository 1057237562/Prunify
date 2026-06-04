use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::types::PrunifyConfig;
use crate::error::PrunifyResult;
use crate::scheme::storage::SchemeStorage;
use crate::scheme::types::Scheme;

/// Loads schemes by merging default schemes with per-project overrides.
///
/// Project overrides COMPLETELY REPLACE default schemes for the same command
/// (no deep merging of rules).
pub struct SchemeLoader {
    default_dir: PathBuf,
}

impl SchemeLoader {
    pub fn new(default_dir: PathBuf) -> Self {
        Self { default_dir }
    }

    /// Load all schemes, merging defaults and project overrides.
    ///
    /// 1. Loads all `.json` scheme files from `self.default_dir`.
    /// 2. Loads all `.json` scheme files from the project override directory
    ///    (determined by `config.scheme_dir` or default `"~/.prunify/schemes/"`).
    /// 3. Project schemes override defaults with the same `command` key.
    ///
    /// Missing or empty directories are skipped (not an error).
    pub fn load(&self, config: &PrunifyConfig) -> PrunifyResult<HashMap<String, Scheme>> {
        let mut schemes = HashMap::new();

        // Load defaults
        let default_schemes = SchemeStorage::load_all(&self.default_dir)?;
        for s in default_schemes {
            schemes.insert(s.command.clone(), s);
        }

        // Determine project override directory
        let project_dir = config
            .scheme_dir
            .clone()
            .unwrap_or_else(|| crate::config::default_prunify_dir().join("schemes"));

        // Load project overrides (completely replaces defaults for same command)
        let project_schemes = SchemeStorage::load_all(&project_dir)?;
        for s in project_schemes {
            schemes.insert(s.command.clone(), s);
        }

        Ok(schemes)
    }
}
