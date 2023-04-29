use crate::{
    external::git_command::GitCommand,
    sdk_service::model::{
        flutter_sdk::FlutterSdk,
        local_flutter_sdk::LocalFlutterSdk,
        remote_flutter_sdk::{GitRefsKind, RemoteFlutterSdk},
    },
    util::{
        chrono_wrapper::Clock,
        list_remote_sdk_cache::{cache_list, lookup_cached_list},
        path_like::PathLike,
    },
};
use anyhow::{Ok, Result};
use log::{debug, warn};
use std::collections::HashSet;

pub struct ShowRemoteSdksArguments<'a> {
    pub cache_directory: &'a PathLike,
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
    cache_directory: &PathLike,
    git_command: &Box<dyn GitCommand>,
    clock: &Box<dyn Clock>,
) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
    const CACHE_FILE_NAME: &str = ".remote_list";

    let cache_file_path = cache_directory.join(CACHE_FILE_NAME);
    let cache_or_none = lookup_cached_list(&cache_file_path, clock);
    if let Some(cache) = cache_or_none {
        debug!("sdk list from cache");
        Ok(cache)
    } else {
        debug!("sdk list from remote");
        let sdks = list_remote_sdks(git_command)?;
        if let Err(err) = cache_list(&cache_file_path, &sdks, clock) {
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
            writeln!(stdout, "{}", sdk.display_name())?;
        } else {
            let is_installed = installed_sdks_set.contains(&sdk.long);
            if is_installed {
                writeln!(stdout, "* {:18} [{}]", sdk.display_name(), &sdk.sha[..7])?;
            } else {
                writeln!(stdout, "  {:18} [{}]", sdk.display_name(), &sdk.sha[..7])?;
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
