use anyhow::{bail, Context as _, Ok, Result};
use log::debug;
use std::{collections::HashSet, process::Command};

use crate::model::remote_flutter_sdk::{GitRefsKind, RemoteFlutterSdk};

pub(crate) trait GitCommand {
    fn clone_flutter_sdk_by_channel(&self, channel: &str, destination: &str) -> Result<()>;
    fn clone_flutter_sdk_by_version(&self, version: &str, destination: &str) -> Result<()>;
    fn list_remote_sdks_by_tags(&self) -> Result<Vec<RemoteFlutterSdk>>;
    fn list_remote_sdks_by_branches(&self) -> Result<Vec<RemoteFlutterSdk>>;
    fn hard_reset_to_ref(&self, refs: &str, working_directory: &str) -> Result<()>;
}

pub(crate) struct GitCommandImpl {}

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
        self.clone_flutter_sdk_by_channel("stable", destination)?;
        self.hard_reset_to_ref(version, destination)
    }

    fn hard_reset_to_ref(&self, refs: &str, working_directory: &str) -> Result<()> {
        const ERROR_TEMPLATE: &str = "Failed to set the snapshot to `{}`";
        let mut command = Command::new("git");
        let command = command
            .current_dir(working_directory)
            .arg("reset")
            .arg("--hard")
            .arg(refs);
        debug!(
            "hard_reset_to_ref(): command: program={:?}: args={:?}",
            command.get_program(),
            command.get_args()
        );
        let child = &mut command
            .spawn()
            .with_context(|| format!("Failed to set the snapshot to `{refs}`"))?;
        let exit_status = &mut child.wait().context(ERROR_TEMPLATE)?;
        if !exit_status.success() {
            bail!(
                "Failed to set the snapshot to `{refs}`: OS state code - {code}",
                code = exit_status.code().unwrap()
            )
        }
        Ok(())
    }

    fn list_remote_sdks_by_tags(&self) -> Result<Vec<RemoteFlutterSdk>> {
        const ERROR_MESSAGE: &str =
            "Failed to fetch remote tags from `https://github.com/flutter/flutter.git`";
        let mut command = Command::new("git");
        let command = command
            .arg("ls-remote")
            .args(["--tags", "--sort=version:refname"])
            .arg("https://github.com/flutter/flutter.git")
            .arg("**/*.*.*");
        debug!(
            "list_remote_sdks_by_tags(): command: program={:?}: args={:?}",
            command.get_program(),
            command.get_args()
        );

        let output = command.output().context(ERROR_MESSAGE)?;
        if !output.status.success() {
            debug!(
                "list_remote_sdks_by_tags(): stderr:\n{}",
                String::from_utf8(output.stderr)?
            );
            bail!("{ERROR_MESSAGE}: {}", output.status);
        }
        let git_output = String::from_utf8(output.stdout)?;
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

    fn list_remote_sdks_by_branches(&self) -> Result<Vec<RemoteFlutterSdk>> {
        const ERROR_MESSAGE: &str =
            "Failed to fetch remote branches from `https://github.com/flutter/flutter.git`";

        let mut command = Command::new("git");
        let command = command
            .arg("ls-remote")
            .args(["--heads", "--refs"])
            .arg("https://github.com/flutter/flutter.git")
            .args(["stable", "beta", "master"]);
        debug!(
            "list_remote_sdks_by_branches(): command: program={:?}: args={:?}",
            command.get_program(),
            command.get_args()
        );

        let output = command.output().context(ERROR_MESSAGE)?;
        if !output.status.success() {
            debug!(
                "list_remote_sdks_by_branches(): stderr:\n{}",
                String::from_utf8(output.stderr)?
            );
            bail!("{ERROR_MESSAGE}: {}", output.status);
        }
        let git_output = String::from_utf8(output.stdout)?;
        debug!("list_remote_sdks_by_branches(): stdout:\n{git_output}");

        let mut lines = git_output.split("\n");
        let git_refs = lines
            .by_ref()
            .map(|line| RemoteFlutterSdk::parse(line))
            .flatten()
            .collect::<Vec<RemoteFlutterSdk>>();
        Ok(git_refs)
    }
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
