use crate::{
    args::FenvLocalArgs,
    context::FenvContext,
    sdk_service::{
        model::flutter_sdk::FlutterSdk,
        results::{LookupResult, VersionFileReadResult},
        sdk_service::SdkService,
    },
    service::service::Service,
    util::io::ConsoleOutput,
};
use anyhow::{bail, Context};
use log::debug;
use std::{io::Write, os::unix::fs};

pub struct FenvLocalService {
    args: FenvLocalArgs,
}

impl FenvLocalService {
    pub fn new(args: FenvLocalArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvLocalService
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    fn execute(
        &self,
        context: &impl FenvContext,
        sdk_service: &impl SdkService,
        output: &mut dyn ConsoleOutput<OUT, ERR>,
    ) -> anyhow::Result<()> {
        match &self.args.prefix {
            Some(prefix) => {
                set_local_version(context, sdk_service, prefix)?;
                install_symlink(context, sdk_service)?;
                Ok(())
            }
            None => {
                if self.args.symlink {
                    install_symlink_and_show_local_version(context, sdk_service, output)
                } else {
                    show_local_version(context, sdk_service, output)
                }
            }
        }
    }
}

fn create_symlink_inner(context: &impl FenvContext, sdk: &impl FlutterSdk) -> anyhow::Result<()> {
    let original_path = context.fenv_sdk_root(&sdk.display_name());
    let symlink_path = context.fenv_dir().join(".flutter");
    debug!("original_path: {original_path}",);
    debug!("symlink_path: {symlink_path}",);
    symlink_path
        .remove_file()
        .with_context(|| format!("Failed to remove the existing symlink: `{symlink_path}`"))?;
    fs::symlink(&original_path, &symlink_path).with_context(|| {
        format!("Failed to create a symlink to the installed version: `{symlink_path}`")
    })
}

fn show_local_version<OUT: Write, ERR: Write>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
) -> anyhow::Result<()> {
    match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        VersionFileReadResult::NotFoundVersionFile => {
            bail!("Could not find any local version file")
        }
        VersionFileReadResult::FoundButNotInstalled {
            stored_version_prefix,
            path_to_version_file,
            is_global: _,
            latest_remote_sdk,
        } => {
            if latest_remote_sdk.is_some() {
                writeln!(
                    output.stderr(),
                     "warn: The specified version `{sdk}` in `{path_to_version_file}` is not installed: do `fenv install && fenv local --symlink`",
                    sdk = stored_version_prefix
                )?;
                writeln!(output.stdout(), "{stored_version_prefix}").map_err(|e| anyhow::anyhow!(e))
            } else {
                bail!("Invalid Flutter SDK: {stored_version_prefix}")
            }
        }
        VersionFileReadResult::FoundAndInstalled {
            store_version_prefix: _,
            path_to_version_file: _,
            is_global: _,
            latest_local_sdk,
            path_to_sdk_root: _,
        } => writeln!(output.stdout(), "{latest_local_sdk}").map_err(|e| anyhow::anyhow!(e)),
        VersionFileReadResult::Err(err) => {
            let file = sdk_service
                .find_nearest_local_version_file(&context.fenv_dir())
                .unwrap();
            Result::Err(anyhow::anyhow!(err))
                .context(format!("Failed to read the local version: `{file}`"))
        }
    }
}

fn install_symlink_and_show_local_version<OUT: Write, ERR: Write>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
) -> anyhow::Result<()> {
    match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        VersionFileReadResult::NotFoundVersionFile => {
            bail!("Could not find any local version file")
        }
        VersionFileReadResult::FoundButNotInstalled {
            stored_version_prefix,
            path_to_version_file,
            is_global: _,
            latest_remote_sdk,
        } => {
            if latest_remote_sdk.is_some() {
                bail!(
                    "The specified version `{}` is not installed (set by `{}`): do `fenv install {}`",
                    stored_version_prefix, path_to_version_file, latest_remote_sdk.unwrap(),
                )
            } else {
                bail!("Invalid Flutter SDK: {stored_version_prefix}")
            }
        }
        VersionFileReadResult::FoundAndInstalled {
            store_version_prefix: _,
            path_to_version_file: _,
            is_global: _,
            latest_local_sdk,
            path_to_sdk_root: _,
        } => {
            create_symlink_inner(context, &latest_local_sdk)?;
            writeln!(output.stdout(), "{latest_local_sdk}").map_err(|e| anyhow::anyhow!(e))
        }
        VersionFileReadResult::Err(err) => {
            let file = sdk_service
                .find_nearest_local_version_file(&context.fenv_dir())
                .unwrap();
            Result::Err(anyhow::anyhow!(err))
                .context(format!("Failed to read the local version: `{file}`"))
        }
    }
}

