use crate::{
    external::git_command::GitCommand,
    sdk_service::{
        model::{
            flutter_sdk::FlutterSdk,
            local_flutter_sdk::LocalFlutterSdk,
            remote_flutter_sdk::{GitRefsKind, RemoteFlutterSdk},
        },
        sdk_service::{RealSdkService, SdkService},
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
    sdk_service: &impl SdkService,
    // git_command: &Box<dyn GitCommand>,
    // clock: &Box<dyn Clock>,
) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
    const CACHE_FILE_NAME: &str = ".remote_list";

    let cache_file_path = cache_directory.join(CACHE_FILE_NAME);
    let cache_or_none = lookup_cached_list(&cache_file_path, clock);
    if let Some(cache) = cache_or_none {
        debug!("sdk list from cache");
        Ok(cache)
    } else {
        debug!("sdk list from remote");
        let sdks = sdk_service.get_available_sdk_list(context)?;
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
