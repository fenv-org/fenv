use super::{
    flutter_command::{FlutterCommand, FlutterCommandImpl},
    install_sdk::{self, install_sdk, InstallSdkArguments},
};
use crate::{
    args::{self, FenvListRemoteArgs},
    context::FenvContext,
    external::git_command::{GitCommand, GitCommandImpl},
    service::{list_remote::list_remote_service::FenvListRemoteService, service::Service},
};
use anyhow::bail;

pub struct FenvInstallService {
    pub args: args::FenvInstallArgs,
    git_command: Box<dyn GitCommand>,
    flutter_command: Box<dyn FlutterCommand>,
}

impl FenvInstallService {
    pub fn new(args: args::FenvInstallArgs) -> Self {
        Self {
            args,
            git_command: Box::from(GitCommandImpl::new()),
            flutter_command: Box::from(FlutterCommandImpl::new()),
        }
    }

    pub fn exists_installing_marker(
        versions_directory: &str,
        target_version_or_channel: &str,
    ) -> bool {
        install_sdk::exists_installing_marker(versions_directory, target_version_or_channel)
    }
}

impl Service for FenvInstallService {
    fn execute<'a>(
        &self,
        context: &impl FenvContext<'a>,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        if self.args.list {
            let list_remote_service = FenvListRemoteService::new(FenvListRemoteArgs {
                bare: self.args.bare,
            });
            list_remote_service.execute(context, stdout)
        } else if let Some(version) = &self.args.version_prefix {
            let args = InstallSdkArguments {
                target_version_or_channel_prefix: version,
                do_precache: self.args.should_precache,
                git_command: &self.git_command,
                flutter_command: &self.flutter_command,
            };
            install_sdk(context, &args)
        } else {
            bail!("A version or a channel prefix is required.")
        }
    }
}
