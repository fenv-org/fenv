pub mod args;
pub mod context;
pub mod external;
pub mod sdk_service;
pub mod service;
pub mod util;

use crate::{
    args::FenvSubcommands,
    service::{
        completions::completions_service::FenvCompletionsService,
        global::global_service::FenvGlobalService, init::init_service::FenvInitService,
        install::install_service::FenvInstallService, latest::latest_service::FenvLatestService,
        list_remote::list_remote_service::FenvListRemoteService,
        local::local_service::FenvLocalService, service::Service,
        uninstall::uninstall_service::FenvUninstallService,
        version::version_service::FenvVersionService,
        version_file::version_file_service::FenvVersionFileService,
        version_name::version_name_service::FenvVersionNameService,
        versions::versions_service::FenvVersionsService,
    },
};
use anyhow::Result;
use args::FenvArgs;
use clap::{Command, CommandFactory, FromArgMatches};
use context::FenvContext;
use indoc::formatdoc;
use log::debug;
use sdk_service::sdk_service::SdkService;
use std::ffi::OsString;
use util::io::ConsoleOutput;

pub fn try_run<I, T, C: FenvContext, S: SdkService, OUT: std::io::Write, ERR: std::io::Write>(
    args: I,
    context: &C,
    sdk_service: &S,
    output: &mut dyn ConsoleOutput<OUT, ERR>,
) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args = matches_args(args);

    debug!("arguments = {args:?}");

    macro_rules! execute_service {
        ($name: ty, $args: expr) => {
            <$name>::new($args.clone()).execute(context, sdk_service, output)
        };
        ($name: ty) => {
            <$name>::new().execute(context, sdk_service, output)
        };
    }

    match &args.command {
        FenvSubcommands::Init(sub_args) => execute_service!(FenvInitService, sub_args),
        FenvSubcommands::Install(sub_args) => execute_service!(FenvInstallService, sub_args),
        FenvSubcommands::Versions | FenvSubcommands::List => execute_service!(FenvVersionsService),
        FenvSubcommands::Completions(sub_args) => {
            execute_service!(FenvCompletionsService, sub_args)
        }
        FenvSubcommands::Global(sub_args) => execute_service!(FenvGlobalService, sub_args),
        FenvSubcommands::VersionFile(sub_args) => {
            execute_service!(FenvVersionFileService, sub_args)
        }
        FenvSubcommands::VersionName(sub_args) => {
            execute_service!(FenvVersionNameService, sub_args)
        }
        FenvSubcommands::Latest(sub_args) => execute_service!(FenvLatestService, sub_args),
        FenvSubcommands::ListRemote(sub_args) => execute_service!(FenvListRemoteService, sub_args),
        FenvSubcommands::Local(sub_args) => execute_service!(FenvLocalService, sub_args),
        FenvSubcommands::Uninstall(sub_args) => execute_service!(FenvUninstallService, sub_args),
        FenvSubcommands::Version(sub_args) => execute_service!(FenvVersionService, sub_args),
    }
}

pub fn build_command() -> Command {
    args::FenvArgs::command()
    .help_template(
        r#"{before-help}{name} v{version} - {about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"#,
    )
    .override_usage("fenv [OPTIONS] <COMMAND> [args..]")
    .after_help(formatdoc! {"
        Example:
          fenv init                  Show setup instructions.
          fenv init -                Output shell command to configure the shell environment for fenv
          fenv install --list        Show the list of the available Flutter SDKs
          fenv install stable        Install the latest `stable`
          fenv install s             Same as `fenv install stable`
          fenv install 3.0.0         Install Flutter `3.0.0`
          fenv install 3.7           Install the latest version of Flutter `3.7.x`
          fenv install 3             Install the latest version of Flutter `3.x.y`
          fenv install               Install the Flutter version specified in the nearest `.flutter-version` file
          fenv versions              Show the list of the installed Flutter SDKs
          fenv list                  Same as `fenv versions`
          fenv list-remote           Same as `fenv install --list`
          fenv global stable         Use `stable` as the global Flutter SDK
          fenv global s              Same as `fenv global stable`
          fenv global                Show the global flutter version
          fenv local 3.0.0           Use `3.0.0` in the current directory and its child directories
          fenv local 3.7             Use the latest version of Flutter `3.7.x`
                                        in the current directory and its child directories
          fenv local 3               Use the latest version of Flutter `3.x.y`
                                        in the current directory and its child directories
          fenv local                 Show the Flutter version specified in the nearest `.flutter-version` file
          fenv local --symlink       Re-install the symlink to the local Flutter SDK in the directory
                                        where the nearest `.flutter-version` file resides.
          fenv version               Show the selected Flutter SDK version and where its version file is located.
          fenv which flutter         Show the full path to the selected `flutter` executable

          To see command-specific options, `fenv <COMMAND> [-h|--help]`

        Note:
          If you installed a specific version of Flutter SDK,
          fenv doesn't allow to execute `flutter upgrade/downgrade/channel` command.

          But, if you installed any of `dev/beta/master/stable` one,
          you can freely execute `flutter upgrade/downgrade`
          even though `flutter channel` is not still permitted.
        "
    })
    .color(clap::ColorChoice::Never)
}

fn matches_args<I, T>(args: I) -> FenvArgs
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let command = build_command();
    let mut matches = &mut command.get_matches_from(args);
    args::FenvArgs::from_arg_matches_mut(&mut matches)
        .map_err(|err| {
            let mut cmd = args::FenvArgs::command();
            err.format(&mut cmd)
        })
        .unwrap()
}

#[cfg(test)]
mod tests {
    use crate::args::FenvInitArgs;

    use super::*;

    #[test]
    fn test_matches_args_init() {
        let args = matches_args(&["fenv", "init"]);
        assert_eq!(
            args,
            FenvArgs {
                debug: false,
                info: false,
                command: FenvSubcommands::Init(FenvInitArgs {
                    detect_shell: false,
                    shell: None,
                    path_mode: None,
                })
            }
        );
    }

    #[test]
    fn test_matches_args_completions() {
        let args = matches_args(&["fenv", "completions", "bash"]);
        assert_eq!(
            args,
            FenvArgs {
                debug: false,
                info: false,
                command: FenvSubcommands::Completions(args::FenvCompletionsArgs {
                    shell: "bash".to_string()
                })
            }
        )
    }
}
