use crate::{
    args::FenvLocalArgs,
    context::FenvContext,
    sdk_service::{results::LookupResult, sdk_service::SdkService},
    service::service::Service,
    util::io::ConsoleOutput,
};
use anyhow::bail;
use std::io::Write;

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
            Some(prefix) => set_local_version(context, sdk_service, prefix),
            None => {
                if self.args.symlink {
                    writeln!(
                        output.stderr(),
                        "fenv: info: `--symlink` option is deprecated. For IDE support, use `fenv workspace` instead."
                    )?;
                }
                show_local_version(context, sdk_service, output)
            }
        }
    }
}

fn show_local_version<OUT: Write, ERR: Write>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
) -> anyhow::Result<()> {
    let result = sdk_service.read_nearest_local_version(context, &context.fenv_dir());
    let summary = sdk_service.ensure_sdk_is_available(&result)?;
    writeln!(output.stdout(), "{}", summary.store_version_prefix)?;
    anyhow::Ok(())
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

    // write a local version file.
    sdk_service.write_local_version(&context.fenv_dir(), &sdk)
}

#[cfg(test)]
mod tests {
    use crate::{
        context::FenvContext, define_mock_valid_git_command,
        external::flutter_command::FlutterCommandImpl, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run, util::chrono_wrapper::SystemClock,
        write_invalid_utf8,
    };
    use std::io::Write;

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
                "Could not find a version file"
            )
        })
    }

    #[test]
    pub fn test_show_local_version_fails_if_specified_version_is_not_installed() {
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
            let result = try_run(&["fenv", "local"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                format!(
                    "The specified version `1.0.0` is not installed (set by `{}/.flutter-version`): do `fenv install`",
                    context.fenv_dir()
                )
            );
            assert!(output.stdout_to_string().is_empty());
            assert!(output.stderr_to_string().is_empty());
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
            write_invalid_utf8!(context.fenv_dir().join(".flutter-version"));
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
                    "Could not read the version file (set by `{}/.flutter-version`): stream did not contain valid UTF-8",
                    context.fenv_dir()
                )
            )
        })
    }

    #[test]
    pub fn test_symlink_option_got_deprecated() {
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
            assert_eq!(
                output.stderr_to_string(),
                "fenv: info: `--symlink` option is deprecated. For IDE support, use `fenv workspace` instead.\n"
            );
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
                "Could not find a version file"
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
                    "The specified version `1.0.0` is not installed (set by `{}/.flutter-version`): do `fenv install`",
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

    #[test]
    pub fn test_show_local_version_fails_if_specified_version_is_invalid() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("invalid")
                .unwrap();
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
                format!(
                    "Invalid Flutter SDK (set by `{}/.flutter-version`): `invalid`",
                    context.fenv_dir()
                )
            )
        })
    }

    #[test]
    pub fn test_install_symlink_and_show_local_version_fails_if_specified_version_is_invalid() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("invalid")
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
                    "Invalid Flutter SDK (set by `{}/.flutter-version`): `invalid`",
                    context.fenv_dir()
                )
            )
        })
    }

    #[test]
    pub fn test_install_symlink_and_show_local_version_fails_if_error_happens_while_reading_local_version_file(
    ) {
        test_with_context(|context, output| {
            // setup
            // prepare the local version file to contain invalid UTF-8 sequence
            write_invalid_utf8!(context.fenv_dir().join(".flutter-version"));
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
                    "Could not read the version file (set by `{}/.flutter-version`): stream did not contain valid UTF-8",
                    context.fenv_dir()
                )
            )
        })
    }
}
