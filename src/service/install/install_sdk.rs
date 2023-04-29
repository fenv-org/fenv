use super::flutter_command::FlutterCommand;
use crate::{
    context::FenvContext, external::git_command::GitCommand, model::flutter_sdk::FlutterSdk,
    service::latest::latest_service::FenvLatestService, util::path_like::PathLike,
};
use anyhow::{anyhow, bail, Context, Ok, Result};
use log::{debug, info};

pub struct InstallSdkArguments<'a> {
    pub target_version_or_channel_prefix: &'a str,
    pub do_precache: bool,
    pub git_command: &'a Box<dyn GitCommand>,
    pub flutter_command: &'a Box<dyn FlutterCommand>,
}

pub fn install_sdk<'a>(context: &impl FenvContext, args: &InstallSdkArguments) -> Result<()> {
    let local_latest_sdk =
        FenvLatestService::latest(context, args.target_version_or_channel_prefix);
    if let Result::Ok(sdk) = local_latest_sdk {
        bail!("`{}` is already installed", sdk.display_name())
    }

    let versions_directory = context.fenv_versions();
    let remote_latest_sdk =
        FenvLatestService::latest_remote(context, args.target_version_or_channel_prefix)?;
    let target_version_or_channel = &remote_latest_sdk.display_name()[..];

    macro_rules! clear_destination_and_early_return_if_err {
        ($result: ident) => {
            if let Err(_e) = $result {
                remove_installing_marker(&versions_directory, target_version_or_channel).ok();
                remove_sdk_root(&versions_directory, target_version_or_channel).ok();
                return Err(_e);
            }
        };
    }

    if exists_installing_marker(&versions_directory, target_version_or_channel) {
        info!(
            "install_sdk(): a previous trial to install `{target_version_or_channel}` \
            ended unsuccessfully: remove the `{destination}`",
            destination = sdk_root_of(&versions_directory, target_version_or_channel).to_string()
        );
        remove_sdk_root(&versions_directory, target_version_or_channel)?;
        remove_installing_marker(&versions_directory, target_version_or_channel)?;
    }

    let marker = installing_marker_of(&versions_directory, target_version_or_channel);
    let destination = sdk_root_of(&versions_directory, target_version_or_channel);
    let flutter_sdk_root_dir = &destination.to_string();

    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            debug!("install_sdk(): create the parent directory: `{parent}`");
            std::fs::create_dir_all(parent).ok();
        }
    }

    // create an empty file to mark as installing the specifying SDK version.
    debug!("install_sdk(): create an installing marker file parent directory: `{marker}`");
    create_installing_marker(&versions_directory, target_version_or_channel)?;

    // download the dedicated sdk.
    let clone_result = match target_version_or_channel {
        "stable" | "beta" | "dev" | "master" => args
            .git_command
            .clone_flutter_sdk_by_channel(target_version_or_channel, flutter_sdk_root_dir),
        _ => args
            .git_command
            .clone_flutter_sdk_by_version(target_version_or_channel, flutter_sdk_root_dir),
    };
    clear_destination_and_early_return_if_err!(clone_result);

    // install the download sdk.
    let doctor_result = args.flutter_command.doctor(flutter_sdk_root_dir);
    clear_destination_and_early_return_if_err!(doctor_result);

    // execute `flutter precache` to install cli tools.
    if args.do_precache {
        let precache_result = args.flutter_command.precache(flutter_sdk_root_dir);
        clear_destination_and_early_return_if_err!(precache_result);
    }

    if let Err(e) = remove_installing_marker(&versions_directory, target_version_or_channel) {
        info!("install_sdk(): failed to remove the `{marker}`: `{e}`")
    }
    Ok(())
}

fn installing_marker_of(
    versions_directory: &PathLike,
    target_version_or_channel: &str,
) -> PathLike {
    versions_directory.join(format!(".install_{target_version_or_channel}"))
}

fn sdk_root_of(versions_directory: &PathLike, target_version_or_channel: &str) -> PathLike {
    versions_directory.join(target_version_or_channel)
}

pub fn exists_installing_marker(
    versions_directory: &PathLike,
    target_version_or_channel: &str,
) -> bool {
    installing_marker_of(versions_directory, target_version_or_channel).exists()
}

fn create_installing_marker(
    versions_directory: &PathLike,
    target_version_or_channel: &str,
) -> Result<()> {
    let marker = installing_marker_of(versions_directory, target_version_or_channel);
    if !marker.exists() {
        marker
            .create_file()
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to create an installing marker: `{marker}`"))?;
    }
    Ok(())
}

fn remove_installing_marker(
    versions_directory: &PathLike,
    target_version_or_channel: &str,
) -> Result<()> {
    let marker = installing_marker_of(versions_directory, target_version_or_channel);
    marker
        .remove_file()
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("Failed to remove an installing marker: `{marker}`"))
}

fn remove_sdk_root(versions_directory: &PathLike, target_version_or_channel: &str) -> Result<()> {
    let sdk_root = sdk_root_of(versions_directory, target_version_or_channel);
    sdk_root
        .remove_dir_all()
        .map_err(|e| anyhow!(e))
        .with_context(|| format!("Failed to remove a sdk root: `{sdk_root}`"))
}
