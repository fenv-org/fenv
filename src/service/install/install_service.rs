use crate::config::Config;
use crate::service::install::list_remote_sdk::list_remote_sdks;
use crate::service::service::Service;
use crate::{args, model::remote_flutter_sdk::RemoteFlutterSdk};
use anyhow::{bail, Ok, Result};

use super::flutter_command::{FlutterCommand, FlutterCommandImpl};
use super::git_command::{GitCommand, GitCommandImpl};
use super::install_sdk::install_sdk;

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
}

impl Service for FenvInstallService {
    fn execute(&self, config: &Config) -> Result<()> {
        if self.args.list {
            let sdks = list_remote_sdks(&self.git_command)?;
            for sdk in sdks {
                if self.args.bare {
                    println!("{}", sdk.short);
                } else {
                    println!("{:20} [{}]", sdk.short, &sdk.sha[..7]);
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