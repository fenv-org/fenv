use crate::{
    args::FenvStartDirArgs, context::FenvContext, invoke_command,
    sdk_service::sdk_service::SdkService, service::service::Service, util::io::ConsoleOutput,
};

pub struct FenvVersionService {
    pub args: FenvStartDirArgs,
}

impl FenvVersionService {
    pub fn new(args: FenvStartDirArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvVersionService
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
        let dir: String = if let Some(dir) = &self.args.dir {
            dir.clone()
        } else {
            context.fenv_dir().to_string()
        };
        let version_name = invoke_command!(context, sdk_service, output, "version-name", &dir)?;
        let version_file = invoke_command!(context, sdk_service, output, "version-file", &dir)?;
        writeln!(output.stdout(), "{version_name} (set by `{version_file}`)")?;
        anyhow::Ok(())
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
    fn test_show_version_succeeds_if_global_version_is_set_and_installed() {
        test_with_context(|context, output| {
            // setup
            // make sure v1.0.0 sdk is installed
            context
                .fenv_versions()
                .join("v1.0.0")
                .create_dir_all()
                .unwrap();
            // prepare the global version file
            context.fenv_root().join("version").writeln("1").unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(&["fenv", "version"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                format!("v1.0.0 (set by `{}/version`)\n", context.fenv_root())
            );
        })
    }

    #[test]
    fn test_show_version_with_directory_succeeds_if_global_version_is_set_and_installed() {
        test_with_context(|context, output| {
            // setup
            // make sure v1.0.0 sdk is installed
            context
                .fenv_versions()
                .join("v1.0.0")
                .create_dir_all()
                .unwrap();
            // prepare the local version file
            context
                .fenv_dir()
                .join("a")
                .join(".flutter-version")
                .writeln("1.0.0")
                .unwrap();
            context
                .fenv_dir()
                .join("a")
                .join("b")
                .join("c")
                .create_dir_all()
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(
                &["fenv", "version", &format!("{}/a/b/c", context.fenv_dir())],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // validation
            assert_eq!(
                output.stdout_to_string(),
                format!(
                    "v1.0.0 (set by `{}/a/.flutter-version`)\n",
                    context.fenv_dir()
                )
            );
        })
    }

    #[test]
    fn test_show_version_fails_if_any_version_file_cannot_be_found() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "version"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Could not find a version file"
            )
        })
    }

    #[test]
    fn test_show_version_fails_if_specified_version_is_not_installed() {
        test_with_context(|context, output| {
            // setup
            context.fenv_root().join("version").writeln("3.7").unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "version"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "The specified version `3.7` is not installed (set by `{}/version`): do `fenv install`",
                    context.fenv_root()
                )
            )
        })
    }
}
