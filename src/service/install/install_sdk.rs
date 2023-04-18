use std::path::PathBuf;

use anyhow::{bail, Ok, Result};
use log::debug;

use super::git_command::GitCommand;

pub(crate) fn install_sdk(
    versions_directory: &str,
    target_version_or_channel: &str,
    _do_pre_cache: bool,
    git_command: &Box<dyn GitCommand>,
) -> Result<()> {
    let destination = PathBuf::from(format!("{versions_directory}/{target_version_or_channel}"));
    if destination.exists() {
        bail!("`{versions_directory}/{target_version_or_channel}` already exists")
    }

    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            debug!(
                "install_sdk(): create the parent directory: `{}`",
                parent.to_str().unwrap()
            );
            std::fs::create_dir_all(parent).ok();
        }
    }
    let clone_result = match target_version_or_channel {
        "stable" | "beta" | "master" => git_command
            .clone_flutter_sdk_by_channel(target_version_or_channel, destination.to_str().unwrap()),
        _ => git_command
            .clone_flutter_sdk_by_version(target_version_or_channel, destination.to_str().unwrap()),
    };
    if let Err(e) = clone_result {
        if destination.exists() {
            std::fs::remove_dir_all(&destination).ok();
        }
        return Err(e);
    }
    Ok(())
}
