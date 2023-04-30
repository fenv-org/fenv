use crate::{
    args::{self, FenvListRemoteArgs},
    context::FenvContext,
    sdk_service::sdk_service::{RealSdkService, SdkService as _},
    service::{list_remote::list_remote_service::FenvListRemoteService, service::Service},
};
use anyhow::bail;

pub struct FenvInstallService {
    pub args: args::FenvInstallArgs,
}

impl FenvInstallService {
    pub fn new(args: args::FenvInstallArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvInstallService {
    fn execute(
        &self,
        context: &impl FenvContext,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        if self.args.list {
            let list_remote_service = FenvListRemoteService::new(FenvListRemoteArgs {
                bare: self.args.bare,
            });
            list_remote_service.execute(context, stdout)
        } else if let Some(version) = &self.args.version_prefix {
            let sdk_service = RealSdkService::new();
            sdk_service.install_sdk(context, version, true, self.args.should_precache)
        } else {
            bail!("A version or a channel prefix is required.")
        }
    }
}
