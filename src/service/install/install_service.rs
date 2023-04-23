use crate::config::Config;

use crate::service::service::Service;

use crate::service::versions::versions_service::FenvVersionsService;
use crate::util::chrono_wrapper::{Clock, SystemClock};
use crate::{args, model::remote_flutter_sdk::RemoteFlutterSdk};
use anyhow::{bail, Result};

use super::flutter_command::{FlutterCommand, FlutterCommandImpl};
use super::git_command::{GitCommand, GitCommandImpl};
use super::install_sdk::{self, install_sdk};
use super::list_remote_sdk::{list_remote_sdks, show_remote_sdks, ShowRemoteSdksArguments};

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
        if self.args.list {
            // let sdks = cached_or_fetch_remote_sdks(&config.fenv_cache(), &self.git_command)?;
            let clock: Box<dyn Clock> = Box::new(SystemClock::new());
            let installed_sdks = FenvVersionsService::list_installed_sdks(config)?;
            let args = ShowRemoteSdksArguments {
                cache_directory: &config.fenv_cache(),
                bare: self.args.bare,
                installed_sdks: &installed_sdks,
                git_command: &self.git_command,
                clock: &clock,
            };
            show_remote_sdks(&args, stdout)
        } else if let Some(version) = &self.args.version {
            install_sdk(
                &config.fenv_versions(),
                &version,
                self.args.should_precache,
                &self.git_command,
                &self.flutter_command,
            )
        } else {
            bail!("Cannot handle arguments: {}", self.args)
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_exists_installing_marker() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/install_service/git_lf-remote_heads.txt");
        println!("{:?}", d);

        // print the contents in "d"
        println!("With text:\n{}", std::fs::read_to_string(d).unwrap());
    }

    struct MockGitCommand;

    // impl GitCommand for MockGitCommand {}

    #[test]
    fn text_list_remote_sdks() {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        root.push("resources/test/install_service/git_lf-remote_heads.txt");
    }
}
