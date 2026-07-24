use zed_extension_api as zed;

struct SpeedReaderExtension;

impl zed::Extension for SpeedReaderExtension {
    fn new() -> Self {
        Self
    }

    fn run_slash_command(
        &self,
        _command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        let wt = worktree.ok_or("Open a project first")?;

        // Determine which file to read
        let file_path = if args.is_empty() {
            // Auto-detect: find the biggest text file in the project
            let candidates = ["README.md", "Cargo.toml", "package.json", "src/lib.rs", "src/main.rs"];
            let mut found = None;
            for c in &candidates {
                if wt.read_text_file(c).is_ok() {
                    found = Some(c.to_string());
                    break;
                }
            }
            found.ok_or("No auto-detected file. Provide a path: `/speed-reader src/main.rs`")?
        } else {
            args.into_iter().next().unwrap()
        };

        // Read the file content
        let content = wt.read_text_file(&file_path)
            .map_err(|e| format!("Can't read `{file_path}`: {e}"))?;

        // Get file stats
        let word_count = content.split_whitespace().count();
        let char_count = content.len();

        // Try to find and launch speed-reader binary
        let launch_msg = match wt.which("speed-reader") {
            Some(bin_path) => {
                let root = wt.root_path();
                let abs_path = format!("{}/{}", root.trim_end_matches('/'), file_path);
                // Launch in background via shell detach
                let cmd = format!("{} {} &", bin_path, abs_path);
                match zed::Command::new("sh").args(["-c", &cmd]).output() {
                    Ok(_) => format!("✅ SpeedReader launched on `{}`\n\n", file_path),
                    Err(e) => format!("⚠️  Launch failed: {e}\n\n"),
                }
            }
            None => {
                "⚠️  `speed-reader` not installed.\nInstall: `cargo install speed-reader`\n\n".to_string()
            }
        };

        let output = format!(
            "## SpeedReader\n\n\
             {launch_msg}\
             File: `{file_path}` — ~{word_count} words ({char_count} chars)\n\n\
             **Controls**: Space=pause+focus, Esc=exit, ←→=skip, ↑↓=speed\n\n\
             Use `/speed-reader <path>` to read a specific file."
        );

        Ok(zed::SlashCommandOutput {
            text: output,
            sections: vec![],
        })
    }
}

zed::register_extension!(SpeedReaderExtension);
