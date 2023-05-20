use log::debug;

use crate::{
    args::FenvUninstallArgs,
    context::FenvContext,
    sdk_service::{results::LookupResult, sdk_service::SdkService},
    service::service::Service,
    util::io::ConsoleOutput,
};

pub struct FenvUninstallService {
    pub args: FenvUninstallArgs,
}

impl FenvUninstallService {
    pub fn new(args: FenvUninstallArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvUninstallService
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
        for prefix in &self.args.prefixes {
            uninstall_version(context, sdk_service, output, prefix)?
        }
        Ok(())
    }
}

fn uninstall_version<OUT, ERR>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    prefix: &str,
) -> anyhow::Result<()>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    debug!("Attempting to uninstall `{}`", prefix);
    let mut lookup_result = sdk_service.find_latest_local(context, prefix);
    if let LookupResult::None = lookup_result {
        writeln!(
            output.stderr(),
            "Could not find any installed sdk: `{}`",
            prefix
        )?;
        return anyhow::Ok(());
    }

    loop {
        match lookup_result {
            LookupResult::Found(sdk) => {
                debug!("Found sdk: `{}`", sdk);
                let result = sdk_service.uninstall(context, &sdk);
                if result.is_err() {
                    break result;
                }
                writeln!(output.stdout(), "{}", sdk)?;
                lookup_result = sdk_service.find_latest_local(context, prefix)
            }
            LookupResult::Err(err) => break Result::Err(anyhow::anyhow!(err)),
            LookupResult::None => break anyhow::Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::FenvContext, define_mock_valid_git_command,
        external::flutter_command::FlutterCommandImpl, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run, util::chrono_wrapper::SystemClock,
    };

    define_mock_valid_git_command!();

    #[test]
    fn test_uninstall_version_succeeds() {
        test_with_context(|context, output| {
            // setup
            let sdks = [
                "3.1.0", "3.0.0", "3.3.0", "3.3.10", "3.7.0", "3.7.1", "3.7.12", "stable",
            ];
            for version in &sdks {
                context
                    .fenv_versions()
                    .join(*version)
                    .create_dir_all()
                    .unwrap();
            }
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(
                &["fenv", "uninstall", "3", "stable"],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                // Removed from the latest version.
                "3.7.12\n3.7.1\n3.7.0\n3.3.10\n3.3.0\n3.1.0\n3.0.0\nstable\n"
            );
            assert!(output.stderr_to_string().is_empty());
            for version in &sdks {
                assert!(!context.fenv_versions().join(*version).exists());
            }
        })
    }

    #[test]
    fn test_uninstall_version_does_not_fails_if_attempts_to_uninstall_nonexistent_sdk() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(
                &["fenv", "uninstall", "stable"],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert!(output.stdout_to_string().is_empty());
            assert_eq!(
                output.stderr_to_string(),
                "Could not find any installed sdk: `stable`\n"
            );
        })
    }
}
