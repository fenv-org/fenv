use std::path::PathBuf;

use anyhow::{bail, Ok, Result};
use log::debug;

use super::{flutter_command::FlutterCommand, git_command::GitCommand};

pub(crate) fn install_sdk(
    versions_directory: &str,
    target_version_or_channel: &str,
    do_precache: bool,
    git_command: &Box<dyn GitCommand>,
    flutter_command: &Box<dyn FlutterCommand>,
) -> Result<()> {
    let destination = PathBuf::from(format!("{versions_directory}/{target_version_or_channel}"));
    if destination.exists() {
        bail!("`{versions_directory}/{target_version_or_channel}` already exists")
    }

    /// Only if the given `result` is an `Err`,
    /// removes the `destination` and its every children and
    /// returns immediately.
    macro_rules! clear_destination_and_early_return_if_err {
        ($result: ident) => {
            if let Err(_e) = $result {
                if destination.exists() {
                    std::fs::remove_dir_all(&destination).ok();
                }
                return Err(_e);
            }
        };
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
    clear_destination_and_early_return_if_err!(clone_result);

    let flutter_bin_dir = destination.to_str().unwrap();
    let doctor_result = flutter_command.doctor(flutter_bin_dir);
    clear_destination_and_early_return_if_err!(doctor_result);

    if do_precache {
        let precache_result = flutter_command.precache(flutter_bin_dir);
        clear_destination_and_early_return_if_err!(precache_result);
    }
    Ok(())
}
