use super::model::{
    flutter_sdk::FlutterSdk,
    remote_flutter_sdk::{GitRefsKind, RemoteFlutterSdk},
};
use crate::{context::FenvContext, external::git_command::GitCommand, util::path_like::PathLike};
use log::debug;
use std::collections::HashSet;

pub struct RemoteSdkRepository;

pub const REMOTE_SDK_REPOSITORY: RemoteSdkRepository = RemoteSdkRepository;

impl RemoteSdkRepository {
    pub fn fetch_available_sdk_list(
        &self,
        git_command: &impl GitCommand,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
        let mut sdks = list_remote_sdks_by_tags(git_command)?;
        sdks.extend(list_remote_sdks_by_branches(git_command)?);
        Ok(sdks)
    }

    pub fn install_sdk(
        &self,
        context: &impl FenvContext,
        git_command: &impl GitCommand,
        sdk: &RemoteFlutterSdk,
    ) -> anyhow::Result<PathLike> {
        match &sdk.kind {
            GitRefsKind::Tag(_) => {
                let destination = context.fenv_sdk_root(&sdk.display_name());
                git_command
                    .clone_flutter_sdk_by_version(&sdk.display_name(), &destination.to_string())?;
                anyhow::Ok(destination)
            }
            GitRefsKind::Head(channel) => {
                let destination = context.fenv_sdk_root(channel);
                git_command.clone_flutter_sdk_by_channel(channel, &destination.to_string())?;
                anyhow::Ok(destination)
            }
        }
    }
}

fn list_remote_sdks_by_tags(
    git_command: &impl GitCommand,
) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
    let git_output = git_command.list_remote_sdks_by_tags()?;
    debug!("list_remote_sdks_by_tags(): stdout:\n{git_output}");

    let mut lines = git_output.split("\n");
    // Holds kind keys for eliminating duplications
    let mut registered_kind_keys: HashSet<String> = HashSet::new();
    let mut git_refs = lines
        .by_ref()
        .map(|line| RemoteFlutterSdk::parse(line))
        .flatten()
        // Remove duplications
        .filter(|sdk| {
            let key = sdk.kind.key();
            if registered_kind_keys.contains(&key) {
                false
            } else {
                registered_kind_keys.insert(key);
                true
            }
        })
        .collect::<Vec<RemoteFlutterSdk>>();
    git_refs.sort_by(|a, b| a.kind.cmp(&b.kind));
    Ok(git_refs)
}

fn list_remote_sdks_by_branches(
    git_command: &impl GitCommand,
) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
    let git_output = git_command.list_remote_sdks_by_branches()?;
    debug!("list_remote_sdks_by_branches(): stdout:\n{git_output}");

    let mut lines = git_output.split("\n");
    let git_refs = lines
        .by_ref()
        .map(|line| RemoteFlutterSdk::parse(line))
        .flatten()
        .collect::<Vec<RemoteFlutterSdk>>();
    Ok(git_refs)
}

impl GitRefsKind {
    /// Extracts a key string from `GitRefsKind`.
    fn key(&self) -> String {
        match self {
            GitRefsKind::Tag(version) => format!(
                "{major}.{minor}.{patch}.{hotfix}",
                major = version.major,
                minor = version.minor,
                patch = version.patch,
                hotfix = version.hotfix,
            ),
            GitRefsKind::Head(branch) => String::from(branch),
        }
    }
}
