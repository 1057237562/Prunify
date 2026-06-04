use std::path::Path;

use crate::error::PrunifyResult;
use crate::scheme::types::Scheme;

pub struct SchemeStorage;

impl SchemeStorage {
    /// Load a single scheme from a JSON file path
    pub fn load(path: &Path) -> PrunifyResult<Scheme> {
        let content = std::fs::read_to_string(path)?;
        let scheme: Scheme = serde_json::from_str(&content)?;
        scheme.validate()?;
        Ok(scheme)
    }

    /// Load all .json scheme files from a directory (non-recursive)
    pub fn load_all(dir: &Path) -> PrunifyResult<Vec<Scheme>> {
        let mut schemes = Vec::new();
        if !dir.exists() {
            return Ok(schemes);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                match Self::load(&path) {
                    Ok(scheme) => schemes.push(scheme),
                    Err(e) => eprintln!("Warning: skipping {}: {}", path.display(), e),
                }
            }
        }
        Ok(schemes)
    }

    /// Validate a scheme file without loading it
    pub fn validate_scheme_file(path: &Path) -> PrunifyResult<()> {
        Self::load(path)?;
        Ok(())
    }
}
