use crate::{
    args::{self, FenvListRemoteArgs},
    context::FenvContext,
    sdk_service::sdk_service::{RealSdkService, SdkService},
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
        sdk_service: &impl SdkService,
        stdout: &mut impl std::io::Write,
    ) -> anyhow::Result<()> {
        if self.args.list {
            let list_remote_service = FenvListRemoteService::new(FenvListRemoteArgs {
                bare: self.args.bare,
            });
            list_remote_service.execute(context, sdk_service, stdout)
        } else if let Some(version) = &self.args.version_prefix {
            sdk_service.install_sdk(context, version, true, self.args.should_precache, true)
        } else {
            // TODO: attempt to install the sdk that `flutter version` returns.
            bail!("A version or a channel prefix is required.")
        }
    }
}
