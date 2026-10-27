use zed_extension_api::{self as zed, LanguageServerId, Result};

struct GpuiRsx;
impl zed::Extension for GpuiRsx {
    fn new() -> Self {
        Self
    }
    fn language_server_command(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let command = worktree.which("cargo-rsx").ok_or(
            "Install the GPUI RSX server with `cargo install cargo-rsx --version 0.8.3 --locked`, then restart Zed.",
        )?;
        Ok(zed::Command {
            command,
            args: vec!["lsp".into()],
            env: Default::default(),
        })
    }
}
zed::register_extension!(GpuiRsx);
