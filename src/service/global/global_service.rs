use crate::{
    args::FenvGlobalArgs,
    context::FenvContext,
    sdk_service::{results::LookupResult, sdk_service::SdkService},
    service::service::Service,
    util::io::ConsoleOutput,
};
use anyhow::bail;

pub struct FenvGlobalService {
    args: FenvGlobalArgs,
}

impl FenvGlobalService {
    pub fn new(args: FenvGlobalArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvGlobalService
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
            Some(version_prefix) => set_global_version(context, sdk_service, version_prefix),
            None => show_global_version(context, sdk_service, output),
        }
    }
}

fn set_global_version<'a>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    prefix: &str,
) -> anyhow::Result<()> {
    let local_sdk = match sdk_service.find_latest_local(context, prefix) {
        LookupResult::Found(sdk) => sdk,
        LookupResult::Err(err) => return Err(anyhow::anyhow!(err)),
        LookupResult::None => {
            if sdk_service.find_latest_remote(context, prefix).is_found() {
                bail!("The specified version is not installed: do `fenv install {prefix} && fenv global {prefix}`")
            } else {
                bail!("Not found any matched flutter sdk version: `{prefix}`")
            }
        }
    };

    sdk_service.write_global_version(context, &local_sdk)
}

fn show_global_version<'a, OUT, ERR>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
) -> anyhow::Result<()>
where
    OUT: std::io::Write,
    ERR: std::io::Write,
{
    let result = sdk_service.read_global_version(context);
    let summary = sdk_service.ensure_sdk_is_available(&result)?;
    writeln!(output.stdout(), "{}", summary.latest_local_sdk)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        define_mock_valid_git_command, external::flutter_command::FlutterCommandImpl,
        sdk_service::sdk_service::RealSdkService, service::macros::test_with_context, try_run,
        util::chrono_wrapper::SystemClock, write_invalid_utf8,
    };
    use std::io::Write;

    define_mock_valid_git_command!();

    #[test]
    fn test_set_global_version_succeeds() {
        test_with_context(|context, output| {
            // setup
            let args = FenvGlobalArgs {
                prefix: Some("stable".to_string()),
            };
            let service = FenvGlobalService::new(args);
            // emulates installation of stable
            context
                .fenv_root()
                .join("versions/stable")
                .create_dir_all()
                .unwrap();

            // execution
            service
                .execute(context, &RealSdkService::new(), output)
                .unwrap();

            // validation
            let version_file_path = context.fenv_root().join("version");
            assert_eq!(
                std::fs::read_to_string(&version_file_path).unwrap(),
                "stable\n"
            );
        });
    }

    #[test]
    fn test_set_global_version_fails_when_not_a_valid_flutter_version() {
        test_with_context(|context, output| {
            // setup
            let args = FenvGlobalArgs {
                prefix: Some("invalid".to_string()),
            };
            let service = FenvGlobalService::new(args);

            // execution
            let result = service.execute(context, &RealSdkService::new(), output);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
                err.to_string(),
                "Not found any matched flutter sdk version: `invalid`"
            );
        });
    }

    #[test]
    fn test_set_global_version_fails_when_no_version_exists() {
        test_with_context(|context, output| {
            // execution
            let result = try_run(
                &["fenv", "global", "stable"],
                context,
                &RealSdkService::new(),
                output,
            );

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
                err.to_string(),
                "The specified version is not installed: do `fenv install stable && fenv global stable`"
            );
        });
    }

    #[test]
    fn test_show_global_version_fails_when_no_global_version_file_exists() {
        test_with_context(|context, output| {
            // setup
            let args = FenvGlobalArgs { prefix: None };
            let service = FenvGlobalService::new(args);

            // execution
            let result = service.execute(context, &RealSdkService::new(), output);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(err.to_string(), "Could not find a version file");
        });
    }

    #[test]
    fn test_show_global_version_fails_when_global_version_exists_but_not_installed() {
        test_with_context(|context, output| {
            // setup
            let args = FenvGlobalArgs { prefix: None };
            let service = FenvGlobalService::new(args);
            // generates global version file
            let version_file_path = context.fenv_root().join("version");
            version_file_path.write("1.0.0").unwrap();

            // execution
            let result = service.execute(context, &RealSdkService::new(), output);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
                err.to_string(),
                format!(
                    "The specified version `1.0.0` is not installed (set by `{}`): do `fenv install && fenv global --symlink`",
                    context.fenv_global_version_file()
                )
            );
        });
    }

    #[test]
    fn test_show_global_version_fails_when_global_version_exists_but_not_valid() {
        test_with_context(|context, output| {
            // setup
            let args = FenvGlobalArgs { prefix: None };
            let service = FenvGlobalService::new(args);
            // generates global version file
            let version_file_path = context.fenv_root().join("version");
            version_file_path.write("invalid").unwrap();

            // execution
            let result = service.execute(context, &RealSdkService::new(), output);

            // validation
            let err = &result.err().unwrap();
            assert_eq!(
                err.to_string(),
                format!(
                    "Invalid Flutter SDK (set by `{}`): `invalid`",
                    context.fenv_root().join("version")
                )
            );
        });
    }

    #[test]
    fn test_show_global_version_succeeds() {
        test_with_context(|context, output| {
            // setup
            let args = FenvGlobalArgs { prefix: None };
            let service = FenvGlobalService::new(args);
            // generates global version file
            let version_file_path = context.fenv_root().join("version");
            version_file_path.write("1.0.0").unwrap();
            // emulates installation of 1.0.0
            context
                .fenv_root()
                .join("versions/1.0.0")
                .create_dir_all()
                .unwrap();

            // execution
            service
                .execute(context, &RealSdkService::new(), output)
                .unwrap();

            // validation: check if stdout and "1.0.0" are equal
            assert_eq!(output.stdout_to_string(), "1.0.0\n")
        });
    }

    #[test]
    fn test_show_global_version_fails_if_error_occurs_while_reading_version_file() {
        test_with_context(|context, output| {
            // setup
            // prepare the local version file to contain invalid UTF-8 sequence
            write_invalid_utf8!(context.fenv_root().join("version"));
            let sdk_service = RealSdkService::from(
                MockValidGitCommand,
                SystemClock::new(),
                FlutterCommandImpl::new(),
            );

            // execution
            let result = try_run(&["fenv", "global"], context, &sdk_service, output);

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
