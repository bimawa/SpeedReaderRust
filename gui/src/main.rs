use clap::Parser;
use speed_reader_core::config::ConfigModel;

#[derive(Parser, Debug)]
#[command(name = "speed-reader", about = "RSVP speed reader overlay for Zed")]
pub struct Cli {
    pub file_path: Option<String>,

    #[arg(long, default_value_t = 300)]
    pub wpm: u32,

    #[arg(long, value_enum, default_value_t = ThemeArg::Dark)]
    pub theme: ThemeArg,

    #[arg(long, default_value_t = 48.0)]
    pub font_size: f32,
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
        config.wpm = self.wpm;
        config.theme_mode = self.theme.clone().into();
        config.font_size = self.font_size;
        config
    }
}

pub fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let config = cli.to_config();
    config.validate().map_err(|e| e.join(", "))?;

    let overlay = overlay::OverlayWindow::new(cli.file_path, config);
    overlay.run()
}

mod overlay;
mod renderer;
mod input;

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
        assert_eq!(args.wpm, 500);
    }

    #[test]
    fn cli_wpm_default() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        assert_eq!(args.wpm, 300);
    }

    #[test]
    fn cli_theme_dark() {
        let args = Cli::try_parse_from(["speed-reader", "--theme", "dark"]).unwrap();
        assert_eq!(args.theme, ThemeArg::Dark);
    }

    #[test]
    fn cli_theme_light() {
        let args = Cli::try_parse_from(["speed-reader", "--theme", "light"]).unwrap();
        assert_eq!(args.theme, ThemeArg::Light);
    }

    #[test]
    fn cli_theme_default() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        assert_eq!(args.theme, ThemeArg::Dark);
    }

    #[test]
    fn cli_font_size() {
        let args = Cli::try_parse_from(["speed-reader", "--font-size", "64.0"]).unwrap();
        assert_eq!(args.font_size, 64.0);
    }

    #[test]
    fn cli_font_size_default() {
        let args = Cli::try_parse_from(["speed-reader"]).unwrap();
        assert_eq!(args.font_size, 48.0);
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
        assert_eq!(args.wpm, 400);
        assert_eq!(args.theme, ThemeArg::Light);
        assert_eq!(args.font_size, 72.0);
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
