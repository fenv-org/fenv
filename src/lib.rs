pub mod args;
pub mod config;
pub mod model;
pub mod service;

use indoc::formatdoc;
use log::debug;
use std::collections::HashMap;

use crate::{
    args::FenvSubcommands,
    config::Config,
    service::{
        init::init_service::FenvInitService, install::install_service::FenvInstallService,
        service::Service, versions::versions_service::FenvVersionsService,
    },
};
use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};

pub fn try_run(args: &Vec<String>, env_vars: &HashMap<String, String>) -> Result<()> {
    let command = args::FenvArgs::command_for_update();
    let mut matches = command
        .help_template(
            r#"{before-help}{name} v{version} - {about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
    "#,
        )
        .after_help(formatdoc! {"
            Example:
              fenv init                  Show setup instructions.
              fenv init -                Output shell command to configure the shell environment for fenv
              fenv install --list        Show the list of the available Flutter SDKs
              fenv install stable        Install the latest `stable`
              fenv install 3.0.0         Install Flutter `3.0.0`
              fenv install               Install the Flutter version specified in the nearest `.flutter-version` file
              fenv global stable         Use `stable` as the global Flutter SDK
              fenv local 3.0.0           Use `3.0.0` in the current directory and its child directories
              fenv local --symlink       Re-install the symlink for the local Flutter SDK
              fenv which flutter         Show the full path to the selected `flutter` executable

            Note:
              If you installed a specific version of Flutter SDK,
              fenv doesn't allow to execute `flutter upgrade/downgrade/channel` command.

              But, if you installed any of `dev/beta/master/stable` one,
              you can freely execute `flutter upgrade/downgrade`
              even though `flutter channel` is not still permitted.
            "
        })
        .color(clap::ColorChoice::Never)
        .get_matches_from(args);
    let args = args::FenvArgs::from_arg_matches_mut(&mut matches)
        .map_err(|err| {
            let mut cmd = args::FenvArgs::command();
            err.format(&mut cmd)
        })
        .unwrap();

    // let args = args::FenvArgs::parse_from(args);
    let config = Config::from(&args, &env_vars)?;

    debug!("config = {config:?}");
    debug!("arguments = {args:?}");

    match &args.command {
        FenvSubcommands::Init(sub_args) => {
            FenvInitService::from(sub_args.clone()).execute(&config, &mut std::io::stdout())
        }
        FenvSubcommands::Install(sub_args) => {
            FenvInstallService::from(sub_args.clone()).execute(&config, &mut std::io::stdout())
        }
        FenvSubcommands::Versions => {
            FenvVersionsService::new().execute(&config, &mut std::io::stdout())
        }
    }
}
