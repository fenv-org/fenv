use crate::{
    args::FenvStartDirArgs,
    context::FenvContext,
    sdk_service::sdk_service::SdkService,
    service::service::Service,
    util::{io::ConsoleOutput, path_like::PathLike},
};

pub struct FenvVersionNameService {
    pub args: FenvStartDirArgs,
}

impl FenvVersionNameService {
    pub fn new(args: FenvStartDirArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvVersionNameService
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
        let start_dir = match &self.args.dir {
            Some(start_dir) => PathLike::from(start_dir.as_str()),
            None => context.fenv_dir(),
        };

        let result = sdk_service.read_nearest_version_file(context, &start_dir);
        let summary = sdk_service.ensure_sdk_is_available(&result)?;
        writeln!(output.stdout(), "{}", summary.latest_local_sdk)?;
        anyhow::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::{
        context::FenvContext, define_mock_valid_git_command,
        external::flutter_command::FlutterCommandImpl, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run, util::chrono_wrapper::SystemClock,
        write_invalid_utf8,
    };

    define_mock_valid_git_command!();

    #[test]
    fn test_show_version_name_succeeds_if_global_version_name_is_found() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            context.fenv_global_version_file().writeln("1").unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(&["fenv", "version-name"], context, &sdk_service, output).unwrap();

            // verification
            assert_eq!(output.stdout_to_string(), "1.0.0\n");
            assert_eq!(output.stderr_to_string(), "");
        })
    }

    #[test]
    fn test_show_version_name_succeeds_if_local_version_name_is_found() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("master")
                .create_dir_all()
                .unwrap();
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("m")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(&["fenv", "version-name"], context, &sdk_service, output).unwrap();

            // verification
            assert_eq!(output.stdout_to_string(), "master\n");
            assert_eq!(output.stderr_to_string(), "");
        })
    }

    #[test]
    fn test_show_version_name_fails_if_no_version_name_is_found() {
        test_with_context(|context, output| {
            // setup
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "version-name"], context, &sdk_service, output);

            // verification
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Could not find a version file"
            );
        })
    }

    #[test]
    fn test_show_version_name_fails_if_global_version_is_not_installed() {
        test_with_context(|context, output| {
            // setup
            context.fenv_global_version_file().writeln("1").unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "version-name"], context, &sdk_service, output);

            // verification
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "The specified version `1` is not installed (set by `{path}`): do `fenv install`",
                    path = context.fenv_root().join("version")
                )
            );
        })
    }

    #[test]
    fn test_show_local_version_name_succeeds_if_local_and_global_version_name_are_found_both() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_versions()
                .join("1.0.0")
                .create_dir_all()
                .unwrap();
            context
                .fenv_versions()
                .join("master")
                .create_dir_all()
                .unwrap();
            context.fenv_global_version_file().writeln("1").unwrap();
            context
                .fenv_dir()
                .join("child")
                .join(".flutter-version")
                .writeln("m")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            try_run(
                &[
                    "fenv",
                    "version-name",
                    &context.fenv_dir().join("child").to_string(),
                ],
                context,
                &sdk_service,
                output,
            )
            .unwrap();

            // verification
            assert_eq!(output.stdout_to_string(), "master\n");
            assert_eq!(output.stderr_to_string(), "");
        })
    }

    #[test]
    fn test_show_version_name_fails_if_version_name_is_invalid() {
        test_with_context(|context, output| {
            // setup
            context
                .fenv_root()
                .join("version")
                .writeln("invalid")
                .unwrap();
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "version-name"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "Invalid Flutter SDK (set by `{}/version`): `invalid`",
                    context.fenv_root()
                )
            )
        })
    }

    #[test]
    fn test_show_version_name_fails_if_global_version_is_invalid() {
        test_with_context(|context, output| {
            // setup
            // prepare the global version file to contain invalid UTF-8 sequence
            write_invalid_utf8!(context.fenv_root().join("version"));
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "version-name"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                format!(
                    "Could not read the version file (set by `{}`): stream did not contain valid UTF-8",
                    context.fenv_root().join("version")
                )
            )
        })
    }
}
