use std::fs;
use std::path::Path;

use crate::config::types::PrunifierConfig;
use crate::error::PrunifierError;
use crate::error::PrunifierResult;

/// Loads `.prunifier.yaml` from disk and deserializes it into `PrunifierConfig`.
///
/// If the path is `None` or the file does not exist, returns `PrunifierConfig::default()`
/// (not an error). Fields absent from the YAML are filled from the default.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load config from the given path.
    ///
    /// - `None` path or missing file → `PrunifierConfig::default()`
    /// - Valid YAML → deserialized config with defaults applied for absent fields
    /// - Invalid YAML → `PrunifierError::ConfigError`
    pub fn load(path: Option<&Path>) -> PrunifierResult<PrunifierConfig> {
        let path = match path {
            Some(p) if p.exists() => p,
            _ => return Ok(PrunifierConfig::default()),
        };

        let contents = fs::read_to_string(path)?;

        let mut config: PrunifierConfig = serde_yaml::from_str(&contents)
            .map_err(|e| PrunifierError::ConfigError(e.to_string()))?;

        // Merge defaults: fields not present in YAML (None) → use default value
        let defaults = PrunifierConfig::default();
        if config.scheme_dir.is_none() {
            config.scheme_dir = defaults.scheme_dir;
        }
        if config.verbose.is_none() {
            config.verbose = defaults.verbose;
        }
        if config.no_color.is_none() {
            config.no_color = defaults.no_color;
        }
        if config.strict.is_none() {
            config.strict = defaults.strict;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn write_temp_yaml(content: &str) -> PathBuf {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join("prunifier_test_config_loader");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join(format!("test_{}.yaml", counter));
        let mut file = std::fs::File::create(&path).expect("create temp file");
        write!(file, "{}", content).expect("write yaml content");
        file.sync_all().expect("sync temp file");
        drop(file);
        path
    }

    #[test]
    fn test_load_yaml_config() {
        let path = write_temp_yaml(
            r#"scheme_dir: ./my-schemes
verbose: true
no_color: false
strict: true"#,
        );

        let config = ConfigLoader::load(Some(&path)).expect("load should succeed");

        assert_eq!(config.scheme_dir, Some(PathBuf::from("./my-schemes")));
        assert_eq!(config.verbose, Some(true));
        assert_eq!(config.no_color, Some(false));
        assert_eq!(config.strict, Some(true));
    }

    #[test]
    fn test_missing_config_uses_defaults() {
        let non_existent = Path::new("/tmp/does_not_exist_prunifier_config.yaml");
        let config = ConfigLoader::load(Some(non_existent))
            .expect("missing file should not error, returns default");

        let defaults = PrunifierConfig::default();
        assert_eq!(config.scheme_dir, defaults.scheme_dir);
        assert_eq!(config.verbose, defaults.verbose);
        assert_eq!(config.no_color, defaults.no_color);
        assert_eq!(config.strict, defaults.strict);
    }

    #[test]
    fn test_invalid_yaml_errors() {
        let path = write_temp_yaml("\tinvalid: yaml");

        let result = ConfigLoader::load(Some(&path));
        assert!(result.is_err(), "invalid YAML should produce an error");

        match result {
            Err(PrunifierError::ConfigError(msg)) => {
                assert!(!msg.is_empty(), "error message should not be empty");
            }
            other => panic!("expected ConfigError, got {:?}", other),
        }
    }

    #[test]
    fn test_partial_config_merges_defaults() {
        let path = write_temp_yaml(r#"verbose: true"#);

        let config = ConfigLoader::load(Some(&path)).expect("load should succeed");

        // Fields set in YAML should be present
        assert_eq!(config.verbose, Some(true));

        // Fields absent from YAML should be filled from defaults
        assert_eq!(config.no_color, Some(false));
        assert_eq!(config.strict, Some(false));
        // scheme_dir default is None
        assert_eq!(config.scheme_dir, None);
    }
}
