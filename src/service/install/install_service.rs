use std::path::PathBuf;

use crate::config::Config;
use crate::service::install::list_remote_sdk::list_remote_sdks;
use crate::service::install::list_remote_sdk_cache::cache_list;
use crate::service::service::Service;
use crate::util::chrono_wrapper::SystemClock;
use crate::{args, model::remote_flutter_sdk::RemoteFlutterSdk};
use anyhow::{bail, Ok, Result};
use log::{debug, warn};

use super::flutter_command::{FlutterCommand, FlutterCommandImpl};
use super::git_command::{GitCommand, GitCommandImpl};
use super::install_sdk::{self, install_sdk};
use super::list_remote_sdk_cache::lookup_cached_list;

pub struct FenvInstallService {
    pub args: args::FenvInstallArgs,
    git_command: Box<dyn GitCommand>,
    flutter_command: Box<dyn FlutterCommand>,
}

impl FenvInstallService {
    pub fn from(args: args::FenvInstallArgs) -> FenvInstallService {
        FenvInstallService {
            args,
            git_command: Box::from(GitCommandImpl::new()),
            flutter_command: Box::from(FlutterCommandImpl::new()),
        }
    }

    pub fn list_remote_sdks() -> Result<Vec<RemoteFlutterSdk>> {
        let git_command: Box<dyn GitCommand> = Box::new(GitCommandImpl::new());
        list_remote_sdks(&git_command)
    }

    pub fn exists_installing_marker(
        versions_directory: &str,
        target_version_or_channel: &str,
    ) -> bool {
        install_sdk::exists_installing_marker(versions_directory, target_version_or_channel)
    }
}

impl Service for FenvInstallService {
    fn execute(&self, config: &Config, stdout: &mut impl std::io::Write) -> Result<()> {
        const CACHE_FILE_NAME: &str = ".remote_list";

        if self.args.list {
            let cache_file_path = PathBuf::from(&config.fenv_cache()).join(CACHE_FILE_NAME);
            let clock = SystemClock::new();
            let cache_or_none = lookup_cached_list(&cache_file_path.to_str().unwrap(), &clock);
            let sdks = if let Some(cache) = cache_or_none {
                debug!("sdk list from cache");
                cache
            } else {
                debug!("sdk list from remote");
                let sdks = list_remote_sdks(&self.git_command)?;
                if let Err(err) = cache_list(&cache_file_path.to_str().unwrap(), &sdks, &clock) {
                    warn!("{}", err);
                }
                sdks
            };
            for sdk in sdks {
                if self.args.bare {
                    writeln!(stdout, "{}", sdk.short)?;
                } else {
                    writeln!(stdout, "{:20} [{}]", sdk.short, &sdk.sha[..7])?;
                }
            }
        } else if let Some(version) = &self.args.version {
            install_sdk(
                &config.fenv_versions(),
                &version,
                self.args.should_precache,
                &self.git_command,
                &self.flutter_command,
            )?
        } else {
            bail!("Cannot handle arguments: {}", self.args)
        }
        Ok(())
    }
}
