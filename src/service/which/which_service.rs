use crate::{
    args::FenvWhichArgs,
    context::FenvContext,
    invoke_command,
    sdk_service::{results::LookupResult, sdk_service::SdkService},
    service::service::Service,
    util::{io::ConsoleOutput, path_like::PathLike},
};
use anyhow::bail;
use is_executable::is_executable;

pub struct FenvWhichService {
    pub args: FenvWhichArgs,
}

impl FenvWhichService {
    pub fn new(args: FenvWhichArgs) -> Self {
        Self { args }
    }
}

impl<OUT, ERR> Service<OUT, ERR> for FenvWhichService
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
        let executable = &self.args.executable;
        let command_path_or_none =
            lookup_executable_in_sdks(context, sdk_service, output, executable)?
                .or_else(|| lookup_executable_in_pub_cache(context, executable));

        match command_path_or_none {
            Some(command_path) => {
                writeln!(output.stdout(), "{}", command_path)?;
                anyhow::Ok(())
            }
            None => bail!("Could not find the specified executable: `{executable}`"),
        }
    }
}

fn lookup_executable_in_sdks<OUT: std::io::Write, ERR: std::io::Write>(
    context: &impl FenvContext,
    sdk_service: &impl SdkService,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
    executable: &str,
) -> anyhow::Result<Option<PathLike>> {
    let version_or_channel = match invoke_command!(context, sdk_service, output, "version-name") {
        Ok(version_or_channel) => version_or_channel,
        Err(err) => {
            if let LookupResult::None =
                sdk_service.find_nearest_version_file(context, &context.fenv_dir())
            {
                return anyhow::Ok(None);
            } else {
                return anyhow::Result::Err(err);
            }
        }
    };

    let prefix = invoke_command!(context, sdk_service, output, "prefix", &version_or_channel)?;
    let command_path = PathLike::from(prefix.as_str()).join("bin").join(executable);
    if is_executable(&command_path) {
        anyhow::Ok(Some(command_path))
    } else {
        anyhow::Ok(None)
    }
}

fn lookup_executable_in_pub_cache(
    context: &impl FenvContext,
    executable: &str,
) -> Option<PathLike> {
    let command_path = context.pub_cache().join("bin").join(executable);
    if is_executable(&command_path) {
        Some(command_path)
    } else {
        None
    }
}

#[cfg(unix)]
#[cfg(test)]
mod tests_unix {
    use crate::{
        context::FenvContext, sdk_service::sdk_service::RealSdkService,
        service::macros::test_with_context, try_run,
    };
    use std::os::unix::prelude::PermissionsExt;

    #[test]
    fn test_show_flutter_filepath_if_everything_is_fine() {
        test_with_context(|context, output| {
            // setup
            // prepare the `flutter` CLI for 3.7.12
            let flutter_path = context.fenv_versions().join("3.7.12/bin/flutter");
            flutter_path.writeln("").unwrap();
            // makes the `flutter` file executable
            let mut permissions = flutter_path.path().metadata().unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&flutter_path, permissions).unwrap();
            // prepare the `.flutter-version` file
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("3")
                .unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            try_run(&["fenv", "which", "flutter"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), format!("{}\n", flutter_path));
            assert!(output.stderr_to_string().is_empty());
        })
    }

    #[test]
    fn test_fails_to_show_flutter_filepath_if_flutter_cli_is_not_executable() {
        test_with_context(|context, output| {
            // setup
            // prepare the `flutter` CLI for 3.7.12
            let flutter_path = context.fenv_versions().join("3.7.12/bin/flutter");
            flutter_path.writeln("").unwrap();
            // prepare the `.flutter-version` file
            context
                .fenv_dir()
                .join(".flutter-version")
                .writeln("3")
                .unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            let result = try_run(&["fenv", "which", "flutter"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Could not find the specified executable: `flutter`"
            );
            assert!(output.stderr_to_string().is_empty());
            assert!(output.stderr_to_string().is_empty());
        })
    }

    #[test]
    fn test_show_melos_filepath_if_everything_is_fine() {
        test_with_context(|context, output| {
            // setup
            // prepare the `melos` CLI
            let melos_path = context.pub_cache().join("bin/melos");
            melos_path.writeln("").unwrap();
            // makes the `melos` file executable
            let mut permissions = melos_path.path().metadata().unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&melos_path, permissions).unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            try_run(&["fenv", "which", "melos"], context, &sdk_service, output).unwrap();

            // validation
            assert_eq!(output.stdout_to_string(), format!("{}\n", melos_path));
            assert!(output.stderr_to_string().is_empty());
        })
    }

    #[test]
    fn test_fails_to_show_melos_filepath_if_melos_cli_is_not_executable() {
        test_with_context(|context, output| {
            // setup
            // prepare the `melos` CLI
            let melos_path = context.pub_cache().join("bin/melos");
            melos_path.writeln("").unwrap();
            let sdk_service = RealSdkService::new();

            // execution
            let result = try_run(&["fenv", "which", "melos"], context, &sdk_service, output);

            // validation
            assert!(result.is_err());
            assert_eq!(
                result.err().unwrap().to_string(),
                "Could not find the specified executable: `melos`"
            );
            assert!(output.stderr_to_string().is_empty());
            assert!(output.stderr_to_string().is_empty());
        })
    }
}
