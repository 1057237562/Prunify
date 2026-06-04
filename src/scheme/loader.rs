use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::types::PrunifyConfig;
use crate::error::PrunifyResult;
use crate::scheme::storage::SchemeStorage;
use crate::scheme::types::Scheme;

/// Loads schemes with a two-level separation: project-local schemes first,
/// then global fallback schemes from `~/.prunify/schemes/`.
///
/// - **Project schemes**: loaded from `config.scheme_dir`, `.prunify/schemes/`,
///   or fall back to `~/.prunify/schemes/` (if no local schemes dir exists).
/// - **Fallback schemes**: loaded from `self.fallback_dir` (`~/.prunify/schemes/`).
///
/// During dispatch, project schemes are consulted first. If a command is not
/// found there, fallback schemes are checked before falling through to passthrough.
pub struct SchemeLoader {
    fallback_dir: PathBuf,
}

impl SchemeLoader {
    pub fn new(fallback_dir: PathBuf) -> Self {
        Self { fallback_dir }
    }

    /// Load all schemes, returning project-level and fallback-level separately.
    ///
    /// 1. Determines the project scheme directory:
    ///    - `config.scheme_dir` if set,
    ///    - otherwise `.prunify/schemes/` if it exists (project-local bundled schemes),
    ///    - otherwise `~/.prunify/schemes/`.
    /// 2. Loads `.json` scheme files from the **project** directory into `project_schemes`.
    /// 3. Loads `.json` scheme files from the **fallback** directory (`~/.prunify/schemes/`)
    ///    *only if it is a different directory* from the project dir — avoiding
    ///    double-loading.
    ///
    /// Missing or empty directories are skipped (not an error).
    pub fn load(
        &self,
        config: &PrunifyConfig,
    ) -> PrunifyResult<(
        HashMap<String, Scheme>,
        HashMap<String, Scheme>,
    )> {
        let mut project_schemes = HashMap::new();

        // Determine project scheme directory
        let project_dir = config
            .scheme_dir
            .clone()
            .unwrap_or_else(|| {
                // Check project-local .prunify/schemes/ before falling back
                // to the home directory default. This enables schemes bundled
                // with a repository to work without a .prunify.yaml config.
                let local = PathBuf::from(".prunify").join("schemes");
                if local.exists() {
                    local
                } else {
                    self.fallback_dir.clone()
                }
            });

        // Load project-level schemes
        let loaded = SchemeStorage::load_all(&project_dir)?;
        for s in loaded {
            project_schemes.insert(s.command.clone(), s);
        }

        // Load fallback schemes only if the directory differs from project dir
        let mut fallback_schemes = HashMap::new();
        if self.fallback_dir != project_dir {
            let loaded = SchemeStorage::load_all(&self.fallback_dir)?;
            for s in loaded {
                fallback_schemes.insert(s.command.clone(), s);
            }
        }

        Ok((project_schemes, fallback_schemes))
    }
}
