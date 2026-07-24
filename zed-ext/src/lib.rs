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
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {

        Ok(zed::SlashCommandOutput {
            text: "SpeedReader launched. Press Space to pause and jump to editor.".into(),
            sections: vec![],
        })
    }
}

zed::register_extension!(SpeedReaderExtension);
