use crate::{
    args::FenvLocalArgs,
    context::FenvContext,
    sdk_service::{
        model::{flutter_sdk::FlutterSdk, local_flutter_sdk::LocalFlutterSdk},
        results::LookupResult,
        sdk_service::{RealSdkService, SdkService},
    },
    service::service::Service,
    util::path_like::PathLike,
};
use anyhow::{bail, Context};
use std::io::Write;

pub struct FenvLocalService {
    args: FenvLocalArgs,
}

impl FenvLocalService {
    pub fn new(args: FenvLocalArgs) -> Self {
        Self { args }
    }
}

impl Service for FenvLocalService {
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        stdout: &mut impl Write,
    ) -> anyhow::Result<()> {
        match &self.args.prefix {
            Some(prefix) => {
                set_local_version(context, stdout, prefix)?;
                install_symlink(context)?;
                Ok(())
            }
            None => {
                show_local_version(context, stdout)?;
                if self.args.symlink {
                    install_symlink(context)?;
                }
                Ok(())
            }
        }
    }
}

fn show_local_version(context: &impl FenvContext, stdout: &mut impl Write) -> anyhow::Result<()> {
    let sdk_service = RealSdkService::new();
    let read_result = match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        LookupResult::Found(result) => result,
        LookupResult::Err(err) => return Result::Err(anyhow::anyhow!(err)),
        LookupResult::None => bail!("Could not find any local version file"),
    };

    if read_result.installed {
        writeln!(stdout, "{}", read_result.sdk).map_err(|e| anyhow::anyhow!(e))
    } else {
        bail!(
            "The specified version in `{version_file}` is not installed: do `fenv install {sdk}`",
            version_file = context.fenv_global_version_file(),
            sdk = read_result.sdk
        )
    }
}

fn install_symlink(context: &impl FenvContext) -> anyhow::Result<()> {
    let sdk_service = RealSdkService::new();
    let read_result = match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        LookupResult::Found(result) => result,
        LookupResult::Err(err) => return Result::Err(anyhow::anyhow!(err)),
        LookupResult::None => bail!("Could not find any local version file"),
    };

    if read_result.installed {
        let original_path = context.fenv_sdk_root(&read_result.sdk.display_name());
        let symlink_path = context.fenv_dir().join(".flutter");
        symlink_path
            .remove_file()
            .with_context(|| format!("Failed to remove the existing symlink: `{symlink_path}`"))?;
        std::os::unix::fs::symlink(&original_path, &symlink_path).with_context(|| {
            format!("Failed to create a symlink to the installed version: `{symlink_path}`")
        })
    } else {
        bail!(
            "The specified version in `{version_file}` is not installed: do `fenv install {sdk}`",
            version_file = context.fenv_global_version_file(),
            sdk = read_result.sdk
        )
    }
}

fn set_local_version(
    context: &impl FenvContext,
    stdout: &mut impl Write,
    prefix: &str,
) -> anyhow::Result<()> {
    let sdk_service = RealSdkService::new();

    todo!()
}
