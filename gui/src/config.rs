use std::fs;
use std::path::PathBuf;

use speed_reader_core::config::ConfigModel;

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
}

pub struct ConfigPersistence;

impl ConfigPersistence {
    pub fn config_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("speed-reader")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<ConfigModel, ConfigError> {
        Self::load_from(&Self::config_path())
    }

    pub fn load_from(path: &std::path::Path) -> Result<ConfigModel, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                return ConfigError::IoError(format!("Config file not found: {}", path.display()));
            }
            ConfigError::IoError(format!("Failed to read config: {}", e))
        })?;

        let config: ConfigModel = serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    pub fn save(config: &ConfigModel) -> Result<(), ConfigError> {
        Self::save_to(config, &Self::config_path())
    }

    pub fn save_to(config: &ConfigModel, path: &std::path::Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::IoError(format!("Failed to create config dir: {}", e)))?;
        }

        let json = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::IoError(format!("Failed to serialize config: {}", e)))?;

        fs::write(path, json)
            .map_err(|e| ConfigError::IoError(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    pub fn apply_cli_overrides(
        config: &mut ConfigModel,
        wpm: Option<u32>,
        theme_mode: Option<speed_reader_core::config::ThemeMode>,
        font_size: Option<f32>,
    ) {
        if let Some(wpm) = wpm {
            config.wpm = wpm;
        }
        if let Some(theme_mode) = theme_mode {
            config.theme_mode = theme_mode;
        }
        if let Some(font_size) = font_size {
            config.font_size = font_size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speed_reader_core::config::ConfigModel;
    use speed_reader_core::config::ThemeMode;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_dir_returns_correct_path() {
        let dir = ConfigPersistence::config_dir();
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        assert_eq!(dir, PathBuf::from(home).join(".config").join("speed-reader"));
    }

    #[test]
    fn test_config_path_returns_correct_path() {
        let path = ConfigPersistence::config_path();
        let dir = ConfigPersistence::config_dir();
        assert_eq!(path, dir.join("config.json"));
    }

    #[test]
    fn test_load_returns_error_when_no_file_exists() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nonexistent").join("config.json");
        let result = ConfigPersistence::load_from(&path);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::IoError(msg) => assert!(msg.contains("not found")),
            _ => panic!("Expected IoError for missing file"),
        }
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.json");

        let mut config = ConfigModel::default();
        config.wpm = 500;
        config.font_size = 64.0;

        ConfigPersistence::save_to(&config, &path).unwrap();
        let loaded = ConfigPersistence::load_from(&path).unwrap();

        assert_eq!(loaded.wpm, 500);
        assert_eq!(loaded.font_size, 64.0);
    }

    #[test]
    fn test_apply_cli_overrides_wpm() {
        let mut config = ConfigModel::default();
        ConfigPersistence::apply_cli_overrides(&mut config, Some(600), Some(ThemeMode::Dark), Some(48.0));
        assert_eq!(config.wpm, 600);
    }

    #[test]
    fn test_apply_cli_overrides_theme() {
        let mut config = ConfigModel::default();
        ConfigPersistence::apply_cli_overrides(&mut config, Some(300), Some(ThemeMode::Light), Some(48.0));
        assert_eq!(config.theme_mode, ThemeMode::Light);
    }

    #[test]
    fn test_apply_cli_overrides_font_size() {
        let mut config = ConfigModel::default();
        ConfigPersistence::apply_cli_overrides(&mut config, Some(300), Some(ThemeMode::Dark), Some(72.0));
        assert_eq!(config.font_size, 72.0);
    }

    #[test]
    fn test_apply_cli_overrides_none_preserves_defaults() {
        let mut config = ConfigModel::default();
        config.wpm = 500;
        ConfigPersistence::apply_cli_overrides(&mut config, None, None, None);
        assert_eq!(config.wpm, 500);
    }

    #[test]
    fn test_config_error_variants_are_distinct() {
        let io_err = ConfigError::IoError("io".to_string());
        let parse_err = ConfigError::ParseError("parse".to_string());
        assert_ne!(io_err, parse_err);
    }

    #[test]
    fn test_save_creates_directory_if_missing() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("nested").join("dir").join("config.json");

        let config = ConfigModel::default();
        ConfigPersistence::save_to(&config, &path).unwrap();

        assert!(path.exists());
        let loaded = ConfigPersistence::load_from(&path).unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn test_load_returns_error_for_empty_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.json");
        fs::write(&path, "").unwrap();

        let result = ConfigPersistence::load_from(&path);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ParseError(_) => {}
            other => panic!("Expected ParseError for empty file, got {:?}", other),
        }
    }
}
