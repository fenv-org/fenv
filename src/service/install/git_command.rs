use anyhow::{Context as _, Ok, Result};

use std::process::Command;

use crate::{spawn_and_capture, spawn_and_wait};

pub trait GitCommand {
    fn clone_flutter_sdk_by_channel(&self, channel: &str, destination: &str) -> Result<()>;
    fn clone_flutter_sdk_by_version(&self, version: &str, destination: &str) -> Result<()>;
    fn list_remote_sdks_by_tags(&self) -> Result<String>;
    fn list_remote_sdks_by_branches(&self) -> Result<String>;
    fn hard_reset_to_refs(&self, working_dir: &str, refs: &str) -> Result<()>;
}

pub struct GitCommandImpl {}

impl GitCommandImpl {
    pub fn new() -> GitCommandImpl {
        GitCommandImpl {}
    }
}

impl GitCommand for GitCommandImpl {
    fn clone_flutter_sdk_by_channel(&self, channel: &str, destination: &str) -> Result<()> {
        let mut command = Command::new("git");
        spawn_and_wait!(
            command
                .arg("clone")
                .args(["-c", "advice.detachedHead=false", "-b", channel])
                .arg("https://github.com/flutter/flutter.git")
                .arg(destination),
            "clone_flutter_sdk_by_channel",
            "Failed to execute `git clone https://github.com/flutter/flutter.git`"
        );
        Ok(())
    }

    fn clone_flutter_sdk_by_version(&self, version: &str, destination: &str) -> Result<()> {
        self.clone_flutter_sdk_by_channel("stable", destination)?;
        self.hard_reset_to_refs(destination, version)
    }

    fn hard_reset_to_refs(&self, working_dir: &str, refs: &str) -> Result<()> {
        let mut command = Command::new("git");
        spawn_and_wait!(
            command
                .current_dir(working_dir)
                .arg("reset")
                .arg("--hard")
                .arg(refs),
            "hard_reset_to_refs",
            "Failed to set the snapshot to `{refs}`"
        );
        Ok(())
    }

    fn list_remote_sdks_by_tags(&self) -> Result<String> {
        let mut command = Command::new("git");
        let git_output = spawn_and_capture!(
            command
                .arg("ls-remote")
                .args(["--tags", "--sort=version:refname"])
                .arg("https://github.com/flutter/flutter.git")
                .arg("**/*.*.*"),
            "list_remote_sdks_by_tags",
            "Failed to fetch remote tags from `https://github.com/flutter/flutter.git`"
        );
        Ok(git_output)
    }

    fn list_remote_sdks_by_branches(&self) -> Result<String> {
        let mut command = Command::new("git");
        let git_output = spawn_and_capture!(
            command
                .arg("ls-remote")
                .args(["--heads", "--refs"])
                .arg("https://github.com/flutter/flutter.git")
                .args(["stable", "dev", "beta", "master"]),
            "list_remote_sdks_by_branches",
            "Failed to fetch remote branches from `https://github.com/flutter/flutter.git`"
        );
        Ok(git_output)
    }
}