fn install_symlink(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
) -> anyhow::Result<()> {
    match sdk_service.read_nearest_local_version(context, &context.fenv_dir()) {
        VersionFileReadResult::NotFoundVersionFile => bail!("Could not find any local version file"),
        VersionFileReadResult::FoundButNotInstalled {
            stored_version_prefix,
            path_to_version_file,
            is_global: _,
            latest_remote_sdk: _,
        } => bail!(
            "The specified version `{sdk}` in `{path_to_version_file}` is not installed: do `fenv install {sdk}`",
            sdk = stored_version_prefix
        ),
        VersionFileReadResult::FoundAndInstalled {
            store_version_prefix: _,
            path_to_version_file: _,
            is_global: _,
            latest_local_sdk: _,
            path_to_sdk_root,
        } => {
            let symlink_path = context.fenv_dir().join(".flutter");
            debug!("original_path: {path_to_sdk_root}",);
            debug!("symlink_path: {symlink_path}",);
            symlink_path
                .remove_file()
                .with_context(|| format!("Failed to remove the existing symlink: `{symlink_path}`"))?;
            fs::symlink(&path_to_sdk_root, &symlink_path).with_context(|| {
                format!("Failed to create a symlink to the installed version: `{symlink_path}`")
            })
        },
        VersionFileReadResult::Err(err) => Result::Err(anyhow::anyhow!(err)),
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
        context::FenvContext, define_mock_valid_git_command,
        external::flutter_command::FlutterCommandImpl, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run, util::chrono_wrapper::SystemClock,
    };
    use std::{io::Write, path::PathBuf};

    define_mock_valid_git_command!();

    #[test]
    pub fn test_show_local_version_fails_if_no_local_version_file_exists() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "local"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Could not find any local version file"
            )
        })
    }

    #[test]
    pub fn test_show_local_version_succeeds_even_if_specified_version_is_not_installed() {
        test_with_context(|context, output| {
            // setup
            // prepare the local version file to contain sdk version 1.0.0
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(&["fenv", "local"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), "1.0.0\n");
            assert_eq!(
                output.stderr_to_string(),
                format!("warn: The specified version `1.0.0` in `{local_version_file}` is not installed: do `fenv install && fenv local --symlink`\n",
                    local_version_file = context.fenv_dir().join(".flutter-version")
                )
            );
        })
    }

    #[test]
    pub fn test_show_local_version_succeeds_if_specified_version_is_installed() {
        test_with_context(|context, output| {
            // setup
            // prepare the local version file to contain sdk version 1.0.0
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(&["fenv", "local"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), "1.0.0\n")
        })
    }

    #[test]
    pub fn test_show_local_version_fails_if_error_happens_while_reading_local_version_file() {
        test_with_context(|context, output| {
            // setup
            // prepare the local version file to contain invalid UTF-8 sequence
            let mut version_file = context
                .fenv_dir()
                .join(".flutter-version")
                .create_file()
                .unwrap();
            version_file.write(&[0xDE, 0xED, 0xBE, 0xEF]).unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "local"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "Failed to read the local version: `{}/.flutter-version`",
                    context.fenv_dir()
                )
            )
        })
    }

    #[test]
    pub fn test_install_symlink_succeeds_if_specified_version_is_installed() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(
                &["fenv", "local", "--symlink"],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), "1.0.0\n");
            let symlink: PathBuf = context.fenv_dir().join(".flutter").path().to_owned();
            assert!(symlink.is_symlink());
            let symlink_destination = symlink.read_link().unwrap();
            let sdk_path = context.fenv_versions().join("1.0.0").path().to_owned();
            assert_eq!(symlink_destination.to_str(), sdk_path.to_str());
        })
    }

    #[test]
    pub fn test_install_symlink_fails_if_no_local_version_file_exists() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(
                &["fenv", "local", "--symlink"],
                context,
                &sdk_service,
                output,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Could not find any local version file"
            )
        })
    }

    #[test]
    pub fn test_install_symlink_fails_if_specified_version_is_not_installed() {
        test_with_context(|context, output| {
            // setup
            // prepare the local version file to contain sdk version 1.0.0
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(
                &["fenv", "local", "--symlink"],
                context,
                &sdk_service,
                output,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                format!(
                    "The specified version `1.0.0` is not installed (set by `{}/.flutter-version`): do `fenv install 1.0.0`",
                    context.fenv_dir()
                )
            )
        })
    }

    #[test]
    pub fn test_set_local_version_succeeds_if_specified_version_is_installed() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(&["fenv", "local", "1.0.0"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), "");
            assert_eq!(
                context
                    .fenv_dir()
                    .join(".flutter-version")
                    .read_to_string()
                    .unwrap(),
                "1.0.0\n"
            );
            let symlink: PathBuf = context.fenv_dir().join(".flutter").path().to_owned();
            assert!(symlink.is_symlink());
            let symlink_destination = symlink.read_link().unwrap();
            let sdk_path = context.fenv_versions().join("1.0.0").path().to_owned();
            assert_eq!(symlink_destination.to_str(), sdk_path.to_str());
        })
    }

    #[test]
    pub fn test_set_local_version_fails_if_specified_version_is_not_installed() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "local", "1"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "The specified version is not installed: do `fenv install 1 && fenv local 1`"
            )
        })
    }

    #[test]
    pub fn test_set_local_version_fails_if_specified_version_does_not_exist() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "local", "invalid"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Not found any matched flutter sdk version: `invalid`"
            )
        })
    }
}
