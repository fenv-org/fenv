pub mod args;
pub mod context;
pub mod external;
pub mod sdk_service;
pub mod service;
pub mod util;

use crate::{
    args::FenvSubcommands,
    context::RealFenvContext,
    service::{
        completions::completions_service::FenvCompletionsService,
        global::global_service::FenvGlobalService, init::init_service::FenvInitService,
        install::install_service::FenvInstallService, latest::latest_service::FenvLatestService,
        list_remote::list_remote_service::FenvListRemoteService, service::Service,
        version_file::version_file_service::FenvVersionFileService,
        versions::versions_service::FenvVersionsService,
    },
};
use anyhow::Result;
use args::FenvArgs;
use clap::{Command, CommandFactory, FromArgMatches};
use indoc::formatdoc;
use log::debug;
use std::collections::HashMap;

pub fn try_run(args: &Vec<String>, env_vars: &HashMap<String, String>) -> Result<()> {
    let args = matches_args(args);
    let context = RealFenvContext::from(&env_vars)?;

    debug!("context = {context:?}");
    debug!("arguments = {args:?}");

    match &args.command {
        FenvSubcommands::Init(sub_args) => {
            FenvInitService::new(sub_args.clone()).execute(&context, &mut std::io::stdout())
        }
        FenvSubcommands::Install(sub_args) => {
            FenvInstallService::new(sub_args.clone()).execute(&context, &mut std::io::stdout())
        }
        FenvSubcommands::Versions | FenvSubcommands::List => {
            FenvVersionsService::new().execute(&context, &mut std::io::stdout())
        }
        FenvSubcommands::Completions(sub_args) => {
            FenvCompletionsService::new(sub_args.clone()).execute(&context, &mut std::io::stdout())
        }
        FenvSubcommands::Global(sub_args) => {
            FenvGlobalService::new(sub_args.clone()).execute(&context, &mut std::io::stdout())
        }
        FenvSubcommands::VersionFile(sub_args) => {
            FenvVersionFileService::new(sub_args.clone()).execute(&context, &mut std::io::stdout())
        }
        FenvSubcommands::Latest(sub_args) => {
            FenvLatestService::new(sub_args.clone()).execute(&context, &mut std::io::stdout())
        }
        FenvSubcommands::ListRemote(sub_args) => {
            FenvListRemoteService::new(sub_args.clone()).execute(&context, &mut std::io::stdout())
        }
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
          fenv local 3.0.0           Use `3.0.0` in the current directory and its child directories
          fenv local --symlink       Re-install the symlink for the local Flutter SDK
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

fn matches_args(args: &Vec<String>) -> FenvArgs {
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
        let args = matches_args(&vec!["fenv".to_string(), "init".to_string()]);
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
        let args = matches_args(&vec![
            "fenv".to_string(),
            "completions".to_string(),
            "bash".to_string(),
        ]);
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
