use anyhow::{Context as _, Ok, Result};
use log::debug;
use std::{collections::HashSet, process::Command};

use crate::{
    model::remote_flutter_sdk::{GitRefsKind, RemoteFlutterSdk},
    spawn_and_capture, spawn_and_wait,
};

pub(crate) trait GitCommand {
    fn clone_flutter_sdk_by_channel(&self, channel: &str, destination: &str) -> Result<()>;
    fn clone_flutter_sdk_by_version(&self, version: &str, destination: &str) -> Result<()>;
    fn list_remote_sdks_by_tags(&self) -> Result<Vec<RemoteFlutterSdk>>;
    fn list_remote_sdks_by_branches(&self) -> Result<Vec<RemoteFlutterSdk>>;
    fn hard_reset_to_refs(&self, working_dir: &str, refs: &str) -> Result<()>;
}

pub(crate) struct GitCommandImpl {}

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

    fn list_remote_sdks_by_tags(&self) -> Result<Vec<RemoteFlutterSdk>> {
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
