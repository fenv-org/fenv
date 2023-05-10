use crate::{
    args::FenvLocalArgs,
    context::FenvContext,
    sdk_service::{model::flutter_sdk::FlutterSdk, results::LookupResult, sdk_service::SdkService},
    service::service::Service,
};
use anyhow::{bail, Context};
use log::debug;
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
                set_local_version(context, sdk_service, prefix)?;
                install_symlink(context, sdk_service)?;
                Ok(())
            }
            None => {
                show_local_version(context, sdk_service, stdout)?;
                if self.args.symlink {
                    install_symlink(context, sdk_service)?;
                }
                Ok(())
            }
        }
    }
}

fn show_local_version(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    stdout: &mut impl Write,
) -> anyhow::Result<()> {
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

fn install_symlink(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
) -> anyhow::Result<()> {
    let read_result = match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        LookupResult::Found(result) => result,
        LookupResult::Err(err) => return Result::Err(anyhow::anyhow!(err)),
        LookupResult::None => bail!("Could not find any local version file"),
    };

    if read_result.installed {
        let original_path = context.fenv_sdk_root(&read_result.sdk.display_name());
        let symlink_path = context.fenv_dir().join(".flutter");
        debug!("original_path: {original_path}",);
        debug!("symlink_path: {symlink_path}",);
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
    sdk_service: &impl SdkService,
    prefix: &str,
) -> anyhow::Result<()> {
    let sdk = match sdk_service.find_latest_local(context, prefix) {
        LookupResult::Found(sdk) => sdk,
        LookupResult::Err(err) => return Err(anyhow::anyhow!(err)),
        LookupResult::None => {
            if sdk_service.find_latest_remote(context, prefix).is_found() {
                bail!("The specified version is not installed: do `fenv install {prefix} && fenv local {prefix}`")
            } else {
                bail!("Not found any matched flutter sdk version: `{prefix}`")
            }
        }
    };

    sdk_service.write_local_version(&context.fenv_dir(), &sdk)
}

#[cfg(test)]
mod tests {
    use crate::{
        define_mock_valid_git_command, external::flutter_command::FlutterCommandImpl,
        sdk_service::sdk_service::RealSdkService, service::macros::test_with_context, try_run,
        util::chrono_wrapper::SystemClock,
    };

    define_mock_valid_git_command!();

    #[test]
    pub fn test_show_local_version_fails_if_no_local_version_file_exists() {
        test_with_context(|context| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let mut stdout: Vec<u8> = vec![];
            let result = try_run(&["fenv", "local"], context, &sdk_service, &mut stdout);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Could not find any local version file"
            )
        })
    }
}
