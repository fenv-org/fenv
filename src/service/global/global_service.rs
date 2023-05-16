use crate::{
    args::FenvGlobalArgs,
    context::FenvContext,
    sdk_service::{
        model::flutter_sdk::FlutterSdk,
        results::{LookupResult, VersionFileReadResult},
        sdk_service::SdkService,
    },
    service::service::Service,
    util::io::ConsoleOutput,
};
use anyhow::{bail, Context, Ok};

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
            None => show_global_version(context, sdk_service, output.stdout()),
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

fn show_global_version<'a>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    stdout: &mut impl std::io::Write,
) -> anyhow::Result<()> {
    match sdk_service.read_global_version(context) {
        VersionFileReadResult::NotFoundVersionFile => {
            bail!("Could not find the global version file")
        }
        VersionFileReadResult::FoundButNotInstalled {
            stored_version_prefix,
            path_to_version_file,
            is_global: _,
            latest_remote_sdk,
        } => {
            if latest_remote_sdk.is_some() {
                bail!(
                    "The specified version `{stored_version_prefix}` is not installed (set by `{path_to_version_file}`): do `fenv install {stored_version_prefix}`",
                )
            } else {
                bail!("Invalid Flutter SDK (set by `{path_to_version_file}`): `{stored_version_prefix}`)")
            }
        }
        VersionFileReadResult::FoundAndInstalled {
            store_version_prefix: _,
            path_to_version_file: _,
            is_global: _,
            latest_local_sdk,
            path_to_sdk_root: _,
        } => {
            writeln!(stdout, "{}", latest_local_sdk.display_name())?;
            Ok(())
        }
        VersionFileReadResult::Err(err) => {
            let error_message = err.to_string();
            Result::Err(err).context(format!(
                "Could not read the global version file (set by `{}`): {error_message}",
                context.fenv_global_version_file(),
            ))
        }
    }
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
            assert_eq!(err.to_string(), "Could not find the global version file");
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
                    "The specified version `1.0.0` is not installed (set by `{}`): do `fenv install 1.0.0`",
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
                    "Invalid Flutter SDK (set by `{}`): `invalid`)",
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
                    "Could not read the global version file (set by `{}`): stream did not contain valid UTF-8",
                    context.fenv_root().join("version")
                )
            )
        })
    }
}
