use std::io::Read;

use clap::Parser;
use speed_reader_core::config::ConfigModel;
use speed_reader_core::tokenizer::tokenize;

#[derive(Parser, Debug)]
#[command(name = "speed-reader", version, about = "RSVP speed reader overlay")]
pub struct Cli {
    pub file_path: Option<String>,

    #[arg(long)]
    pub wpm: Option<u32>,

    #[arg(long, value_enum)]
    pub theme: Option<ThemeArg>,

    #[arg(long)]
    pub font_size: Option<f32>,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum ThemeArg {
    Light,
    Dark,
}

impl From<ThemeArg> for speed_reader_core::config::ThemeMode {
    fn from(arg: ThemeArg) -> Self {
        match arg {
            ThemeArg::Light => speed_reader_core::config::ThemeMode::Light,
            ThemeArg::Dark => speed_reader_core::config::ThemeMode::Dark,
        }
    }
}

impl Cli {
    pub fn to_config(&self) -> ConfigModel {
        let mut config = ConfigModel::default();
        if let Some(wpm) = self.wpm {
            config.wpm = wpm;
        }
        if let Some(theme) = &self.theme {
            config.theme_mode = theme.clone().into();
        }
        if let Some(font_size) = self.font_size {
            config.font_size = font_size;
        }
        config
    }
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let mut config = config::ConfigPersistence::load().unwrap_or_else(|_| cli.to_config());
    config::ConfigPersistence::apply_cli_overrides(
        &mut config,
        cli.wpm,
        cli.theme.as_ref().map(|t| t.clone().into()),
        cli.font_size,
    );
    config.validate().map_err(|e| e.join(", "))?;

    let text = match &cli.file_path {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{path}': {e}"))?,
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("Failed to read stdin: {e}"))?;
            buf
        }
    };

    let tokens =
        tokenize(&text).map_err(|e| format!("Failed to tokenize text: {e:?}"))?;

    if tokens.is_empty() {
        return Err("No tokens to display".into());
    }

    let overlay = overlay::OverlayWindow::new(cli.file_path, config, text, tokens);
    overlay.run()
}

mod overlay;
mod renderer;
mod input;
mod config;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parses_file_path() {
        let args = Cli::try_parse_from(["speed-reader", "test.txt"]).unwrap();
        assert_eq!(args.file_path.as_deref(), Some("test.txt"));
    }

    #[test]
    fn cli_no_file_path() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        assert!(args.file_path.is_none());
    }

    #[test]
    fn cli_wpm_override() {
        let args = Cli::try_parse_from(["speed-reader", "--wpm", "500", "file.txt"]).unwrap();
        assert_eq!(args.wpm, Some(500));
    }

    #[test]
    fn cli_wpm_default() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        assert!(args.wpm.is_none());
    }

    #[test]
    fn cli_theme_dark() {
        let args = Cli::try_parse_from(["speed-reader", "--theme", "dark"]).unwrap();
        assert_eq!(args.theme, Some(ThemeArg::Dark));
    }

    #[test]
    fn cli_theme_light() {
        let args = Cli::try_parse_from(["speed-reader", "--theme", "light"]).unwrap();
        assert_eq!(args.theme, Some(ThemeArg::Light));
    }

    #[test]
    fn cli_theme_default() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        assert!(args.theme.is_none());
    }

    #[test]
    fn cli_font_size() {
        let args = Cli::try_parse_from(["speed-reader", "--font-size", "64.0"]).unwrap();
        assert_eq!(args.font_size, Some(64.0));
    }

    #[test]
    fn cli_font_size_default() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        assert!(args.font_size.is_none());
    }

    #[test]
    fn cli_combined_args() {
        let args = Cli::try_parse_from([
            "speed-reader",
            "--wpm",
            "400",
            "--theme",
            "light",
            "--font-size",
            "72.0",
            "chapter.txt",
        ])
        .unwrap();
        assert_eq!(args.file_path.as_deref(), Some("chapter.txt"));
        assert_eq!(args.wpm, Some(400));
        assert_eq!(args.theme, Some(ThemeArg::Light));
        assert_eq!(args.font_size, Some(72.0));
    }

    #[test]
    fn cli_to_config_applies_overrides() {
        let args = Cli::try_parse_from([
            "speed-reader",
            "--wpm",
            "600",
            "--theme",
            "light",
            "--font-size",
            "56.0",
        ])
        .unwrap();
        let config = args.to_config();
        assert_eq!(config.wpm, 600);
        assert_eq!(
            config.theme_mode,
            speed_reader_core::config::ThemeMode::Light
        );
        assert_eq!(config.font_size, 56.0);
    }

    #[test]
    fn cli_to_config_defaults_match_core() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        let config = args.to_config();
        let default_config = ConfigModel::default();
        assert_eq!(config.wpm, default_config.wpm);
        assert_eq!(config.theme_mode, default_config.theme_mode);
        assert_eq!(config.font_size, default_config.font_size);
    }

    #[test]
    fn cli_help_contains_expected_flags() {
        let mut cmd = Cli::command();
        let help = render_help(&mut cmd);
        assert!(help.contains("--wpm"), "help should mention --wpm");
        assert!(help.contains("--theme"), "help should mention --theme");
        assert!(help.contains("--font-size"), "help should mention --font-size");
    }

    fn render_help(cmd: &mut clap::Command) -> String {
        let mut buf = Vec::new();
        cmd.write_help(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
