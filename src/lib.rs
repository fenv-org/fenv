pub mod args;
pub mod config;
pub mod model;
pub mod service;

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
use clap::Parser;

pub fn try_run(args: &Vec<String>, env_vars: &HashMap<String, String>) -> Result<()> {
    let args = args::FenvArgs::parse_from(args);
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
