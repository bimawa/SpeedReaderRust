use zed_extension_api as zed;

struct SpeedReaderExtension;

impl zed::Extension for SpeedReaderExtension {
    fn new() -> Self {
        Self
    }

    fn run_slash_command(
        &self,
        _command: zed::SlashCommand,
        _args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        // Get the worktree
        let worktree = worktree.ok_or("No worktree available. Open a file first.")?;

        // Try to find the speed-reader binary
        let binary_info = match worktree.which("speed-reader") {
            Some(path) => format!("✅ `speed-reader` found at: `{path}`\n\n"),
            None => "⚠️  `speed-reader` binary not found in PATH.\n   Install with: `cargo install speed-reader`\n\n".into(),
        };

        // Read the root path and list files
        let _root = worktree.root_path();
        let mut total_words = 0usize;
        let mut file_list = String::new();

        // For now, try to read a few common file patterns
        for candidate in &["README.md", "src/lib.rs", "Cargo.toml"] {
            match worktree.read_text_file(candidate) {
                Ok(content) => {
                    let words = content.split_whitespace().count();
                    total_words += words;
                    file_list.push_str(&format!("  - `{candidate}`: {words} words\n"));
                }
                Err(_) => continue,
            }
        }

        if file_list.is_empty() {
            file_list = "  (no files auto-detected — select text and copy to SpeedReader)\n".into();
        }

        // Calculate estimated reading time
        let est_secs = if total_words > 0 {
            (total_words as f64 / 300.0 * 60.0) as u64
        } else {
            0
        };

        let output = format!(
            "## SpeedReader\n\n\
             {binary_info}\
             Files:\n{file_list}\n\
             **Total**: ~{total_words} words (~{est_secs}s at 300 WPM)\n\n\
             **Usage**:\n\
             1. Select text in your buffer and copy it (Cmd+C)\n\
             2. Run `speed-reader` in your terminal\n\
             3. Press `Space` to pause — Zed will jump to the reading position\n\
             4. Press `Esc` to exit"
        );

        Ok(zed::SlashCommandOutput {
            text: output,
            sections: vec![],
        })
    }
}

zed::register_extension!(SpeedReaderExtension);
