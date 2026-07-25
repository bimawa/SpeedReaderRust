use serde::{Deserialize, Serialize};

/// Модель конфигурации SpeedReader.
/// Requirement: 4.1, 4.2, 4.3, Design: ConfigModel section

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeColors {
    pub bg: String,
    pub text: String,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub light: ThemeColors,
    pub dark: ThemeColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            light: ThemeColors {
                bg: "#FFFFFF".into(),
                text: "#1A1A1A".into(),
                accent: "#E53935".into(),
            },
            dark: ThemeColors {
                bg: "#1A1A1A".into(),
                text: "#F5F5F5".into(),
                accent: "#FF5252".into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigModel {
    pub wpm: u32,
    pub theme_mode: ThemeMode,
    pub font_size: f32,
    pub skip_amount: u32,
    pub speed_step: u32,
    pub theme: Theme,
    pub editor_cmd: String,
}

impl Default for ConfigModel {
    fn default() -> Self {
        Self {
            wpm: 300,
            theme_mode: ThemeMode::Dark,
            font_size: 48.0,
            skip_amount: 5,
            speed_step: 10,
            theme: Theme::default(),
            editor_cmd: "zed".into(),
        }
    }
}

impl ConfigModel {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.wpm < 50 || self.wpm > 1000 {
            errors.push(format!("WPM {} out of range [50, 1000]", self.wpm));
        }
        if self.font_size < 12.0 || self.font_size > 200.0 {
            errors.push(format!("font_size {} out of range [12, 200]", self.font_size));
        }
        if self.skip_amount == 0 {
            errors.push("skip_amount must be > 0".into());
        }
        if self.speed_step == 0 {
            errors.push("speed_step must be > 0".into());
        }
        if self.editor_cmd.is_empty() {
            errors.push("editor_cmd must not be empty".into());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }


    pub fn current_colors(&self) -> &ThemeColors {
        match self.theme_mode {
            ThemeMode::Light => &self.theme.light,
            ThemeMode::Dark => &self.theme.dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_valid() {
        let config = ConfigModel::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_wpm_out_of_range() {
        let mut config = ConfigModel::default();
        config.wpm = 10;
        assert!(config.validate().is_err());
        config.wpm = 2000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_font_size_out_of_range() {
        let mut config = ConfigModel::default();
        config.font_size = 8.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_current_colors_dark() {
        let config = ConfigModel::default();
        let colors = config.current_colors();
        assert_eq!(colors.bg, "#1A1A1A");
    }

    #[test]
    fn test_current_colors_light() {
        let mut config = ConfigModel::default();
        config.theme_mode = ThemeMode::Light;
        let colors = config.current_colors();
        assert_eq!(colors.bg, "#FFFFFF");
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = ConfigModel::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConfigModel = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
