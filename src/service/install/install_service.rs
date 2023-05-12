use crate::{
    args::{self, FenvListRemoteArgs},
    context::FenvContext,
    sdk_service::{model::flutter_sdk::FlutterSdk, results::LookupResult, sdk_service::SdkService},
    service::{list_remote::list_remote_service::FenvListRemoteService, service::Service},
};
use anyhow::{bail, Context, Ok};

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
            return list_remote_service.execute(context, sdk_service, stdout);
        }

        if let Some(version) = &self.args.version_prefix {
            return sdk_service.install_sdk(
                context,
                version,
                true,
                self.args.should_precache,
                true,
            );
        }

        let read_result = match sdk_service.read_nearest_local_version(context, &context.fenv_dir())
        {
            LookupResult::Found(read_result) => read_result,
            LookupResult::Err(err) => {
                let version_file = sdk_service
                    .find_nearest_local_version_file(&context.fenv_dir())
                    .unwrap();
                return Result::Err(err).context(format!(
                    "Failed to read the local version at `{version_file}`"
                ));
            }
            LookupResult::None => bail!("Could not find any local version file"),
        };

        if read_result.installed {
            eprintln!("`{}` is already installed", read_result.sdk);
            return Ok(());
        }

        sdk_service.install_sdk(
            context,
            &read_result.sdk.display_name(),
            true,
            self.args.should_precache,
            true,
        )
    }
}
