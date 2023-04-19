use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Ok, Result};
use log::{debug, info};

use super::{flutter_command::FlutterCommand, git_command::GitCommand};

pub fn install_sdk(
    versions_directory: &str,
    target_version_or_channel: &str,
    do_precache: bool,
    git_command: &Box<dyn GitCommand>,
    flutter_command: &Box<dyn FlutterCommand>,
) -> Result<()> {
    macro_rules! clear_destination_and_early_return_if_err {
        ($result: ident) => {
            if let Err(_e) = $result {
                remove_installing_marker(versions_directory, target_version_or_channel).ok();
                remove_sdk_root(versions_directory, target_version_or_channel).ok();
                return Err(_e);
            }
        };
    }

    if is_installed(versions_directory, target_version_or_channel) {
        bail!("`{}` is already installed", target_version_or_channel)
    }

    if exists_installing_marker(versions_directory, target_version_or_channel) {
        info!(
            "install_sdk(): a previous trial to install `{target_version_or_channel}` \
            ended unsuccessfully: remove the `{destination}`",
            destination = sdk_root_of(versions_directory, target_version_or_channel)
                .to_str()
                .unwrap()
        );
        remove_sdk_root(versions_directory, target_version_or_channel)?;
        remove_installing_marker(versions_directory, target_version_or_channel)?;
    }

    let marker = installing_marker_of(versions_directory, target_version_or_channel);
    let destination = sdk_root_of(versions_directory, target_version_or_channel);

    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            debug!(
                "install_sdk(): create the parent directory: `{}`",
                parent.to_str().unwrap()
            );
            std::fs::create_dir_all(parent).ok();
        }
    }

    // create an empty file to mark as installing the specifying SDK version.
    debug!(
        "install_sdk(): create an installing marker file parent directory: `{}`",
        &marker.to_str().unwrap()
    );
    create_installing_marker(versions_directory, target_version_or_channel)?;

    // download the dedicated sdk.
    let clone_result = match target_version_or_channel {
        "stable" | "beta" | "dev" | "master" => git_command
            .clone_flutter_sdk_by_channel(target_version_or_channel, destination.to_str().unwrap()),
        _ => git_command
            .clone_flutter_sdk_by_version(target_version_or_channel, destination.to_str().unwrap()),
    };
    clear_destination_and_early_return_if_err!(clone_result);

    // install the download sdk.
    let flutter_bin_dir = destination.to_str().unwrap();
    let doctor_result = flutter_command.doctor(flutter_bin_dir);
    clear_destination_and_early_return_if_err!(doctor_result);

    // execute `flutter precache` to install cli tools.
    if do_precache {
        let precache_result = flutter_command.precache(flutter_bin_dir);
        clear_destination_and_early_return_if_err!(precache_result);
    }

    remove_installing_marker(versions_directory, target_version_or_channel).ok();
    Ok(())
}

fn installing_marker_of(versions_directory: &str, target_version_or_channel: &str) -> PathBuf {
    PathBuf::from(format!(
        "{versions_directory}/.install_{target_version_or_channel}"
    ))
}

fn sdk_root_of(versions_directory: &str, target_version_or_channel: &str) -> PathBuf {
    PathBuf::from(format!("{versions_directory}/{target_version_or_channel}"))
}

fn exists_installing_marker(versions_directory: &str, target_version_or_channel: &str) -> bool {
    installing_marker_of(versions_directory, target_version_or_channel).exists()
}

fn exists_sdk_root(versions_directory: &str, target_version_or_channel: &str) -> bool {
    sdk_root_of(versions_directory, target_version_or_channel).exists()
}

fn is_installed(versions_directory: &str, target_version_or_channel: &str) -> bool {
    !exists_installing_marker(versions_directory, target_version_or_channel)
        && exists_sdk_root(versions_directory, target_version_or_channel)
}

fn create_installing_marker(
    versions_directory: &str,
    target_version_or_channel: &str,
) -> Result<()> {
    let marker = installing_marker_of(versions_directory, target_version_or_channel);
    if !marker.exists() {
        std::fs::File::create(&marker)
            .map_err(|e| anyhow!(e))
            .with_context(|| {
                format!(
                    "Failed to create an installing marker: `{}`",
                    marker.to_str().unwrap()
                )
            })?;
    }
    Ok(())
}

fn remove_installing_marker(
    versions_directory: &str,
    target_version_or_channel: &str,
) -> Result<()> {
    let marker = installing_marker_of(versions_directory, target_version_or_channel);
    if marker.exists() {
        std::fs::remove_file(&marker)
            .map_err(|e| anyhow!(e))
            .with_context(|| {
                format!(
                    "Failed to remove an installing marker: `{}`",
                    marker.to_str().unwrap()
                )
            })?
    }
    Ok(())
}

fn remove_sdk_root(versions_directory: &str, target_version_or_channel: &str) -> Result<()> {
    let sdk_root = sdk_root_of(versions_directory, target_version_or_channel);
    if sdk_root.exists() {
        std::fs::remove_dir_all(&sdk_root)
            .map_err(|e| anyhow!(e))
            .with_context(|| {
                format!(
                    "Failed to remove a sdk root: `{}`",
                    sdk_root.to_str().unwrap()
                )
            })?;
    }
    Ok(())
}
