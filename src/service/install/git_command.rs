use anyhow::{bail, Context as _, Ok, Result};
use log::debug;
use std::process::Command;

pub trait GitCommand {
    fn clone_flutter_sdk_by_channel(&self, channel: &str, destination: &str) -> Result<()>;
    fn clone_flutter_sdk_by_version(&self, version: &str, destination: &str) -> Result<()>;
}

pub struct GitCommandImpl {}

impl GitCommandImpl {
    pub fn new() -> GitCommandImpl {
        GitCommandImpl {}
    }
}

impl GitCommand for GitCommandImpl {
    fn clone_flutter_sdk_by_channel(&self, channel: &str, destination: &str) -> Result<()> {
        const ERROR_MESSAGE: &str =
            "Failed to execute `git clone https://github.com/flutter/flutter.git`";
        let mut command = Command::new("git");
        let command = command
            .arg("clone")
            .args(["-c", "advice.detachedHead=false", "-b", channel])
            .arg("https://github.com/flutter/flutter.git")
            .arg(destination);
        debug!(
            "clone_flutter_sdk_by_channel(): command: program={:?}: args={:?}",
            command.get_program(),
            command.get_args()
        );
        let child = &mut command.spawn().context(ERROR_MESSAGE)?;
        let exit_status = &mut child.wait().context(ERROR_MESSAGE)?;
        if !exit_status.success() {
            bail!(
                "{ERROR_MESSAGE}: OS state code - {}",
                exit_status.code().unwrap()
            )
        }
        Ok(())
    }

    fn clone_flutter_sdk_by_version(&self, version: &str, destination: &str) -> Result<()> {
        const ERROR_TEMPLATE: &str = "Failed to set the snapshot to `{}`";
        self.clone_flutter_sdk_by_channel("stable", destination)?;
        let mut command = Command::new("git");
        let command = command
            .current_dir(destination)
            .arg("reset")
            .arg("--hard")
            .arg(version);
        debug!(
            "clone_flutter_sdk_by_version(): command: program={:?}: args={:?}",
            command.get_program(),
            command.get_args()
        );
        let child = &mut command
            .spawn()
            .with_context(|| format!("Failed to set the snapshot to `{version}`"))?;
        let exit_status = &mut child.wait().context(ERROR_TEMPLATE)?;
        if !exit_status.success() {
            bail!(
                "Failed to set the snapshot to `{version}`: OS state code - {code}",
                code = exit_status.code().unwrap()
            )
        }
        Ok(())
    }
}
