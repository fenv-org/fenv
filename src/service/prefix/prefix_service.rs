use crate::{
    args::FenvPrefixArgs, context::FenvContext, invoke_command,
    sdk_service::sdk_service::SdkService, service::service::Service, util::io::ConsoleOutput,
};

pub struct FenvPrefixService {
    pub args: FenvPrefixArgs,
}

impl FenvPrefixService {
    pub fn new(args: FenvPrefixArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvPrefixService
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
        let version_prefix = match &self.args.prefix {
            Some(prefix) => prefix.to_owned(),
            None => invoke_command!(context, sdk_service, output, "version-name")?,
        };
        let version_or_channel =
            invoke_command!(context, sdk_service, output, "latest", &version_prefix)?;
        writeln!(
            output.stdout(),
            "{}",
            context.fenv_sdk_root(&version_or_channel).to_string()
        )?;
        Ok(())
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
    fn test_prefix_succeeds_with_prefix() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("stable")
                .create_dir_all()
                .unwrap();

            // execution
            try_run(
                &["fenv", "prefix", "s"],
                context,
                &RealSdkService::new(),
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                format!("{}\n", context.fenv_versions().join("stable"))
            );
            assert!(output.stderr_to_string().is_empty())
        })
    }

    #[test]
    fn test_prefix_succeeds_without_prefix_if_global_version_file_exists() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("1.22.6")
                .create_dir_all()
                .unwrap();
            context.fenv_root().join("version").writeln("v1").unwrap();

            // execution
            try_run(&["fenv", "prefix"], context, &RealSdkService::new(), output).unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                format!("{}\n", context.fenv_versions().join("1.22.6"))
            );
            assert!(output.stderr_to_string().is_empty())
        })
    }

    #[test]
    fn test_prefix_succeeds_without_prefix_if_local_version_file_exists() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("1.22.6")
                .create_dir_all()
                .unwrap();
            context.fenv_root().join("version").writeln("2").unwrap();
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("1")
                .unwrap();

            // execution
            try_run(&["fenv", "prefix"], context, &RealSdkService::new(), output).unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                format!("{}\n", context.fenv_versions().join("1.22.6"))
            );
            assert!(output.stderr_to_string().is_empty())
        })
    }

    #[test]
    fn test_prefix_fails_with_prefix_if_specified_version_is_not_installed() {
        test_with_context(|context, output| {
            // execution
            let result = try_run(
                &["fenv", "prefix", "stable"],
                context,
                &RealSdkService::new(),
                output,
            );

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Not found any matched flutter sdk version: `stable`",
            )
        })
    }

    #[test]
    fn test_prefix_fails_without_prefix_if_specified_version_is_not_installed() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("2")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "prefix"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "The specified version `2` is not installed (set by `{}/.flutter-version`): do `fenv install && fenv local --symlink`",
                    context.fenv_dir()
                ),
            )
        })
    }
}
