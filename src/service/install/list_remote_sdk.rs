use std::{collections::HashSet, path::PathBuf};

use anyhow::{Ok, Result};
use log::{debug, warn};

use crate::{
    model::{
        local_flutter_sdk::LocalFlutterSdk,
        remote_flutter_sdk::{GitRefsKind, RemoteFlutterSdk},
    },
    service::install::list_remote_sdk_cache::{cache_list, lookup_cached_list},
    util::chrono_wrapper::Clock,
};

use super::git_command::GitCommand;

pub struct ShowRemoteSdksArguments<'a> {
    pub cache_directory: &'a str,
    pub git_command: &'a Box<dyn GitCommand>,
    pub installed_sdks: &'a [LocalFlutterSdk],
    pub clock: &'a Box<dyn Clock>,
    pub bare: bool,
}

pub fn show_remote_sdks(
    args: &ShowRemoteSdksArguments,
    stdout: &mut impl std::io::Write,
) -> anyhow::Result<()> {
    let sdks = cached_or_fetch_remote_sdks(args.cache_directory, args.git_command, args.clock)?;
    display_remote_sdks(&sdks, args.installed_sdks, stdout, args.bare)
}

pub fn cached_or_fetch_remote_sdks(
    cache_directory: &str,
    git_command: &Box<dyn GitCommand>,
    clock: &Box<dyn Clock>,
) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
    const CACHE_FILE_NAME: &str = ".remote_list";

    let cache_file_path = PathBuf::from(cache_directory).join(CACHE_FILE_NAME);
    let cache_or_none = lookup_cached_list(&cache_file_path.to_str().unwrap(), clock);
    if let Some(cache) = cache_or_none {
        debug!("sdk list from cache");
        Ok(cache)
    } else {
        debug!("sdk list from remote");
        let sdks = list_remote_sdks(git_command)?;
        if let Err(err) = cache_list(&cache_file_path.to_str().unwrap(), &sdks, clock) {
            warn!("{}", err);
        }
        Ok(sdks)
    }
}

fn display_remote_sdks(
    sdks: &[RemoteFlutterSdk],
    installed_sdks: &[LocalFlutterSdk],
    stdout: &mut impl std::io::Write,
    bare: bool,
) -> anyhow::Result<()> {
    let installed_sdks_set: HashSet<String> =
        installed_sdks.iter().map(|sdk| sdk.refs_name()).collect();

    for sdk in sdks {
        if bare {
            writeln!(stdout, "{}", sdk.short)?;
        } else {
            let is_installed = installed_sdks_set.contains(&sdk.long);
            if is_installed {
                writeln!(stdout, "* {:18} [{}]", sdk.short, &sdk.sha[..7])?;
            } else {
                writeln!(stdout, "  {:18} [{}]", sdk.short, &sdk.sha[..7])?;
            }
        }
    }
    Ok(())
}

pub fn list_remote_sdks(git_command: &Box<dyn GitCommand>) -> Result<Vec<RemoteFlutterSdk>> {
    let mut sdks = list_remote_sdks_by_tags(&git_command)?;
    sdks.extend(list_remote_sdks_by_branches(&git_command)?);
    Ok(sdks)
}

fn list_remote_sdks_by_tags(git_command: &Box<dyn GitCommand>) -> Result<Vec<RemoteFlutterSdk>> {
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
    git_command: &Box<dyn GitCommand>,
) -> Result<Vec<RemoteFlutterSdk>> {
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
