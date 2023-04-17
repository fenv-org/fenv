use crate::config::Config;
use crate::service::service::Service;
use crate::{
    args, model::remote_flutter_sdk::GitRefsKind, model::remote_flutter_sdk::RemoteFlutterSdk,
};
use anyhow::{bail, Context as _, Ok, Result};
use log::debug;

use std::collections::HashSet;
use std::process::Command;

pub struct FenvInstallService {
    pub args: args::FenvInstallArgs,
}

impl FenvInstallService {
    pub fn from(args: args::FenvInstallArgs) -> FenvInstallService {
        return FenvInstallService { args };
    }

    pub fn list_remote_sdks() -> Result<Vec<RemoteFlutterSdk>> {
        let mut sdks = list_remote_sdks_by_tags()?;
        sdks.extend(list_remote_sdks_by_branches()?);
        Ok(sdks)
    }
}

impl Service for FenvInstallService {
    fn execute(&self, _: &Config) -> Result<()> {
        if self.args.list {
            let sdks = FenvInstallService::list_remote_sdks()?;
            for sdk in sdks {
                if self.args.bare {
                    println!("{}", sdk.short);
                } else {
                    println!("{:20} [{}]", sdk.short, &sdk.sha[0..7]);
                }
            }
        } else {
            bail!("Cannot handle arguments: {}", self.args)
        }
        Ok(())
    }
}

impl GitRefsKind {
    /// Extracts a key string from `GitRefsKind`.
    fn key(&self) -> String {
        match self {
            GitRefsKind::Tag(version) => format!(
                "{}.{}.{}.{}",
                version.major, version.minor, version.patch, version.hotfix
            ),
            GitRefsKind::Head(branch) => String::from(branch),
        }
    }
}

fn list_remote_sdks_by_tags() -> Result<Vec<RemoteFlutterSdk>> {
    const ERROR_MESSAGE: &str =
        "Failed to fetch remote tags from `https://github.com/flutter/flutter.git`";

    let command_output = Command::new("git")
        .arg("ls-remote")
        .args(["--tags", "--sort=version:refname"])
        .arg("https://github.com/flutter/flutter.git")
        .arg("**/*.*.*")
        .output()
        .context(ERROR_MESSAGE)?;
    if !command_output.status.success() {
        debug!(
            "list_remote_sdks_by_tags(): stderr:\n{}",
            String::from_utf8(command_output.stderr)?
        );
        bail!("{}: {}", ERROR_MESSAGE, command_output.status);
    }
    let git_output = String::from_utf8(command_output.stdout)?;
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

fn list_remote_sdks_by_branches() -> Result<Vec<RemoteFlutterSdk>> {
    const ERROR_MESSAGE: &str =
        "Failed to fetch remote branches from `https://github.com/flutter/flutter.git`";

    let command_output = Command::new("git")
        .arg("ls-remote")
        .args(["--heads", "--refs"])
        .arg("https://github.com/flutter/flutter.git")
        .args(["stable", "beta", "master"])
        .output()
        .context(ERROR_MESSAGE)?;
    if !command_output.status.success() {
        debug!(
            "list_remote_sdks_by_branches(): stderr:\n{}",
            String::from_utf8(command_output.stderr)?
        );
        bail!("{}: {}", ERROR_MESSAGE, command_output.status);
    }
    let git_output = String::from_utf8(command_output.stdout)?;
    debug!("list_remote_sdks_by_branches(): stdout:\n{git_output}");
    let mut lines = git_output.split("\n");
    let git_refs = lines
        .by_ref()
        .map(|line| RemoteFlutterSdk::parse(line))
        .flatten()
        .collect::<Vec<RemoteFlutterSdk>>();
    Ok(git_refs)
}
