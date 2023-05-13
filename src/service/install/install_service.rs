use crate::{
    args::{self, FenvListRemoteArgs},
    context::FenvContext,
    sdk_service::{model::flutter_sdk::FlutterSdk, results::LookupResult, sdk_service::SdkService},
    service::{list_remote::list_remote_service::FenvListRemoteService, service::Service},
    util::io::ConsoleOutput,
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

impl<OUT, ERR> Service<OUT, ERR> for FenvInstallService
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
        if self.args.list {
            let list_remote_service = FenvListRemoteService::new(FenvListRemoteArgs {
                bare: self.args.bare,
            });
            return list_remote_service.execute(context, sdk_service, output);
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
            LookupResult::None => {
                bail!("Could not find any local version file. Specify a version to install.")
            }
        };

        if read_result.is_installed() {
            writeln!(
                output.stderr(),
                "`{}` is already installed",
                read_result.sdk
            )?;
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::{
        context::FenvContext, define_mock_flutter_command, define_mock_valid_git_command,
        sdk_service::sdk_service::RealSdkService, service::macros::test_with_context, try_run,
        util::chrono_wrapper::SystemClock,
    };

    define_mock_valid_git_command!();
    define_mock_flutter_command!();

    #[test]
    pub fn test_install_without_prefix_succeeds() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_dir()
                .join(".flutter-version")
                .write("stable")
                .unwrap();
            let sdk_service =
                RealSdkService::from(MockValidGitCommand, SystemClock::new(), MockFlutterCommand);

            // precondition
            assert!(!context.fenv_versions().join("stable").exists());

            // execution
            try_run(&["fenv", "install"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), "");
            assert!(context.fenv_versions().join("stable").is_dir())
        })
    }

    #[test]
    pub fn test_install_without_prefix_succeeds_even_if_specified_version_is_already_installed() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_dir()
                .join(".flutter-version")
                .write("stable")
                .unwrap();
            context
                .fenv_versions()
                .join("stable")
                .create_dir_all()
                .unwrap();
            let sdk_service =
                RealSdkService::from(MockValidGitCommand, SystemClock::new(), MockFlutterCommand);

            // execution
            try_run(&["fenv", "install"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), "");
            assert_eq!(output.stderr_to_string(), "`stable` is already installed\n");
            assert!(context.fenv_versions().join("stable").is_dir())
        })
    }

    #[test]
    pub fn test_install_without_prefix_fails_if_error_happens_while_reading_version_file() {
        test_with_context(|context, output| {
            // setup
            // Prepare a version file that contains invalid UTF-8 sequence.
            let mut version_file = context
                .fenv_dir()
                .join(".flutter-version")
                .create_file()
                .unwrap();
            version_file.write(&[0xDE, 0xAD, 0xBE, 0xEF]).unwrap();
            let sdk_service =
                RealSdkService::from(MockValidGitCommand, SystemClock::new(), MockFlutterCommand);

            // execution
            let result = try_run(&["fenv", "install"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "Failed to read the local version at `{filepath}`",
                    filepath = context.fenv_dir().join(".flutter-version")
                )
            )
        })
    }

    #[test]
    pub fn test_install_without_prefix_fails_if_no_version_file_exists() {
        test_with_context(|context, output| {
            // setup
            let sdk_service =
                RealSdkService::from(MockValidGitCommand, SystemClock::new(), MockFlutterCommand);

            // execution
            let result = try_run(&["fenv", "install"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Could not find any local version file. Specify a version to install."
            )
        })
    }
}
